// @ts-check
// Words on a color: black or white by its lightness as OKLab measures it (black from L 0.7157, where APCA flips for a
// grey: a hue then reads as a grey of its lightness does), each color measured once.

const ink = new Map();
/** A canvas a color is read back from (any CSS color: a name, hex, rgb(), oklch()). @type {CanvasRenderingContext2D | null} */
let pen = null;

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
