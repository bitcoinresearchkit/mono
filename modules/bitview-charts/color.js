// @ts-check
// Words on a color: white or black, by its lightness (where APCA turns for a gray: see whiteReads), each color measured
// once; a series' tag (its fill, its words). And a fill's or a stroke's style as a vertical gradient (a
// fade, or a color split at a height), kept till it changes.

const ink = new Map();
/** A canvas a color is written by (any CSS color: a name, hex, rgb(), oklch()), read from its writing. @type {CanvasRenderingContext2D | null} */
let pen = null;

const solids = new Map();

/**
 * `color` without its alpha (a tag's ground, as the chart library's: a line drawn at half opacity tagged in its full
 * color), each color read once.
 * @param {string} color
 */
export function solid(color) {
  let full = solids.get(color);
  if (full !== undefined) return full;
  pen ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
  // (As a canvas writes it back: #rrggbb opaque, rgba(…) with an alpha, other spaces in their own form with "/ alpha"
  // (oklch, display-p3): that alpha dropped, exactly.)
  pen.fillStyle = "#000";
  pen.fillStyle = color;
  // (None at all (transparent): as it is, nothing to show.)
  const read = String(pen.fillStyle), m = /^rgba\((\d+), (\d+), (\d+), ([^)]+)\)$/.exec(read);
  const slash = m ? null : /\s*\/\s*([^)/]+)\)$/.exec(read), alpha = m ? Number(m[4]) : slash ? parseFloat(slash[1]) / (slash[1].trim().endsWith("%") ? 100 : 1) : 1;
  full = !(alpha > 0) ? color : m ? `rgb(${m[1]}, ${m[2]}, ${m[3]})` : slash ? read.slice(0, slash.index) + ")" : read;
  if (solids.size > 512) solids.clear();
  solids.set(color, full);
  return full;
}

/** White or black, whichever reads better on `color` (see whiteReads). @param {string} color */
export function inkOn(color) {
  let text = ink.get(color);
  if (text !== undefined) return text;
  text = whiteReads(rgb(read(color))) ? "#ffffff" : "#000000";
  if (ink.size > 512) ink.clear();
  ink.set(color, text);
  return text;
}

const tints = new Map();

/**
 * A tag's look for a series in `color`: its fill that very color (the line's); its words white or black by the fill's
 * lightness (see whiteReads), each given back a whisper of the fill's hue in OKLCH (lightness 0.97 or 0.24, chroma
 * 0.06 at most, the fill's hue: as much as sRGB allows there). No edge: a series the ground's color is the page's
 * choice. A see-through color, none (nothing shown). Each color worked out once.
 * @param {string} color
 * @returns {{ fill: string, ink: string }}
 */
export function tint(color) {
  let look = tints.get(color);
  if (look !== undefined) return look;
  const seen = read(color);
  if (seen[3] === 0) look = { fill: "transparent", ink: "transparent" };
  else {
    const fill = rgb(seen), [, a0, b0] = oklab(seen), hue = Math.atan2(b0, a0), chroma = Math.min(0.06, Math.hypot(a0, b0)), light = whiteReads(fill);
    // (Its hue at the ink's lightness, its chroma the most sRGB allows there, 0.06 at most.)
    const l = light ? 0.97 : 0.24;
    let lo = 0, hi = chroma;
    if (fits(lin(l, hi * Math.cos(hue), hi * Math.sin(hue)))) lo = hi;
    else
      for (let k = 0; k < 24; k++) {
        const mid = (lo + hi) / 2;
        if (fits(lin(l, mid * Math.cos(hue), mid * Math.sin(hue)))) lo = mid;
        else hi = mid;
      }
    look = { fill: hex(fill), ink: hex(rgb(lin(l, lo * Math.cos(hue), lo * Math.sin(hue)))) };
  }
  if (tints.size > 512) tints.clear();
  tints.set(color, look);
  return look;
}

