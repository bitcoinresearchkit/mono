// @ts-check

const ink = new Map();
/** @type {CanvasRenderingContext2D | null} */
let pen = null;

const solids = new Map();

/**
 * @param {string} color
 */
export function solid(color) {
  let full = solids.get(color);
  if (full !== undefined) return full;
  pen ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
  // (As a canvas writes it back: #rrggbb opaque, rgba(…) with an alpha, other spaces with "/ alpha": that alpha dropped.)
  pen.fillStyle = "#000";
  pen.fillStyle = color;
  const read = String(pen.fillStyle), m = /^rgba\((\d+), (\d+), (\d+), ([^)]+)\)$/.exec(read);
  const slash = m ? null : /\s*\/\s*([^)/]+)\)$/.exec(read), alpha = m ? Number(m[4]) : slash ? parseFloat(slash[1]) / (slash[1].trim().endsWith("%") ? 100 : 1) : 1;
  full = !(alpha > 0) ? color : m ? `rgb(${m[1]}, ${m[2]}, ${m[3]})` : slash ? read.slice(0, slash.index) + ")" : read;
  if (solids.size > 512) solids.clear();
  solids.set(color, full);
  return full;
}

/** @param {string} color */
export function inkOn(color) {
  let text = ink.get(color);
  if (text !== undefined) return text;
  text = whiteOn(rgb(read(color))) ? "#ffffff" : "#000000";
  if (ink.size > 512) ink.clear();
  ink.set(color, text);
  return text;
}

const tints = new Map();

/**
 * No edge: a series the ground's color is the page's choice.
 * @param {string} color
 * @returns {{ fill: string, ink: string }}
 */
export function tint(color) {
  let look = tints.get(color);
  if (look !== undefined) return look;
  const seen = read(color);
  if (seen[3] === 0) look = { fill: "transparent", ink: "transparent" };
  else {
    const fill = rgb(seen);
    look = { fill: hex(fill), ink: hex(wordsOn(fill)) };
  }
  if (tints.size > 512) tints.clear();
  tints.set(color, look);
  return look;
}

/**
 * @param {string} color @param {string} ground
 * @returns {{ fill: string, ink: string, edge: string }}
 */
export function outlined(color, ground) {
  const { fill } = tint(color);
  return fill === "transparent" ? { fill, ink: fill, edge: fill } : { fill: ground, ink: legible(fill, ground), edge: fill };
}

const overs = new Map();

/** @param {string} color @param {string} ground */
export function over(color, ground) {
  const key = `${color}|${ground}`;
  let out = overs.get(key);
  if (out !== undefined) return out;
  const top = read(color), under = read(ground), share = top[3] / 255;
  out = hex([0, 1, 2].map((c) => Math.round(top[c] * share + under[c] * (1 - share))));
  if (overs.size > 64) overs.clear();
  overs.set(key, out);
  return out;
}

const legibles = new Map();

/**
 * Its OKLab lightness moved toward white or black just as far as it takes to read.
 * @param {string} color @param {string} ground
 */
export function legible(color, ground) {
  const key = `${color}|${ground}`;
  let out = legibles.get(key);
  if (out !== undefined) return out;
  const seen = rgb(read(color)), under = rgb(read(ground)), back = luminance(under), [L, a0, b0] = oklab(seen), hue = Math.atan2(b0, a0), chroma = Math.hypot(a0, b0);
  const reads = (/** @type {number[]} */ c) => ratio(c, under) >= FLOOR;
  out = hex(reads(seen) ? seen : toward(L, 1.05 / (back + 0.05) >= (back + 0.05) / 0.05 ? 1 : 0, hue, chroma, reads));
  if (legibles.size > 64) legibles.clear();
  legibles.set(key, out);
  return out;
}

/**
 * WCAG 2's relative luminance.
 * @param {number[]} c
 */
function luminance(c) {
  const f = (/** @type {number} */ v) => ((v /= 255) <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  return 0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2]);
}

/**
 * WCAG 2's contrast.
 * @param {number[]} x @param {number[]} y
 */
