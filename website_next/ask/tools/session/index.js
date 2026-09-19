import { AskRefs } from "../refs.js";
import {
  ROUTE_INSTRUCTION,
  actionInstruction,
  actionTool,
  apiArgumentTool,
  availableActions,
  capabilityMetrics,
  directAction,
  generalCapabilities,
  requestedChartStyles,
  routeTools,
} from "./capabilities.js";
import { loadSessionContext } from "./context.js";
import { collectEvidence, collectSourceOptions } from "./evidence.js";
import { CapabilityExecutor } from "./executor.js";
import { normalize, relevance, tokenAffinity } from "../text.js";
import { recordArguments, selectApiRecord } from "../api/records.js";
import {
  explicitArguments,
  hasRequiredArguments,
  reusableArguments,
  validatedArguments,
} from "../api/routing.js";

const RETRIEVAL_TERMS = new Set([
  "display",
  "fetch",
  "get",
  "inspect",
  "lookup",
  "read",
  "retrieve",
  "show",
]);

/** @param {unknown[]} values */
function schemaTokens(values) {
  return new Set(
    normalize(values.filter(Boolean).join(" "))
      .split(" ")
      .filter((token) => token.length >= 3),
  );
}

/** @param {Set<string>} query @param {Set<string>} document */
function overlapCount(query, document) {
  return [...query].filter((token) => document.has(token)).length;
}

/** @param {Set<string>} query @param {Set<string>} document */
function semanticOverlapCount(query, document) {
  return [...query].filter((token) =>
    [...document].some((candidate) =>
      tokenAffinity(token, candidate) >= 0.75
    )
  ).length;
}

/** @param {string} question */
function explicitPosition(question) {
  const date = question.match(
    /(?<![A-Za-z0-9])\d{4}-\d{2}-\d{2}(?![A-Za-z0-9])/,
  )?.[0];
  if (date) return date;
  const values = [...new Set(
    question.match(
      /(?<![A-Za-z0-9])[-+]?\d[\d,]*(?:\.\d+)?(?![A-Za-z0-9])/g,
    ) ?? [],
  )];
  return values.length === 1 ? values[0].replaceAll(",", "") : undefined;
}

/** @param {Awaited<ReturnType<typeof collectEvidence>>} evidence @param {string} question @param {boolean} [requiresExample] */
function preferredGuide(evidence, question, requiresExample = false) {
  return evidence.guideOptions
    .filter(({ guide }) => !requiresExample || guide.example)
    .map((option) => ({
      option,
      relevance: Math.max(
        relevance(question, option.guide.title),
        relevance(evidence.context.knowledge?.title, option.guide.title),
      ),
    }))
    .sort((left, right) => right.relevance - left.relevance)[0]?.option;
}

export class AskToolSession {
  /** @param {import("../source/index.js").AskSource} source */
  constructor(source) {
    this.source = source;
  }

  /** @param {string} question @param {import("../../storage.js").StoredMessage[]} history @param {(status: string) => void} onStatus */
  async begin(question, history, onStatus) {
    const refs = new AskRefs();
    this.refs = refs;
    const context = await loadSessionContext(
      history,
      () => onStatus("Indexing context…"),
    );
    if (context.api) {
      const record = selectApiRecord(context.api.records, question);
      if (record !== undefined) {
        context.api.arguments = {
          ...context.api.arguments,
          ...recordArguments(record, context.api.operation.response.type),
        };
      }
    }
    onStatus("Searching tools…");
    const evidence = await collectEvidence({
      question,
      context,
      refs,
      onStatus,
    });
    this.question = question;
    this.evidence = evidence;
    this.executor = new CapabilityExecutor({
      question,
      evidence,
      refs,
      source: this.source,
    });
  }

  question = "";
  /** @type {Awaited<ReturnType<typeof collectEvidence>> | undefined} */
  evidence;
  /** @type {CapabilityExecutor | undefined} */
  executor;
  /** @type {AskRefs | undefined} */
  refs;

  routeTools() {
    if (!this.evidence) throw new Error("Tool session is not ready");
    return routeTools(this.evidence);
  }

  /** @param {unknown} subject */
  continuesKnowledge(subject) {
    const title = this.evidence?.context.knowledge?.title;
    if (typeof subject !== "string" || !title) return false;
    return Math.max(
      relevance(subject, title),
      relevance(title, subject),
    ) >= 10;
  }

