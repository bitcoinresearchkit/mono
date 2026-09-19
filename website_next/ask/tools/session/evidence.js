import { searchApi } from "../api/index.js";
import { searchLearn } from "../learn.js";
import {
  mentionedMetricNames,
  metricByName,
  metricVariants,
  searchMetrics,
} from "../metrics/index.js";
import { normalize } from "../text.js";
import {
  explicitArguments,
  hasRequiredArguments,
  reusableArguments,
} from "../api/routing.js";

const MAX_METRICS = 5;
const MAX_API = 6;
const MAX_API_CANDIDATES = 64;
const MAX_SOURCE = 6;
const MAX_GUIDES = 2;
const MAX_SOURCE_SCHEMA_QUERIES = 2;

/** @param {string} value */
function label(value) {
  return value.replaceAll("_", " ");
}

/** @template T @param {T[]} values @param {(value: T) => string} key */
function unique(values, key) {
  const seen = new Set();
  return values.filter((value) => {
    const itemKey = key(value);
    if (seen.has(itemKey)) return false;
    seen.add(itemKey);
    return true;
  });
}

/** @param {unknown} value */
function searchTerms(value) {
  return new Set(
    normalize(value).split(" ").filter((term) => term.length >= 3),
  );
}

/**
 * Turn generated response-schema matches into source-level symbols. A nested
 * response field already tells us both its owning Rust type and its field
 * name, so source lookup can be precise without maintaining API-to-code maps.
 *
 * @param {string} question
 * @param {import("../api/index.js").ApiOperation[]} operations
 */
export function schemaSourceQueries(question, operations) {
  const query = searchTerms(question);
  if (!query.size) return [];

  const documents = operations.flatMap((operation) =>
    operation.response.fields.map((field, index) => ({
      operation,
      field,
      index,
      names: searchTerms(field.name),
      terms: searchTerms(
        `${operation.summary} ${field.name} ${field.ownDescription}`,
      ),
    }))
  );
  const frequency = new Map();
  for (const { terms } of documents) {
    for (const term of terms) {
      frequency.set(term, (frequency.get(term) ?? 0) + 1);
    }
  }
  const candidates = documents.map(
    ({ operation, field, index, names, terms }) => {
      const matched = [...query].filter((term) => terms.has(term));
      const named = matched.filter((term) => names.has(term)).length;
      const parts = field.name.split(".");
      const parentName = parts.slice(0, -1).join(".");
      const parent = parentName
        ? operation.response.fields.find(({ name }) => name === parentName)
        : undefined;
      const owner = parent?.type || operation.response.type;
      const fieldName = parts.at(-1) ?? field.name;
      return {
        query: `${owner} ${fieldName}`,
        matched: matched.length,
        named,
        specificity: matched.reduce((sum, term) =>
          sum + Math.log(
            (documents.length + 1) /
              ((frequency.get(term) ?? documents.length) + 1),
          ) + 1, 0),
        rank: Number(operation.score ?? 0),
        index,
      };
    },
  )
    .filter(({ matched, named }) => matched >= 2 && named > 0)
    .sort((left, right) =>
      right.specificity - left.specificity ||
      right.matched - left.matched ||
      right.rank - left.rank ||
      left.index - right.index
    );
  const strongest = candidates[0]?.specificity ?? 0;
  return unique(
    candidates
      .filter(({ specificity }) => specificity === strongest)
      .map(({ query }) => query),
    (query) => normalize(query),
  ).slice(0, MAX_SOURCE_SCHEMA_QUERIES);
}

/** @param {any} metric */
function acceptsMetric(metric) {
  return Number(metric.matchedTerms ?? 0) >= 2;
}

/** @param {any} operation @param {string} question @param {any} previous */
function acceptsApi(operation, question, previous) {
  const specificity = Number(operation.specificity ?? 0);
  const reusable = Boolean(reusableArguments(operation, previous));
  const required = operation.parameters.some(
    (/** @type {any} */ parameter) => parameter.required,
  );
  if (!required) {
    return Number(operation.titleMatchedTerms ?? 0) > 0 &&
      specificity >= 2.5;
  }
  const supplied = hasRequiredArguments(
    operation,
    explicitArguments(operation, question),
  );
  return supplied || reusable
    ? Number(operation.titleMatchedTerms ?? 0) > 0 ||
      reusable && Number(operation.matchedTerms ?? 0) > 0
    : Number(operation.titleMatchedTerms ?? 0) > 0;
}

/** @param {any} match */
function acceptsSource(match) {
  return Number(match.matched ?? 0) >=
    Math.min(2, Number(match.queryTerms ?? 0));
}

