import { executeApi } from "../api/execute.js";
import { reusableArguments } from "../api/routing.js";
import { createChartArtifact } from "../chart.js";
import { resolveChartUnit } from "../chart/units.js";
import { readMetric } from "../data.js";
import { metricVariants, searchMetrics } from "../metrics/index.js";
import { renderData, renderEvidence } from "../render.js";
import { normalize } from "../text.js";
import { schemaSourceQueries } from "./evidence.js";

const CHART_VIEWS = new Set(["line", "area", "stacked", "bar", "dots"]);
const CHART_SCALES = new Set(["linear", "log"]);
const SOURCE_USAGE_TERMS = new Set([
  "called",
  "usage",
  "usages",
]);
const SOURCE_DEFINITION_TERMS = new Set([
  "declared",
  "defined",
  "definition",
  "implemented",
  "implementation",
]);

/** @param {unknown} value @param {string} name */
function requiredString(value, name) {
  if (typeof value !== "string" || !value.trim()) {
    throw new Error(`The AI did not provide ${name}`);
  }
  return value.trim();
}

/** @param {unknown} value */
function uniqueRefs(value) {
  if (
    !Array.isArray(value) ||
    !value.length ||
    value.some((ref) => typeof ref !== "string")
  ) {
    throw new Error("The AI did not select verified evidence");
  }
  return [...new Set(value)];
}

/** @param {string} value */
function label(value) {
  return normalize(value);
}

/** @param {string | undefined} value */
function inlineCode(value) {
  if (!value) return [];
  return value
    .split("`")
    .filter((_, index) => index % 2 === 1)
    .map((term) => term.trim())
    .filter(Boolean);
}

/** @param {string | undefined} value */
function sourceSubject(value) {
  const subject = inlineCode(value).find((term) =>
    !term.includes("/") && !term.includes("\\")
  );
  if (!subject) return undefined;
  return subject.match(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/)?.[1] ?? subject;
}

/** @param {string} value */
function querySubject(value) {
  const query = value.trim();
  if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(query)) return query;
  return query.match(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/)?.[1] ??
    query.match(
      /\b(?:[A-Za-z][A-Za-z0-9]*_[A-Za-z0-9_]+|[a-z][A-Za-z0-9]*[A-Z][A-Za-z0-9]*)\b/,
    )?.[0];
}

/** @param {string} value */
function sourceFocus(value) {
  const words = new Set(normalize(value).split(" "));
  if ([...words].some((word) => SOURCE_USAGE_TERMS.has(word))) {
    return /** @type {const} */ ("usage");
  }
  if ([...words].some((word) => SOURCE_DEFINITION_TERMS.has(word))) {
    return /** @type {const} */ ("definition");
  }
  return undefined;
}

/** @param {string} content @param {string} field */
function computesField(content, field) {
  const code = content
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .split("\n")
    .map((line) => line.split("//")[0])
    .join("\n");
  return normalize(code).includes(field) &&
    ["+=", "-=", "*=", "/=", " + ", " - ", " * ", " / "]
      .some((operator) => code.includes(operator));
}

/** @param {Awaited<ReturnType<import("../source/index.js").AskSource["search"]>>} result @param {string} query */
function rankedSchemaMatches(result, query) {
  const parts = query.trim().split(/\s+/);
  const field = normalize(parts.at(-1) ?? "");
  const owner = normalize(parts.slice(0, -1).join(" "));
  const terms = normalize(query).split(" ").filter(Boolean);
  return result.matches.filter((match) =>
    normalize(match.content).includes(field)
  ).sort((left, right) => {
    const leftContent = normalize(left.content);
    const rightContent = normalize(right.content);
    const leftOwner = owner && leftContent.includes(owner) ? 1 : 0;
    const rightOwner = owner && rightContent.includes(owner) ? 1 : 0;
    const leftComputes = computesField(left.content, field) ? 1 : 0;
    const rightComputes = computesField(right.content, field) ? 1 : 0;
    const leftPath = terms.filter((term) =>
      normalize(left.path).split(" ").includes(term)
    ).length;
    const rightPath = terms.filter((term) =>
      normalize(right.path).split(" ").includes(term)
    ).length;
    return rightComputes - leftComputes ||
      rightOwner - leftOwner ||
      rightPath - leftPath ||
      Number(right.score ?? 0) - Number(left.score ?? 0);
  });
}

