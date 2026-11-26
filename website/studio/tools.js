// @ts-check
// The drawing tools and a drawing's words: each kind's name, key and tip (said for the device), its points, sizes,
// ends and backs, a drawing's look.
import { CLICK, TOUCH } from "/studio/dom.js";
import { ICONS } from "/studio/icons.js";
import { isColor } from "/studio/color.js";
import { DASHES, OPACITIES, WIDTHS } from "/studio/vocabulary.js";
import { capitalized } from "/studio/words.js";
/** @import { DrawingKind } from "/studio/vocabulary.js" */

const TOOLS = /** @type {Record<DrawingKind, { name: string, noun: string, verb: string, key: string, tip: string, keys: string, icon: string }>} */ ({
  trend: { name: "Trend line", noun: "trend line", verb: "Draw a trend line", key: "L", tip: `Drag, or ${CLICK} its two ends`, keys: "Arrows to each end, ↵ to place it", icon: ICONS.trend },
  path: { name: "Path", noun: "path", verb: "Draw a path", key: "P", tip: `${capitalized(CLICK)} each of its points, the last again to finish`, keys: "Arrows to each point, ↵ to place it, ↵ again to finish", icon: ICONS.path },
  hline: { name: "Horizontal line", noun: "horizontal line", verb: "Draw a horizontal line", key: "H", tip: `${capitalized(CLICK)} at its value`, keys: "Arrows to its value, ↵ to place it", icon: ICONS.hline },
  vline: { name: "Vertical line", noun: "vertical line", verb: "Draw a vertical line", key: "V", tip: `${capitalized(CLICK)} at its date`, keys: "Arrows to its date, ↵ to place it", icon: ICONS.vline },
  rect: { name: "Rectangle", noun: "rectangle", verb: "Draw a rectangle", key: "R", tip: `Drag, or ${CLICK} two corners`, keys: "Arrows to each corner, ↵ to place it", icon: ICONS.rect },
  ellipse: { name: "Ellipse", noun: "ellipse", verb: "Draw an ellipse", key: "E", tip: `Drag, or ${CLICK} two corners`, keys: "Arrows to each corner, ↵ to place it", icon: ICONS.ellipse },
  text: { name: "Text", noun: "text", verb: "Write on the chart", key: "T", tip: `${capitalized(CLICK)} where it goes, then type`, keys: "Arrows to where it goes, ↵, then type", icon: ICONS.text },
  brush: { name: "Brush", noun: "stroke", verb: "Draw with the brush", key: "B", tip: "Draw freely", keys: "Draw freely, with a pointer", icon: ICONS.brush },
  ruler: { name: "Ruler", noun: "ruler", verb: "Measure a change", key: "M", tip: "Drag across the chart to measure a change", keys: "Arrows to each end, ↵ to place it", icon: ICONS.ruler },
});
const KINDS = /** @type {DrawingKind[]} */ (Object.keys(TOOLS));
const LENS_SAYS = `the chart under ${TOUCH ? "your finger" : "the pointer"} at twice the size`;
const PAIRED = new Set(["trend", "rect", "ellipse", "ruler"]);
const THROUGH = new Set(["trend", "path"]);
const POINTED = new Set([...PAIRED, "path"]);
const BOXES = new Set(["rect", "ellipse", "ruler"]);
const LINES = new Set(["trend", "hline", "vline"]);
const LABEL_MAX = 80, labelText = (/** @type {string} */ text) => text.replace(/\s+/g, " ").trim().slice(0, LABEL_MAX);
// Bounded so all of it fits in a link.
const MAX_DRAWINGS = 100, MAX_STROKE = 300;
// Time to the second, the value unrounded: rounding moves it on a deep zoom.
const pointOf = (/** @type {number} */ time, /** @type {number} */ value) => /** @type {[number, number]} */ ([Math.round(time), value]);
/** @typedef {"arrow" | "extend"} End @type {[End | "", string, string][]} */
const ENDS = [["", "Plain", "Ends at its point"], ["arrow", "Arrow", "An arrowhead at its point"], ["extend", "Extended", "On past its point, to the chart's edge"]];
const isEnd = (/** @type {unknown} */ value) => value === "arrow" || value === "extend";
const SIZES = [11, 13, 16, 22], TEXT_SIZE = 13;
const SIZE_NAMES = { 11: "Small", 13: "Medium", 16: "Large", 22: "Huge" }, SIZE_SHORT = { 11: "S", 13: "M", 16: "L", 22: "XL" };
const BACKS = [["", "None", "Just the words"], ["outline", "Outline", "A border around the words"], ["soft", "Soft", "On a soft fill of its color"], ["soft-outline", "Soft, outlined", "On a soft fill, a border around"], ["solid", "Solid", "On its color, the words in contrast"]];
const presetSize = (/** @type {number} */ size) => SIZES.reduce((best, each) => (Math.abs(each - size) <= Math.abs(best - size) ? each : best));
const PEN_COLOR = "yellow";
function lookOfDrawing(kind, input) {
  return {
    color: isColor(input?.color) ? input.color : PEN_COLOR,
    width: WIDTHS.includes(input?.width) ? input.width : 1.5,
    dash: DASHES.some(([dash]) => dash === input?.dash) ? input.dash : 0,
    ...(kind !== "ruler" && OPACITIES.includes(input?.opacity) && input.opacity < 1 ? { opacity: input.opacity } : {}),
    ...(THROUGH.has(kind) && isEnd(input?.start) ? { start: input.start } : {}),
    ...(THROUGH.has(kind) && isEnd(input?.end) ? { end: input.end } : {}),
    ...((kind === "text" || LINES.has(kind)) && Number.isFinite(input?.size) ? { size: presetSize(input.size) } : {}),
    ...((kind === "text" || LINES.has(kind)) && BACKS.some(([back]) => back && back === input?.back) ? { back: input.back } : {}),
  };
}
export {
  BACKS, BOXES, ENDS, KINDS, LABEL_MAX, labelText, LENS_SAYS, LINES, lookOfDrawing, MAX_DRAWINGS, MAX_STROKE, PAIRED,
  PEN_COLOR, POINTED, pointOf, presetSize, SIZE_NAMES, SIZE_SHORT, SIZES, TEXT_SIZE, THROUGH, TOOLS,
};
