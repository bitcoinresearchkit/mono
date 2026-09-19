import {
  createApiAnswerTool,
  finishApiAnswer,
  summarizeApiAnswer,
} from "./api/answer.js";
import { prewarmApiIndex, terminateApiIndex } from "./api/index.js";
import {
  apiRows,
  recordArguments,
  selectApiRecord,
} from "./api/records.js";
import { prewarmMetricIndex, terminateMetricIndex } from "./metrics/index.js";
import { renderEvidence } from "./render.js";
import { AskToolSession } from "./session/index.js";
import { arithmeticAnswer } from "./source/arithmetic.js";
import { AskSource } from "./source/index.js";
import { normalize, relevance } from "./text.js";

const NUMBER = /\d+(?:[.,]\d+)*/g;

/** @param {unknown} value */
function numbers(value) {
  return String(value)
    .match(NUMBER)
    ?.map((number) => number.replaceAll(",", "")) ?? [];
}

/**
 * Quantities in an ungrounded answer are worth a second look. This deliberately
 * checks syntax rather than guessing the user's intent or maintaining a list of
 * Bitcoin concepts.
 *
 * @param {string} answer
 * @param {import("../model.js").ChatMessage[]} messages
 */
function unsupportedNumbers(answer, messages) {
  const supported = new Set(numbers(
    messages
      .filter(({ role }) => role === "user")
      .map(({ content }) => content)
      .join(" "),
  ));
  return new Set(numbers(answer).filter((number) => !supported.has(number)));
}

/**
 * The small model can repeat an unsupported quantity after being asked to
 * revise it. Keep the useful grounded sentences instead of trusting a second
 * model pass to police itself.
 *
 * @param {string} answer
 * @param {import("../model.js").ChatMessage[]} messages
 */
function removeUnsupportedQuantitySentences(answer, messages) {
  const unsupported = unsupportedNumbers(answer, messages);
  if (!unsupported.size) return answer.trim();

  const kept = [...new Intl.Segmenter(undefined, { granularity: "sentence" })
    .segment(answer)]
    .map(({ segment }) => segment.trim())
    .filter((segment) =>
      !numbers(segment).some((number) => unsupported.has(number))
    );
  return kept.join(" ").trim();
}

/** @param {string} reference */
function codeSubject(reference) {
  const value = reference.trim();
  if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(value)) return value;
  return value.match(/^([A-Za-z_][A-Za-z0-9_]*)\s*\(/)?.[1];
}

/**
 * @typedef {Object} ToolOutcome
 * @property {boolean} done
 * @property {string} [output]
 * @property {import("../storage.js").StoredArtifact[]} [artifacts]
 * @property {string[]} [metricPaths]
 * @property {import("../storage.js").ApiContext} [apiContext]
 * @property {import("../storage.js").SourceContext[]} [sourceContext]
 * @property {import("../storage.js").KnowledgeContext} [knowledgeContext]
 * @property {{ question: string, title?: string, metrics: { name: string, path: string, unit?: string }[], facts: string[], contextFacts?: string[], excerpts: import("../storage.js").SourceContext[], subjects?: string[], renderFacts?: boolean, validateNumbers?: boolean }} [grounding]
 * @property {{ question: string, previousFields: string[], previousArguments?: Record<string, unknown>, previousRecords?: unknown[], operation: { key: string, method: string, path: string, summary: string, description: string, parameters: { name: string }[], response: { type: string, fields?: { name: string, type: string, description?: string }[] } }, arguments: Record<string, unknown>, requestPath: string, data: unknown, truncated: boolean }} [apiGrounding]
 *
 * @typedef {Object} AskAnswer
 * @property {string} output
 * @property {import("../storage.js").StoredArtifact[]} artifacts
 * @property {string[]} [metricPaths]
 * @property {import("../storage.js").ApiContext} [apiContext]
 * @property {import("../storage.js").SourceContext[]} [sourceContext]
 * @property {import("../storage.js").KnowledgeContext} [knowledgeContext]
 * @property {string} [capability]
 * @property {import("../storage.js").StoredChat} chat
 */