function ratio(x, y) {
  const a = luminance(x), b = luminance(y);
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

/** WCAG 2's floor for small words (12 px). */
const FLOOR = 4.5;

/**
 * A whisper of the fill's hue in OKLCH; where that falls short of FLOOR, moved toward pure white or black just as far.
 * @param {number[]} fill
 */
function wordsOn(fill) {
  const white = whiteOn(fill), [, a, b] = oklab(fill), hue = Math.atan2(b, a), chroma = Math.min(0.06, Math.hypot(a, b)), L = white ? 0.97 : 0.24;
  const reads = (/** @type {number[]} */ c) => ratio(c, fill) >= FLOOR, ink = vivid(L, hue, chroma);
  return reads(ink) ? ink : toward(L, white ? 1 : 0, hue, chroma, reads);
}

/** @param {number} l @param {number} hue @param {number} most */
function vivid(l, hue, most) {
  const at = (/** @type {number} */ c) => lin(l, c * Math.cos(hue), c * Math.sin(hue));
  let lo = 0, hi = most;
  if (fits(at(hi))) lo = hi;
  else
    for (let k = 0; k < 24; k++) {
      const mid = (lo + hi) / 2;
      if (fits(at(mid))) lo = mid;
      else hi = mid;
    }
  return rgb(at(lo));
}

/**
 * Found by halves, its hue kept.
 * @param {number} L @param {number} to @param {number} hue @param {number} chroma @param {(c: number[]) => boolean} reads
 */
function toward(L, to, hue, chroma, reads) {
  let lo = L, hi = to;
  for (let k = 0; k < 24; k++) {
    const mid = (lo + hi) / 2;
    if (reads(vivid(mid, hue, chroma))) hi = mid;
    else lo = mid;
  }
  return vivid(hi, hue, chroma);
}

/**
 * APCA's lightness contrast without its offsets and clamp: only two inks on one fill are compared.
 * @param {number[]} text @param {number[]} back
 */
function lc(text, back) {
  const y = (/** @type {number[]} */ c) => {
    const v = 0.2126729 * (c[0] / 255) ** 2.4 + 0.7151522 * (c[1] / 255) ** 2.4 + 0.072175 * (c[2] / 255) ** 2.4;
    return v < 0.022 ? v + (0.022 - v) ** 1.414 : v;
  };
  const t = y(text), b = y(back);
  return b > t ? b ** 0.56 - t ** 0.57 : t ** 0.62 - b ** 0.65;
}

const WHITE = [255, 255, 255], BLACK = [0, 0, 0];

/**
 * APCA's pick, unless under FLOOR (WCAG 2); then the other, never under it (white's and black's ratios multiply to 21).
 * @param {number[]} back
 */
function whiteOn(back) {
  const white = lc(WHITE, back) >= lc(BLACK, back);
  return ratio(white ? WHITE : BLACK, back) >= FLOOR ? white : !white;
}

/** @param {ArrayLike<number>} c */
function rgb(c) {
  return c.length === 4 ? [c[0], c[1], c[2]] : Array.from(c, (v) => Math.round(255 * (Math.min(1, Math.max(0, v)) <= 0.0031308 ? 12.92 * Math.min(1, Math.max(0, v)) : 1.055 * Math.min(1, Math.max(0, v)) ** (1 / 2.4) - 0.055)));
}

/** At or under it a color has no hue of its own (a gray): mixed, it takes its neighbor's. */
const GRAY = 0.02;

/**
 * In OKLCH the shorter way round, vivid between; a see-through stop takes its neighbor's color (no fade to black).
 * @param {readonly (readonly [number, string])[]} stops @param {number} [steps]
 * @returns {string[]}
 */
export function gradientTable(stops, steps = 256) {
  const rows = mixes(stops, steps);
  return Array.from({ length: steps }, (_, k) => {
    const at = k * 4, a = rows[at + 3];
    return `oklch(${+rows[at].toFixed(4)} ${+rows[at + 1].toFixed(4)} ${+rows[at + 2].toFixed(1)}${a < 255 ? ` / ${+(a / 255).toFixed(3)}` : ""})`;
  });
}

/** @param {readonly (readonly [number, string])[]} stops @param {number} steps */
function mixes(stops, steps) {
  const lchs = stops.map(([, color]) => lchOf(color)), rows = new Float64Array(steps * 4), last = stops.length - 1;
  for (let k = 0; k < steps; k++) {
    const at = steps > 1 ? (k / (steps - 1)) * 100 : 0, next = stops.findIndex(([where]) => where >= at);
    const [i, j, t] = next <= 0 ? [next < 0 ? last : 0, next < 0 ? last : 0, 0] : [next - 1, next, (at - stops[next - 1][0]) / Math.max(1e-9, stops[next][0] - stops[next - 1][0])];
    const a = lchs[i][3] === 0 ? lchs[j] : lchs[i], b = lchs[j][3] === 0 ? lchs[i] : lchs[j];
    const from = a[1] <= GRAY ? b[2] : a[2], to = b[1] <= GRAY ? a[2] : b[2], turn = ((((to - from) % 360) + 540) % 360) - 180;
    (rows[k * 4] = a[0] + (b[0] - a[0]) * t), (rows[k * 4 + 1] = a[1] + (b[1] - a[1]) * t), (rows[k * 4 + 2] = (((from + turn * t) % 360) + 360) % 360);
    rows[k * 4 + 3] = lchs[i][3] + (lchs[j][3] - lchs[i][3]) * t;
  }
  return rows;
}

/**
 * An `oklch(…)` read as written: none of its chroma clipped to sRGB, as a canvas's writing would.
 * @param {string} color @returns {number[]}
 */
function lchOf(color) {
  const call = /^oklch\(\s*([\d.e+-]+%?)\s+([\d.e+-]+%?)\s+([\d.e+-]+)(?:deg)?\s*(?:\/\s*([\d.e+-]+%?)\s*)?\)$/.exec(color.trim());
  const num = (/** @type {string} */ part, /** @type {number} */ whole) => (part.endsWith("%") ? (parseFloat(part) / 100) * whole : parseFloat(part));
  if (call) return [num(call[1], 1), num(call[2], 0.4), parseFloat(call[3]), call[4] === undefined ? 255 : Math.round(Math.min(1, Math.max(0, num(call[4], 1))) * 255)];
  const channels = read(color), [L, a, b] = oklab(channels);
  return [L, Math.hypot(a, b), ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360, channels[3]];
}

/** @param {number} place @param {number} n */
export const placeIndex = (place, n) => Math.round((place < 0 ? 0 : place > 1 ? 1 : place) * (n - 1));

/**
 * From the canvas's own writing of it, never a read pixel: an anti-fingerprinting browser (Mullvad) adds noise.
 * @param {string} color
 */
export function channels(color) {
  return read(color);
}

/** @param {string} color */
function read(color) {
  pen ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
  pen.fillStyle = "#000";
  pen.fillStyle = color;
  return parsed(String(pen.fillStyle));
}

/** @param {string} text */
function parsed(text) {
  if (text[0] === "#") {
    const hex = (/** @type {number} */ at) => parseInt(text.slice(at, at + 2), 16);
    return [hex(1), hex(3), hex(5), text.length > 7 ? hex(7) : 255];
  }
  const call = /^([a-z0-9-]+)\((.*)\)$/.exec(text.trim());
  if (!call) return [0, 0, 0, 255];
  const [, name, body] = call, parts = body.replace("/", " / ").split(/[\s,]+/).filter(Boolean), slash = parts.indexOf("/");
  const num = (/** @type {string | undefined} */ part, /** @type {number} */ whole) => (part === undefined || part === "none" ? 0 : part.endsWith("%") ? (parseFloat(part) / 100) * whole : parseFloat(part));
  const values = slash >= 0 ? parts.slice(0, slash) : parts, alpha = slash >= 0 ? num(parts[slash + 1], 1) : (name === "rgba" || name === "rgb") && parts.length === 4 ? num(parts[3], 1) : 1;
  /** @type {number[]} */
  let channels = [0, 0, 0];
  if (name === "rgb" || name === "rgba") channels = values.slice(0, 3).map((part) => num(part, 255));
  else if (name === "oklch" || name === "oklab") {
    const l = num(values[0], 1), first = num(values[1], 0.4), second = name === "oklch" ? (num(values[2], 360) * Math.PI) / 180 : num(values[2], 0.4);
    channels = rgb(name === "oklch" ? lin(l, first * Math.cos(second), first * Math.sin(second)) : lin(l, first, second));
  } else if (name === "lab" || name === "lch") {
    const l = num(values[0], 100), first = num(values[1], name === "lab" ? 125 : 150), second = name === "lch" ? (num(values[2], 360) * Math.PI) / 180 : num(values[2], 125);
    const a = name === "lab" ? first : first * Math.cos(second), b = name === "lab" ? second : first * Math.sin(second), fy = (l + 16) / 116, fx = fy + a / 500, fz = fy - b / 200;
    const inv = (/** @type {number} */ t) => (t ** 3 > 216 / 24389 ? t ** 3 : (116 * t - 16) / (24389 / 27));
    channels = rgb(fromXYZ(d50to65([0.3457 / 0.3585 * inv(fx), l > 8 ? fy ** 3 : l / (24389 / 27), ((1 - 0.3457 - 0.3585) / 0.3585) * inv(fz)])));
  } else if (name === "color") {
    const space = values[0], [x, y, z] = values.slice(1, 4).map((part) => num(part, 1));
    const srgb = (/** @type {number} */ v) => Math.sign(v) * (Math.abs(v) <= 0.04045 ? Math.abs(v) / 12.92 : ((Math.abs(v) + 0.055) / 1.055) ** 2.4);
    if (space === "srgb") channels = [x * 255, y * 255, z * 255];
    else if (space === "srgb-linear") channels = rgb([x, y, z]);
    else if (space === "display-p3") channels = rgb(fromXYZ(times(P3, [srgb(x), srgb(y), srgb(z)])));
    else if (space === "a98-rgb") channels = rgb(fromXYZ(times(A98, [x, y, z].map((v) => Math.sign(v) * Math.abs(v) ** (563 / 256)))));
    else if (space === "rec2020") {
      // (Its curve as browsers draw it (Chromium's pixels match it), not the 2.4 gamma CSS Color 4 moved to.)
      const f = (/** @type {number} */ v) => Math.sign(v) * (Math.abs(v) < 0.081242858298635 ? Math.abs(v) / 4.5 : ((Math.abs(v) + 0.0992968268094) / 1.0992968268094) ** (1 / 0.45));
      channels = rgb(fromXYZ(times(REC2020, [f(x), f(y), f(z)])));
    } else if (space === "prophoto-rgb") {
      const f = (/** @type {number} */ v) => Math.sign(v) * (Math.abs(v) <= 16 / 512 ? Math.abs(v) / 16 : Math.abs(v) ** 1.8);
      channels = rgb(fromXYZ(d50to65(times(PROPHOTO, [f(x), f(y), f(z)]))));
    } else if (space === "xyz" || space === "xyz-d65") channels = rgb(fromXYZ([x, y, z]));
    else if (space === "xyz-d50") channels = rgb(fromXYZ(d50to65([x, y, z])));
  }
  return [...channels.map((v) => Math.round(Math.min(255, Math.max(0, v)))), Math.round(Math.min(1, Math.max(0, alpha)) * 255)];
}

const P3 = [0.4865709486482162, 0.26566769316909306, 0.1982172852343625, 0.2289745640697488, 0.6917385218365064, 0.079286914093745, 0, 0.04511338185890264, 1.043944368900976];
const A98 = [0.5766690429101305, 0.1855582379065463, 0.1882286462349947, 0.29734497525053605, 0.6273635662554661, 0.07529145849399788, 0.02703136138641234, 0.07068885253582723, 0.9913375368376388];
const REC2020 = [0.6369580483012914, 0.14461690358620832, 0.1688809751641721, 0.2627002120112671, 0.6779980715188708, 0.05930171646986196, 0, 0.028072693049087428, 1.060985057710791];
const PROPHOTO = [0.7977604896723027, 0.13518583717574031, 0.0313493495815248, 0.2880711282292934, 0.7118432178101014, 0.00008565396060525902, 0, 0, 0.8251046025104601];
const D50TO65 = [0.955473421488075, -0.02309845494876471, 0.06325924320057072, -0.0283697093338637, 1.0099953980813041, 0.021041441191917323, 0.012314014864481998, -0.020507649298898964, 1.330365926242124];
const TOSRGB = [3.2409699419045226, -1.537383177570094, -0.4986107602930034, -0.9692436362808796, 1.8759675015077202, 0.04155505740717559, 0.05563007969699366, -0.20397695888897652, 1.0569715142428786];
/** A 3x3 matrix (by rows) times a vector. @param {number[]} m @param {number[]} v */
const times = (m, v) => [m[0] * v[0] + m[1] * v[1] + m[2] * v[2], m[3] * v[0] + m[4] * v[1] + m[5] * v[2], m[6] * v[0] + m[7] * v[1] + m[8] * v[2]];
/** @param {number[]} v */
const d50to65 = (v) => times(D50TO65, v);
/** @param {number[]} v */
const fromXYZ = (v) => times(TOSRGB, v);

/** sRGB channels (0 to 255) as OKLab. @param {ArrayLike<number>} rgb */
function oklab(rgb) {
  const f = (/** @type {number} */ v) => ((v /= 255) <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4), r = f(rgb[0]), g = f(rgb[1]), b = f(rgb[2]);
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b), m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b), s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return [0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s, 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s, 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s];
}

