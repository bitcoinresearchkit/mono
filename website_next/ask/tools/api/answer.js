import { renderApiAnswer } from "../render.js";
import { normalize, relevance, tokenAffinity } from "../text.js";
import { focusApiData } from "./result.js";

const MAX_FIELDS = 10;

/** @param {string} type */
function dimension(type) {
  const value = type.toLowerCase();
  if (value.includes("sats")) return "sats";
  if (value.includes("vsize")) return "vB";
  if (value.includes("feerate")) return "sat/vB";
  return value;
}

/** @param {string} type */
function displayedUnit(type) {
  const value = dimension(type);
  if (value === "sats") return " sats";
  if (value === "vB") return " vB";
  if (value === "sat/vB") return " sat/vB";
  if (value.includes("timestamp")) return "";
  if (value === "number" || value === "integer" || value === "float") return "";
  return ` ${type}`;
}

/** @param {string} question */
function languageHints(question) {
  const words = new Set(question.toLowerCase().split(/[^a-z0-9]+/).filter(Boolean));
  return [
    ...(words.has("when") ? ["time timestamp date"] : []),
    ...(words.has("many") ? ["count number total"] : []),
    ...(words.has("much") ? ["amount value total"] : []),
  ];
}

/** @param {string} query @param {string} document */
function lexicalAffinity(query, document) {
  const queryWords = normalize(query).split(" ").filter(Boolean);
  const documentWords = normalize(document).split(" ").filter(Boolean);
  return Math.max(
    0,
    ...queryWords.flatMap((queryWord) =>
      documentWords.map((documentWord) =>
        tokenAffinity(queryWord, documentWord)
      )
    ),
  );
}

/**
 * @typedef {Object} ApiAnswerField
 * @property {string} ref
 * @property {string} name
 * @property {string} type
 * @property {string} [description]
 * @property {string} [ownDescription]
 * @property {string | number | boolean} value
 * @property {number} score
 *
 * @typedef {Object} ApiAnswerSpec
 * @property {ApiAnswerField[]} fields
 * @property {ApiAnswerField} [previous]
 * @property {ApiAnswerField} [resolved]
 * @property {ApiAnswerField} [direct]
 * @property {ApiAnswerField[]} related
 * @property {ApiAnswerField[]} ambiguous
 * @property {any[]} tools
 */

/** @param {unknown} value @param {string[]} path */
function valueAt(value, path) {
  let current = value;
  for (const key of path) {
    if (!current || typeof current !== "object" || !Object.hasOwn(current, key)) {
      return undefined;
    }
    current = /** @type {Record<string, unknown>} */ (current)[key];
  }
  return current;
}

/** @param {unknown} value @param {string} [type] */
function formattedValue(value, type = "") {
  if (typeof value === "number") {
    if (dimension(type).includes("timestamp")) {
      return new Date(value * 1_000).toLocaleString(undefined, {
        dateStyle: "medium",
        timeStyle: "medium",
      });
    }
    return new Intl.NumberFormat("en-US", {
      maximumFractionDigits: 8,
    }).format(value);
  }
  if (typeof value === "boolean") return value ? "yes" : "no";
  return String(value);
}

