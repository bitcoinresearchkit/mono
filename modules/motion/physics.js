// @ts-check
// Functions of elapsed time, never of frames. Velocities stay in the value's own units a millisecond, scaled by the caller:
// normalizing here is a trap (a spring from a value to itself has no travel to scale by).

import { reduced } from "./reduced.js";

/**
 * Apple's spring (WWDC23 10158): bounce -1 to 1, 0 critically damped.
 * @param {{ from: number, to: number, velocity?: number, duration?: number, bounce?: number, rest?: number }} spring
 */
export function spring({ from, to, velocity = 0, duration = 500, bounce = 0, rest = 0.01 }) {
  if (!(Math.abs(bounce) < 1)) throw new Error(`spring: bounce ${bounce} is outside -1 to 1 (at ±1 it never rests)`);
  if (reduced()) (from = to), (velocity = 0);
  const w0 = (2 * Math.PI) / duration, zeta = bounce >= 0 ? 1 - bounce : 1 / (1 + bounce), x0 = from - to;
  /** @type {(t: number) => number} */ let x;
  /** @type {(t: number) => number} */ let v;
  if (zeta < 1) {
    const wd = w0 * Math.sqrt(1 - zeta * zeta), a = x0, b = (velocity + zeta * w0 * x0) / wd;
    x = (t) => Math.exp(-zeta * w0 * t) * (a * Math.cos(wd * t) + b * Math.sin(wd * t));
    v = (t) => Math.exp(-zeta * w0 * t) * ((b * wd - zeta * w0 * a) * Math.cos(wd * t) - (a * wd + zeta * w0 * b) * Math.sin(wd * t));
  } else if (zeta === 1) {
    const b = velocity + w0 * x0;
    x = (t) => Math.exp(-w0 * t) * (x0 + b * t);
    v = (t) => Math.exp(-w0 * t) * (b - w0 * (x0 + b * t));
  } else {
    const s = w0 * Math.sqrt(zeta * zeta - 1), r1 = -zeta * w0 + s, r2 = -zeta * w0 - s, c2 = (velocity - r1 * x0) / (r2 - r1), c1 = x0 - c2;
    x = (t) => c1 * Math.exp(r1 * t) + c2 * Math.exp(r2 * t);
    v = (t) => c1 * r1 * Math.exp(r1 * t) + c2 * r2 * Math.exp(r2 * t);
  }
  return {
    to,
    /** @param {number} t */
    at: (t) => to + x(Math.max(0, t)),
    /** Units a millisecond. @param {number} t */
    velocity: (t) => v(Math.max(0, t)),
    /** @param {number} t */
    done: (t) => Math.abs(x(Math.max(0, t))) < rest && Math.abs(v(Math.max(0, t))) * 16 < rest,
  };
}

/**
 * `rate`: -ln of the speed kept a millisecond.
 * @param {{ from: number, velocity: number, rate: number, least?: number }} glide
 */
export function glide({ from, velocity, rate, least = 0.02 }) {
  if (reduced()) velocity = 0;
  const travel = velocity / rate, duration = Math.abs(velocity) > least ? Math.log(Math.abs(velocity) / least) / rate : 0;
  /** @param {number} t */
  const within = (t) => Math.min(Math.max(0, t), duration);
  return {
    to: from + travel,
    duration,
    /** Stops short of `to` by least / rate. @param {number} t */
    at: (t) => from + travel * (1 - Math.exp(-rate * within(t))),
    /** @param {number} t */
    velocity: (t) => (t < duration ? velocity * Math.exp(-rate * Math.max(0, t)) : 0),
    /** @param {number} t */
    done: (t) => t >= duration,
  };
}

/**
 * Least-squares slope, not the last two samples: iOS input timestamps are noisy.
 * @param {number} [span] ms
 */
export function velocityTracker(span = 50) {
  const SIZE = 16, ts = new Float64Array(SIZE), xs = new Float64Array(SIZE);
  let end = 0, size = 0;
  return {
    reset() {
      size = 0;
    },
    /** @param {number} t @param {number} x */
    add(t, x) {
      (ts[end] = t), (xs[end] = x), (end = (end + 1) % SIZE), (size = Math.min(SIZE, size + 1));
    },
    /** @param {number} now */
    velocity(now) {
      const last = (end + SIZE - 1) % SIZE;
      if (size < 2 || now - ts[last] >= span) return 0;
      // Centered on the last sample: large timestamps lose no precision.
      let n = 0, st = 0, sx = 0, stt = 0, stx = 0;
      for (let k = 0; k < size; k++) {
        const i = (last - k + SIZE) % SIZE, t = ts[i] - ts[last], x = xs[i] - xs[last];
        if (t < -span) break;
        n++, (st += t), (sx += x), (stt += t * t), (stx += t * x);
      }
      const d = n * stt - st * st;
      return n > 1 && d > 0 ? (n * stx - st * sx) / d : 0;
    },
  };
}