/** OKLab as linear sRGB. @param {number} L @param {number} a @param {number} b */
function lin(L, a, b) {
  const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3, m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3, s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s, -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s, -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s];
}

/** Whether linear sRGB channels are within sRGB. @param {number[]} c */
function fits(c) {
  return c.every((v) => v >= -1e-4 && v <= 1 + 1e-4);
}

/** sRGB channels (0 to 255) as #rrggbb. @param {number[]} c */
function hex(c) {
  return "#" + c.map((v) => v.toString(16).padStart(2, "0")).join("");
}

/** @type {Map<string, string[]>} */
const blends = new Map();

/**
 * Worked out once a pair, so a gradient made anew as its scale refits costs only its stops.
 * @param {string} from @param {string} to
 */
function blended(from, to) {
  const key = from + "|" + to;
  let stops = blends.get(key);
  if (stops === undefined) {
    stops = gradientTable([[0, from], [100, to]], 9);
    if (blends.size > 256) blends.clear();
    blends.set(key, stops);
  }
  return stops;
}

export class Gradient {
  /** @param {readonly string[]} colors @param {number} lo @param {number} hi */
  constructor(colors, lo, hi) {
    // (Never changed: its channels and Fill's styles of it are kept by its identity.)
    /** @readonly */
    this.colors = colors;
    /** @readonly */
    this.lo = lo;
    /** @readonly */
    this.hi = hi;
    /** @type {number[][] | null} */
    this.rgba = null;
  }