  /** @param {unknown} explicitSubject @param {unknown} topic */
  continuesGeneral(explicitSubject, topic) {
    if (!this.evidence?.context.knowledge) return false;
    if (this.continuesKnowledge(topic)) return true;
    if (typeof explicitSubject === "string") {
      return !explicitSubject.trim() ||
        this.continuesKnowledge(explicitSubject);
    }
    return false;
  }

  routeMessages() {
    if (!this.evidence) throw new Error("Tool session is not ready");
    const { context, apiOptions } = this.evidence;
    /** @type {import("../../model.js").ChatMessage[]} */
    const messages = [
      {
        role: /** @type {const} */ ("system"),
        content: ROUTE_INSTRUCTION,
      },
    ];
    if (context.knowledge?.description) {
      messages.push({
        role: /** @type {const} */ ("assistant"),
        content: context.knowledge.description,
      });
    }
    messages.push(
      {
        role: /** @type {const} */ ("user"),
        content: JSON.stringify({
          request: this.question,
          context: {
            activeCapability: context.chart
              ? "chart"
              : context.api
                ? "api"
                : context.metrics.length
                  ? "metric"
                  : context.source.length
                    ? "source"
                    : context.knowledge
                      ? "general"
                    : undefined,
            ...(context.chart
              ? {
                  activeChart: {
                    title: context.chart.chart.title,
                    series: context.chart.chart.series.map(
                      (/** @type {any} */ { path, label }) => ({ path, label }),
                    ),
                  },
                }
              : {}),
            ...(context.metrics.length
              ? {
                  metrics: context.metrics.map(
                    (/** @type {any} */ metric) => ({
                      name: metric.name,
                      path: metric.path,
                    }),
                  ),
                }
              : {}),
            ...(context.recentMetrics.length
              ? {
                  recentMetrics: context.recentMetrics.map(
                    (/** @type {any} */ metric) => ({
                      name: metric.name,
                      path: metric.path,
                    }),
                  ),
                }
              : {}),
            ...(context.api
              ? {
                  api: {
                    operation: context.api.operation.label,
                    arguments: context.api.arguments,
                  },
                }
              : {}),
            ...(context.source.length
              ? {
                  source: context.source.map(
                    (/** @type {any} */ source) => source.path,
                  ),
                  sourceSubjects: context.knowledge?.subjects ?? [],
                }
              : {}),
            ...(context.knowledge
              ? {
                  concept: {
                    title: context.knowledge.title,
                    subjects: context.knowledge.subjects ?? [],
                  },
                }
              : {}),
          },
          apiMatches: apiOptions.map(({ label, operation }) => ({
            label,
            required: operation.parameters
              .filter((/** @type {any} */ parameter) => parameter.required)
              .map((/** @type {any} */ parameter) => parameter.description),
          })),
        }),
      },
    );
    return messages;
  }