/**
 * @param {import("../model.js").AskModel} model
 * @param {NonNullable<ToolOutcome["grounding"]>} grounding
 * @param {(status: string) => void} onStatus
 */
async function answerFromEvidence(model, grounding, onStatus) {
  const arithmetic = arithmeticAnswer(grounding);
  if (arithmetic) {
    return {
      output: renderEvidence({
        facts: [arithmetic, ...grounding.facts],
        sources: grounding.excerpts.slice(0, 2),
        excerpts: [],
      }),
      sourceContext: grounding.excerpts.slice(0, 2),
      knowledgeContext: {
        title: grounding.metrics[0].name,
        description: arithmetic,
      },
    };
  }

  onStatus("Answering from source…");
  const evidence = [
    `Request: ${grounding.question}`,
    grounding.metrics.length
      ? `Verified metrics:\n${grounding.metrics.map(({ name, path, unit }) =>
          `- ${name} | ${path}${unit ? ` | unit: ${unit}` : ""}`
        ).join("\n")}`
      : "",
    grounding.facts.length || grounding.contextFacts?.length
      ? `Verified facts:\n${
          [...grounding.facts, ...(grounding.contextFacts ?? [])]
            .map((fact) => `- ${fact}`).join("\n")
        }`
      : "",
    grounding.excerpts.length
      ? `Verified source excerpts, strongest first:\n${grounding.excerpts.map(
          ({ path, startLine, endLine, content }, index) =>
            `[${index + 1}] ${path}:${startLine}${endLine ? `-${endLine}` : ""}\n${content}`,
        ).join("\n\n")}`
      : "",
  ].filter(Boolean).join("\n\n");
  const messages = [
    {
      role: /** @type {const} */ ("system"),
      content: "Use only the verified evidence. Answer the exact request in at most 45 words. When an example is requested, instantiate the evidence as a clearly hypothetical sequence with named actors or objects and concrete actions instead of summarizing it; use symbolic labels or qualitative amounts rather than unsupported quantities. Metric names and units are exact. Never add a fact absent from the evidence. Do not cite, number, name, or quote source files; the renderer appends source links.",
    },
    {
      role: /** @type {const} */ ("user"),
      content: `${evidence}\n\nUse only the verified evidence above. Do not explain what code identifiers mean unless the evidence does.`,
    },
  ];
  let result = await model.generate(
    messages,
    () => {},
    [],
    "none",
    { maxTokens: 72 },
  );
  if (
    grounding.validateNumbers &&
    unsupportedNumbers(result.text, messages).size
  ) {
    result = await model.generate(
      [
        ...messages,
        {
          role: "assistant",
          content: result.text,
        },
        {
          role: "user",
          content: "Replace the draft with the exact requested answer using no numeric quantities absent from the verified evidence. For a hypothetical example, reconstruct it directly from the evidence: name individual entities with symbolic labels such as A and B, then state the before state, the action, and the after state. Use only entities and actions supported by the evidence, with no numerals or value calculations. Return only the replacement answer.",
        },
      ],
      () => {},
      [],
      "none",
      { maxTokens: 72 },
    );
  }
  if (!result.text.trim()) {
    result = await model.generate(
      [
        ...messages,
        {
          role: "user",
          content: "Return one concise direct answer to the request using only the verified evidence. If the request asks for an example, use the example already present in the evidence. Return only the answer.",
        },
      ],
      () => {},
      [],
      "none",
      { maxTokens: 72 },
    );
  }
  const draft = (
    grounding.validateNumbers
      ? removeUnsupportedQuantitySentences(result.text, messages)
      : result.text
  ).trim();
  const evidenceText = normalize([
    ...grounding.metrics.flatMap(({ name, path, unit }) => [
      name,
      path,
      unit,
    ]),
    ...grounding.facts,
    ...(grounding.contextFacts ?? []),
    ...grounding.excerpts.flatMap(({ path, content }) => [path, content]),
  ].filter(Boolean).join(" "));
  const inline = [...draft.matchAll(/`([^`\n]+)`/g)].map((match) => match[1]);
  const paths = draft.match(/(?:[A-Za-z0-9_.-]+\/){2,}[A-Za-z0-9_.-]+/g) ?? [];
  const unsupportedReference = [...inline, ...paths].some((reference) => {
    const normalized = normalize(reference);
    return normalized && !evidenceText.includes(normalized);
  });
  const answer = unsupportedReference ? "" : draft;
  const answerSubjects = [...new Set(
    inline.map(codeSubject).filter((subject) => subject !== undefined),
  )];
  const groundedSubject = grounding.subjects?.[0];
  const referencedSubjects = answerSubjects.length > 1 && groundedSubject
    ? answerSubjects.filter((subject) => subject !== groundedSubject)
    : answerSubjects;
  const subjects = grounding.subjects?.length
    ? grounding.subjects
    : referencedSubjects;
  const sources = grounding.excerpts.slice(0, 2);
  const fallback = !answer && sources[0]
    ? "I found related source, but not enough verified evidence for a precise answer."
    : "";
  return {
    output: renderEvidence({
      facts: [
        answer || fallback,
        ...(grounding.renderFacts === false ? [] : grounding.facts),
      ].filter(Boolean),
      sources,
      excerpts: [],
    }),
    sourceContext: sources,
    knowledgeContext: answer
      ? {
          title: grounding.metrics[0]?.name ??
            grounding.title ??
            subjects[0] ??
            grounding.question.slice(0, 160),
          description: answer,
          ...(subjects.length
            ? { subjects }
            : {}),
        }
      : undefined,
  };
}

/** @param {string} question */
function requestedArithmetic(question) {
  const words = new Set(normalize(question).split(" "));
  const matches = /** @type {const} */ ([
    { action: "add", words: ["add", "plus"] },
    { action: "subtract", words: ["subtract", "minus"] },
    { action: "multiply", words: ["multiply", "times"] },
    { action: "divide", words: ["divide", "rate"] },
  ]).filter(({ words: candidates }) =>
    candidates.some((word) => words.has(word))
  );
  return matches.length === 1 ? matches[0].action : undefined;
}

/** @param {string} question @param {string} field */
function fieldPosition(question, field) {
  const words = normalize(question).split(" ");
  const fieldWords = new Set(
    normalize(field.split(".").at(-1)).split(" ").filter((word) =>
      word.length > 2
    ),
  );
  const positions = words
    .map((word, index) => fieldWords.has(word) ? index : -1)
    .filter((index) => index >= 0);
  return positions.length ? Math.min(...positions) : -1;
}

/** @param {{ name: string, description?: string, ownDescription?: string }} field */
function apiFieldText(field) {
  return `${field.name} ${field.ownDescription || field.description || ""}`;
}

/**
 * Match clauses to independently described schema fields. Returns nothing
 * unless every clause has one clear winner.
 *
 * @param {string[]} clauses
 * @param {import("./api/answer.js").ApiAnswerField[]} fields
 * @param {string[] | undefined} previousFields
 */
function matchApiClauses(clauses, fields, previousFields) {
  const parents = new Set(
    (previousFields ?? [])
      .map((name) => name.split(".").slice(0, -1).join("."))
      .filter(Boolean),
  );
  const preferredParent = parents.size === 1 ? [...parents][0] : undefined;
  /** @param {import("./api/answer.js").ApiAnswerField[]} candidates */
  const resolve = (candidates) => {
    /** @type {import("./api/answer.js").ApiAnswerField[]} */
    const chosen = [];
    for (const clause of clauses) {
      const ranked = candidates
        .filter(({ ref }) => !chosen.some((field) => field.ref === ref))
        .map((field) => ({
          field,
          score: relevance(clause, apiFieldText(field)) +
            (
              preferredParent &&
                field.name.split(".").slice(0, -1).join(".") === preferredParent
                ? 2
                : 0
            ),
        }))
        .sort((left, right) => right.score - left.score);
      const [best, runnerUp] = ranked;
      if (
        !best ||
        best.score < 2 ||
        best.score - (runnerUp?.score ?? 0) < 0.4
      ) {
        return [];
      }
      chosen.push(best.field);
    }
    return chosen;
  };
  const previousNames = new Set(previousFields ?? []);
  const previous = fields.filter(({ name }) => previousNames.has(name));
  const contextual = previous.length >= clauses.length
    ? resolve(previous)
    : [];
  return contextual.length ? contextual : resolve(fields);
}

/**
 * @param {string} question
 * @param {import("./api/answer.js").ApiAnswerField[]} fields
 * @param {string[] | undefined} previousFields
 */
function coordinatedApiFields(question, fields, previousFields) {
  const clauses = normalize(question)
    .split(" and ")
    .map((clause) => clause.trim())
    .filter(Boolean);
  return clauses.length < 2
    ? []
    : matchApiClauses(clauses, fields, previousFields);
}

/**
 * @param {string} question
 * @param {"add" | "subtract" | "multiply" | "divide"} arithmetic
 * @param {import("./api/answer.js").ApiAnswerField[]} fields
 * @param {string[] | undefined} previousFields
 */
function arithmeticApiFields(question, arithmetic, fields, previousFields) {
  const normalized = ` ${normalize(question)} `;
  const separator = arithmetic === "subtract"
    ? " from "
    : arithmetic === "divide"
      ? " by "
      : " and ";
  const parts = normalized
    .split(separator)
    .map((part) => part.trim())
    .filter(Boolean);
  if (parts.length !== 2) return [];
  const clauses = arithmetic === "subtract"
    ? [parts[1], parts[0]]
    : parts;
  return matchApiClauses(clauses, fields, previousFields);
}

/**
 * @param {import("../model.js").AskModel} model
 * @param {NonNullable<ToolOutcome["apiGrounding"]>} grounding
 * @param {(status: string) => void} onStatus
 */
async function answerFromApiGrounding(model, grounding, onStatus) {
  const apiAnswer = createApiAnswerTool(grounding);
  const normalizedQuestion = normalize(grounding.question);
  const question = ` ${normalizedQuestion} `;
  const questionWords = new Set(normalizedQuestion.split(" "));
  const arithmetic = requestedArithmetic(grounding.question);
  const asksForSeveral = questionWords.has("and");
  const parameterNames = new Set(
    grounding.operation.parameters.map(({ name }) => normalize(name)),
  );
  const mentionedResponse = (
    grounding.operation.response.fields ?? []
  ).some(({ name }) => {
    const field = normalize(name.split(".").at(-1));
    return field && !parameterNames.has(field) &&
      question.includes(` ${field} `);
  });
  const directFields = apiAnswer.fields.filter((field) => {
    const name = normalize(field.name.split(".").at(-1));
    return name && question.includes(` ${name} `);
  });
  const coordinated = coordinatedApiFields(
    grounding.question,
    apiAnswer.fields,
    grounding.previousFields,
  );
  const calculated = arithmetic
    ? arithmeticApiFields(
      grounding.question,
      arithmetic,
      apiAnswer.fields,
      grounding.previousFields,
    )
    : [];
  if (calculated.length > 1) {
    return {
      output: finishApiAnswer(
        "calculate_api_fields",
        {
          operator: arithmetic,
          operands: calculated.map(({ ref }) => ref),
          label: "result",
        },
        apiAnswer.fields,
        grounding,
      ),
      fields: calculated.map(({ name }) => name),
    };
  }
  if (coordinated.length > 1) {
    if (arithmetic) {
      return {
        output: finishApiAnswer(
          "calculate_api_fields",
          {
            operator: arithmetic,
            operands: coordinated.map(({ ref }) => ref),
            label: "result",
          },
          apiAnswer.fields,
          grounding,
        ),
        fields: coordinated.map(({ name }) => name),
      };
    }
    return {
      output: finishApiAnswer(
        "select_api_fields",
        { fields: coordinated.map(({ ref }) => ref) },
        apiAnswer.fields,
        grounding,
      ),
      fields: coordinated.map(({ name }) => name),
    };
  }
  if (
    arithmetic === "divide" &&
    directFields.length === 1
  ) {
    const numerator = directFields[0];
    const rateDenominators = apiAnswer.fields.filter((field) =>
      field.ref !== numerator.ref &&
      typeof field.value === "number" &&
      ["vsize", "weight"].some((type) =>
        normalize(field.type).includes(type)
      )
    );
    if (rateDenominators.length) {
      const denominator = rateDenominators.find(({ type }) =>
        normalize(type).includes("vsize")
      ) ?? rateDenominators[0];
      return {
        output: finishApiAnswer(
          "calculate_api_rate",
          {
            left: numerator.ref,
            right: denominator.ref,
            label: `${
              numerator.name.split(".").at(-1)?.replaceAll("_", " ")
            } rate`,
          },
          apiAnswer.fields,
          grounding,
        ),
        fields: [numerator.name, denominator.name],
      };
    }
    const denominators = apiAnswer.fields.filter((field) =>
      field.ref !== numerator.ref &&
      typeof field.value === "number" &&
      field.type !== numerator.type &&
      !directFields.some(({ ref }) => ref === field.ref)
    );
    if (denominators.length === 1) {
      const denominator = denominators[0];
      return {
        output: finishApiAnswer(
          "calculate_api_fields",
          {
            operator: "divide",
            left: numerator.ref,
            right: denominator.ref,
            label: `${
              numerator.name.split(".").at(-1)?.replaceAll("_", " ")
            } rate`,
          },
          apiAnswer.fields,
          grounding,
        ),
        fields: [numerator.name, denominator.name],
      };
    }
  }
  const previous = apiAnswer.previous;
  const previousOperand = arithmetic && previous
    ? directFields.find(({ ref }) => ref !== previous.ref) ??
      (() => {
        const compatible = apiAnswer.fields
          .filter((field) =>
            field.ref !== previous.ref &&
            typeof field.value === "number" &&
            field.type === previous.type
          )
          .sort((left, right) => right.score - left.score);
        return compatible[0]?.score >= 6 &&
            compatible[0].score - (compatible[1]?.score ?? 0) >= 0.5
          ? compatible[0]
          : undefined;
      })()
    : undefined;
  if (arithmetic && previous && previousOperand) {
    const current = previousOperand;
    const previousPosition = fieldPosition(grounding.question, previous.name);
    const currentPosition = fieldPosition(grounding.question, current.name);
    const fromPosition = normalizedQuestion.split(" ").indexOf("from");
    const reverseSubtract = arithmetic === "subtract" &&
      fromPosition >= 0 &&
      previousPosition >= 0 &&
      previousPosition < fromPosition &&
      currentPosition > fromPosition;
    const [left, right] = reverseSubtract
      ? [current, previous]
      : previousPosition >= 0 &&
          currentPosition >= 0 &&
          currentPosition < previousPosition
        ? [current, previous]
        : [previous, current];
    const label = `${left.name.split(".").at(-1)?.replaceAll("_", " ")} ${
      arithmetic === "add"
        ? "plus"
        : arithmetic === "subtract"
          ? "minus"
          : arithmetic === "multiply"
            ? "times"
            : "divided by"
    } ${right.name.split(".").at(-1)?.replaceAll("_", " ")}`;
    return {
      output: finishApiAnswer(
        "calculate_api_fields",
        {
          operator: arithmetic,
          left: left.ref,
          right: right.ref,
          label,
        },
        apiAnswer.fields,
        grounding,
      ),
      fields: [left.name, right.name],
    };
  }
  if (
    !arithmetic &&
    !asksForSeveral &&
    apiAnswer.resolved
  ) {
    const field = apiAnswer.resolved;
    return {
      output: finishApiAnswer(
        "select_api_field",
        { field: field.ref },
        apiAnswer.fields,
        grounding,
      ),
      fields: [field.name],
    };
  }
  if (apiAnswer.ambiguous.length > 1) {
    const choices = apiAnswer.ambiguous
      .map((field) =>
        `**${field.name.replaceAll(".", " · ").replaceAll("_", " ")}**${
          field.description ? ` — ${field.description}` : ""
        }`
      )
      .join("\n- ");
    return {
      output: finishApiAnswer(
        "answer_api_text",
        {
          text: `I found multiple matching fields:\n- ${choices}\n\nWhich one do you mean?`,
        },
        apiAnswer.fields,
        grounding,
      ),
      fields: apiAnswer.ambiguous.map(({ name }) => name),
    };
  }
  if (
    !grounding.previousFields?.length &&
    !mentionedResponse
  ) {
    return summarizeApiAnswer(grounding);
  }
  if (!arithmetic && !asksForSeveral && apiAnswer.direct) {
    const field = apiAnswer.direct;
    return {
      output: finishApiAnswer(
        "select_api_field",
        { field: field.ref },
        apiAnswer.fields,
        grounding,
      ),
      fields: [field.name],
    };
  }
  if (!arithmetic && apiAnswer.related.length > 1) {
    return {
      output: finishApiAnswer(
        "select_api_fields",
        { fields: apiAnswer.related.map(({ ref }) => ref) },
        apiAnswer.fields,
        grounding,
      ),
      fields: apiAnswer.related.map(({ name }) => name),
    };
  }
  onStatus("Answering from API…");
  const instruction = apiAnswer.fields.length
    ? `Answer the exact newest request using only the verified API result. Call exactly one matching tool. Select one raw field only when that field itself was requested; select_many when several raw fields were requested${apiAnswer.previous ? "; continue the preceding numeric answer when the request applies arithmetic to it" : ""}. When the requested concept is narrower than an aggregate field, derive it from matching component fields. Never replace requested arithmetic with a convenient field. For subtraction and division, keep operands in the request's arithmetic order: minuend or dividend first. Preserve identifiers and units. Never invent missing values.`
    : "Answer the exact request using only the verified API result. Call answer_api_text exactly once. Preserve identifiers and units. Never invent missing values.";
  const prompt = {
    question: grounding.question,
    previous: apiAnswer.previous
      ? {
        ref: apiAnswer.previous.ref,
        name: apiAnswer.previous.name,
        type: apiAnswer.previous.type,
        value: apiAnswer.previous.value,
      }
      : undefined,
    fields: apiAnswer.fields.map(({ ref, name, type, description, value }) => ({
      ref,
      name,
      type,
      description,
      value,
    })),
  };
  const generateAnswer = () =>
    model.generate(
      [
        {
          role: "system",
          content: instruction,
        },
        { role: "user", content: JSON.stringify(prompt) },
      ],
      () => {},
      apiAnswer.tools,
      { name: "answer_api" },
      { maxTokens: 72 },
    );
  let answer = await generateAnswer();
  let call = answer.toolCalls[0];
  if (!call || call.name !== "answer_api") {
    return summarizeApiAnswer(grounding);
  }
  const actionFor = (/** @type {Record<string, unknown>} */ arguments_) =>
    arguments_.action === "select"
    ? "select_api_field"
    : arguments_.action === "select_many"
      ? "select_api_fields"
    : arguments_.action === "continue"
      ? "continue_api_calculation"
      : arguments_.action === "calculate"
        ? "calculate_api_fields"
        : arguments_.action === "text"
          ? "answer_api_text"
          : "";
  let actionName = actionFor(call.arguments);
  if (!actionName) return summarizeApiAnswer(grounding);
  const selectedRefs = actionName === "select_api_field"
    ? [call.arguments.field]
    : actionName === "select_api_fields"
      ? Array.isArray(call.arguments.fields) ? call.arguments.fields : []
    : actionName === "continue_api_calculation"
      ? [apiAnswer.previous?.ref, call.arguments.operand]
      : typeof call.arguments.left === "string" &&
          typeof call.arguments.right === "string"
        ? [call.arguments.left, call.arguments.right]
      : Array.isArray(call.arguments.operands)
        ? call.arguments.operands
        : [];
  const selected = new Set(selectedRefs.map(String));
  try {
    return {
      output: finishApiAnswer(
        actionName,
        call.arguments,
        apiAnswer.fields,
        grounding,
      ),
      fields: apiAnswer.fields
        .filter(({ ref }) => selected.has(ref))
        .map(({ name }) => name),
    };
  } catch {
    return summarizeApiAnswer(grounding);
  }
}