  /** @param {number} v */
  index(v) {
    return placeIndex(this.hi > this.lo ? (v - this.lo) / (this.hi - this.lo) : 0.5, this.colors.length);
  }

  /** @param {number} v */
  color(v) {
    return v === v ? this.colors[this.index(v)] : null;
  }

  /** @param {Float64Array} values @param {Uint16Array | null} out */
  indexes(values, out) {
    const n = values.length, into = out && out.length === n ? out : new Uint16Array(n);
    for (let i = 0; i < n; i++) into[i] = values[i] === values[i] ? this.index(values[i]) : 0;
    return into;
  }

  channels() {
    return (this.rgba ??= this.colors.map(read));
  }
}

/** @typedef {{ value: number, color: string }} Zone */

/**
 * As triples into `out`, bottom up. A value under the foot or unshowable holds all from there up; over the top, none.
 * @param {readonly { value: number }[]} zones @param {{ y: (v: number) => number }} scale @param {number} height @param {number[]} out
 */
export function zoneSpans(zones, scale, height, out) {
  out.length = 0;
  let under = -1, foot = height;
  for (let k = 0; k < zones.length; k++) {
    const at = scale.y(zones[k].value);
    if (at !== at || at >= height) {
      under = k;
      continue;
    }
    if (at <= 0) break;
    out.push(under, at, foot);
    (foot = at), (under = k);
  }
  out.push(under, 0, foot);
  return out;
}

