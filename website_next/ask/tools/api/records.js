import { normalize } from "../text.js";

const ORDINALS = new Map([
  ["first", 0],
  ["second", 1],
  ["third", 2],
  ["fourth", 3],
  ["fifth", 4],
  ["sixth", 5],
  ["seventh", 6],
  ["eighth", 7],
  ["ninth", 8],
  ["tenth", 9],
]);

/** @param {unknown} data */
export function apiRows(data) {
  if (Array.isArray(data)) return data;
  return data &&
      typeof data === "object" &&
      "sample" in data &&
      Array.isArray(data.sample)
    ? data.sample
    : undefined;
}

/**
 * @param {unknown} data
 * @param {string} question
 * @param {Record<string, unknown>} [previousArguments]
 */
export function selectApiRecord(data, question, previousArguments = {}) {
  const rows = apiRows(data);
  if (!rows?.length) return undefined;

  const previous = Object.entries(previousArguments);
  const matching = rows.filter((row) => {
    if (!row || typeof row !== "object" || Array.isArray(row)) return false;
    const shared = previous.filter(([name]) => Object.hasOwn(row, name));
    return shared.length &&
      shared.every(([name, value]) => String(row[name]) === String(value));
  });
  if (matching.length === 1) return matching[0];

  const words = new Set(normalize(question).split(" "));
  if (words.has("last")) return rows.at(-1);
  for (const [word, index] of ORDINALS) {
    if (words.has(word) && index < rows.length) return rows[index];
  }
  const numbered = normalize(question).match(
    /\b(\d+)\s*(?:st|nd|rd|th)\b/,
  );
  const index = numbered ? Number(numbered[1]) - 1 : -1;
  return index >= 0 && index < rows.length ? rows[index] : undefined;
}

/** @param {unknown} record @param {string} responseType */
export function recordArguments(record, responseType) {
  if (
    typeof record === "string" ||
    typeof record === "number" ||
    typeof record === "boolean"
  ) {
    const name = normalize(responseType.replace(/\[\]$/, "")).replaceAll(
      " ",
      "_",
    );
    return name && !["string", "number", "integer", "boolean"].includes(name)
      ? { [name]: record }
      : {};
  }
  return record && typeof record === "object" && !Array.isArray(record)
    ? Object.fromEntries(
      Object.entries(record).filter(([, value]) =>
        typeof value === "string" ||
        typeof value === "number" ||
        typeof value === "boolean"
      ),
    )
    : {};
}