/**
 * @param {import("../model.js").AskModel} model
 * @param {NonNullable<ToolOutcome["apiGrounding"]>} grounding
 * @param {(status: string) => void} onStatus
 */
async function answerFromApi(model, grounding, onStatus) {
  const previousRows = apiRows(grounding.previousRecords);
  const currentRows = apiRows(grounding.data);
  const previousRecord = selectApiRecord(
    previousRows,
    grounding.question,
    grounding.previousArguments,
  );
  const record = previousRecord ?? selectApiRecord(
    currentRows,
    grounding.question,
    grounding.previousArguments,
  );
  const contextRows = previousRecord ? previousRows : currentRows;
  const answered = await answerFromApiGrounding(
    model,
    record ? { ...grounding, data: record } : grounding,
    onStatus,
  );
  return {
    ...answered,
    ...(record
      ? {
          contextArguments: {
            ...grounding.arguments,
            ...recordArguments(record, grounding.operation.response.type),
          },
        }
      : {}),
    ...(contextRows?.length
      ? { contextRecords: contextRows.slice(0, 4) }
      : {}),
  };
}

export function createAskTools() {
  const source = new AskSource();
  /** @type {AbortController | undefined} */
  let controller;

  return {
    prewarm() {
      return Promise.all([
        prewarmApiIndex(),
        prewarmMetricIndex(),
        source.prewarm(),
      ]);
    },

    /**
     * @param {Object} options
     * @param {string} options.question
     * @param {import("../storage.js").StoredMessage[]} options.history
     * @param {import("../model.js").AskModel} options.model
     * @param {() => Promise<{ chat: import("../storage.js").StoredChat, messages: import("../model.js").ChatMessage[] }>} options.prepare
     * @param {(update: import("../model.js").TokenUpdate) => void} options.onToken
     * @param {(status: string) => void} options.onStatus
     * @returns {Promise<AskAnswer>}
     */
    async answer({
      question,
      history,
      model,
      prepare,
      onToken: _onToken,
      onStatus,
    }) {
      controller = new AbortController();
      const { signal } = controller;

      try {
        const session = new AskToolSession(source);
        const [prepared] = await Promise.all([
          prepare(),
          session.begin(question, history, onStatus),
        ]);
        signal.throwIfAborted();

        /** @type {{ action: string, call?: import("../model.js").ToolCall } | undefined} */
        const direct = session.directRoute();
        let call = direct?.call;
        let action = direct?.action ?? "";
        let continueContext = false;
        if (!action) {
          onStatus("Choosing capability…");
          const routeTools = session.routeTools();
          const route = await model.generate(
            session.routeMessages(),
            () => {},
            routeTools,
            { name: "choose_capability" },
            { maxTokens: 48 },
          );
          const selected = route.toolCalls[0];
          const selectedCapability = selected?.name === "choose_capability" &&
              typeof selected.arguments.capability === "string"
            ? selected.arguments.capability
            : "";
          const selectedContinuesContext = selected?.name ===
                "choose_capability" &&
              selected.arguments.continuesContext === true;
          action = selectedCapability;
          if (action === "answer_general" && selectedContinuesContext) {
            action = session.contextualGeneralAction(true) ?? action;
          }
          if (!action) {
            throw new Error("The AI did not choose a valid capability");
          }
          call = session.directCall(action);
        }

        signal.throwIfAborted();
        await session.prepareAction(action, onStatus);
        signal.throwIfAborted();
        if (!call || action === "explain_metric_calculation") {
          call = session.directCall(action) ?? call;
        }
        if (!call) {
          onStatus("Understanding request…");
          if (action === "answer_general") {
            const messages = session.actionMessages(action);
            const tool = session.actionTool(action);
            const result = await model.generate(
              messages,
              () => {},
              [tool],
              { name: action },
              { maxTokens: 128 },
            );
            const generalCall = result.toolCalls[0];
            continueContext = session.continuesGeneral(
              generalCall?.arguments.explicitSubject,
              generalCall?.arguments.topic,
            );
            const contextualAction = session.contextualGeneralAction(
              continueContext,
            );
            if (contextualAction && contextualAction !== action) {
              action = contextualAction;
              call = session.directCall(action);
            } else {
              const rawAnswer =
                typeof generalCall?.arguments.answer === "string"
                  ? generalCall.arguments.answer
                  : "";
              const answer = (
                generalCall?.arguments.quantityUse === "hypothetical"
                  ? rawAnswer.trim()
                  : removeUnsupportedQuantitySentences(rawAnswer, messages)
              ) || (
                unsupportedNumbers(rawAnswer, messages).size
                  ? "What would you like numbers for—for example, a metric, transaction, address, or block?"
                  : "I do not have enough verified context to answer that without guessing."
              );
              if (generalCall?.name === action) {
                call = {
                  ...generalCall,
                  arguments: {
                    ...generalCall.arguments,
                    answer,
                  },
                };
              }
            }
          } else {
            const result = await model.generate(
              session.actionMessages(action),
              () => {},
              [session.actionTool(action)],
              { name: action },
              { maxTokens: action === "call_api" ? 128 : 64 },
            );
            call = result.toolCalls[0];
          }
        }
        if (!call || call.name !== action) {
          throw new Error("The AI did not complete the selected capability");
        }
        if (action === "call_api") {
          const ref = typeof call.arguments.ref === "string"
            ? call.arguments.ref
            : "";
          if (!ref) throw new Error("The AI did not select an API operation");
          let arguments_ = session.apiArguments(ref);
          if (
            !session.hasApiArguments(ref, arguments_) &&
            Object.keys(arguments_).length
          ) {
            onStatus("Reading API arguments…");
            const argumentsResult = await model.generate(
              session.apiArgumentMessages(ref),
              () => {},
              [session.apiArgumentTool(ref)],
              { name: "provide_api_arguments" },
              { maxTokens: 96 },
            );
            const argumentsCall = argumentsResult.toolCalls[0];
            if (
              !argumentsCall ||
              argumentsCall.name !== "provide_api_arguments"
            ) {
              throw new Error("The AI did not provide valid API arguments");
            }
            arguments_ = {
              ...(argumentsCall.arguments.arguments ?? {}),
              ...arguments_,
            };
          }
          arguments_ = session.validateApiArguments(ref, arguments_);
          const missing = session.missingApiArguments(ref, arguments_);
          if (missing.length) {
            const subject = missing
              .map((/** @type {any} */ parameter) =>
                parameter.description || parameter.name
              )
              .join(" and ");
            return {
              output: `Which ${subject} should I use?`,
              artifacts: [],
              capability: action,
              chat: prepared.chat,
            };
          }
          call = {
            name: action,
            arguments: {
              ref,
              arguments: arguments_,
            },
          };
        }

        const outcome = /** @type {ToolOutcome} */ (
          await session.execute(call, onStatus, signal)
        );
        if (outcome.apiGrounding) {
          const answered = await answerFromApi(
            model,
            outcome.apiGrounding,
            onStatus,
          );
          return {
            output: answered.output,
            artifacts: [],
            capability: action,
            apiContext: outcome.apiContext
              ? {
                ...outcome.apiContext,
                ...(answered.contextArguments
                  ? { arguments: answered.contextArguments }
                  : {}),
                ...(answered.contextRecords
                  ? { records: answered.contextRecords }
                  : {}),
                ...(answered.fields.length
                  ? { fields: answered.fields }
                  : {}),
              }
              : undefined,
            chat: prepared.chat,
          };
        }
        if (outcome.grounding) {
          const grounded = await answerFromEvidence(
            model,
            outcome.grounding,
            onStatus,
          );
          return {
            output: grounded.output,
            artifacts: [],
            capability: action,
            metricPaths: outcome.metricPaths,
            sourceContext: grounded.sourceContext,
            knowledgeContext: grounded.knowledgeContext,
            chat: prepared.chat,
          };
        }
        return {
          output: outcome.output ?? "",
          artifacts: outcome.artifacts ?? [],
          capability: action,
          metricPaths: outcome.metricPaths,
          apiContext: outcome.apiContext,
          sourceContext: action === "answer_general" && !continueContext
            ? undefined
            : outcome.sourceContext,
          knowledgeContext: outcome.knowledgeContext,
          chat: prepared.chat,
        };
      } finally {
        controller = undefined;
        onStatus("");
      }
    },

    stop() {
      controller?.abort();
      source.terminate();
    },

    terminate() {
      controller?.abort();
      controller = undefined;
      source.terminate();
      terminateApiIndex();
      terminateMetricIndex();
    },
  };
}