/** @param {any} grounding */
export function summarizeApiAnswer(grounding) {
  const data = focusApiData(grounding.data, grounding.arguments);
  const responseFields = /** @type {{ name: string, type: string, description?: string, ownDescription?: string }[]} */ (
    grounding.operation.response.fields ?? []
  );
  if (Array.isArray(data)) {
    const rows = data.slice(0, 4);
    const fields = responseFields.slice(0, 4);
    const total = grounding.data &&
        typeof grounding.data === "object" &&
        !Array.isArray(grounding.data) &&
        typeof grounding.data.count === "number"
      ? grounding.data.count
      : data.length;
    const primitiveRows = rows.every((row) =>
      typeof row === "string" ||
      typeof row === "number" ||
      typeof row === "boolean"
    );
    if (primitiveRows) {
      const itemType = grounding.operation.response.type.replace(/\[\]$/, "");
      return {
        output: renderApiAnswer(
          [
            `${total.toLocaleString()} record${total === 1 ? "" : "s"}${
              total > rows.length ? ` · showing ${rows.length}` : ""
            }`,
            ...rows.map((row, index) =>
              `${index + 1}. ${formattedValue(row, itemType)}`
            ),
          ].join("\n\n"),
          grounding.operation,
        ),
        fields: [],
      };
    }
    if (!rows.length || !fields.length) {
      return {
        output: renderApiAnswer(
          "The API returned no compact records to display.",
          grounding.operation,
        ),
        fields: [],
      };
    }
    const output = [
      `${total.toLocaleString()} record${total === 1 ? "" : "s"}${
        total > rows.length ? ` · showing ${rows.length}` : ""
      }`,
      ...rows.map((row, index) => {
        const values = fields
          .map((field) => ({
            field,
            value: valueAt(row, field.name.split(".")),
          }))
          .filter(({ value }) =>
            typeof value === "string" ||
            typeof value === "number" ||
            typeof value === "boolean"
          )
          .map(({ field, value }) =>
            `**${field.name.replaceAll(".", " · ").replaceAll("_", " ")}**: ${
              formattedValue(value, field.type)
            }${typeof value === "number" ? displayedUnit(field.type) : ""}`
          );
        return `${index + 1}. ${values.join(" · ")}`;
      }),
    ].join("\n\n");
    return {
      output: renderApiAnswer(output, grounding.operation),
      fields: fields.map(({ name }) => name),
    };
  }
  const fields = responseFields
    .map((field) => ({
      ...field,
      value: valueAt(data, field.name.split(".")),
    }))
    .filter((/** @type {any} */ { value }) =>
      typeof value === "string" ||
      typeof value === "number" ||
      typeof value === "boolean"
    )
    .slice(0, 8);
  if (!fields.length) {
    return {
      output: renderApiAnswer(
        "The API returned no compact primitive fields to display.",
        grounding.operation,
      ),
      fields: [],
    };
  }
  const output = fields
    .map((/** @type {any} */ field) =>
      `- **${field.name.replaceAll(".", " · ").replaceAll("_", " ")}**: ${
        formattedValue(field.value, field.type)
      }${typeof field.value === "number" ? displayedUnit(field.type) : ""}`
    )
    .join("\n");
  return {
    output: renderApiAnswer(output, grounding.operation),
    fields: fields.map((/** @type {any} */ { name }) => name),
  };
}

