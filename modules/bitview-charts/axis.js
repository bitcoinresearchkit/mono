// @ts-check
// The time axis under the panes: the starts of years, months, days or hours (whichever leave room enough between their
// labels), each at its first point, its label under it (UTC).

import { kept } from "./format.js";

const DAY = 86400;
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
/** Each unit's steps, from the finest: [seconds a step (about), step in its unit, its unit]. */
const STEPS = /** @type {const} */ ([
  [3600, 1, "hour"], [6 * 3600, 6, "hour"], [DAY, 1, "day"], [7 * DAY, 7, "day"], [30 * DAY, 1, "month"],
  [91 * DAY, 3, "month"], [182 * DAY, 6, "month"], [365 * DAY, 1, "year"], [5 * 365 * DAY, 5, "year"], [10 * 365 * DAY, 10, "year"],
]);

/** A time's UTC year and month (0 to 11), written here: no Date made each frame (Hinnant's civil from days). */
let civilYear = 0, civilMonth = 0;
/** @param {number} t seconds */
function civil(t) {
  const z = Math.floor(t / DAY) + 719468, era = Math.floor(z / 146097), doe = z - era * 146097;
  const yoe = Math.floor((doe - Math.floor(doe / 1460) + Math.floor(doe / 36524) - Math.floor(doe / 146096)) / 365);
  const doy = doe - (365 * yoe + Math.floor(yoe / 4) - Math.floor(yoe / 100)), mp = Math.floor((5 * doy + 2) / 153), m = mp < 10 ? mp + 3 : mp - 9;
  civilYear = yoe + era * 400 + (m <= 2 ? 1 : 0);
  civilMonth = m - 1;
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
    // (Each unit's labels: a month's start is its name among days, its year's among months.)
    const date = (/** @type {number} */ t) => new Date(t * 1000);
    this.labels = {
      year: kept((t) => String(date(t).getUTCFullYear())),
      month: kept((t) => (date(t).getUTCMonth() ? MONTHS[date(t).getUTCMonth()] : String(date(t).getUTCFullYear()))),
      day: kept((t) => (date(t).getUTCDate() === 1 ? MONTHS[date(t).getUTCMonth()] : String(date(t).getUTCDate()))),
      hour: kept((t) => (date(t).getUTCHours() ? `${String(date(t).getUTCHours()).padStart(2, "0")}:00` : String(date(t).getUTCDate()))),
    };
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
   * @param {import("./timeline.js").Timeline} time @param {import("./pane.js").Theme} theme @param {number} dpr @param {number} plot
   */
  draw(time, theme, dpr, plot) {
    const g = this.g, times = time.times, n = times.length;
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.fillStyle = theme.background;
    g.fillRect(0, 0, this.width, this.height);
    if (n < 2) return;
    const a = Math.max(0, Math.floor(time.from)), b = Math.min(n - 1, Math.ceil(time.to));
    if (b <= a) return;
    // (Seconds a pixel, over what's in view; labels at least 72 pixels apart.)
    const perPx = (times[b] - times[a]) / Math.max(1, time.x(b) - time.x(a)), room = 72;
    let step = STEPS[STEPS.length - 1];
    for (let k = 0; k < STEPS.length; k++)
      if (STEPS[k][0] / perPx >= room) {
        step = STEPS[k];
        break;
      }
    const [seconds, size, unit] = step, label = this.labels[unit];
    g.fillStyle = theme.text;
    g.font = theme.font;
    g.textAlign = "center";
    g.textBaseline = "middle";
    // (Years and months by the calendar, from a multiple of the step; days and hours by whole steps since 1970: the
    // same starts whatever's in view, so they move with it.)
    civil(times[a]);
    const calendar = unit === "year" || unit === "month";
    let y = unit === "year" ? Math.floor(civilYear / size) * size : civilYear, m = unit === "month" ? Math.floor(civilMonth / size) * size : 0;
    let t = calendar ? Date.UTC(y, m, 1) / 1000 : Math.floor(times[a] / seconds) * seconds, last = -Infinity;
    for (let guard = 0; guard < 400 && t <= times[b]; guard++) {
      const i = at(times, t);
      if (i <= b) {
        const x = time.x(i);
        if (x - last >= room && x > 12 && x < plot - 12) (g.fillText(label(t), x, this.height / 2), (last = x));
      }
      if (unit === "year") t = Date.UTC((y += size), 0, 1) / 1000;
      else if (unit === "month") t = Date.UTC(y, (m += size), 1) / 1000;
      else t += seconds;
    }
  }
}
