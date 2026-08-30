import { colors } from "../../utils/colors.js";
import { units } from "../../chart/units.js";
import { createMetric } from "./metrics/index.js";

/** @typedef {import("../storage.js").ChartArtifact} ChartArtifact */

const VIEWS = new Set(["line", "area", "stacked", "bar", "dots"]);
const SCALES = new Set(["linear", "log"]);
const PALETTE = ["orange", "blue", "green", "violet", "yellow", "red"];

/** @param {unknown} value @param {string} name */
function requiredString(value, name) {
  if (typeof value !== "string" || !value.trim()) {
    throw new Error(`Chart ${name} is required`);
  }
  return value.trim();
}

/** @param {Record<string, unknown>} args @returns {ChartArtifact} */
export function createChartArtifact(args) {
  const title = requiredString(args.title, "title");
  const unit = requiredString(args.unit, "unit");
  const view = args.view === undefined ? "line" : requiredString(args.view, "view");
  const scale = args.scale === undefined
    ? "linear"
    : requiredString(args.scale, "scale");

  if (!Object.hasOwn(units, unit)) throw new Error(`Unknown chart unit: ${unit}`);
  if (!VIEWS.has(view)) throw new Error(`Unknown chart view: ${view}`);
  if (!SCALES.has(scale)) throw new Error(`Unknown chart scale: ${scale}`);
  if (!Array.isArray(args.series) || !args.series.length || args.series.length > 6) {
    throw new Error("A chart needs between one and six series");
  }

  const usedColors = new Set(
    args.series.flatMap((value) =>
      value && typeof value === "object" &&
        typeof /** @type {Record<string, unknown>} */ (value).color === "string"
        ? [/** @type {Record<string, string>} */ (value).color]
        : []
    ),
  );
  const series = args.series.map((value, index) => {
    if (!value || typeof value !== "object") throw new Error("Invalid chart series");
    const item = /** @type {Record<string, unknown>} */ (value);
    const path = requiredString(item.path, "series path")
      .replace(/^bitview\.series\./, "")
      .replace(/^series\./, "");
    const label = requiredString(item.label, "series label");
    const color = item.color === undefined
      ? PALETTE.find((candidate) => !usedColors.has(candidate)) ?? PALETTE[index]
      : requiredString(item.color, "series color");

    if (!Object.hasOwn(colors, color)) throw new Error(`Unknown chart color: ${color}`);
    usedColors.add(color);
    createMetric(path);
    return {
      path,
      label,
      color: /** @type {ChartArtifact["chart"]["series"][number]["color"]} */ (color),
    };
  });

  return {
    type: /** @type {const} */ ("chart"),
    id: crypto.randomUUID(),
    chart: {
      title,
      unit: /** @type {ChartArtifact["chart"]["unit"]} */ (unit),
      view: /** @type {ChartArtifact["chart"]["view"]} */ (view),
      scale: /** @type {ChartArtifact["chart"]["scale"]} */ (scale),
      series,
    },
  };
}