/** Kept till its colors or heights change. */
export class Fill {
  constructor() {
    /** @type {CanvasGradient | null} */
    this.gradient = null;
    this.key = "";
    /** @type {number[]} */
    this.spans = [];
    /** @type {CanvasGradient | string | null} */
    this.byValue = null;
    /** @type {Gradient | null} */
    this.of = null;
    /** @type {Gradient | null} */
    this.fadeOf = null;
    (this.seen = NaN), (this.ya = NaN), (this.yb = NaN), (this.ym = NaN);
    this.ys = new Float64Array(0);
  }

  /**
     * Made again only as either gradient, the height or the scale change, told by three heights read a draw.
     * @param {CanvasRenderingContext2D} g @param {Gradient} gradient @param {{ y: (v: number) => number }} scale @param {number} height @param {Gradient | null} [fade]
     * @returns {CanvasGradient | string}
     */
  graded(g, gradient, scale, height, fade = null) {
    const lo = gradient.lo, hi = gradient.hi, ya = scale.y(lo), yb = scale.y(hi), ym = scale.y(hi > 0 ? hi / 2 : (lo + hi) / 2);
    if (this.byValue && this.of === gradient && this.fadeOf === fade && this.seen === height && Object.is(this.ya, ya) && Object.is(this.yb, yb) && Object.is(this.ym, ym)) return this.byValue;
    (this.of = gradient), (this.fadeOf = fade), (this.seen = height), (this.ya = ya), (this.yb = yb), (this.ym = ym);
    const colors = gradient.colors, n = colors.length, mid = placeIndex(0.5, n);
    if (!(hi > lo)) return (this.byValue = fade ? this.style(g, colors[mid], fade.colors[mid], 0, height) : colors[mid]);
    if (!(height > 0)) return (this.byValue = colors[mid]);
    const ys = this.ys.length >= n ? this.ys : (this.ys = new Float64Array(n));
    let f = -1, l = -1;
    for (let k = 0; k < n; k++) {
      const y = (ys[k] = scale.y(n > 1 ? lo + ((hi - lo) * k) / (n - 1) : lo));
      if (y - y === 0) (f = f < 0 ? k : f), (l = k);
    }
    if (f < 0) return (this.byValue = colors[0]);
    const top = gradient.channels(), foot = fade ? fade.channels() : null, s = ys[l] > ys[f] ? 1 : -1, made = g.createLinearGradient(0, 0, 0, height);
    // (By pixels, as the canvas mixes; past them, the end's.)
    const at = (/** @type {number} */ y) => {
      if (f === l || (y - ys[f]) * s <= 0) return f;
      if ((y - ys[l]) * s >= 0) return l;
      let k = f;
      while (k + 1 < l && (ys[k + 1] - y) * s <= 0) k++;
      const d = ys[k + 1] - ys[k];
      return d ? k + (y - ys[k]) / d : k;
    };
    const mix = (/** @type {number} */ p, /** @type {number} */ down, /** @type {number} */ c) => {
      const k = Math.floor(p), t = p - k, j = Math.min(k + 1, n - 1), over = top[k][c] + (top[j][c] - top[k][c]) * t;
      return foot ? over + (foot[k][c] + (foot[j][c] - foot[k][c]) * t - over) * down : over;
    };
    const stop = (/** @type {number} */ y, /** @type {number} */ p, down = y / height) =>
      made.addColorStop(down, `rgba(${Math.round(mix(p, down, 0))}, ${Math.round(mix(p, down, 1))}, ${Math.round(mix(p, down, 2))}, ${(mix(p, down, 3) / 255).toFixed(3)})`);
    stop(0, at(0));
    for (let k = f; k <= l; k++) if (ys[k] > 0 && ys[k] < height) stop(ys[k], k);
    stop(height, at(height));
    return (this.byValue = made);
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {string} color @param {string | null} fade @param {number} top @param {number} bottom
   */
  style(g, color, fade, top, bottom) {
    if (!fade || fade === color || !(bottom > top)) return color;
    return this.#made(g, color, fade, top, bottom);
  }

  /**
   * A zone's value its scale can't show (a log scale's 0 or under) holds all of it.
   * @param {CanvasRenderingContext2D} g @param {string} own @param {readonly Zone[]} zones @param {{ y: (v: number) => number }} scale @param {number} height
   * @returns {string | CanvasGradient}
   */
  zoned(g, own, zones, scale, height) {
    const spans = zoneSpans(zones, scale, height, this.spans), colorOf = (/** @type {number} */ k) => (k < 0 ? own : zones[k].color);
    let key = `${height}`, one = true;
    for (let s = 0; s < spans.length; s += 3) (key += `|${colorOf(spans[s])}|${spans[s + 1]}`), (one &&= colorOf(spans[s]) === colorOf(spans[0]));
    if (one || !(height > 0)) return colorOf(spans[0]);
    if (!this.gradient || this.key !== key) {
      const gradient = g.createLinearGradient(0, 0, 0, height);
      // (Drawn, not measured: a measure strokes each zone apart.)
      for (let s = spans.length - 3; s >= 0; s -= 3) gradient.addColorStop(spans[s + 1] / height, colorOf(spans[s])), gradient.addColorStop(spans[s + 2] / height, colorOf(spans[s]));
      (this.gradient = gradient), (this.key = key);
    }
    return this.gradient;
  }

  /** @param {CanvasRenderingContext2D} g @param {string} from @param {string} to @param {number} top @param {number} bottom */
  #made(g, from, to, top, bottom) {
    const key = `${from}|${to}|${top}|${bottom}`;
    if (!this.gradient || this.key !== key) {
      const gradient = g.createLinearGradient(0, top, 0, bottom);
      // (Mixed in OKLCH, not the canvas's sRGB.)
      const stops = blended(from, to);
      for (let k = 0; k < stops.length; k++) gradient.addColorStop(k / (stops.length - 1), stops[k]);
      const above = painted(from), below = painted(to);
      gradientPaint.set(gradient, above || below ? [above ? -Infinity : top, below ? Infinity : bottom] : [Infinity, -Infinity]);
      (this.gradient = gradient), (this.key = key);
    }
    return this.gradient;
  }
}

/** @param {any} style */
export function painted(style) {
  if (typeof style !== "string") {
    const bounds = gradientPaint.get(style);
    return !bounds || bounds[0] < bounds[1];
  }
  let seen = paints.get(style);
  if (seen === undefined) {
    pen ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
    pen.fillStyle = "#000";
    pen.fillStyle = style;
    const read = String(pen.fillStyle);
    seen = !(/,\s*0\)$/.test(read) || /\/\s*0\)$/.test(read));
    if (paints.size > 512) paints.clear();
    paints.set(style, seen);
  }
  return seen;
}
/** @type {Map<string, boolean>} */
const paints = new Map();

/**
 * Unknown styles leave it as it is.
 * @param {CanvasGradient} style @param {Float64Array} box
 */
export function clipPaint(style, box) {
  const bounds = gradientPaint.get(style);
  if (bounds) (box[1] = Math.max(box[1], bounds[0])), (box[3] = Math.min(box[3], bounds[1]));
}
/** Their painted heights: no pixel reads. @type {WeakMap<CanvasGradient, number[]>} */
const gradientPaint = new WeakMap();
