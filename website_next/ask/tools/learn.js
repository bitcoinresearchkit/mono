import { bitview } from "../../utils/client.js";
import { normalize, relevance, tokenAffinity } from "./text.js";

/** @type {Promise<any[]> | undefined} */
let catalogPromise;

/** @param {any} node @param {string[]} breadcrumbs */
function record(node, breadcrumbs) {
  if (!node.chart) return undefined;
  const series = node.chart.series.flatMap((/** @type {any} */ entry) => {
    try {
      const metric = entry.metric(bitview);
      return metric?.name
        ? [{ name: metric.name, label: entry.label }]
        : [];
    } catch {
      return [];
    }
  });
  return {
    title: node.chart.title ?? node.title,
    sectionTitle: node.title,
    breadcrumbs,
    description: node.description ?? "",
    ...(node.example ? { example: node.example } : {}),
    unit: node.chart.unit?.id,
    series,
  };
}

/** @param {any[]} nodes @param {string[]} ancestors @param {any[]} output */
function walk(nodes, ancestors, output) {
  for (const node of nodes) {
    const breadcrumbs = [...ancestors, node.title];
    const item = record(node, breadcrumbs);
    if (item) output.push(item);
    if (node.children) walk(node.children, breadcrumbs, output);
  }
}

async function learnCatalog() {
  catalogPromise ??= import("../../learn/data/index.js").then(({ sections }) => {
    /** @type {any[]} */
    const output = [];
    walk(sections, [], output);
    return output;
  });
  return catalogPromise;
}

/** @param {string} query @param {number} [limit] */
export async function searchLearn(query, limit = 12) {
  const queryTokens = normalize(query).split(" ").filter(Boolean);
  return (await learnCatalog())
    .map((/** @type {any} */ item) => {
      const titleTokens = normalize(item.title)
        .split(" ")
        .filter((token) => token.length >= 4);
      const titleCoverage = titleTokens.length
        ? titleTokens.filter((title) =>
            queryTokens.some((token) => tokenAffinity(title, token) >= 0.7)
          ).length / titleTokens.length
        : 0;
      return {
        ...item,
        titleCoverage,
        score:
          titleCoverage * 50 +
        relevance(query, `${item.title} ${item.sectionTitle}`) * 3 +
        relevance(query, item.breadcrumbs.join(" ")) * 2 +
        relevance(
          query,
          `${item.description} ${item.series.flatMap((/** @type {any} */ series) => [series.label, series.name]).join(" ")}`,
        ),
      };
    })
    .filter(({ score }) => score >= 16)
    .sort((left, right) => right.score - left.score)
    .slice(0, limit);
}
