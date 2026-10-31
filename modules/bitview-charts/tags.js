// @ts-check
// A page's own tags on an axis (a drawing's value on a price axis, its date on the time axis): gathered each time the
// axis is drawn from its pane's series that give some (see Custom's axisTags and timeTags), drawn as a series' tags
// are, the axis' labels left out under them.

import { inkOn } from "./color.js";

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