  directRoute() {
    if (!this.evidence) return undefined;
    if (this.evidence.variantMiss) {
      const metric = this.evidence.context.metrics[0];
      return {
        action: "clarify",
        call: {
          name: "clarify",
          arguments: {
            question: `I could not find a matching variant of ${
              normalize(metric?.name ?? "the active metric").replaceAll("_", " ")
            }. Which available cohort or variant should I use?`,
          },
        },
      };
    }
    let action = directAction(this.evidence, this.question);
    if (
      action === "explain_metric_calculation" &&
      !this.evidence.metricOptions.some(
        (/** @type {any} */ { origin }) => origin === "mentioned",
      ) &&
      this.evidence.apiOptions.some(({ operation }) =>
        operation.parameters.some((parameter) => parameter.required) &&
        hasRequiredArguments(
          operation,
          explicitArguments(operation, this.question),
        )
      )
    ) {
      action = undefined;
    }
    if (action === "search_source") {
      if (
        this.evidence.context.source.length &&
        !this.evidence.context.knowledge?.subjects?.length
      ) {
        return undefined;
      }
      return {
        action,
        call: {
          name: action,
          arguments: { query: this.question },
        },
      };
    }
    if (action) {
      return {
        action,
        call: this.directCall(action),
      };
    }
    const at = explicitPosition(this.question);
    const metricAt = at
      ? capabilityMetrics(this.evidence, "read_metric_at")
      : [];
    if (at && metricAt.length === 1) {
      return {
        action: "read_metric_at",
        call: {
          name: "read_metric_at",
          arguments: {
            refs: [metricAt[0].ref],
            at,
          },
        },
      };
    }
    const supplied = this.evidence.apiOptions
      .map(({ ref, operation }) => ({
        ref,
        operation,
        arguments: explicitArguments(operation, this.question),
      }))
      .filter(({ operation, arguments: arguments_ }) =>
        hasRequiredArguments(operation, arguments_) &&
        (
          operation.parameters.some((parameter) => parameter.required) ||
          Number(operation.titleMatchedTerms ?? 0) >= 2
        )
      )
      .sort(({ operation: left }, { operation: right }) =>
        Number(right.titleMatchedTerms ?? 0) -
          Number(left.titleMatchedTerms ?? 0) ||
        right.response.fields.length - left.response.fields.length ||
        Number(right.specificity ?? 0) - Number(left.specificity ?? 0) ||
        Number(right.score ?? 0) - Number(left.score ?? 0)
      );
    const priorArguments = new Set(
      Object.values(this.evidence.context.api?.arguments ?? {}).map(String),
    );
    const newResource = !this.evidence.context.api ||
      supplied.some(({ arguments: arguments_ }) =>
        Object.values(arguments_).some((value) =>
          !priorArguments.has(String(value))
        )
      );
    if (newResource && supplied[0]) {
      return {
        action: "call_api",
        call: {
          name: "call_api",
          arguments: { ref: supplied[0].ref },
        },
      };
    }
    if (
      this.evidence.context.metrics.length ||
      this.evidence.context.chart
    ) {
      return undefined;
    }

    const questionTokens = schemaTokens([this.question]);
    if (
      !this.evidence.context.api &&
      !this.evidence.metricOptions.some(
        (/** @type {any} */ { origin }) => origin === "mentioned",
      ) &&
      [...questionTokens].some((token) => RETRIEVAL_TERMS.has(token))
    ) {
      const records = this.evidence.apiOptions
        .map(({ ref, operation }) => ({
          ref,
          operation,
          score: overlapCount(
            questionTokens,
            schemaTokens([operation.label]),
          ),
        }))
        .filter(({ score }) => score > 0)
        .sort((left, right) =>
          right.score - left.score ||
          right.operation.response.fields.length -
            left.operation.response.fields.length
        );
      if (
        records[0] &&
        records[0].score > (records[1]?.score ?? 0)
      ) {
        return {
          action: "call_api",
          call: {
            name: "call_api",
            arguments: { ref: records[0].ref },
          },
        };
      }
    }

    const query = questionTokens;
    const apiContext = this.evidence.context.api;
    const contextKey = apiContext?.operation.key;
    const contextOperation = apiContext?.operation;
    const currentFieldNames = schemaTokens(
      contextOperation?.response.fields.map(
        (/** @type {any} */ field) => field.name,
      ) ?? [],
    );
    const detail = contextOperation?.response.type.endsWith("[]") &&
        semanticOverlapCount(query, currentFieldNames) === 0
      ? this.evidence.apiOptions
        .filter(({ operation }) =>
          operation.key !== contextKey &&
          !operation.response.type.endsWith("[]") &&
          operation.response.fields.length >
            contextOperation.response.fields.length &&
          Boolean(reusableArguments(operation, apiContext))
        )
        .sort(({ operation: left }, { operation: right }) =>
          Number(right.titleMatchedTerms ?? 0) -
            Number(left.titleMatchedTerms ?? 0) ||
          Number(right.specificity ?? 0) - Number(left.specificity ?? 0) ||
          right.response.fields.length - left.response.fields.length ||
          Number(right.score ?? 0) - Number(left.score ?? 0)
        )[0]
      : undefined;
    if (detail) {
      return {
        action: "call_api",
        call: {
          name: "call_api",
          arguments: { ref: detail.ref },
        },
      };
    }

    const apiMatches = [];
    for (const { ref, operation } of this.evidence.apiOptions) {
      const returned = schemaTokens(
        operation.response.fields.flatMap((/** @type {any} */ field) => [
          field.name,
          field.description,
        ]),
      );
      const fieldMatches = semanticOverlapCount(query, returned);
      const suppliedResource =
        operation.parameters.some((parameter) => parameter.required) &&
        hasRequiredArguments(
          operation,
          explicitArguments(operation, this.question),
        );
      const inheritedResource = reusableArguments(
        operation,
        this.evidence.context.api,
      );
      if (
        fieldMatches > 0 &&
        (suppliedResource || inheritedResource || operation.key === contextKey)
      ) {
        apiMatches.push({
          score: fieldMatches,
          context: operation.key === contextKey,
          action: "call_api",
          call: {
            name: "call_api",
            arguments: { ref },
          },
        });
      }
    }
    apiMatches.sort((left, right) => right.score - left.score);
    const activeMatch = apiMatches.find(({ context }) => context);
    if (activeMatch) return activeMatch;
    if (
      apiMatches[0] &&
      apiMatches[0].score > (apiMatches[1]?.score ?? 0)
    ) {
      return apiMatches[0];
    }

    if (supplied.length) {
      return {
        action: "call_api",
        call: {
          name: "call_api",
          arguments: { ref: supplied[0].ref },
        },
      };
    }
    if (
      this.evidence.context.knowledge &&
      !this.evidence.context.knowledge.subjects?.length &&
      (
        this.evidence.context.source.length ||
        this.evidence.guideOptions.some(
          (/** @type {any} */ { origin }) => origin === "context",
        )
      )
    ) {
      return { action: "answer_general" };
    }
    return undefined;
  }