/** @param {any} grounding @returns {ApiAnswerSpec} */
export function createApiAnswerTool(grounding) {
  const data = focusApiData(grounding.data, grounding.arguments);
  const responseFields = /** @type {{ name: string, type: string, description?: string, ownDescription?: string }[]} */ (
    grounding.operation.response.fields ?? []
  );
  const previousName = grounding.previousFields?.length === 1
    ? grounding.previousFields[0]
    : undefined;
  const previousParents = new Set(
    (grounding.previousFields ?? []).map((/** @type {string} */ name) =>
      name.split(".").slice(0, -1).join(".")
    ).filter(Boolean),
  );
  const previousParent = previousParents.size === 1
    ? [...previousParents][0]
    : undefined;
  const parameterNames = new Set(
    grounding.operation.parameters.map(
      (/** @type {{ name: string }} */ parameter) => parameter.name,
    ),
  );
  const primitive = responseFields
    .map((field) => ({
      ...field,
      value: valueAt(data, field.name.split(".")),
    }))
    .filter((field) =>
      typeof field.value === "string" ||
      typeof field.value === "number" ||
      typeof field.value === "boolean"
    )
    .map((field, index) => ({
      ...field,
      index,
      score: relevance(
        grounding.question,
        `${field.name} ${field.ownDescription || field.description || ""}`,
      ) +
        relevance(grounding.question, field.name) +
        lexicalAffinity(grounding.question, field.name) * 8 +
        relevance(
          grounding.question,
          field.ownDescription || field.description || "",
        ) +
        Math.max(
          0,
          ...languageHints(grounding.question).map((hint) =>
            relevance(
              hint,
              `${field.name} ${field.type} ${
                field.ownDescription || field.description || ""
              }`,
            )
          ),
        ) -
        Math.max(0, field.name.split(".").length - 1) * 2 +
        (
          previousParent &&
            field.name.split(".").slice(0, -1).join(".") === previousParent
            ? 2
            : 0
        ),
    }))
    .sort((left, right) => {
      return right.score - left.score || left.index - right.index;
    });
  const answerCandidates = primitive.filter(({ name }) =>
    name !== previousName &&
    !parameterNames.has(name.split(".").at(-1) ?? name)
  );
  const best = answerCandidates
    .sort((left, right) => right.score - left.score || left.index - right.index)[0];
  const runnerUp = answerCandidates
    .filter(({ name }) => name !== best?.name)
    .sort((left, right) => right.score - left.score || left.index - right.index)[0];
  const direct = best && best.score >= 6 &&
      best.score >= (runnerUp?.score ?? 0) + 0.5
    ? best
    : undefined;
  const siblings = best
    ? primitive.filter((field) =>
      field.name.split(".").at(-1) === best.name.split(".").at(-1) &&
      best.score - field.score < 1 &&
      field.score > 0
    )
    : [];
  const matchingParent = previousParent
    ? siblings.filter((field) =>
      field.name.split(".").slice(0, -1).join(".") === previousParent
    )
    : [];
  const ambiguousNames = new Set(
    (matchingParent.length === 1 ? [] : siblings).map(({ name }) => name),
  );
  const current = primitive.filter(({ name }) => name !== previousName);
  const previousField = previousName
    ? primitive.find(({ name }) => name === previousName)
    : undefined;
  const selected = [
    ...current.slice(0, MAX_FIELDS - (previousField ? 1 : 0)),
    ...(previousField ? [previousField] : []),
  ];
  const fields = selected
    .map((field, index) => ({
      ...field,
      value: /** @type {string | number | boolean} */ (field.value),
      ref: `n${index + 1}`,
    }));
  const related = best
    ? fields.filter((field) =>
      field.score >= 6 && best.score - field.score < 3
    )
    : [];
  const previousChoices = fields
    .filter(({ name }) => grounding.previousFields?.includes(name))
    .sort((left, right) => right.score - left.score);
  const resolved = previousChoices.length > 1 &&
      previousChoices[0].score >= previousChoices[1].score + 5
    ? previousChoices[0]
    : undefined;
  const previous = previousName
    ? fields.find((field) =>
      field.name === previousName && typeof field.value === "number"
    )
    : undefined;
  const numericFields = fields.filter((field) => typeof field.value === "number");
  const calculationSplit = numericFields.findIndex((field, index) =>
    index >= 2 && numericFields[index - 1].score - field.score >= 3
  );
  const calculationFields = calculationSplit >= 2
    ? numericFields.slice(0, calculationSplit)
    : numericFields;
  /**
   * @param {string} name
   * @param {string} description
   * @param {Record<string, any>} properties
   * @param {string[]} required
   */
  const functionTool = (name, description, properties, required) => ({
    type: "function",
    function: {
      name,
      description,
      parameters: {
        type: "object",
        properties,
        required,
        additionalProperties: false,
      },
    },
  });
  const label = {
    type: "string",
    description: "Short user-facing name for the result.",
  };
  const operator = {
    type: "string",
    enum: ["add", "subtract", "multiply", "divide"],
    description: "The arithmetic operation explicitly requested by the user.",
  };
  const reference = {
    type: "string",
    enum: fields.map(({ ref }) => ref),
    description: "Verified field ref from the user message.",
  };
  const numericReference = {
    type: "string",
    enum: calculationFields.map(({ ref }) => ref),
    description: "Verified numeric field ref from the user message.",
  };
  const tools = [
    functionTool(
      "answer_api",
      [
        "Choose select for one raw primitive field.",
        "Choose select_many when several raw primitive fields were requested.",
        "Choose calculate to derive the result from component fields, including a narrower concept than an aggregate.",
        previous
          ? `Choose continue only to apply arithmetic to preceding ${previous.ref}=${previous.name}.`
          : "",
        "Choose text for a nonnumeric answer copied or summarized from verified data.",
      ].filter(Boolean).join(" "),
      {
        action: {
          type: "string",
          enum: [
            ...(fields.length ? ["select"] : []),
            ...(fields.length > 1 ? ["select_many"] : []),
            ...(calculationFields.length >= 2 ? ["calculate"] : []),
            ...(previous ? ["continue"] : []),
            "text",
          ],
        },
        ...(fields.length
          ? {
            field: reference,
            fields: {
              type: "array",
              minItems: 2,
              maxItems: Math.min(10, fields.length),
              items: reference,
              description:
                "Ordered verified field refs when several raw fields were requested.",
            },
            ...(calculationFields.length === 2
              ? {
                operator,
                left: {
                  ...numericReference,
                  description: "Left arithmetic operand: the minuend or dividend for subtract or divide.",
                },
                right: {
                  ...numericReference,
                  description: "Right arithmetic operand: the subtrahend or divisor for subtract or divide.",
                },
              }
              : calculationFields.length > 2
                ? {
                  operator,
                  operands: {
                    type: "array",
                    minItems: 2,
                    maxItems: 10,
                    items: numericReference,
                    description: "Ordered verified numeric fields for calculate.",
                  },
                }
                : {}),
          }
          : {}),
        ...(previous
          ? {
            operand: {
              type: "string",
              enum: numericFields
                .filter(({ ref }) => ref !== previous.ref)
                .map(({ ref }) => ref),
              description: `Second operand after fixed ${previous.ref}.`,
            },
          }
          : {}),
        label,
        text: {
          type: "string",
          description: "Concise nonnumeric answer containing no invented values.",
        },
      },
      ["action"],
    ),
  ];

  return {
    fields,
    previous,
    resolved,
    direct: direct ? fields.find(({ name }) => name === direct.name) : undefined,
    related,
    ambiguous: fields.filter(({ name }) => ambiguousNames.has(name)),
    tools,
  };
}

