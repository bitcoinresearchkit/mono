// @ts-check
// A page's own tags on an axis (a drawing's value on a price axis, its date on the time axis): gathered each time the
// axis is drawn from its pane's series that give some (see Custom's axisTags and timeTags), drawn as a series' tags
// are, the axis' labels left out under them.

import { inkOn } from "./color.js";

/** A pill's height (CSS pixels): a tag's, a badge's. */
export const PILL = 18;

/**
 * A pill (every tag and badge the chart draws: its series' and a page's on the axes, the crosshair's): `words` on
 * `color` in `ink`, as wide as its words and 6 pixels each side, `height` tall (its corners 4, less where it's lower), its
 * middle at `y`; its left at `x` (`align` "left"), its right ("right") or its middle ("center"). Too low to read (under
 * 10 pixels: crowded tags), its color alone. With a `border`, edged in it (a pixel, inside). Its width returned.
 * @param {CanvasRenderingContext2D} g @param {string} words @param {string} color @param {string} ink @param {number} x
 * @param {number} y @param {"left" | "right" | "center"} align @param {number} [height] @param {string} [border]
 */
export function pill(g, words, color, ink, x, y, align, height = PILL, border = "") {
  const width = g.measureText(words).width + 12, left = align === "left" ? x : align === "right" ? x - width : x - width / 2, radius = Math.max(0, Math.min(4, height / 2));
  g.fillStyle = color;
  g.beginPath();
  g.roundRect(left, y - height / 2, width, height, radius);
  g.fill();
  if (border && height >= 4) {
    g.strokeStyle = border;
    g.lineWidth = 1;
    g.beginPath();
    g.roundRect(left + 0.5, y - height / 2 + 0.5, width - 1, height - 1, Math.max(0, radius - 0.5));
    g.stroke();
  }
  if (height < 10) return width;
  g.fillStyle = ink;
  g.textAlign = "center";
  g.fillText(words, left + width / 2, y);
  return width;
}

export class Tags {
  constructor() {
    /** Each one's place along the axis (CSS pixels), words, color, and its words' (`count` of them). */
    this.at = new Float64Array(8);
    /** @type {string[]} */
    this.texts = [];
    /** @type {string[]} */
    this.colors = [];
    /** @type {string[]} */
    this.inks = [];
    this.count = 0;
    /**
     * One added: at `at`, its words `text`, on `color` (in `ink`, else black or white by it); none where it's not a
     * number. Made once: given to every series asked.
     * @param {number} at @param {string} text @param {string} color @param {string} [ink]
     */
    this.add = (at, text, color, ink) => {
      if (at !== at || at === Infinity || at === -Infinity) return;
      if (this.count >= this.at.length) ((grown) => (grown.set(this.at), (this.at = grown)))(new Float64Array(this.at.length * 2));
      const k = this.count++;
      (this.at[k] = at), (this.texts[k] = text), (this.colors[k] = color), (this.inks[k] = ink || inkOn(color));
    };
  }

  /** Whether one's within `near` of `at`. @param {number} at @param {number} near */
  covers(at, near) {
    for (let k = 0; k < this.count; k++) if (Math.abs(this.at[k] - at) < near) return true;
    return false;
  }
}
