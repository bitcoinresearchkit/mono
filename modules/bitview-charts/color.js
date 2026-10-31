// @ts-check
// Words on a color: black or white by its lightness as OKLab measures it (black from L 0.7157, where APCA flips for a
// grey: a hue then reads as a grey of its lightness does), each color measured once. And a fill's or a stroke's style
// as a vertical gradient (a fade, or a color split at a height), kept till it changes.

const ink = new Map();
/** A canvas a color is read back from (any CSS color: a name, hex, rgb(), oklch()). @type {CanvasRenderingContext2D | null} */
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
  pen ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d", { willReadFrequently: true }));
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

/** Black or white, whichever reads on `color`. @param {string} color */
export function inkOn(color) {
  let text = ink.get(color);
  if (text !== undefined) return text;
  pen ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d", { willReadFrequently: true }));
  pen.clearRect(0, 0, 1, 1);
  pen.fillStyle = "#000";
  pen.fillStyle = color;
  pen.fillRect(0, 0, 1, 1);
  const [r, g, b] = pen.getImageData(0, 0, 1, 1).data, lin = (/** @type {number} */ v) => ((v /= 255) <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  const R = lin(r), G = lin(g), B = lin(b);
  const l = Math.cbrt(0.4122214708 * R + 0.5363325363 * G + 0.0514459929 * B), m = Math.cbrt(0.2119034982 * R + 0.6806995451 * G + 0.1073969566 * B), s = Math.cbrt(0.0883024619 * R + 0.2817188376 * G + 0.6299787005 * B);
  text = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s < 0.7157 ? "#ffffff" : "#000000";
  if (ink.size > 512) ink.clear();
  ink.set(color, text);
  return text;
}

const tints = new Map();

/**
 * A tag's look for a series in `color` on `ground` (the axis'): its fill its color (lightness moved away from the ground
 * till it stands 4.5:1 off it: a dark series' lifted on a dark ground, a light one's deepened on a light one) 30% of the
 * way from the ground, a tint as visible for a dark series as a bright one; its words that color moved on till they read
 * 7:1 against the fill. Its hue kept throughout, its chroma lessened only as far as each lightness tried needs to stay in
 * sRGB. Moved lighter on a dark ground, darker on a light one; where that can't read (a mid-grey ground), its words, then
 * its fill too, the other way, whichever reads best. A see-through color, none (nothing shown). Each pair worked out once.
 * @param {string} color @param {string} ground
 */
export function tint(color, ground) {
  const key = color + "|" + ground;
  let look = tints.get(key);
  if (look !== undefined) return look;
  const seen = read(color);
  if (seen[3] === 0) look = { ink: "transparent", fill: "transparent" };
  else {
    const g = oklab(read(ground)), floor = lin(g[0], g[1], g[2]), [l0, a0, b0] = oklab(seen), hue = Math.atan2(b0, a0), chroma = Math.hypot(a0, b0);
    // (At a lightness, its color: its chroma the most sRGB allows there, its own at most.)
    const at = (/** @type {number} */ l) => {
      let lo = 0, hi = chroma;
      if (fits(lin(l, hi * Math.cos(hue), hi * Math.sin(hue)))) lo = hi;
      else
        for (let k = 0; k < 24; k++) {
          const mid = (lo + hi) / 2;
          if (fits(lin(l, mid * Math.cos(hue), mid * Math.sin(hue)))) lo = mid;
          else hi = mid;
        }
      return [l, lo * Math.cos(hue), lo * Math.sin(hue)];
    };
    // (Its lightness moved by `step` from `l` till `reads` it, or as far as it goes.)
    const away = (/** @type {number} */ l, /** @type {number} */ step, /** @type {(c: number[]) => boolean} */ reads) => {
      let c = at(l);
      for (let k = 0; k < 100 && !reads(c) && (step > 0 ? l < 1 : l > 0); k++) (l = Math.min(1, Math.max(0, l + step))), (c = at(l));
      return c;
    };
    // (Its fill, its color moved one way; its words moved on that way, or back the other where that reads better: their
    // contrast with them.)
    const made = (/** @type {number} */ step) => {
      const base = away(l0, step, (c) => contrast(lin(c[0], c[1], c[2]), floor) >= 4.5), fill = lin(g[0] + (base[0] - g[0]) * 0.3, g[1] + (base[1] - g[1]) * 0.3, g[2] + (base[2] - g[2]) * 0.3);
      const words = (/** @type {number} */ way) => ((c) => lin(c[0], c[1], c[2]))(away(base[0], way, (c) => contrast(lin(c[0], c[1], c[2]), fill) >= 7));
      const on = words(step), back = contrast(on, fill) >= 7 ? on : words(-step), ink = contrast(back, fill) > contrast(on, fill) ? back : on;
      return { ink, fill, reads: contrast(ink, fill) };
    };
    const first = made(g[0] < 0.5 ? 0.01 : -0.01), other = first.reads >= 7 ? first : made(g[0] < 0.5 ? -0.01 : 0.01), best = other.reads > first.reads ? other : first;
    look = { ink: hex(best.ink), fill: hex(best.fill) };
  }
  if (tints.size > 512) tints.clear();
  tints.set(key, look);
  return look;
}

/** Two linear sRGB colors' contrast (WCAG's). @param {number[]} x @param {number[]} y */
function contrast(x, y) {
  const a = luminance(x), b = luminance(y);
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

/** A color's sRGB channels, 0 to 255, as a canvas reads it. @param {string} color */
function read(color) {
  pen ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d", { willReadFrequently: true }));
  pen.clearRect(0, 0, 1, 1);
  pen.fillStyle = "#000";
  pen.fillStyle = color;
  pen.fillRect(0, 0, 1, 1);
  return pen.getImageData(0, 0, 1, 1).data;
}

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

/** Linear sRGB's relative luminance (WCAG's). @param {number[]} c */
function luminance(c) {
  const [r, g, b] = c.map((v) => Math.min(1, Math.max(0, v)));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** Linear sRGB as #rrggbb. @param {number[]} c */
function hex(c) {
  return "#" + c.map((v) => Math.round(255 * (Math.min(1, Math.max(0, v)) <= 0.0031308 ? 12.92 * Math.min(1, Math.max(0, v)) : 1.055 * Math.min(1, Math.max(0, v)) ** (1 / 2.4) - 0.055)).toString(16).padStart(2, "0")).join("");
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
      gradient.addColorStop(0, from);
      if (split) gradient.addColorStop(top, from), gradient.addColorStop(top, to);
      gradient.addColorStop(1, to);
      (this.gradient = gradient), (this.from = from), (this.to = to), (this.top = top), (this.bottom = bottom), (this.split = split);
    }
    return this.gradient;
  }
}