  /** @param {string} action @param {(status: string) => void} onStatus */
  async prepareAction(action, onStatus) {
    if (
      action !== "explain_metric_calculation" ||
      !this.evidence ||
      !this.refs
    ) return;
    this.evidence.sourceOptions = await collectSourceOptions({
      question: this.question,
      evidence: this.evidence,
      source: this.source,
      refs: this.refs,
      onStatus,
    });
  }

  /** @param {string} action */
  actionTool(action) {
    const evidence = this.evidence;
    if (!evidence) throw new Error("Tool session is not ready");
    if (!availableActions(evidence).includes(action)) {
      throw new Error(`Unavailable AI capability: ${action}`);
    }
    return actionTool(evidence, action);
  }

  /** @param {string} action */
  directCall(action) {
    const evidence = this.evidence;
    if (!evidence) return undefined;

    const metrics = capabilityMetrics(evidence, action);
    if (action === "search_source") {
      return {
        name: action,
        arguments: { query: this.question },
      };
    }
    if (action === "describe_capabilities") {
      return {
        name: action,
        arguments: { capabilities: generalCapabilities() },
      };
    }
    if (action === "show_guide_example") {
      const guide = preferredGuide(evidence, this.question, true);
      return guide
        ? {
            name: action,
            arguments: { refs: [guide.ref] },
          }
        : undefined;
    }
    if (
      action === "add_chart_series" ||
      action === "remove_chart_series" ||
      action === "replace_chart_series"
    ) {
      const activePaths = new Set(
        evidence.context.chart?.chart.series.map(
          (/** @type {{ path: string }} */ { path }) => path,
        ) ?? [],
      );
      const mentioned = evidence.metricOptions.filter(
        ({ metric, origin }) =>
          origin === "mentioned" &&
          (
            action === "add_chart_series"
              ? !activePaths.has(metric.path)
              : activePaths.has(metric.path)
          ),
      );
      if (mentioned.length === 1) {
        return {
          name: action,
          arguments: { refs: [mentioned[0].ref] },
        };
      }
    }
    if (
      (
        action === "read_latest_metric" ||
        action === "select_metric_variant" ||
        action === "build_metric_chart" ||
        action === "add_chart_series" ||
        action === "remove_chart_series" ||
        action === "replace_chart_series"
      ) &&
      metrics.length === 1
    ) {
      return {
        name: action,
        arguments: { refs: [metrics[0].ref] },
      };
    }

    if (action === "list_metric_cohorts_variants") {
      const variants = evidence.metricOptions.filter(
        ({ origin }) => origin === "variant",
      );
      if (variants.length === 1) {
        return {
          name: action,
          arguments: { refs: [variants[0].ref] },
        };
      }
      const bases = evidence.metricOptions.filter(
        ({ origin }) => origin === "mentioned" || origin === "context",
      );
      if (!variants.length && bases.length === 1) {
        return {
          name: action,
          arguments: { refs: [bases[0].ref] },
        };
      }
      return undefined;
    }

    if (action === "set_chart_view_scale") {
      const styles = requestedChartStyles(this.question);
      if (styles.length) {
        return {
          name: action,
          arguments: { styles },
        };
      }
    }

    if (action !== "explain_metric_calculation") return undefined;
    const { context, metricOptions, sourceOptions, guideOptions } = evidence;
    const contextual = context.metrics[0]
      ? metricOptions.find(({ metric }) =>
          metric.path === context.metrics[0].path
        )
      : undefined;
    const mentioned = metricOptions.find(({ origin }) => origin === "mentioned");
    const currentGuide = guideOptions.some(
      ({ origin }) => origin === "current",
    );
    const metric = mentioned ?? (currentGuide ? undefined : contextual);
    const grounding = sourceOptions[0] ?? guideOptions[0];
    if (!grounding) return undefined;
    return {
      name: action,
      arguments: {
        refs: [grounding.ref],
        ...(metric ? { metrics: [metric.ref] } : {}),
      },
    };
  }