/** @param {any} guide */
function acceptsGuide(guide) {
  return Number(guide.titleCoverage ?? 0) >= 0.75;
}

/** @param {any} metric */
async function sourceMetricSubject(metric) {
  const variants = await metricVariants(metric, "");
  const names = variants?.series.map((/** @type {any} */ series) =>
    normalize(series.name).split(" ")
  ) ?? [];
  if (names.length < 2) return label(metric.name);

  const shortest = Math.min(...names.map((name) => name.length));
  const suffix = [];
  for (let offset = 1; offset <= shortest; offset += 1) {
    const word = names[0].at(-offset);
    if (!word || names.some((name) => name.at(-offset) !== word)) break;
    suffix.unshift(word);
  }
  return suffix.length >= 2 ? suffix.join(" ") : label(metric.name);
}

/**
 * Search generated metric, API, and guide catalogs before asking the model
 * what the request means. Repository excerpts are loaded only if selected.
 *
 * @param {Object} options
 * @param {string} options.question
 * @param {Awaited<ReturnType<typeof import("./context.js").loadSessionContext>>} options.context
 * @param {import("../refs.js").AskRefs} options.refs
 * @param {(status: string) => void} options.onStatus
 */
export async function collectEvidence({
  question,
  context,
  refs,
  onStatus,
}) {
  const [foundMetrics, foundApi, currentGuides, contextGuides, mentionedNames] =
    await Promise.all([
    searchMetrics(
      [question],
      MAX_METRICS,
      [],
      () => onStatus("Indexing metrics…"),
    ),
    searchApi(
      [question],
      MAX_API_CANDIDATES,
      () => onStatus("Indexing API…"),
    ),
    searchLearn(question, MAX_GUIDES),
    context.knowledge?.title
      ? searchLearn(context.knowledge.title, MAX_GUIDES)
      : [],
    mentionedMetricNames(question),
  ]);
  const foundGuides = unique(
    [
      ...currentGuides
        .filter(acceptsGuide)
        .map((guide) => ({ ...guide, origin: "current" })),
      ...contextGuides
        .filter(acceptsGuide)
        .map((guide) => ({ ...guide, origin: "context" })),
    ],
    (guide) => guide.breadcrumbs.join("/"),
  );
  const mentionedMetrics = (await Promise.all(
    mentionedNames.map((name) => metricByName(name)),
  )).filter(Boolean);
  const foundByPath = new Map(
    foundMetrics.map((metric) => [metric.path, metric]),
  );
  const normalizedQuestion = normalize(question);
  const linkedMetrics = mentionedMetrics.map((/** @type {any} */ metric) => {
    const match = foundByPath.get(metric.path);
    const normalizedName = normalize(metric.name);
    const explicit = normalizedQuestion === normalizedName ||
      normalizedName.split(" ").length > 1 ||
      Number(match?.matchedTerms ?? 0) >= 2;
    return {
      ...metric,
      ...(match ?? {}),
      origin: explicit ? "mentioned" : "search",
    };
  });
  const variantResults = await Promise.all(
    context.metrics.map((/** @type {any} */ metric) =>
      metricVariants(metric, question)
    )
  );
  const variantMetrics = variantResults.flatMap((variants) =>
    variants?.series
      .filter((/** @type {any} */ { specificity }) =>
        Number(specificity ?? 0) >= 3
      )
      .map((/** @type {any} */ metric) => {
        const name = label(metric.name);
        const selector = label(metric.selector);
        return {
          ...metric,
          label: name === selector || name.startsWith(`${selector} `)
            ? name
            : `${selector} ${name}`.trim(),
          origin: "variant",
        };
      }) ?? []
  );
  const variantMiss = context.metrics.length > 0 &&
    variantResults.some(Boolean) &&
    variantMetrics.length === 0 &&
    !linkedMetrics.some(({ origin }) => origin === "mentioned") &&
    foundMetrics.some(acceptsMetric);

  const metrics = unique(
    [
      ...linkedMetrics.filter(({ origin }) => origin === "mentioned"),
      ...variantMetrics,
      ...context.metrics.map((/** @type {any} */ metric) => ({
        ...metric,
        origin: "context",
      })),
      ...context.recentMetrics.map((/** @type {any} */ metric) => ({
        ...metric,
        origin: "recent",
      })),
      ...linkedMetrics.filter(({ origin }) => origin !== "mentioned"),
      ...foundMetrics.filter(acceptsMetric),
    ],
    (metric) => metric.path,
  ).slice(0, MAX_METRICS);
  const api = unique(
    [
      ...(context.api ? [context.api.operation] : []),
      ...foundApi
        .filter((operation) => acceptsApi(operation, question, context.api))
        .sort((left, right) =>
          Number(right.titleMatchedTerms ?? 0) -
            Number(left.titleMatchedTerms ?? 0) ||
          Number(Boolean(reusableArguments(right, context.api))) -
            Number(Boolean(reusableArguments(left, context.api))) ||
          right.response.fields.length - left.response.fields.length ||
          Number(right.matchedTerms ?? 0) -
            Number(left.matchedTerms ?? 0) ||
          Number(right.score ?? 0) - Number(left.score ?? 0)
        ),
    ],
    (operation) => operation.key,
  ).slice(0, MAX_API);
  const guides = unique(
    foundGuides,
    (guide) => guide.breadcrumbs.join("/"),
  ).slice(0, MAX_GUIDES);

  const metricOptions = metrics.map((metric) => ({
    ref: refs.issue(
      "metric",
      metric,
      metric.path,
      metric.label ?? label(metric.name),
    ),
    label: metric.label ?? label(metric.name),
    metric,
    origin: metric.origin ?? "search",
  }));
  const apiOptions = api.map((operation) => ({
    ref: refs.issue("api", operation, operation.key, operation.label),
    label: operation.label,
    operation,
  }));
  const sourceOptions = context.source.map((/** @type {any} */ match) => ({
    ref: refs.issue(
      "source",
      match,
      `${match.revision}:${match.path}:${match.startLine}`,
      match.path.split("/").at(-1) ?? match.path,
    ),
    label: `${match.path}:${match.startLine}`,
    source: match,
  }));
  const guideOptions = guides.map((guide) => ({
    ref: refs.issue(
      "guide",
      guide,
      guide.breadcrumbs.join("/"),
      guide.title,
    ),
    label: guide.title,
    guide,
    origin: guide.origin,
  }));

  return {
    metricOptions,
    apiOptions,
    apiCandidates: foundApi,
    sourceOptions,
    guideOptions,
    context,
    variantMiss,
  };
}

