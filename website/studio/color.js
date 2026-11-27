// @ts-check
// Colors as layers have them: which values are colors, the swatches and the turns new layers take, OKLCH to sRGB
// and back held inside what sRGB shows, the light theme's tone curve.
import { clamp } from "/studio/math.js";
import { OKLCH } from "/studio/palette.js";

const TONE = /** @type {[number, number][]} */ ([[0, 1], [0.5, 0.64], [0.75, 0.58], [1, 0]]);
function onCurve(/** @type {[number, number][]} */ knots, /** @type {number} */ x) {
  const i = clamp(knots.findIndex(([at]) => at >= x), 1, knots.length - 1), [x0, y0] = knots[i - 1], [x1, y1] = knots[i];
  return clamp(y0 + ((x - x0) * (y1 - y0)) / (x1 - x0), Math.min(y0, y1), Math.max(y0, y1));
}
const warmLift = (/** @type {number} */ c, /** @type {number} */ h) => (h > 55 && h < 120 ? 0.1 * Math.sin(((h - 55) / 65) * Math.PI) * Math.min(1, c / 0.1) : 0);
const SWATCHES = ["ink", "gray", "red", "orange", "amber", "yellow", "avocado", "lime", "green", "emerald", "teal", "cyan", "sky", "blue", "indigo", "violet", "purple", "fuchsia", "pink", "rose"];
const PALETTE = ["ink", "orange", "blue", "green", "purple", "red", "teal", "yellow", "sky", "pink", "lime", "gray"];
const OKLCH_TEXT = /^oklch\((0|1|0?\.\d+) (0|0?\.\d+) (\d{1,3}(\.\d+)?)\)$/;
const PAGE_INKS = /** @type {Record<string, string>} */ ({ ink: "--ink", soft: "--soft" });
/** @param {string} color */
const pageInk = (color) => (Object.hasOwn(PAGE_INKS, color) ? PAGE_INKS[color] : undefined);
const isNamed = (color) => Boolean(pageInk(color)) || Object.hasOwn(OKLCH, color);
/** Only the palette's colors (its names, and oklch mixes of them) follow the theme; a hex is drawn as given. @param {string} color */
const ofPalette = (color) => isNamed(color) || OKLCH_TEXT.test(color);
const isColor = (color) => typeof color === "string" && (isNamed(color) || /^#[0-9a-f]{6}$/i.test(color) || (OKLCH_TEXT.test(color) && Number(color.split(" ")[2].slice(0, -1)) <= 360));
function nameDistance(l1, c1, h1, l2, c2, h2) {
  const dh = 2 * Math.sqrt(c1 * c2) * Math.sin(((h1 - h2) * Math.PI) / 360);
  return (l1 - l2) ** 2 + 2 * (c1 - c2) ** 2 + 3 * dh ** 2;
}
/**
 * Chroma is lowered until it fits, so hue and lightness hold.
 * @returns {string}
 */
function oklchToHex(l, c, h) {
  const wanted = oklchToLinear(l, c, h), rgb = inside(wanted) ? wanted : oklchToLinear(l, limit(l, h), h);
  return `#${rgb.map((v) => Math.round(gamma(clamp(v, 0, 1)) * 255).toString(16).padStart(2, "0")).join("")}`;
}
/** @returns {[number, number, number]} */
function oklchToLinear(l, c, h) {
  return labToLinear(l, c * Math.cos((h * Math.PI) / 180), c * Math.sin((h * Math.PI) / 180));
}
/** @returns {[number, number, number]} */
function labToLinear(l, a, b, out = /** @type {[number, number, number]} */ ([0, 0, 0])) {
  const L = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3, M = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3, S = (l - 0.0894841775 * a - 1.291485548 * b) ** 3;
  out[0] = 4.0767416621 * L - 3.3077115913 * M + 0.2309699292 * S;
  out[1] = -1.2684380046 * L + 2.6097574011 * M - 0.3413193965 * S;
  out[2] = -0.0041960863 * L - 0.7034186147 * M + 1.707614701 * S;
  return out;
}
// Plain comparisons: a picker's surface asks very often, on a phone without its JIT.
const inside = (/** @type {number[]} */ rgb) => rgb[0] >= -1e-4 && rgb[0] <= 1 + 1e-4 && rgb[1] >= -1e-4 && rgb[1] <= 1 + 1e-4 && rgb[2] >= -1e-4 && rgb[2] <= 1 + 1e-4;
const gamma = (x) => (x <= 0.0031308 ? 12.92 * x : 1.055 * x ** (1 / 2.4) - 0.055);
const maxChromas = new Map();
function maxChroma(l, h) {
  const key = `${l.toFixed(3)}:${h.toFixed(1)}`;
  if (maxChromas.has(key)) return maxChromas.get(key);
  const most = limit(l, h);
  if (maxChromas.size > 20000) maxChromas.clear();
  maxChromas.set(key, most);
  return most;
}
function limit(l, h, steps = 16) {
  const cos = Math.cos((h * Math.PI) / 180), sin = Math.sin((h * Math.PI) / 180), rgb = /** @type {[number, number, number]} */ ([0, 0, 0]);
  let low = 0, high = 0.4;
  for (let k = 0; k < steps; k++) {
    const mid = (low + high) / 2;
    if (inside(labToLinear(l, mid * cos, mid * sin, rgb))) low = mid;
    else high = mid;
  }
  return low;
}
/** @returns {[number, number, number]} */
function hexToOklch(hex) {
  const [r, g, b] = [1, 3, 5].map((i) => { const x = parseInt(hex.slice(i, i + 2), 16) / 255; return x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4; });
  const L = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b), M = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b), S = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  const l = 0.2104542553 * L + 0.793617785 * M - 0.0040720468 * S, a = 1.9779984951 * L - 2.428592205 * M + 0.4505937099 * S, bb = 0.0259040371 * L + 0.7827717662 * M - 0.808675766 * S;
  return [l, Math.hypot(a, bb), (((Math.atan2(bb, a) * 180) / Math.PI) + 360) % 360];
}

export {
  gamma, hexToOklch, inside, isColor, isNamed, ofPalette, pageInk, limit, maxChroma, nameDistance, oklchToHex, oklchToLinear, onCurve,
  PALETTE, SWATCHES, TONE, warmLift,
};
