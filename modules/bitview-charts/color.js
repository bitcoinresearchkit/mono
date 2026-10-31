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
  // (Painted alone on a clear pixel and read back, in any CSS form (oklch, display-p3 too): opaque, as it is; else its
  // red, green and blue.)
  pen.clearRect(0, 0, 1, 1);
  pen.fillStyle = "#000";
  pen.fillStyle = color;
  pen.fillRect(0, 0, 1, 1);
  const [r, g, b, a] = pen.getImageData(0, 0, 1, 1).data;
  full = a === 255 || a === 0 ? color : `rgb(${r}, ${g}, ${b})`;
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
