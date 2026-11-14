// @ts-check
// The page and its generated addresses use the same words, independent of runtime chart normalization.
import { shownStep } from "./words.js";

export const STUDIO_TITLE = "Bitview Studio: charts of Bitcoin's on-chain data";
export const STUDIO_DESCRIPTION = "Chart any of Bitview's Bitcoin on-chain series, or start from thousands of ready-made charts: overlay, stack, derive with formulas, draw, and share a chart as a link. Free, in the browser.";
export const STUDIO_ORIGIN = "https://bitview.space/studio";
const INVITATION = "Open it, change it and share it in Bitview Studio, free in the browser.";
/** @param {string[]} path @param {Record<string, string>} labels */
const place = (path, labels) => path.map((_, i) => shownStep(labels, path.slice(0, i + 1))).join(" › ");
/** @param {{ t: string, p: string[] }} preset @param {Record<string, string>} labels */
export function presetInfo(preset, labels) {
  return {
    title: `${preset.t} chart · Bitview Studio`,
    description: `${preset.t}: a chart of Bitcoin's on-chain data in ${place(preset.p.slice(0, -1), labels)}, live from bitview.space. ${INVITATION}`,
  };
}
/** @param {string[]} path @param {Record<string, string>} labels @param {string[]} inside */
export function sectionInfo(path, labels, inside) {
  const list = (/** @type {string[]} */ names) => names.length > 1 ? `${names.slice(0, -1).join(", ")} and ${names.at(-1)}` : names[0] ?? "";
  return {
    title: `${path.length ? shownStep(labels, path) : "Presets"} · Bitview Studio`,
    description: path.length
      ? `Charts of Bitcoin's on-chain data in ${place(path, labels)}: ${inside.length > 4 ? `${inside.slice(0, 4).join(", ")} and more` : list(inside)}. Open one, change it and share it in Bitview Studio, free in the browser.`
      : "Thousands of ready-made charts of Bitcoin's on-chain data: market, network, mining, distribution, investing and more. Open one, change it and share it in Bitview Studio, free in the browser.",
  };
}

/** @import { Preset } from "./presets.js" */
import { inBands, bandNames } from "./bands.js";
/** The real source series, including hidden inputs, cutoff expansions and implicit halving dates.
 * @param {Preset} preset @param {any} tree
 */
export function presetSources(preset, tree) {
  const names = new Set();
  for (const layer of preset.c.layers) {
    if (layer.series) names.add(layer.series);
    if (layer.formula) {
      const formula = layer.formula.replace(/\b(over|under)\(([a-z][a-z0-9_]*)\)/g, (/** @type {string} */ _, /** @type {string} */ mode, /** @type {string} */ name) => {
        const found = inBands(tree, name);
        if (!found) throw new Error(`No band sources for ${mode}(${name})`);
        return bandNames(found, mode).join(" + ");
      });
      for (const match of formula.matchAll(/\b[a-z][a-z0-9_]*\b/g)) {
        const name = match[0];
        // A function's name is consumed whole, never its suffix; date/year are formula values.
        if (formula.slice(match.index + name.length).trimStart().startsWith("(") || name === "date" || name === "year") continue;
        names.add(name);
      }
      if (/\bhalving\s*\(|\dhalving\b/.test(formula)) names.add("halving_epoch");
    }
  }
  return [...names];
}
/** @param {Preset} preset @param {any} tree */
export function presetDistributions(preset, tree) {
  const index = preset.c.index ?? "day1";
  const series = presetSources(preset, tree).flatMap((name) => ["json", "csv"].map((format) => ({
    "@type": "DataDownload", name, encodingFormat: format === "json" ? "application/json" : "text/csv",
    contentUrl: `https://bitview.space/api/series/${name}/${index}?format=${format}`,
  })));
  // URPD exposes JSON only. Never advertise a nonexistent CSV representation.
  const urpd = new Set(preset.c.layers.flatMap((layer) => [layer.heatmap, ...(layer.profile ?? [])].filter(Boolean).map((cohort) => `${cohort}${layer.weight ? `?weight=${layer.weight}` : ""}`)));
  return [...series, ...[...urpd].map((cohort) => ({ "@type": "DataDownload", name: `Cost basis distribution: ${cohort}`, encodingFormat: "application/json", contentUrl: `https://bitview.space/api/urpd/${cohort}` }))];
}
/** @param {Preset} preset @param {Record<string, string>} labels @param {any} tree @param {string} canonical */
export function presetDataset(preset, labels, tree, canonical) {
  return { "@context": "https://schema.org", "@type": "Dataset", name: preset.t, description: presetInfo(preset, labels).description, url: canonical,
    isAccessibleForFree: true, creator: { "@type": "Organization", name: "Bitview", url: "https://bitview.space" },
    distribution: presetDistributions(preset, tree) };
}

/** The app's structured data, shared by Home, sections and browser navigation.
 * @param {string} canonical @param {string} description
 */
export function studioApplication(canonical, description) {
  return { "@context": "https://schema.org", "@type": "WebApplication", name: "Bitview Studio", url: canonical, description,
    applicationCategory: "FinanceApplication", operatingSystem: "Any (a web browser)", isAccessibleForFree: true,
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" }, image: "https://bitview.space/assets/cards/studio.jpg",
    publisher: { "@type": "Organization", name: "Bitview", url: "https://bitview.space" } };
}