/** @param {string} value */
function regexEscape(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** @param {string} content @param {string} subject */
function containsSymbol(content, subject) {
  return new RegExp(`\\b${regexEscape(subject)}\\b`).test(content);
}

/** @param {string} content @param {string} subject */
function declaresSymbol(content, subject) {
  return new RegExp(
    `\\b(?:fn|function|class|struct|enum|trait|interface)\\s+${
      regexEscape(subject)
    }\\b`,
  ).test(content);
}

/** @param {{ content: string, startLine: number }} source @param {string} subject */
function declarationLine(source, subject) {
  const declaration = new RegExp(
    `\\b(?:fn|function|class|struct|enum|trait|interface)\\s+${
      regexEscape(subject)
    }\\b`,
  );
  const index = source.content.split("\n").findIndex((line) =>
    declaration.test(line)
  );
  return index < 0 ? source.startLine : source.startLine + index;
}

/** @param {any[]} excerpts @param {string} subject */
function usageCallers(excerpts, subject) {
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(subject)) return [];
  const call = new RegExp(`\\b${regexEscape(subject)}\\s*\\(`);
  const declaration = /\b(?:fn|function|def)\s+([A-Za-z_][A-Za-z0-9_]*)/;
  const callers = [];
  for (const excerpt of excerpts) {
    let owner;
    let ownerType;
    for (const line of excerpt.content.split("\n")) {
      const implemented = line.match(
        /\bimpl(?:<[^>]*>)?\s+(?:[A-Za-z_][A-Za-z0-9_:<>]*\s+for\s+)?([A-Za-z_][A-Za-z0-9_]*)/,
      )?.[1];
      if (implemented) ownerType = implemented;
      const declarationMatch = line.match(declaration);
      const declared = declarationMatch?.[1];
      if (declared) {
        const indented = /^\s/.test(line);
        const module = excerpt.path.split("/").at(-1)?.replace(/\.[^.]+$/, "");
        owner = ownerType
          ? `${ownerType}::${declared}`
          : indented && module
            ? `${module}::${declared}`
            : declared;
      }
      if (
        !owner ||
        owner === subject ||
        declared ||
        !call.test(line.split("//")[0])
      ) continue;
      callers.push({ name: owner, source: excerpt });
    }
  }
  return [...new Map(
    callers.map((caller) => [
      `${caller.source.path}:${caller.name}`,
      caller,
    ]),
  ).values()];
}

export class CapabilityExecutor {
  /**
   * @param {Object} options
   * @param {string} options.question
   * @param {any} options.evidence
   * @param {import("../refs.js").AskRefs} options.refs
   * @param {import("../source/index.js").AskSource} options.source
   */
  constructor({ question, evidence, refs, source }) {
    this.question = question;
    this.evidence = evidence;
    this.refs = refs;
    this.source = source;
  }

  /** @param {unknown} value @param {"metric" | "source" | "guide"} kind */
  selected(value, kind) {
    return uniqueRefs(value).map((ref) => ({
      ref,
      value: this.refs.get(ref, kind),
    }));
  }

  /** @param {Record<string, unknown>} arguments_ */
  answerGeneral(arguments_) {
    const answer = requiredString(arguments_.answer, "an answer");
    const topic = requiredString(arguments_.topic, "an answer topic");
    return {
      done: true,
      output: answer,
      sourceContext: this.evidence.context.source,
      knowledgeContext: {
        title: topic.slice(0, 160),
        description: answer,
      },
    };
  }

  /** @param {Record<string, unknown>} arguments_ */
  guideExample(arguments_) {
    const [{ value: guide }] = this.selected(arguments_.refs, "guide");
    const example = requiredString(guide.example, "a guide example");
    return {
      done: true,
      output: example,
      sourceContext: this.evidence.context.source,
      knowledgeContext: {
        title: guide.title,
        description: example,
      },
    };
  }