  /** @param {boolean} continueContext */
  contextualGeneralAction(continueContext) {
    if (!continueContext || !this.evidence) return undefined;
    if (
      this.evidence.context.source.length &&
      this.evidence.context.knowledge?.subjects?.length
    ) {
      return "search_source";
    }
    return directAction(this.evidence, this.question, true);
  }

  /** @param {string} action @param {boolean} [continueContext] */
  actionMessages(action, continueContext = true) {
    const scopedEvidence = this.evidence;
    if (!scopedEvidence) throw new Error("Tool session is not ready");
    const { context, apiOptions, sourceOptions, guideOptions } =
      scopedEvidence;
    const actionMetrics = capabilityMetrics(scopedEvidence, action);
    /** @type {Record<string, unknown>} */
    const evidence = {
      request: this.question,
    };
    if (
      action === "set_chart_view_scale" ||
      action === "add_chart_series" ||
      action === "remove_chart_series" ||
      action === "replace_chart_series"
    ) {
      evidence.activeChart = context.chart?.chart;
    }
    if ([
      "read_latest_metric",
      "read_metric_at",
      "read_metric_range",
      "build_metric_chart",
      "add_chart_series",
      "remove_chart_series",
      "replace_chart_series",
      "list_metric_cohorts_variants",
      "select_metric_variant",
      "explain_metric_calculation",
    ].includes(action)) {
      evidence.metrics = actionMetrics.map(
        (/** @type {any} */ { ref, label, metric, origin }) => ({
          ref,
          label,
          origin,
          path: metric.path,
          indexes: action.startsWith("read_metric") ||
              action === "read_latest_metric"
            ? metric.indexes
            : undefined,
          unit: metric.suggestedUnit,
        }),
      );
    }
    if (
      action === "explain_metric_calculation" ||
      action === "search_source"
    ) {
      evidence.source = sourceOptions.map(
        (/** @type {any} */ { ref, source }) => ({
        ref,
        path: source.path,
        startLine: source.startLine,
        content: source.content.slice(0, 220),
        }),
      );
      evidence.guides = guideOptions.map(({ ref, guide }) => ({
        ref,
        title: guide.title,
        description: guide.description,
      }));
      if (context.source.length) {
        evidence.previousSource = context.source.map(
          (/** @type {any} */ source) => ({
            path: source.path,
            startLine: source.startLine,
            content: source.content,
          }),
        );
      }
      if (context.knowledge) {
        evidence.previousContext = context.knowledge;
      }
    }
    if (action === "call_api") {
      evidence.api = apiOptions.map(({ ref, operation }) => ({
        ref,
        operation: `${operation.method} ${operation.path}`,
        summary: operation.summary || operation.label,
        description: operation.description,
        parameters: operation.parameters,
        response: {
          type: operation.response.type,
          description: operation.response.description,
          fields: operation.response.fields
            .slice(0, 12)
            .map((/** @type {any} */ { name, type, description }) => ({
              name,
              type,
              description,
            })),
        },
      }));
      if (context.api) {
        evidence.previousApi = {
          operation: context.api.operation.label,
          arguments: context.api.arguments,
        };
      }
    }
    if (
      (action === "answer_general" || action === "find_chart_metrics") &&
      context.knowledge
    ) {
      evidence.context = context.knowledge;
    }
    if (action === "answer_general") {
      if (!context.knowledge || !continueContext) {
        return [
          {
            role: /** @type {const} */ ("system"),
            content: actionInstruction(action),
          },
          {
            role: /** @type {const} */ ("user"),
            content: this.question,
          },
        ];
      }
      const verified = Boolean(
        context.source.length ||
          guideOptions.some(({ origin }) => origin === "context"),
      );
      const verifiedGuideFacts = guideOptions
        .filter(({ origin, guide }) => origin === "context" && guide.description)
        .map(({ guide }) => guide.description);
      return [
        {
          role: /** @type {const} */ ("system"),
          content: `${actionInstruction(action)}
The immediately preceding assistant message is the active conversation context. Its topic is "${
            context.knowledge.title
          }" and it is ${verified ? "verified" : "not verified"}. ${
            verified
              ? "Use only that answer and its direct logical consequences when the new request continues it."
              : "Do not treat it as verified evidence."
          }${
            verifiedGuideFacts.length
              ? `\nVerified supporting facts:\n${
                  verifiedGuideFacts.map((fact) => `- ${fact}`).join("\n")
                }`
              : ""
          }`,
        },
        {
          role: /** @type {const} */ ("assistant"),
          content: context.knowledge.description,
        },
        {
          role: /** @type {const} */ ("user"),
          content: this.question,
        },
      ];
    }
    return [
      {
        role: /** @type {const} */ ("system"),
        content: actionInstruction(action),
      },
      {
        role: /** @type {const} */ ("user"),
        content: JSON.stringify(evidence),
      },
    ];
  }

