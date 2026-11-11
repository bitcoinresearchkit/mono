// @ts-check
// The time axis under the panes: the starts of years, months, days, hours or minutes (whichever leave room enough
// between their labels), each at its first point, its label under it (UTC).

import { kept } from "./format.js";
import { Tags, pill } from "./tags.js";

const DAY = 86400;
/** Labels' least room between them (CSS pixels). */
const ROOM = 72;
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
/** Each unit's steps, from the finest: [seconds a step (about), step in its unit, its unit]. */
const STEPS = /** @type {const} */ ([
  [600, 10, "minute"], [1800, 30, "minute"], [3600, 1, "hour"], [6 * 3600, 6, "hour"], [DAY, 1, "day"], [7 * DAY, 7, "day"], [30 * DAY, 1, "month"],
  [91 * DAY, 3, "month"], [182 * DAY, 6, "month"], [365 * DAY, 1, "year"], [2 * 365 * DAY, 2, "year"], [5 * 365 * DAY, 5, "year"], [10 * 365 * DAY, 10, "year"],
]);

/** A time's UTC year and month (0 to 11), written here: no Date made each frame (Hinnant's civil from days). */
let civilYear = 0, civilMonth = 0, civilDay = 0;
/** @param {number} t seconds */
function civil(t) {
  const z = Math.floor(t / DAY) + 719468, era = Math.floor(z / 146097), doe = z - era * 146097;
  const yoe = Math.floor((doe - Math.floor(doe / 1460) + Math.floor(doe / 36524) - Math.floor(doe / 146096)) / 365);
  const doy = doe - (365 * yoe + Math.floor(yoe / 4) - Math.floor(yoe / 100)), mp = Math.floor((5 * doy + 2) / 153), m = mp < 10 ? mp + 3 : mp - 9;
  civilYear = yoe + era * 400 + (m <= 2 ? 1 : 0);
  civilMonth = m - 1;
  civilDay = doy - Math.floor((153 * mp + 2) / 5) + 1;
}

/** What a label at `t` starts, as a page's writer is told: a year, a month, a day, or none (a time of day). @param {number} t */
function kindOf(t) {
  if ((((t % DAY) + DAY) % DAY) !== 0) return "time";
  civil(t);
  return civilDay !== 1 ? "day" : civilMonth ? "month" : "year";
}

/** The first index whose time is at or past `t` (the length, none). @param {Float64Array} times @param {number} t */
export function at(times, t) {
  let lo = 0, hi = times.length;
  while (lo < hi) {
    const mid = (lo + hi) >>> 1;
    times[mid] < t ? (lo = mid + 1) : (hi = mid);
  }
  return lo;
}

export class TimeAxis {
  constructor() {
    this.canvas = document.createElement("canvas");
    this.canvas.style.cssText = "position: absolute; left: 0; display: block";
    const g = this.canvas.getContext("2d", { alpha: false });
    if (!g) throw new Error("bitview-charts: no 2D canvas");
    this.g = g;
    this.height = 0;
    this.width = 0;
    /** Its labels as last drawn: each one's x and words, `drawnCount` of them (the crosshair's date covers those it touches). */
    this.drawnX = new Float64Array(64);
    /** @type {string[]} */
    this.drawnWords = [];
    this.drawnCount = 0;
    // (Daily labels always include their month; January is named by its year among monthly ticks.)
    const date = (/** @type {number} */ t) => new Date(t * 1000);
    const day = (/** @type {Date} */ d) => `${d.getUTCDate()} ${MONTHS[d.getUTCMonth()]}${d.getUTCDate() === 1 && d.getUTCMonth() === 0 ? ` ${d.getUTCFullYear()}` : ""}`;
    // (A time of day as hours and minutes; midnight, its day.)
    const clock = kept((t) => {
      const d = date(t);
      return d.getUTCHours() || d.getUTCMinutes() ? `${String(d.getUTCHours()).padStart(2, "0")}:${String(d.getUTCMinutes()).padStart(2, "0")}` : day(d);
    });
    this.labels = {
      year: kept((t) => String(date(t).getUTCFullYear())),
      month: kept((t) => (date(t).getUTCMonth() ? MONTHS[date(t).getUTCMonth()] : String(date(t).getUTCFullYear()))),
      day: kept((t) => day(date(t))),
      hour: clock,
      minute: clock,
    };
    /**
     * A page's own labels: each one's words from its time and what it starts ("year", "month", "day" or "time"); "", no
     * label there; null, its own. None: its own all along.
     * @type {((t: number, kind: string) => string | null) | null}
     */
    this.format = null;
    /** The panes' series' own tags on it as last drawn, and each one's half width. */
    this.page = new Tags();
    this.halves = new Float64Array(8);
  }

  /** @param {number} top @param {number} width @param {number} height @param {number} dpr */
  place(top, width, height, dpr) {
    const w = Math.round(width * dpr), h = Math.round(height * dpr);
    if (this.canvas.width !== w || this.canvas.height !== h) (this.canvas.width = w), (this.canvas.height = h);
    Object.assign(this.canvas.style, { top: `${top}px`, width: `${width}px`, height: `${height}px` });
    this.width = width;
    this.height = height;
  }