/**
 * Whether white words read better than black on `back` (sRGB channels, 0 to 255): by its lightness (OKLab), white under
 * 0.7157 (where APCA turns for a gray, #a4a4a4), so a color of one lightness goes one way whatever its hue (APCA itself
 * gave a light salmon or lavender white words; WCAG 2's ratio gets mid blues and purples wrong). Studio's own words
 * on a color the same.
 * @param {number[]} back
 */
function whiteReads(back) {
  return oklab(back)[0] < 0.7157;
}

/** sRGB channels (0 to 255) from linear ones, or a read color's first three. @param {ArrayLike<number>} c */
function rgb(c) {
  return c.length === 4 ? [c[0], c[1], c[2]] : Array.from(c, (v) => Math.round(255 * (Math.min(1, Math.max(0, v)) <= 0.0031308 ? 12.92 * Math.min(1, Math.max(0, v)) : 1.055 * Math.min(1, Math.max(0, v)) ** (1 / 2.4) - 0.055)));
}

/**
 * A gradient's 256 colors (RGBA, 0 to 255, a color each 4), its `stops` (any CSS colors, evenly apart) blended in OKLab
 * (no muddy midpoints: a hue's lightness and chroma move together), their opacities straight (a see-through stop the
 * color of the stop it fades to or from: a fade not through black, nor through another stop's hue); one stop, its color.
 * @param {string[]} stops
 */
export function gradientTable(stops) {
  const table = new Uint8ClampedArray(256 * 4), labs = stops.map((stop) => [...oklab(read(stop)), read(stop)[3]]), last = labs.length - 1;
  for (let k = 0; k < 256; k++) {
    const at = last > 0 ? (k / 255) * last : 0, i = Math.min(Math.max(0, last - 1), Math.floor(at)), t = last > 0 ? at - i : 0, a = labs[i], b = labs[Math.min(i + 1, last)];
    // (A see-through end of the segment its other end's color: each visible stop fades in its own.)
    const from = a[3] === 0 ? b : a, to = b[3] === 0 ? a : b, c = rgb(lin(from[0] + (to[0] - from[0]) * t, from[1] + (to[1] - from[1]) * t, from[2] + (to[2] - from[2]) * t));
    (table[k * 4] = c[0]), (table[k * 4 + 1] = c[1]), (table[k * 4 + 2] = c[2]), (table[k * 4 + 3] = a[3] + (b[3] - a[3]) * t);
  }
  return table;
}