  /** @param {string} ref */
  apiArgumentTool(ref) {
    if (!this.refs) throw new Error("Tool session is not ready");
    return apiArgumentTool(this.refs.get(ref, "api"));
  }

  /** @param {string} ref */
  apiArgumentMessages(ref) {
    if (!this.refs) throw new Error("Tool session is not ready");
    const operation = this.refs.get(ref, "api");
    return [
      {
        role: /** @type {const} */ ("system"),
        content: "Call provide_api_arguments exactly once. Copy only parameter values explicitly supplied by the newest request or verified API context. Omit everything else.",
      },
      {
        role: /** @type {const} */ ("user"),
        content: JSON.stringify({
          request: this.question,
          operation: {
            method: operation.method,
            path: operation.path,
            parameters: operation.parameters,
          },
          ...(this.evidence?.context.api
            ? {
                previousArguments: this.evidence.context.api.arguments,
              }
            : {}),
        }),
      },
    ];
  }

  /** @param {string} ref */
  apiArguments(ref) {
    if (!this.refs) throw new Error("Tool session is not ready");
    const operation = this.refs.get(ref, "api");
    return {
      ...(reusableArguments(operation, this.evidence?.context.api) ?? {}),
      ...explicitArguments(operation, this.question),
    };
  }

  /** @param {string} ref @param {Record<string, unknown>} arguments_ */
  hasApiArguments(ref, arguments_) {
    if (!this.refs) throw new Error("Tool session is not ready");
    return hasRequiredArguments(this.refs.get(ref, "api"), arguments_);
  }

  /** @param {string} ref @param {Record<string, unknown>} arguments_ */
  validateApiArguments(ref, arguments_) {
    if (!this.refs) throw new Error("Tool session is not ready");
    return validatedArguments(
      this.refs.get(ref, "api"),
      arguments_,
      this.question,
      this.evidence?.context.api,
    );
  }

  /** @param {string} ref @param {Record<string, unknown>} arguments_ */
  missingApiArguments(ref, arguments_) {
    if (!this.refs) throw new Error("Tool session is not ready");
    return this.refs.get(ref, "api").parameters
      .filter((/** @type {any} */ parameter) =>
        parameter.required && !Object.hasOwn(arguments_, parameter.name)
      );
  }

  /** @param {{ name: string, arguments: Record<string, unknown> }} call @param {(status: string) => void} onStatus @param {AbortSignal} signal */
  execute(call, onStatus, signal) {
    if (!this.executor) throw new Error("Tool session is not ready");
    return this.executor.execute(call, onStatus, signal);
  }
}