  /**
   * Drawn for the view: its labels, and the panes' series' own tags (a drawing's date, see Custom's timeTags) over them,
   * a label none where one covers it.
   * @param {import("./timeline.js").Timeline} time @param {import("./pane.js").Theme} theme @param {number} dpr @param {number} plot
   * @param {import("./pane.js").Pane[]} panes
   */
  draw(time, theme, dpr, plot, panes) {
    const g = this.g, page = this.page;
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.fillStyle = theme.background;
    g.fillRect(0, 0, this.width, this.height);
    g.font = theme.font;
    (page.count = 0), (page.length = plot), (this.drawnCount = 0);
    for (const pane of panes) for (let k = 0; k < pane.series.length; k++) if (!pane.series[k].hidden && !pane.series[k].faint) pane.series[k].timeTags?.(page.add);
    // (Each tag's half width, measured once a draw.)
    if (this.halves.length < page.count) this.halves = new Float64Array(page.count * 2);
    // (Each where it's drawn: within the plot (one out of it none, as a value's).)
    for (let k = 0; k < page.count; k++) (this.halves[k] = g.measureText(page.texts[k]).width / 2 + 6), (page.at[k] = Math.min(Math.max(page.at[k], this.halves[k]), plot - this.halves[k]));
    this.#labels(time, theme, plot, page);
    g.textBaseline = "middle";
    for (let k = 0; k < page.count; k++) pill(g, page.texts[k], page.colors[k], page.inks[k], page.at[k], this.height / 2, "center");
  }

  /** Whether a label at `x` overlaps a page's tag, with four pixels between them. @param {number} x @param {Tags} page @param {number} half */
  #covered(x, page, half) {
    for (let k = 0; k < page.count; k++) if (Math.abs(page.at[k] - x) < this.halves[k] + half + 4) return true;
    return false;
  }

  /**
   * Its labels for the view, none under a page's tag: by the finest step leaving room between them; where that one
   * has no start in view (a few days of data in a narrow stretch: no month begins in them), the finer ones in turn, down
   * to its closest points' spacing in view (finer would name no more).
   * @param {import("./timeline.js").Timeline} time @param {import("./pane.js").Theme} theme @param {number} plot @param {Tags} page
   */
  #labels(time, theme, plot, page) {
    const times = time.times, n = times.length;
    if (n < 2) return;
    const a = Math.max(0, Math.floor(time.from)), b = Math.min(n - 1, Math.ceil(time.to));
    if (b <= a) return;
    // (Seconds a pixel, over what's in view; labels at least 72 pixels apart.)
    const perPx = (times[b] - times[a]) / Math.max(1, time.x(b) - time.x(a));
    let k = 0;
    while (k < STEPS.length - 1 && STEPS[k][0] / perPx < ROOM) k++;
    if (this.#row(time, theme, plot, page, a, b, STEPS[k])) return;
    let gap = Infinity;
    for (let i = a + 1; i <= b; i++) if (times[i] - times[i - 1] < gap) gap = times[i] - times[i - 1];
    while (k > 0 && STEPS[k - 1][0] >= gap && !this.#row(time, theme, plot, page, a, b, STEPS[--k]));
  }

  /**
   * Its labels from point `a` to `b` by `step`, at least ROOM apart, each at the first point within a step (none
   * where the data skips a whole step: a weekend's days not named on the Monday), named as the step (a month's name at
   * its first trading day) or, by hours and minutes, as the point's own time (points at :05 named so): how many it drew.
   * @param {import("./timeline.js").Timeline} time @param {import("./pane.js").Theme} theme @param {number} plot @param {Tags} page
   * @param {number} a @param {number} b @param {(typeof STEPS)[number]} step
   */
  #row(time, theme, plot, page, a, b, step) {
    const g = this.g, times = time.times, [seconds, size, unit] = step, label = this.labels[unit];
    let drawn = 0;
    g.fillStyle = theme.text;
    g.textAlign = "center";
    g.textBaseline = "middle";
    // (Years and months by the calendar, from a multiple of the step; days and hours by whole steps since 1970: the
    // same starts whatever's in view, so they move with it.)
    civil(times[a]);
    const calendar = unit === "year" || unit === "month";
    let y = unit === "year" ? Math.floor(civilYear / size) * size : civilYear, m = unit === "month" ? Math.floor(civilMonth / size) * size : 0;
    let t = calendar ? Date.UTC(y, m, 1) / 1000 : Math.floor(times[a] / seconds) * seconds, last = -Infinity, lastRight = -Infinity;
    for (let guard = 0; guard < 400 && t <= times[b]; guard++) {
      const i = at(times, t), next = unit === "year" ? Date.UTC((y += size), 0, 1) / 1000 : unit === "month" ? Date.UTC(y, (m += size), 1) / 1000 : t + seconds;
      if (i <= b && times[i] < next) {
        // A calendar label names its actual period, including a first partial year/month; a multi-year tick before
        // the first point is skipped when it would name another year (2008 for January 2009).
        if (calendar) {
          civil(times[i]);
          const start = Date.UTC(civilYear, unit === "year" ? 0 : civilMonth, 1) / 1000;
          if (start !== t) { t = next; continue; }
        }
        const x = time.x(i);
        // Measure the final label: day/year text and page-supplied words have the same edge and overlap bounds.
        const when = calendar ? t : times[i], words = this.format ? this.format(when, kindOf(when)) : null;
        if (words !== "" && x - last >= ROOM) {
          const text = words ?? label(when), half = g.measureText(text).width / 2;
          if (x < half || x + half > plot || x - half < lastRight + 4 || this.#covered(x, page, half)) { t = next; continue; }
          g.fillText(text, x, this.height / 2);
          if (this.drawnCount === this.drawnX.length) ((grown) => (grown.set(this.drawnX), (this.drawnX = grown)))(new Float64Array(this.drawnCount * 2));
          (this.drawnX[this.drawnCount] = x), (this.drawnWords[this.drawnCount++] = text), (last = x), (lastRight = x + half), drawn++;
        }
      }
      t = next;
    }
    return drawn;
  }
}