/**
 * Source retrieval is intentionally lazy. Routing, API reads, metric values,
 * and chart edits do not need repository excerpts.
 *
 * @param {Object} options
 * @param {string} options.question
 * @param {any} options.evidence
 * @param {import("../source/index.js").AskSource} options.source
 * @param {import("../refs.js").AskRefs} options.refs
 * @param {(status: string) => void} options.onStatus
 */
export async function collectSourceOptions({
  question,
  evidence,
  source,
  refs,
  onStatus,
}) {
  const currentGuide = evidence.guideOptions.some(
    (/** @type {any} */ { origin }) => origin === "current",
  );
  const metricOption = evidence.metricOptions.find(
    (/** @type {any} */ { origin }) => origin === "mentioned",
  ) ?? (currentGuide ? undefined : evidence.metricOptions[0]);
  const metric = metricOption?.metric;
  const metricSubject = metric
    ? await sourceMetricSubject(metric)
    : undefined;
  const schemaQueries = schemaSourceQueries(
    question,
    evidence.apiCandidates ?? [],
  );
  const queries = [
    ...(metricSubject
      ? [{
          query: metricSubject,
          focus: /** @type {const} */ ("definition"),
        }, {
          query: metricSubject,
          focus: /** @type {const} */ ("implementation"),
        }]
      : []),
    ...schemaQueries.map((query) => ({
      query,
      focus: /** @type {const} */ ("implementation"),
    })),
    { query: question, focus: undefined },
  ].filter((value, index, values) =>
    values.findIndex((candidate) =>
      candidate.query === value.query && candidate.focus === value.focus
    ) === index
  );
  onStatus("Searching source…");
  const results = await Promise.all(
    queries.map(({ query, focus }) =>
      source.search(
        query,
        undefined,
        focus,
        ({ loaded, total }) =>
          onStatus(`Indexing source · ${loaded} / ${total}`),
      )
    ),
  );
  const sources = unique(
    [
      ...results.flatMap((result) =>
        result.matches
          .filter(acceptsSource)
          .map((/** @type {any} */ match) => ({
            ...match,
            revision: result.revision,
          }))
      ).sort((left, right) =>
        Number(right.score ?? 0) - Number(left.score ?? 0)
      ),
      ...evidence.context.source,
    ],
    (match) => `${match.revision}:${match.path}:${match.startLine}`,
  ).slice(0, MAX_SOURCE);
  return sources.map((match) => ({
    ref: refs.issue(
      "source",
      match,
      `${match.revision}:${match.path}:${match.startLine}`,
      match.path.split("/").at(-1) ?? match.path,
    ),
    label: `${match.path}:${match.startLine}`,
    source: match,
  }));
}