/**
 * A color's sRGB channels and opacity (0 to 255), as the canvas reads it: from the canvas's own writing of it (#rrggbb,
 * rgb() or rgba(), oklch(), oklab(), lab(), lch(), color() in any of CSS's spaces: any CSS color comes back as one,
 * wide-gamut ones clipped to sRGB), never from
 * a pixel drawn and read (a browser resisting fingerprinting, as Mullvad does, gives a read pixel noise). Anything else,
 * black.
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

/** A canvas's writing of a color as its sRGB channels and opacity (0 to 255; see read). @param {string} text */
function parsed(text) {
  if (text[0] === "#") {
    const hex = (/** @type {number} */ at) => parseInt(text.slice(at, at + 2), 16);
    return [hex(1), hex(3), hex(5), text.length > 7 ? hex(7) : 255];
  }
  const call = /^([a-z0-9-]+)\((.*)\)$/.exec(text.trim());
  if (!call) return [0, 0, 0, 255];
  const [, name, body] = call, parts = body.replace("/", " / ").split(/[\s,]+/).filter(Boolean), slash = parts.indexOf("/");
  // (A number, a percentage of `whole`, or none.)
  const num = (/** @type {string | undefined} */ part, /** @type {number} */ whole) => (part === undefined || part === "none" ? 0 : part.endsWith("%") ? (parseFloat(part) / 100) * whole : parseFloat(part));
  const values = slash >= 0 ? parts.slice(0, slash) : parts, alpha = slash >= 0 ? num(parts[slash + 1], 1) : (name === "rgba" || name === "rgb") && parts.length === 4 ? num(parts[3], 1) : 1;
  /** @type {number[]} */
  let channels = [0, 0, 0];
  if (name === "rgb" || name === "rgba") channels = values.slice(0, 3).map((part) => num(part, 255));
  else if (name === "oklch" || name === "oklab") {
    const l = num(values[0], 1), first = num(values[1], 0.4), second = name === "oklch" ? (num(values[2], 360) * Math.PI) / 180 : num(values[2], 0.4);
    channels = rgb(name === "oklch" ? lin(l, first * Math.cos(second), first * Math.sin(second)) : lin(l, first, second));
  } else if (name === "lab" || name === "lch") {
    // (CIE Lab (D50): to XYZ, adapted to D65, to linear sRGB.)
    const l = num(values[0], 100), first = num(values[1], name === "lab" ? 125 : 150), second = name === "lch" ? (num(values[2], 360) * Math.PI) / 180 : num(values[2], 125);
    const a = name === "lab" ? first : first * Math.cos(second), b = name === "lab" ? second : first * Math.sin(second), fy = (l + 16) / 116, fx = fy + a / 500, fz = fy - b / 200;
    const inv = (/** @type {number} */ t) => (t ** 3 > 216 / 24389 ? t ** 3 : (116 * t - 16) / (24389 / 27));
    channels = rgb(fromXYZ(d50to65([0.3457 / 0.3585 * inv(fx), l > 8 ? fy ** 3 : l / (24389 / 27), ((1 - 0.3457 - 0.3585) / 0.3585) * inv(fz)])));
  } else if (name === "color") {
    const space = values[0], [x, y, z] = values.slice(1, 4).map((part) => num(part, 1));
    // (sRGB's curve, and Display P3's and A98's (as sRGB's; A98's a power) undone to linear.)
    const srgb = (/** @type {number} */ v) => Math.sign(v) * (Math.abs(v) <= 0.04045 ? Math.abs(v) / 12.92 : ((Math.abs(v) + 0.055) / 1.055) ** 2.4);
    if (space === "srgb") channels = [x * 255, y * 255, z * 255];
    else if (space === "srgb-linear") channels = rgb([x, y, z]);
    else if (space === "display-p3") channels = rgb(fromXYZ(times(P3, [srgb(x), srgb(y), srgb(z)])));
    else if (space === "a98-rgb") channels = rgb(fromXYZ(times(A98, [x, y, z].map((v) => Math.sign(v) * Math.abs(v) ** (563 / 256)))));
    else if (space === "rec2020") {
      // (Its curve as browsers draw it (Chromium's pixels match this one, 2026), not the 2.4 gamma CSS Color 4 moved to.)
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

// (CSS Color 4's matrices: each space's linear channels to XYZ (D65; ProPhoto's D50), D50 to D65, XYZ to linear sRGB.)
const P3 = [0.4865709486482162, 0.26566769316909306, 0.1982172852343625, 0.2289745640697488, 0.6917385218365064, 0.079286914093745, 0, 0.04511338185890264, 1.043944368900976];
const A98 = [0.5766690429101305, 0.1855582379065463, 0.1882286462349947, 0.29734497525053605, 0.6273635662554661, 0.07529145849399788, 0.02703136138641234, 0.07068885253582723, 0.9913375368376388];
const REC2020 = [0.6369580483012914, 0.14461690358620832, 0.1688809751641721, 0.2627002120112671, 0.6779980715188708, 0.05930171646986196, 0, 0.028072693049087428, 1.060985057710791];
const PROPHOTO = [0.7977604896723027, 0.13518583717574031, 0.0313493495815248, 0.2880711282292934, 0.7118432178101014, 0.00008565396060525902, 0, 0, 0.8251046025104601];
const D50TO65 = [0.955473421488075, -0.02309845494876471, 0.06325924320057072, -0.0283697093338637, 1.0099953980813041, 0.021041441191917323, 0.012314014864481998, -0.020507649298898964, 1.330365926242124];
const TOSRGB = [3.2409699419045226, -1.537383177570094, -0.4986107602930034, -0.9692436362808796, 1.8759675015077202, 0.04155505740717559, 0.05563007969699366, -0.20397695888897652, 1.0569715142428786];
/** A 3x3 matrix (by rows) times a vector. @param {number[]} m @param {number[]} v */
const times = (m, v) => [m[0] * v[0] + m[1] * v[1] + m[2] * v[2], m[3] * v[0] + m[4] * v[1] + m[5] * v[2], m[6] * v[0] + m[7] * v[1] + m[8] * v[2]];
/** XYZ (D50) as XYZ (D65). @param {number[]} v */
const d50to65 = (v) => times(D50TO65, v);
/** XYZ (D65) as linear sRGB. @param {number[]} v */
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

/** Two colors' blends, by the pair. @type {Map<string, string[]>} */
const blends = new Map();

/**
 * A fade's stops from `from` to `to`, nine evenly apart, blended in OKLab (see gradientTable): worked out once a pair,
 * so a gradient made anew (its heights changed: a baseline's base as its scale refits) costs only its stops.
 * @param {string} from @param {string} to
 */
function blended(from, to) {
  const key = from + "|" + to;
  let stops = blends.get(key);
  if (stops === undefined) {
    const table = gradientTable([from, to]);
    stops = Array.from({ length: 9 }, (_, k) => {
      const at = Math.round((k / 8) * 255) * 4;
      return `rgba(${table[at]}, ${table[at + 1]}, ${table[at + 2]}, ${(table[at + 3] / 255).toFixed(3)})`;
    });
    if (blends.size > 256) blends.clear();
    blends.set(key, stops);
  }
  return stops;
}

/** A fill's or a stroke's style: a color, or a vertical gradient (CSS pixels) kept till its colors or heights change. */
export class Fill {
  constructor() {
    /** @type {CanvasGradient | null} */
    this.gradient = null;
    this.from = "";
    this.to = "";
    this.top = NaN;
    this.bottom = NaN;
    this.split = false;
  }

  /**
   * `color`, or where it fades to `fade` (none: none), a gradient from height `top` to `bottom`.
   * @param {CanvasRenderingContext2D} g @param {string} color @param {string | null} fade @param {number} top @param {number} bottom
   */
  style(g, color, fade, top, bottom) {
    if (!fade || fade === color || !(bottom > top)) return color;
    return this.#made(g, color, fade, top, bottom, false);
  }

  /**
   * `color` above height `at` and `under` below it, of a canvas `height` tall (a baseline's line: its color changing
   * where it crosses its base).
   * @param {CanvasRenderingContext2D} g @param {string} color @param {string} under @param {number} at @param {number} height
   */
  splitAt(g, color, under, at, height) {
    if (under === color || !(height > 0)) return color;
    return this.#made(g, color, under, Math.min(Math.max(at / height, 0), 1), height, true);
  }

  /** @param {CanvasRenderingContext2D} g @param {string} from @param {string} to @param {number} top @param {number} bottom @param {boolean} split */
  #made(g, from, to, top, bottom, split) {
    if (!this.gradient || this.from !== from || this.to !== to || this.top !== top || this.bottom !== bottom || this.split !== split) {
      const gradient = split ? g.createLinearGradient(0, 0, 0, bottom) : g.createLinearGradient(0, top, 0, bottom);
      if (split) gradient.addColorStop(0, from), gradient.addColorStop(top, from), gradient.addColorStop(top, to), gradient.addColorStop(1, to);
      else {
        // (Blended in OKLab, not the canvas's sRGB: nine stops, an eighth apart.)
        const stops = blended(from, to);
        for (let k = 0; k < stops.length; k++) gradient.addColorStop(k / (stops.length - 1), stops[k]);
      }
      (this.gradient = gradient), (this.from = from), (this.to = to), (this.top = top), (this.bottom = bottom), (this.split = split);
    }
    return this.gradient;
  }
}
