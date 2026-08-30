import { formatValue } from "./data.js";

/**
 * @typedef {Object} MetricRead
 * @property {string} label
 * @property {string | undefined} unit
 * @property {string} index
 * @property {number | string} start
 * @property {string | undefined} stamp
 * @property {unknown[]} values
 * @property {string} [requested]
 */

/**
 * @typedef {Object} SourceEvidence
 * @property {string} revision
 * @property {string} path
 * @property {number} startLine
 * @property {number} [endLine]
 */

const SOURCE_URL = "https://github.com/bitcoinresearchkit/bitview/blob";

/** @param {SourceEvidence} source */
function sourceKey(source) {
  return `${source.revision}:${source.path}:${source.startLine}:${source.endLine ?? source.startLine}`;
}

/** @param {SourceEvidence} source */
function sourceLink(source) {
  const end = source.endLine && source.endLine !== source.startLine
    ? `-${source.endLine}`
    : "";
  const path = source.path.split("/").map(encodeURIComponent).join("/");
  const lines = `#L${source.startLine}${end ? `-L${source.endLine}` : ""}`;
  const url = `${SOURCE_URL}/${encodeURIComponent(source.revision)}/${path}${lines}`;
  return `[\`${source.path}:${source.startLine}${end}\`](${url})`;
}

/** @param {{ facts: string[], sources: SourceEvidence[], excerpts: (SourceEvidence & { content: string })[] }} evidence */
export function renderEvidence(evidence) {
  const sections = [...new Set(evidence.facts)].filter(Boolean);
  const cited = new Set();
  for (const excerpt of evidence.excerpts) {
    sections.push(`${sourceLink(excerpt)}\n\n\`\`\`\n${excerpt.content}\n\`\`\``);
    cited.add(sourceKey(excerpt));
  }
  const sources = [...new Map(
    evidence.sources.map((source) => [sourceKey(source), source]),
  ).values()].filter((source) => !cited.has(sourceKey(source)));
  if (sources.length) {
    sections.push(`Source${sources.length === 1 ? "" : "s"}: ${sources.map(sourceLink).join(", ")}`);
  }
  return sections.join("\n\n") || "I could not find enough verified evidence to answer that.";
}

/** @param {MetricRead[]} results */
export function renderData(results) {
  return results.map((result) => {
    const returnedPosition = result.index === "height"
      ? String(result.start)
      : result.stamp;
    if (
      result.requested &&
      returnedPosition &&
      result.requested !== returnedPosition
    ) {
      const returned = result.index === "height"
        ? `block ${returnedPosition}`
        : returnedPosition;
      return `**${result.label}**: no exact value was returned for ${result.requested}; the server returned ${returned} instead.`;
    }
    if (result.values.length === 1 && typeof result.values[0] === "number") {
      const position = result.index === "height"
        ? ` at block ${result.start}`
        : result.stamp
          ? ` at ${result.stamp}`
          : "";
      return `**${result.label}**: ${formatValue(result.values[0], result.unit)}${position}`;
    }
    const values = /** @type {number[]} */ (
      result.values.filter((value) => typeof value === "number")
    );
    if (!values.length) return `**${result.label}**: no values returned.`;
    return `**${result.label}**: ${values.length} values; latest ${formatValue(values[values.length - 1], result.unit)}.`;
  }).join("\n");
}

/** @param {string} answer @param {{ method: string, path: string }} operation */
export function renderApiAnswer(answer, operation) {
  return `${answer.trim()}\n\nData: \`${operation.method} ${operation.path}\``;
}
