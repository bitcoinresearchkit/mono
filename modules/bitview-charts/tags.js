// @ts-check

import { inkOn } from "./color.js";

export const PILL = 18;

/**
 * @param {CanvasRenderingContext2D} g @param {string} words @param {string} color @param {string} ink @param {number} x
 * @param {number} y @param {"left" | "right" | "center"} align @param {number} [height] @param {string} [border]
 * @param {readonly [number, number] | null} [within]
 */
export function pill(g, words, color, ink, x, y, align, height = PILL, border = "", within = null) {
  const width = g.measureText(words).width + 12, at = align === "left" ? x : align === "right" ? x - width : x - width / 2, radius = Math.max(0, Math.min(4, height / 2));
  const left = within ? Math.max(within[0], Math.min(at, within[1] - width)) : at;
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
    this.at = new Float64Array(8);
    /** @type {string[]} */
    this.texts = [];
    /** @type {string[]} */
    this.colors = [];
    /** @type {string[]} */
    this.inks = [];
    this.count = 0;
    this.length = Infinity;
    /**
     * Made once: given to every series asked.
     * @param {number} at @param {string} text @param {string} color @param {string} [ink]
     */
    this.add = (at, text, color, ink) => {
      if (!(at > -0.5 && at < this.length + 0.5)) return;
      if (this.count >= this.at.length) ((grown) => (grown.set(this.at), (this.at = grown)))(new Float64Array(this.at.length * 2));
      const k = this.count++;
      (this.at[k] = at), (this.texts[k] = text), (this.colors[k] = color), (this.inks[k] = ink || inkOn(color));
    };
  }

  /** @param {number} at @param {number} near */
  covers(at, near) {
    for (let k = 0; k < this.count; k++) if (Math.abs(this.at[k] - at) < near) return true;
    return false;
  }
}
