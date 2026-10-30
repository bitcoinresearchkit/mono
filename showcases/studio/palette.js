// Studio's colors, one palette wherever they're drawn (the page, its presets): Bitview's named colors, the holder
// groups' (their comparisons, Compass's clouds, Bitcoin clouds, profiles, by the keys the series tree names them by), and
// the ramps families and things in time take.

/**
 * Bitview's palette, as the website defines it: lightness, chroma, hue in OKLCH. A layer's color is one of these
 * names, "ink" (the page's own), an exact "oklch(l c h)", or a hex; each is drawn in sRGB (the chart reads only
 * that), converted on demand.
 * @type {Record<string, [number, number, number]>}
 */
export const OKLCH = {
  gray: [0.55, 0, 0], red: [0.607, 0.241, 26.328], orange: [0.6764, 0.191, 44.41], amber: [0.7175, 0.1835, 64.199],
  yellow: [0.738, 0.173, 80.9405], avocado: [0.72, 0.19, 110], lime: [0.708, 0.2165, 131.267], green: [0.675, 0.2065, 149.3965],
  emerald: [0.646, 0.1575, 162.8525], teal: [0.652, 0.129, 183.6035], cyan: [0.662, 0.1345, 218.472], sky: [0.6365, 0.1635, 239.6445],
  blue: [0.5845, 0.2295, 261.348], indigo: [0.548, 0.2475, 277.0415], violet: [0.5735, 0.2655, 292.863], purple: [0.5925, 0.2765, 303.1105],
  fuchsia: [0.629, 0.294, 322.523], pink: [0.624, 0.245, 357.444], rose: [0.6155, 0.2495, 17.012],
};

/**
 * Each holder group's color: all coins green, short-term holders orange, long-term blue; the age cutoffs around them, the
 * younger warm (under 4 months yellow, under 6 red), the older cool (over 4 months cyan, over 6 violet).
 * @type {Readonly<Record<string, string>>}
 */
export const HOLDER_COLORS = Object.freeze({ all: "green", sth: "orange", lth: "blue", under4m: "yellow", under6m: "red", over4m: "cyan", over6m: "violet" });

/** A holder group's color, by its key (none: not a holder group). @param {string} key */
export const holderColor = (key) => (Object.hasOwn(HOLDER_COLORS, key) ? HOLDER_COLORS[key] : undefined);

/**
 * The `i`th of `count` members' color (a family's: the age bands from the youngest): hues around the wheel, evenly, from
 * warm to pink, all as light and as vivid. @param {number} i @param {number} count
 */
export const wheelColor = (i, count, from = 25, to = 320) => `oklch(0.72 0.15 ${Math.round(count > 1 ? from + ((to - from) * i) / (count - 1) : from)})`;

/** Bitview's hues from cool to warm, as things in time take them, the oldest the coolest. */
const IN_TIME = ["indigo", "blue", "sky", "cyan", "teal", "emerald", "green", "lime", "avocado", "yellow", "amber", "orange"];
/**
 * The `i`th of `count` in time, the oldest first (halving cycles, years, a family's members by when): Bitview's hues
 * from cool to warm, the newest orange, as far apart as their number lets them: by name, up to twelve; more, each mixed
 * between the two nearest. Each its own hue, in either theme (you, 2026-10-31). @param {number} i @param {number} count
 */
export function recencyColor(i, count) {
  const at = count > 1 ? (i * (IN_TIME.length - 1)) / (count - 1) : IN_TIME.length - 1;
  if (count <= IN_TIME.length) return IN_TIME[Math.round(at)];
  const below = Math.floor(at), t = at - below, [l0, c0, h0] = OKLCH[IN_TIME[below]], [l1, c1, h1] = OKLCH[IN_TIME[Math.min(below + 1, IN_TIME.length - 1)]];
  return `oklch(${(l0 + (l1 - l0) * t).toFixed(3)} ${(c0 + (c1 - c0) * t).toFixed(4)} ${(h0 + (h1 - h0) * t).toFixed(1)})`;
}
