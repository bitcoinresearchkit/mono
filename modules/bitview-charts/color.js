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
 * A tag's look for a series in `color`: its fill that very color (the line's); its words white or black, whichever APCA
 * finds reads better on it (WCAG 2's ratio gets mid blues and purples wrong), each given back a whisper of the fill's hue
 * in OKLCH (lightness 0.97 or 0.24, chroma 0.06 at most, the fill's hue: as much as sRGB allows there); its edge the
 * fill's hue a step (0.12 in OKLCH lightness) away from `ground` (the axis'): lighter on a dark ground, darker on a light
 * one, a hairline that sets the tag off whatever its fill (a black series on a dark ground, a white one on a light
 * ground). One formula on any ground. A see-through color, none (nothing shown). Each pair worked out once.
 * @param {string} color @param {string} ground
 * @returns {{ fill: string, ink: string, border: string }}
 */
export function tint(color, ground) {
  const key = color + "|" + ground;
  let look = tints.get(key);
  if (look !== undefined) return look;
  const seen = read(color);
  if (seen[3] === 0) look = { fill: "transparent", ink: "transparent", border: "transparent" };
  else {
    const fill = rgb(seen), [l0, a0, b0] = oklab(seen), hue = Math.atan2(b0, a0), chroma = Math.hypot(a0, b0), light = apca([255, 255, 255], fill) >= apca([0, 0, 0], fill);
    // (Its hue at a lightness, its chroma the most sRGB allows there, `most` at most.)
    const at = (/** @type {number} */ l, /** @type {number} */ most) => {
      let lo = 0, hi = most;
      if (fits(lin(l, hi * Math.cos(hue), hi * Math.sin(hue)))) lo = hi;
      else
        for (let k = 0; k < 24; k++) {
          const mid = (lo + hi) / 2;
          if (fits(lin(l, mid * Math.cos(hue), mid * Math.sin(hue)))) lo = mid;
          else hi = mid;
        }
      return hex(rgb(lin(l, lo * Math.cos(hue), lo * Math.sin(hue))));
    };
    const away = oklab(read(ground))[0] < 0.5 ? 0.12 : -0.12;
    look = { fill: hex(fill), ink: at(light ? 0.97 : 0.24, Math.min(0.06, chroma)), border: at(Math.min(1, Math.max(0, l0 + away)), chroma) };
  }
  if (tints.size > 512) tints.clear();
  tints.set(color, look);
  return look;
}

/**
 * APCA's lightness contrast of words in `text` on `back` (sRGB channels, 0 to 255), as a magnitude (Lc, 0 to ~106): how
 * readable they are, light on dark and dark on light alike (WCAG 2's ratio overstates dark colors' contrast).
 * @param {number[]} text @param {number[]} back
 */
function apca(text, back) {
  const y = (/** @type {number[]} */ c) => { const v = 0.2126729 * (c[0] / 255) ** 2.4 + 0.7151522 * (c[1] / 255) ** 2.4 + 0.072175 * (c[2] / 255) ** 2.4; return v > 0.022 ? v : v + (0.022 - v) ** 1.414; };
  const t = y(text), b = y(back);
  if (Math.abs(b - t) < 0.0005) return 0;
  const s = b > t ? (b ** 0.56 - t ** 0.57) * 1.14 : (b ** 0.65 - t ** 0.62) * 1.14;
  return Math.abs(s) < 0.1 ? 0 : (Math.abs(s) - 0.027) * 100;
}

/** sRGB channels (0 to 255) from linear ones, or a read color's first three. @param {ArrayLike<number>} c */
function rgb(c) {
  return c.length === 4 ? [c[0], c[1], c[2]] : Array.from(c, (v) => Math.round(255 * (Math.min(1, Math.max(0, v)) <= 0.0031308 ? 12.92 * Math.min(1, Math.max(0, v)) : 1.055 * Math.min(1, Math.max(0, v)) ** (1 / 2.4) - 0.055)));
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

/** sRGB channels (0 to 255) as #rrggbb. @param {number[]} c */
function hex(c) {
  return "#" + c.map((v) => v.toString(16).padStart(2, "0")).join("");
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
