import { QuickMatch, QuickMatchConfig } from "../../../modules/quickmatch-js/src/index.js";
import { normalize, tokenAffinity } from "../text.js";
import { operationsFromOpenApi } from "./openapi.js";

const SEARCH_CANDIDATES = 256;

/**
 * @typedef {import("./index.js").ApiOperation} ApiOperation
 * @typedef {ApiOperation & { document: string, tokens: string[], titleTokens: string[] }} IndexedOperation
 */

/** @param {string} value */
function searchable(value) {
  return normalize(value);
}

/** @param {ApiOperation} operation @returns {IndexedOperation} */
function indexOperation(operation) {
  const { summary, description, parameters, response } = operation;
  const fields = response.fields
    .flatMap((field) => [field.name, field.description])
    .filter(Boolean)
    .join(" ");
  const parameterText = parameters
    .flatMap((parameter) => [
      parameter.name,
      parameter.type,
      parameter.valueType,
      parameter.primitive,
      parameter.description,
      ...(parameter.enum ?? []),
    ])
    .filter(Boolean)
    .join(" ");
  const titleDocument = searchable(
    `${operation.key} ${summary} ${parameters.flatMap((parameter) => [parameter.name, parameter.type]).join(" ")}`,
  );
  const document = searchable(
    `${operation.key} ${summary} ${description} ${parameterText} ${response.type} ${response.description} ${fields}`,
  );
  return {
    ...operation,
    document,
    tokens: [...new Set(document.split(" ").filter(Boolean))],
    titleTokens: [...new Set(titleDocument.split(" ").filter(Boolean))],
  };
}

/** @param {IndexedOperation} operation @returns {ApiOperation} */
function publicOperation(operation) {
  const { document, tokens, titleTokens, ...value } = operation;
  return value;
}

/** @param {string} url */
async function buildState(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`OpenAPI unavailable (${response.status})`);
  const operations = operationsFromOpenApi(await response.json()).map(indexOperation);
  operations.sort((left, right) => left.path.localeCompare(right.path));
  const config = new QuickMatchConfig()
    .withLimit(SEARCH_CANDIDATES)
    .withTrigramBudget(0)
    .withMinScore(2)
    .withSeparators("_- :/.|{}");
  const byDocument = new Map(operations.map((operation) => [operation.document, operation]));
  const byKey = new Map(operations.map((operation) => [operation.key, operation]));
  const documentFrequency = new Map();
  for (const operation of operations) {
    for (const token of operation.tokens) {
      documentFrequency.set(token, (documentFrequency.get(token) ?? 0) + 1);
    }
  }
  const matcher = new QuickMatch(operations.map((operation) => operation.document), config);
  return { operations, config, byDocument, byKey, documentFrequency, matcher };
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

/** @param {Awaited<ReturnType<typeof buildState>>} index @param {string} query @param {number} limit */
function searchOne(index, query, limit) {
  const normalized = searchable(query)
    .split(" ")
    .filter((word) => word.length >= 3 && word.length < 32)
    .join(" ");
  if (!normalized) return [];
  const words = [...new Set(normalized.split(" ").filter(Boolean))];
  const full = index.matcher.matchesWith(
    normalized,
    index.config.withLimit(SEARCH_CANDIDATES),
  );
  const fullRanks = new Map(full.map((document, rank) => [document, rank]));
  const lexical = index.operations
    .map((operation) => {
      const tokens = new Set(operation.tokens);
      const titleTokens = new Set(operation.titleTokens);
      let score = 0;
      let specificity = 0;
      let matched = 0;
      let titleMatched = 0;
      for (const word of words) {
        if (!tokens.has(word)) continue;
        matched += 1;
        const frequency = index.documentFrequency.get(word) ?? index.operations.length;
        const idf = Math.log((index.operations.length + 1) / (frequency + 1)) + 1;
        specificity += idf;
        const titleMatch = [...titleTokens].some((token) =>
          tokenAffinity(word, token) >= 0.75
        );
        if (titleMatch) titleMatched += 1;
        score += idf * (titleMatch ? 3 : 1);
      }
      return { operation, matched, titleMatched, score, specificity };
    })
    .filter(({ matched }) => matched > 0)
    .sort((left, right) =>
      right.score - left.score ||
      right.matched - left.matched ||
      (fullRanks.get(left.operation.document) ?? SEARCH_CANDIDATES) -
        (fullRanks.get(right.operation.document) ?? SEARCH_CANDIDATES) ||
      left.operation.path.localeCompare(right.operation.path)
    );
  if (lexical.length) {
    return lexical.slice(0, limit).map(({
      operation,
      matched,
      titleMatched,
      score,
      specificity,
    }, rank) => ({
      ...publicOperation(operation),
      matchedQuery: query,
      matchedTerms: matched,
      titleMatchedTerms: titleMatched,
      specificity,
      score: Math.round(score * 1_000) - rank,
    }));
  }

  const scores = new Map();
  for (const word of words) {
    if (!word) continue;
    const matches = index.matcher.matchesWith(
      word,
      index.config.withLimit(SEARCH_CANDIDATES),
    );
    for (const [rank, document] of matches.entries()) {
      const score = scores.get(document) ?? { matched: 0, ranks: 0 };
      score.matched += 1;
      score.ranks += rank;
      scores.set(document, score);
    }
  }
  return [...new Set([...full, ...scores.keys()])]
    .sort((left, right) => {
      const a = scores.get(left) ?? { matched: 0, ranks: SEARCH_CANDIDATES };
      const b = scores.get(right) ?? { matched: 0, ranks: SEARCH_CANDIDATES };
      return b.matched - a.matched ||
        (fullRanks.get(left) ?? SEARCH_CANDIDATES) -
          (fullRanks.get(right) ?? SEARCH_CANDIDATES) ||
        a.ranks - b.ranks ||
        left.localeCompare(right);
    })
    .slice(0, limit)
    .map((document, rank) => ({
      ...publicOperation(
        /** @type {IndexedOperation} */ (index.byDocument.get(document)),
      ),
      matchedQuery: query,
      matchedTerms: 0,
      titleMatchedTerms: 0,
      score: 1_000 - rank,
    }));
}

/** @param {Awaited<ReturnType<typeof buildState>>} index @param {string[]} queries @param {number} limit */
function search(index, queries, limit) {
  const groups = queries.map((query) => searchOne(index, query, limit));
  const output = [];
  const seen = new Set();
  for (let rank = 0; output.length < limit; rank += 1) {
    let added = false;
    for (const group of groups) {
      const operation = group[rank];
      if (!operation || seen.has(operation.key)) continue;
      seen.add(operation.key);
      output.push(operation);
      added = true;
      if (output.length === limit) break;
    }
    if (!added) break;
  }
  return output;
}

self.addEventListener("message", async (event) => {
  const { id, type, data } = event.data;
  try {
    const index = await state(id, data.url);
    let result;
    if (type === "prewarm") {
      result = true;
    } else if (type === "search") {
      result = search(index, data.queries, data.limit);
    } else if (type === "byKey") {
      const operation = index.byKey.get(data.key);
      result = operation ? publicOperation(operation) : undefined;
    } else {
      throw new Error(`Unknown API request: ${type}`);
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
