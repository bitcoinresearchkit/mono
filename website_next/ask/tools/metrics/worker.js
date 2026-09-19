import { QuickMatch, QuickMatchConfig } from "../../../modules/quickmatch-js/src/index.js";
import { normalize, relevance } from "../text.js";
import { metricsFromSeries } from "./series.js";
import { unitFromType } from "./unit.js";

const SEARCH_CANDIDATES = 1_024;

/** @typedef {{ path: string, name: string, indexes: string[], type: string, document: string }} CatalogMetric */

/** @param {string} value */
function searchable(value) {
  return normalize(value);
}

/** @param {string} value */
function queryVocabulary(value) {
  const words = searchable(value).split(" ").filter(Boolean);
  const vocabulary = new Set(words);
  for (let start = 0; start < words.length; start += 1) {
    for (
      let length = 2;
      length <= 5 && start + length <= words.length;
      length += 1
    ) {
      vocabulary.add(
        words.slice(start, start + length).map((word) => word[0]).join(""),
      );
    }
  }
  return vocabulary;
}

/** @param {number} limit */
function createConfig(limit = SEARCH_CANDIDATES) {
  // The model supplies semantic wording, where unmatched words are usually context rather than typos.
  return new QuickMatchConfig()
    .withLimit(limit)
    .withTrigramBudget(0)
    .withMinScore(2)
    .withSeparators("_- :/.|");
}

/** @param {string} url */
async function buildState(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`Series catalog unavailable (${response.status})`);
  const items = metricsFromSeries(await response.json()).map((metric) => ({
    ...metric,
    document: searchable(`${metric.name} ${metric.path}`),
  }));
  /** @type {Map<string, CatalogMetric>} */
  const byName = new Map();
  /** @type {Map<string, CatalogMetric>} */
  const byPath = new Map();
  const metricNames = [...new Set(items.map((metric) => searchable(metric.name)))];
  /** @type {Map<string, CatalogMetric>} */
  const byDocument = new Map();
  const documentFrequency = new Map();
  for (const metric of items) {
    if (!byName.has(metric.name)) byName.set(metric.name, metric);
    const normalizedName = searchable(metric.name);
    if (!byName.has(normalizedName)) byName.set(normalizedName, metric);
    byPath.set(metric.path, metric);
    if (!byDocument.has(metric.document)) byDocument.set(metric.document, metric);
    for (const token of new Set(metric.document.split(" ").filter(Boolean))) {
      documentFrequency.set(
        token,
        (documentFrequency.get(token) ?? 0) + 1,
      );
    }
  }

  const config = createConfig();
  const matcher = new QuickMatch(items.map(({ document }) => document), config);
  /** @type {Map<string, { matcher: QuickMatch, config: QuickMatchConfig }>} */
  const scoped = new Map();
  return {
    items,
    byName,
    byPath,
    metricNames,
    byDocument,
    documentFrequency,
    matcher,
    config,
    scoped,
  };
}

/** @param {Awaited<ReturnType<typeof buildState>>} index @param {string} query */
function mentions(index, query) {
  const value = ` ${searchable(query)} `;
  const names = index.metricNames
    .filter((name) => name && value.includes(` ${name} `))
    .filter((name, _, matches) =>
      !matches.some((candidate) =>
        candidate !== name &&
        candidate.length > name.length &&
        ` ${candidate} `.includes(` ${name} `)
      )
    )
    .sort((left, right) => right.length - left.length || left.localeCompare(right));
  return names.slice(0, 4);
}

/** @type {Promise<Awaited<ReturnType<typeof buildState>>> | undefined} */
let statePromise;
let stateUrl = "";

/** @param {string} id @param {string} url */
function state(id, url) {
  if (!statePromise || url !== stateUrl) {
    stateUrl = url;
    self.postMessage({ id, status: "progress" });
    const pending = buildState(url);
    statePromise = pending;
    void pending.catch(() => {
      if (statePromise !== pending) return;
      statePromise = undefined;
      stateUrl = "";
    });
  }
  return statePromise;
}

/** @param {CatalogMetric} metric */
function publicMetric(metric) {
  return {
    path: metric.path,
    name: metric.name,
    indexes: metric.indexes,
    type: metric.type,
    suggestedUnit: unitFromType(metric.type),
  };
}

