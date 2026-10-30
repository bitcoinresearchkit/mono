// The holder groups' colors, one palette wherever they're drawn: Studio's presets (their comparisons, Compass's clouds,
// Bitcoin clouds) and its profiles, by the same keys the series tree names them by.

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

/**
 * The `i`th of `count` in time, the oldest first (halving cycles, years, a family's members by when): a quiet violet,
 * then orchid and coral, to a bright amber, the newest, each brighter and warmer; on the light theme (its lightness
 * turned over) the newest the deepest, the hues still parting them. Four in a chart (one, and the three before it) are
 * told apart by every kind of color vision, in either theme. @param {number} i @param {number} count
 */
export const recencyColor = (i, count) => ((t) => `oklch(${(0.52 + 0.31 * t).toFixed(3)} ${(0.19 - 0.01 * t).toFixed(3)} ${((285 + 143 * t) % 360).toFixed(1)})`)(count > 1 ? i / (count - 1) : 1);