/** @param {string} name @param {Record<string, unknown>} action @param {ApiAnswerField[]} fields @param {any} grounding */
export function finishApiAnswer(name, action, fields, grounding) {
  const byRef = new Map(fields.map((field) => [field.ref, field]));
  if (name === "calculate_api_rate") {
    const numerator = byRef.get(String(action.left));
    const denominator = byRef.get(String(action.right));
    if (
      !numerator ||
      !denominator ||
      typeof numerator.value !== "number" ||
      typeof denominator.value !== "number"
    ) {
      throw new Error("The AI returned invalid rate fields");
    }
    const denominatorDimension = dimension(denominator.type);
    const divisor = denominatorDimension === "vB"
      ? denominator.value
      : denominatorDimension === "weight"
        ? Math.ceil(denominator.value / 4)
        : 0;
    if (!divisor) throw new Error("Cannot calculate this rate");
    const value = dimension(numerator.type) === "sats"
      ? Math.ceil(numerator.value * 1_000 / divisor) / 1_000
      : numerator.value / divisor;
    const label = typeof action.label === "string" && action.label.trim()
      ? action.label.trim().replaceAll("_", " ")
      : "rate";
    const unit = dimension(numerator.type) === "sats" ? " sat/vB" : "";
    return renderApiAnswer(
      `**${label}**: ${
        new Intl.NumberFormat("en-US", { maximumFractionDigits: 8 }).format(value)
      }${unit}`,
      grounding.operation,
    );
  }
  if (name === "select_api_field") {
    const field = byRef.get(String(action.field));
    if (!field) throw new Error("The AI selected an unknown API field");
    const label = typeof action.label === "string" && action.label.trim()
      ? action.label.trim().replaceAll("_", " ")
      : field.name.replaceAll(".", " · ").replaceAll("_", " ");
    const formatted = formattedValue(field.value, field.type);
    return renderApiAnswer(
      `**${label}**: ${formatted}${
        typeof field.value === "number" ? displayedUnit(field.type) : ""
      }`,
      grounding.operation,
    );
  }
  if (name === "select_api_fields") {
    const refs = Array.isArray(action.fields) ? action.fields : [];
    const selected = refs.map((ref) => byRef.get(String(ref))).filter((field) => field !== undefined);
    if (selected.length < 2) {
      throw new Error("The AI selected too few API fields");
    }
    const output = selected.map((field) => {
      const formatted = formattedValue(field.value, field.type);
      return `- **${
        field.name.replaceAll(".", " · ").replaceAll("_", " ")
      }**: ${formatted}${
        typeof field.value === "number" ? displayedUnit(field.type) : ""
      }`;
    }).join("\n");
    return renderApiAnswer(output, grounding.operation);
  }
  if (name === "answer_api_text") {
    const text = typeof action.text === "string" ? action.text.trim() : "";
    if (!text) throw new Error("The AI returned an empty API answer");
    return renderApiAnswer(text, grounding.operation);
  }
  if (name === "continue_api_calculation") {
    const previousNames = Array.isArray(grounding.previousFields)
      ? grounding.previousFields
      : [];
    const previous = previousNames.length === 1
      ? fields.find((field) => field.name === previousNames[0])
      : undefined;
    const operand = byRef.get(String(action.operand));
    if (
      !previous ||
      !operand ||
      typeof previous.value !== "number" ||
      typeof operand.value !== "number" ||
      previous.ref === operand.ref
    ) {
      throw new Error("The AI returned an invalid API follow-up calculation");
    }
    if (
      typeof action.operator !== "string" ||
      !["add", "subtract", "multiply", "divide"].includes(action.operator)
    ) {
      throw new Error("The AI returned an invalid API calculation operator");
    }
    if (action.operator === "divide" && operand.value === 0) {
      throw new Error("Cannot divide by zero");
    }
    const value = action.operator === "add"
      ? previous.value + operand.value
      : action.operator === "subtract"
        ? previous.value - operand.value
        : action.operator === "multiply"
          ? previous.value * operand.value
          : previous.value / operand.value;
    const previousDimension = dimension(previous.type);
    const operandDimension = dimension(operand.type);
    const unit = action.operator === "divide"
      ? previousDimension === operandDimension
        ? ""
        : ` ${previousDimension}/${operandDimension}`
      : previousDimension === operandDimension
        ? displayedUnit(previous.type)
        : "";
    const label = typeof action.label === "string" && action.label.trim()
      ? action.label.trim().replaceAll("_", " ")
      : "result";
    const formatted = new Intl.NumberFormat("en-US", {
      maximumFractionDigits: 8,
    }).format(value);
    return renderApiAnswer(`**${label}**: ${formatted}${unit}`, grounding.operation);
  }
  if (name !== "calculate_api_fields") {
    throw new Error("The AI returned an invalid API calculation");
  }
  if (
    typeof action.operator === "string" &&
    ["add", "subtract", "multiply", "divide"].includes(action.operator) &&
    (
      (
        typeof action.left === "string" &&
        typeof action.right === "string"
      ) ||
      (
        Array.isArray(action.operands) &&
        action.operands.length >= 2
      )
    )
  ) {
    const refs = typeof action.left === "string" &&
        typeof action.right === "string"
      ? [action.left, action.right]
      : /** @type {unknown[]} */ (action.operands);
    const selected = refs.map((ref) => byRef.get(String(ref)));
    if (selected.some((field) => !field)) throw new Error("Unknown calculation field");
    if (selected.some((field) => typeof field?.value !== "number")) {
      throw new Error("A calculation requires numeric fields");
    }
    const numeric = /** @type {(ApiAnswerField & { value: number })[]} */ (selected);
    const [first, ...rest] = numeric;
    const types = new Set(numeric.map(({ type }) => dimension(type)));
    if (
      (action.operator === "add" || action.operator === "subtract") &&
      types.size > 1
    ) {
      const choices = numeric
        .map((field) =>
          `**${field.name.replaceAll(".", " · ").replaceAll("_", " ")}** (${field.type})`
        )
        .join(", ");
      return renderApiAnswer(
        `Those fields use different units: ${choices}. Which one do you want?`,
        grounding.operation,
      );
    }
    const value = rest.reduce((result, field) => {
      if (action.operator === "add") return result + field.value;
      if (action.operator === "subtract") return result - field.value;
      if (action.operator === "multiply") return result * field.value;
      if (field.value === 0) throw new Error("Cannot divide by zero");
      return result / field.value;
    }, first.value);
    const unit = action.operator === "divide" && numeric.length === 2
      ? dimension(first.type) === dimension(numeric[1].type)
        ? ""
        : ` ${dimension(first.type)}/${dimension(numeric[1].type)}`
      : types.size === 1
        ? displayedUnit(first.type)
        : "";
    const label = typeof action.label === "string" && action.label.trim()
      ? action.label.trim().replaceAll("_", " ")
      : "result";
    const formatted = new Intl.NumberFormat("en-US", { maximumFractionDigits: 8 }).format(value);
    return renderApiAnswer(`**${label}**: ${formatted}${unit}`, grounding.operation);
  }
  throw new Error("The AI returned an invalid API calculation");
}