/** @param {Awaited<ReturnType<typeof buildState>>} index @param {string[]} prefixes */
function scopedIndex(index, prefixes) {
  const inScope = (/** @type {CatalogMetric} */ metric) =>
    !prefixes.length || prefixes.some((prefix) =>
      metric.path === prefix || metric.path.startsWith(`${prefix}.`)
    );
  if (!prefixes.length) return { matcher: index.matcher, config: index.config, inScope };

  const key = [...prefixes].sort().join("|");
  let scoped = index.scoped.get(key);
  if (!scoped) {
    const config = createConfig();
    const documents = index.items.filter(inScope).map(({ document }) => document);
    scoped = { matcher: new QuickMatch(documents, config), config };
    index.scoped.set(key, scoped);
  }
  return { ...scoped, inScope };
}

/** @param {Awaited<ReturnType<typeof buildState>>} index @param {string} query @param {number} limit @param {string[]} prefixes */
function searchOne(index, query, limit, prefixes) {
  const scope = scopedIndex(index, prefixes);
  const normalizedQuery = searchable(query);
  const full = scope.matcher.matchesWith(
    normalizedQuery,
    scope.config.withLimit(SEARCH_CANDIDATES),
  );
  const fullRanks = new Map(full.map((document, rank) => [document, rank]));
  const scores = new Map();
  for (const word of [...new Set(normalizedQuery.split(" "))]) {
    if (!word) continue;
    const matches = scope.matcher.matchesWith(
      word,
      scope.config.withLimit(SEARCH_CANDIDATES),
    );
    for (const [rank, document] of matches.entries()) {
      const score = scores.get(document) ?? { matched: 0, ranks: 0 };
      score.matched += 1;
      score.ranks += rank;
      scores.set(document, score);
    }
  }
  const documents = [...new Set([...full, ...scores.keys()])]
    .sort((left, right) => {
      const a = scores.get(left) ?? { matched: 0, ranks: SEARCH_CANDIDATES };
      const b = scores.get(right) ?? { matched: 0, ranks: SEARCH_CANDIDATES };
      return b.matched - a.matched ||
        (fullRanks.get(left) ?? SEARCH_CANDIDATES) -
          (fullRanks.get(right) ?? SEARCH_CANDIDATES) ||
        a.ranks - b.ranks ||
        left.length - right.length ||
        left.localeCompare(right);
    });

  return documents
    .map((document) => index.byDocument.get(document))
    .filter((metric) => metric && scope.inScope(metric))
    .slice(0, limit)
    .map((metric, rank) => {
      const value = /** @type {CatalogMetric} */ (metric);
      const documentTokens = new Set(value.document.split(" "));
      const queryTokens = [...new Set(normalizedQuery.split(" ").filter(Boolean))];
      const matchedTokens = queryTokens.filter((token) =>
        documentTokens.has(token)
      );
      return {
        ...publicMetric(value),
        matchedQuery: query,
        matchedTerms: matchedTokens.length,
        specificity: matchedTokens.reduce((sum, token) => {
          const frequency = index.documentFrequency.get(token) ??
            index.items.length;
          return sum +
            Math.log((index.items.length + 1) / (frequency + 1)) + 1;
        }, 0),
        relevance: relevance(query, value.document),
        score: 1_000 - rank,
      };
    });
}

/** @param {Awaited<ReturnType<typeof buildState>>} index @param {string[]} queries @param {number} limit @param {string[]} prefixes */
function search(index, queries, limit, prefixes) {
  const groups = queries.map((query) => searchOne(index, query, limit, prefixes));
  const output = [];
  const seen = new Set();

  for (let rank = 0; output.length < limit; rank += 1) {
    let added = false;
    for (const group of groups) {
      const metric = group[rank];
      if (!metric || seen.has(metric.path)) continue;
      seen.add(metric.path);
      output.push(metric);
      added = true;
      if (output.length === limit) break;
    }
    if (!added) break;
  }
  return output;
}

/**
 * @param {Awaited<ReturnType<typeof buildState>>} index
 * @param {string} name
 * @param {string} path
 * @param {string} query
 */