  /** @param {Record<string, unknown>} arguments_ */
  clarify(arguments_) {
    return {
      done: true,
      output: requiredString(arguments_.question, "a clarification question"),
    };
  }

  /** @param {Record<string, unknown>} arguments_ */
  describeCapabilities(arguments_) {
    const capabilities = Array.isArray(arguments_.capabilities)
      ? arguments_.capabilities.filter((value) => typeof value === "string")
      : [];
    if (!capabilities.length) {
      throw new Error("The assistant capabilities are unavailable");
    }
    return {
      done: true,
      output: `I can:\n\n${capabilities.map((value) => `- ${value}`).join("\n")}`,
    };
  }

  /** @param {Record<string, unknown>} arguments_ */
  async explain(arguments_) {
    const selected = [
      ...uniqueRefs(arguments_.refs),
      ...(Array.isArray(arguments_.metrics)
        ? uniqueRefs(arguments_.metrics)
        : []),
    ];
    const sources = [];
    const metrics = [];
    const guides = [];

    for (const ref of selected) {
      const kind = this.refs.kind(ref);
      if (kind === "source") sources.push(this.refs.get(ref, "source"));
      else if (kind === "metric") metrics.push(this.refs.get(ref, "metric"));
      else if (kind === "guide") guides.push(this.refs.get(ref, "guide"));
      else throw new Error(`${ref} cannot ground an explanation`);
    }

    return {
      done: true,
      metricPaths: metrics.map((metric) => metric.path),
      sourceContext: sources,
      grounding: {
        question: this.question,
        metrics: metrics.map((metric) => ({
          name: label(metric.name),
          path: metric.path,
          unit: metric.suggestedUnit,
        })),
        facts: guides.map((guide) => guide.description).filter(Boolean),
        excerpts: sources,
      },
    };
  }