function variants(index, name, path, query) {
  const selectedPath = path.split(".");
  /** @type {typeof index.items} */
  let candidates = [];
  for (let length = selectedPath.length - 1; length >= 2; length -= 1) {
    const suffix = selectedPath.slice(-length).join(".");
    const matching = index.items.filter((candidate) =>
      candidate.path === suffix || candidate.path.endsWith(`.${suffix}`)
    );
    if (matching.length > 1) {
      candidates = matching;
      break;
    }
  }
  if (!candidates.length) {
    const suffix = `_${name}`;
    candidates = index.items.filter((candidate) =>
      candidate.name === name || candidate.name.endsWith(suffix)
    );
  }
  if (candidates.length <= 1) return undefined;

  let commonSuffix = candidates[0].path.split(".");
  for (const candidate of candidates.slice(1)) {
    const path = candidate.path.split(".");
    let count = 0;
    while (
      count < commonSuffix.length &&
      count < path.length &&
      commonSuffix[commonSuffix.length - 1 - count] === path[path.length - 1 - count]
    ) count += 1;
    commonSuffix = count ? commonSuffix.slice(-count) : [];
  }

  const selectors = candidates.map((candidate) => {
    const path = candidate.path.split(".");
    return commonSuffix.length ? path.slice(0, -commonSuffix.length) : path;
  });
  let commonPrefix = selectors[0] ?? [];
  for (const selector of selectors.slice(1)) {
    let count = 0;
    while (
      count < commonPrefix.length &&
      count < selector.length &&
      commonPrefix[count] === selector[count]
    ) count += 1;
    commonPrefix = commonPrefix.slice(0, count);
  }

  const preferredPaths = new Map(
    index.matcher.matchesWith(searchable(query), index.config.withLimit(SEARCH_CANDIDATES))
      .map((document, rank) => [index.byDocument.get(document)?.path, rank]),
  );
  const queryTerms = queryVocabulary(query);
  const selectorTokens = selectors.map((selector) =>
    [...new Set(
      searchable(selector.slice(commonPrefix.length).join(" "))
        .split(" ")
        .filter(Boolean),
    )]
  );
  const selectorFrequency = new Map();
  for (const tokens of selectorTokens) {
    for (const token of tokens) {
      selectorFrequency.set(token, (selectorFrequency.get(token) ?? 0) + 1);
    }
  }
  const ranked = candidates
    .map((candidate, candidateIndex) => {
      const matches = selectorTokens[candidateIndex].filter((token) =>
        queryTerms.has(token)
      );
      return {
        ...publicMetric(candidate),
        selector: selectors[candidateIndex],
        rank: preferredPaths.get(candidate.path) ?? SEARCH_CANDIDATES,
        queryMatches: matches.length,
        specificity: matches.reduce((sum, token) =>
          sum +
          Math.log(
            (candidates.length + 1) /
              ((selectorFrequency.get(token) ?? candidates.length) + 1),
          ) + 1, 0),
      };
    })
    .sort((left, right) =>
      right.specificity - left.specificity ||
      right.queryMatches - left.queryMatches ||
      Number(right.name === name) - Number(left.name === name) ||
      left.rank - right.rank ||
      left.path.localeCompare(right.path)
    );

  const groups = new Map();
  for (const { selector } of ranked) {
    const varying = selector.slice(commonPrefix.length);
    const family = varying[0] ?? commonPrefix.at(-1) ?? "root";
    const value = varying.slice(1).join(" / ") || varying[0] || "all";
    const group = groups.get(family) ?? { family, count: 0, examples: [] };
    group.count += 1;
    if (group.examples.length < 5) group.examples.push(value);
    groups.set(family, group);
  }

  return {
    totalSeries: ranked.length,
    groups: [...groups.values()].slice(0, 8),
    series: ranked.slice(0, 16).map((
      {
        path,
        name: metricName,
        suggestedUnit,
        indexes,
        type,
        selector: selectorParts,
        queryMatches,
        specificity,
      },
    ) => {
      const selector = selectorParts
        .slice(commonPrefix.length)
        .join(" ") || selectorParts.at(-1) || "";
      return {
        path,
        name: metricName,
        suggestedUnit,
        indexes,
        type,
        selector,
        matchedTerms: queryMatches,
        specificity,
      };
    }),
  };
}

self.addEventListener("message", async (event) => {
  const { id, type, data } = event.data;
  try {
    const index = await state(id, data.url);
    let result;
    if (type === "prewarm") {
      result = true;
    } else if (type === "search") {
      result = search(index, data.queries, data.limit, data.prefixes);
    } else if (type === "mentions") {
      result = mentions(index, data.query);
    } else if (type === "byName") {
      const metric = index.byName.get(data.name);
      result = metric ? publicMetric(metric) : undefined;
    } else if (type === "byPaths") {
      result = data.paths
        .map((/** @type {string} */ path) => index.byPath.get(path))
        .filter(Boolean)
        .map(publicMetric);
    } else if (type === "variants") {
      result = variants(index, data.name, data.path ?? "", data.query);
    } else {
      throw new Error(`Unknown metric request: ${type}`);
    }
    self.postMessage({ id, status: "complete", data: result });
  } catch (error) {
    self.postMessage({
      id,
      status: "error",
      data: error instanceof Error ? error.message : String(error),
    });
  }
});