  /** @param {Record<string, unknown>} arguments_ @param {(status: string) => void} onStatus */
  async searchSource(arguments_, onStatus) {
    const query = requiredString(arguments_.query, "a source search query");
    const requestedFocus = sourceFocus(this.question);
    const structuredSubjects =
      this.evidence.context.knowledge?.subjects ?? [];
    const candidateSubject = querySubject(query);
    const explicitCodeSubject = candidateSubject &&
      (
        inlineCode(this.question).includes(candidateSubject) ||
        this.question.includes(`${candidateSubject}(`)
      );
    const exactSubject = candidateSubject &&
        (
          !structuredSubjects.length ||
          structuredSubjects.includes(candidateSubject) ||
          explicitCodeSubject
        )
      ? candidateSubject
      : undefined;
    const schemaQueries = schemaSourceQueries(
      this.question,
      this.evidence.apiCandidates ?? [],
    );
    const subject = exactSubject ??
      structuredSubjects[0] ??
      sourceSubject(this.evidence.context.knowledge?.description);
    const context = this.evidence.context.source;
    const subjectContext = subject
      ? context.filter((/** @type {{ content: string }} */ source) =>
          source.content.includes(subject)
        )
      : [];
    const scopedContext = subjectContext.length ? subjectContext : context;
    const paths = [...new Set(scopedContext.map(
      (/** @type {{ path: string }} */ { path }) => path,
    ))].slice(0, 2);
    const contextualQuery = subject
      ? `${query} ${subject}`
      : query;
    const searches = [
      { query, path: undefined, focus: requestedFocus },
      ...schemaQueries.map((schemaQuery) => ({
        query: schemaQuery,
        path: undefined,
        focus: /** @type {const} */ ("implementation"),
      })),
      ...(contextualQuery === query
        ? []
        : [{
            query: contextualQuery,
            path: undefined,
            focus: requestedFocus,
          }]),
      ...(subject
        ? [{
            query: subject,
            path: undefined,
            focus: requestedFocus ?? /** @type {const} */ ("implementation"),
          }]
        : []),
      ...paths.map((path) => ({
        query: contextualQuery,
        path,
        focus: requestedFocus,
      })),
      ...(subject
        ? [{
            query: subject,
            path: undefined,
            focus: /** @type {const} */ ("usage"),
          }]
        : []),
    ];

    onStatus("Searching source…");
    const results = await Promise.all(
      searches.map(({ query: scopedQuery, path, focus }) =>
        this.source.search(
          scopedQuery,
          path,
          focus,
          ({ loaded, total }) =>
            onStatus(`Indexing source · ${loaded} / ${total}`),
        )
      ),
    );
    const scopedResults = results.filter((_, index) => searches[index].path);
    const schemaResults = results.slice(1, 1 + schemaQueries.length);
    const contextualIndex = 1 + schemaQueries.length;
    const contextualResult = contextualQuery === query
      ? undefined
      : results[contextualIndex];
    const subjectResult = subject
      ? results[
        contextualIndex + (contextualQuery === query ? 0 : 1)
      ]
      : undefined;
    const usageResult = subject ? results.at(-1) : undefined;
    const rawResult = results[0];
    const definition = requestedFocus === "definition" && exactSubject
      ? subjectResult?.matches.find((/** @type {any} */ match) =>
          declaresSymbol(match.content, exactSubject)
        )
      : undefined;
    if (definition && exactSubject && subjectResult) {
      const source = {
        ...definition,
        revision: subjectResult.revision,
      };
      const line = declarationLine(source, exactSubject);
      const description = `\`${exactSubject}\` is defined in \`${source.path}\` at line ${line}.`;
      return {
        done: true,
        output: renderEvidence({
          facts: [description],
          sources: [source],
          excerpts: [],
        }),
        sourceContext: [source],
        knowledgeContext: {
          title: exactSubject,
          description,
          subjects: [exactSubject],
        },
      };
    }
    const seeded = this.evidence.sourceOptions.map(
      (/** @type {{ source: any }} */ { source }) => source,
    );
    const schemaMatches = schemaResults.flatMap((result, index) =>
      rankedSchemaMatches(result, schemaQueries[index]).slice(0, 2)
        .map((/** @type {any} */ match) => ({
          ...match,
          revision: result.revision,
        }))
    );
    if (schemaQueries.length && !schemaMatches.length) {
      return {
        done: true,
        output: "I could not find enough verified source evidence to answer that.",
      };
    }
    const candidates = [...new Map([
      ...subjectContext,
      ...(paths.length ? [] : seeded),
      ...schemaMatches,
      ...(subjectResult?.matches.slice(0, 3).map(
        (/** @type {any} */ match) => ({
          ...match,
          revision: subjectResult.revision,
        }),
      ) ?? []),
      ...(usageResult?.matches.slice(0, 6).map(
        (/** @type {any} */ match) => ({
          ...match,
          revision: usageResult.revision,
        }),
      ) ?? []),
      ...scopedResults.flatMap((result) =>
        result.matches.slice(0, 1).map((/** @type {any} */ match) => ({
          ...match,
          revision: result.revision,
        }))
      ),
      ...(contextualResult?.matches.slice(0, 3).map(
        (/** @type {any} */ match) => ({
          ...match,
          revision: contextualResult.revision,
        }),
      ) ?? []),
      ...(paths.length ? seeded : []),
      ...rawResult.matches.slice(0, 3).map((/** @type {any} */ match) => ({
        ...match,
        revision: rawResult.revision,
      })),
    ].map((excerpt) => [
      `${excerpt.revision}:${excerpt.path}:${excerpt.startLine}`,
      excerpt,
    ])).values()];
    const contextualExplanation = subjectContext.length &&
      requestedFocus !== "usage";
    const excerpts = (contextualExplanation
      ? subjectContext
      : exactSubject
        ? candidates.filter(({ content }) =>
            containsSymbol(content, exactSubject)
          )
        : candidates).slice(0, 3);
    const callers = subject && usageResult
      ? usageCallers(
          usageResult.matches.map((/** @type {any} */ match) => ({
            ...match,
            revision: usageResult.revision,
          })),
          subject,
        )
      : [];
    if (requestedFocus === "usage" && subject && callers.length) {
      const sources = [...new Map(
        callers.map(({ source }) => [
          `${source.revision}:${source.path}:${source.startLine}`,
          source,
        ]),
      ).values()].slice(0, 3);
      const names = callers.map(({ name }) => name);
      const description = `\`${subject}\` is called by ${
        names.map((name) => `\`${name}\``).join(", ")
      }.`;
      return {
        done: true,
        output: renderEvidence({
          facts: [description],
          sources,
          excerpts: [],
        }),
        sourceContext: sources,
        knowledgeContext: {
          title: subject,
          description,
          subjects: names.map((name) => name.split("::").at(-1) ?? name),
        },
      };
    }
    if (!excerpts.length) {
      return {
        done: true,
        output: "I could not find enough verified source evidence to answer that.",
      };
    }
    return {
      done: true,
      sourceContext: excerpts,
      grounding: {
        question: this.question,
        metrics: [],
        facts: [],
        contextFacts: [
          ...(this.evidence.context.knowledge?.description
            ? [this.evidence.context.knowledge.description]
            : []),
          ...(callers.length
            ? [
              `Verified functions that call \`${subject}\`: ${
                callers.map(({ name }) => `\`${name}\``).join(", ")
              }.`,
            ]
            : []),
        ],
        excerpts,
        ...(subject ? { subjects: [subject] } : {}),
      },
    };
  }

  /** @param {Record<string, unknown>} arguments_ */
  async listVariants(arguments_) {
    const [{ value: metric }] = this.selected(arguments_.refs, "metric");
    if (metric.origin === "variant") {
      return this.selectVariant(arguments_);
    }
    const variants = await metricVariants(metric, this.question);
    if (!variants) {
      return {
        done: true,
        output: `I found one **${label(metric.name)}** series and no cohort variants.`,
        metricPaths: [metric.path],
      };
    }
    const groups = variants.groups
      .map((group) =>
        group.examples.length === 1 && group.examples[0] === group.family
          ? label(group.family)
          : `${label(group.family)}: ${group.examples.map(label).join(", ")}`
      )
      .join("; ");
    return {
      done: true,
      output: `**${label(metric.name)}** has ${variants.totalSeries} available series. Variant groups: ${groups}.`,
      metricPaths: [metric.path],
    };
  }

  /** @param {Record<string, unknown>} arguments_ @param {(status: string) => void} onStatus */
  async findChartMetrics(arguments_, onStatus) {
    const query = requiredString(arguments_.query, "a metric search query");
    const context = this.evidence.context.knowledge;
    const queries = [...new Set([
      query,
      this.question,
      context?.title,
      context?.description,
    ].filter((value) => typeof value === "string" && value.trim()))];
    onStatus("Searching metrics…");
    const metrics = (await searchMetrics(queries, 12))
      .filter(({ matchedTerms }) => Number(matchedTerms ?? 0) > 0)
      .filter((metric, index, values) =>
        values.findIndex(({ name }) => name === metric.name) === index
      )
      .slice(0, 6);
    if (!metrics.length) {
      return {
        done: true,
        output: "I could not find a chart metric matching that description.",
      };
    }
    return {
      done: true,
      output: `I found these chart metrics:\n\n${
        metrics.map((metric) =>
          `- **${label(metric.name)}**${
            metric.suggestedUnit ? ` · ${metric.suggestedUnit}` : ""
          }`
        ).join("\n")
      }`,
      metricPaths: metrics.map(({ path }) => path),
    };
  }

  /** @param {Record<string, unknown>} arguments_ */
  selectVariant(arguments_) {
    const [{ value: metric }] = this.selected(arguments_.refs, "metric");
    return {
      done: true,
      output: `Selected **${metric.label ?? label(metric.name)}**.`,
      metricPaths: [metric.path],
    };
  }

  /** @param {Record<string, unknown>} arguments_ @param {"latest" | "at" | "range"} mode */
  async read(arguments_, mode) {
    const selected = this.selected(arguments_.refs, "metric");
    if (mode === "at") requiredString(arguments_.at, "a block height or date");
    const results = await Promise.all(
      selected.map(({ value: metric }) =>
        readMetric(metric, { ...arguments_, mode })
      ),
    );
    return {
      done: true,
      output: renderData(results),
      metricPaths: selected.map(({ value }) => value.path),
    };
  }

  /** @param {any[]} chosen @param {any} [existing] @param {"add" | "remove" | "replace"} [operation] */
  chart(chosen, existing, operation) {
    const prior = /** @type {any[]} */ (existing?.chart.series ?? []);
    const added = chosen.map((metric) => ({
      path: metric.path,
      label: metric.label ?? label(metric.name),
    }));
    const selectedPaths = new Set(added.map(({ path }) => path));
    const series = !existing || operation === "replace"
      ? added
      : operation === "remove"
        ? prior.filter(({ path }) => !selectedPaths.has(path))
        : [
            ...prior,
            ...added.filter(({ path }) =>
              !prior.some((item) => item.path === path)
            ),
          ];
    if (!series.length) throw new Error("A chart needs at least one series");

    const { unit, conflicts } = resolveChartUnit(
      chosen,
      existing?.chart.unit,
      operation ?? "replace",
    );
    if (conflicts.length) {
      return {
        done: true,
        output: `Those metrics use different units (${conflicts.map((value) => value === "number" ? "**unitless**" : `**${value.toUpperCase()}**`).join(" and ")}), so they need separate charts. Which one should I chart?`,
        artifacts: [],
      };
    }
    const artifact = createChartArtifact({
      title: series.map((item) => item.label).join(" and "),
      unit,
      view: existing?.chart.view,
      scale: existing?.chart.scale,
      series,
    });
    const known = new Map(
      [...this.evidence.context.metrics, ...chosen]
        .map((metric) => [metric.path, metric]),
    );
    return {
      done: true,
      output: `${existing ? "Updated" : "Built"} **${artifact.chart.title}** with ${artifact.chart.series.map((item) => item.label).join(", ")}.`,
      artifacts: [artifact],
      metricPaths: [...new Set([
        ...artifact.chart.series.map(({ path }) => path),
        ...(operation === "remove" || operation === "replace"
          ? chosen.map(({ path }) => path)
          : []),
      ])].filter((path) => known.has(path)),
    };
  }

  /** @param {Record<string, unknown>} arguments_ */
  buildChart(arguments_) {
    const selected = this.selected(arguments_.refs, "metric")
      .map(({ value }) => value);
    const chosen = [...new Map([
      ...(arguments_.includeContext === true
        ? this.evidence.context.metrics
        : []),
      ...selected,
    ].map((metric) => [metric.path, metric])).values()];
    return this.chart(chosen);
  }

  /**
   * @param {Record<string, unknown>} arguments_
   * @param {"add" | "remove" | "replace"} operation
   */
  editChart(arguments_, operation) {
    const active = this.evidence.context.chart;
    if (!active) throw new Error("There is no active chart to update");
    const chosen = this.selected(arguments_.refs, "metric")
      .map(({ value }) => value);
    return this.chart(chosen, active, operation);
  }

  /** @param {Record<string, unknown>} arguments_ */
  styleChart(arguments_) {
    const active = this.evidence.context.chart;
    if (!active) throw new Error("There is no active chart to update");
    const styles = Array.isArray(arguments_.styles)
      ? arguments_.styles.filter((style) => typeof style === "string")
      : [];
    const view = styles.find((style) => CHART_VIEWS.has(style));
    const scale = styles.find((style) => CHART_SCALES.has(style));
    if (!view && !scale) throw new Error("The AI did not select a chart style");
    const artifact = createChartArtifact({
      title: active.chart.title,
      unit: active.chart.unit,
      view: view ?? active.chart.view,
      scale: scale ?? active.chart.scale,
      series: active.chart.series,
    });
    return {
      done: true,
      output: `Updated **${artifact.chart.title}** to use ${[
        view ? `${view} view` : "",
        scale ? `${scale} scale` : "",
      ].filter(Boolean).join(" and ")}.`,
      artifacts: [artifact],
      metricPaths: artifact.chart.series.map(({ path }) => path),
    };
  }

  /**
   * @param {Record<string, unknown>} arguments_
   * @param {(status: string) => void} onStatus
   * @param {AbortSignal} signal
   */
  async callApi(arguments_, onStatus, signal) {
    const ref = requiredString(arguments_.ref, "an API reference");
    const operation = this.refs.get(ref, "api");
    const supplied = arguments_.arguments &&
        typeof arguments_.arguments === "object" &&
        !Array.isArray(arguments_.arguments)
      ? /** @type {Record<string, unknown>} */ (arguments_.arguments)
      : {};
    const reused = reusableArguments(operation, this.evidence.context.api);
    onStatus("Reading API…");
    const result = await executeApi(
      operation,
      { ...(reused ?? {}), ...supplied },
      signal,
    );
    return {
      done: true,
      apiContext: {
        key: operation.key,
        arguments: result.arguments,
      },
      apiGrounding: {
        question: this.question,
        previousFields:
          this.evidence.context.api?.operation.key === operation.key
            ? this.evidence.context.api.fields ?? []
            : [],
        previousArguments: this.evidence.context.api?.arguments ?? {},
        previousRecords:
          this.evidence.context.api?.operation.key === operation.key
            ? this.evidence.context.api.records ?? []
            : [],
        ...result,
      },
    };
  }

  /**
   * @param {{ name: string, arguments: Record<string, unknown> }} call
   * @param {(status: string) => void} onStatus
   * @param {AbortSignal} signal
   */
  async execute(call, onStatus, signal) {
    signal.throwIfAborted();
    if (call.name === "answer_general") {
      return this.answerGeneral(call.arguments);
    }
    if (call.name === "show_guide_example") {
      return this.guideExample(call.arguments);
    }
    if (call.name === "describe_capabilities") {
      return this.describeCapabilities(call.arguments);
    }
    if (call.name === "clarify") return this.clarify(call.arguments);
    if (call.name === "explain_metric_calculation") {
      return await this.explain(call.arguments);
    }
    if (call.name === "search_source") {
      return await this.searchSource(call.arguments, onStatus);
    }
    if (call.name === "list_metric_cohorts_variants") {
      return await this.listVariants(call.arguments);
    }
    if (call.name === "find_chart_metrics") {
      return await this.findChartMetrics(call.arguments, onStatus);
    }
    if (call.name === "select_metric_variant") {
      return this.selectVariant(call.arguments);
    }
    if (call.name === "read_latest_metric") {
      onStatus("Reading data…");
      return await this.read(call.arguments, "latest");
    }
    if (call.name === "read_metric_at") {
      onStatus("Reading data…");
      return await this.read(call.arguments, "at");
    }
    if (call.name === "read_metric_range") {
      onStatus("Reading data…");
      return await this.read(call.arguments, "range");
    }
    if (call.name === "build_metric_chart") {
      onStatus("Building chart…");
      return this.buildChart(call.arguments);
    }
    if (call.name === "add_chart_series") {
      onStatus("Updating chart…");
      return this.editChart(call.arguments, "add");
    }
    if (call.name === "remove_chart_series") {
      onStatus("Updating chart…");
      return this.editChart(call.arguments, "remove");
    }
    if (call.name === "replace_chart_series") {
      onStatus("Updating chart…");
      return this.editChart(call.arguments, "replace");
    }
    if (call.name === "set_chart_view_scale") {
      onStatus("Updating chart…");
      return this.styleChart(call.arguments);
    }
    if (call.name === "call_api") {
      return await this.callApi(call.arguments, onStatus, signal);
    }
    throw new Error(`Unsupported AI capability: ${call.name}`);
  }
}
