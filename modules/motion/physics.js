// @ts-check
// Motion as functions of elapsed time, never of frames: a frame at any time (60, 120, a late one, a worker's between
// the page's) reads where the motion is then, and two threads reading the same motion at the same time agree. A time
// before its start reads as its start (a frame stamped before the press that began it).
// Velocities are in the value's own units a millisecond: a hand-off from a finger (CSS pixels a millisecond) is scaled
// into the moved value's units by its caller (a chart's view: pixels over its spacing), and nothing is normalized in
// here. (Normalizing is a known trap: a spring working in 0–100 handed a velocity in the value's own units hands off
// the wrong speed, and a spring from a value to itself has no travel to scale by and drops its velocity.)
// A motion made under reduced motion (reduced.js) resolves at once: a spring at its end, a glide none (no fling). One
// made before the reader turns it on runs out.

import { reduced } from "./reduced.js";

/**
 * A spring from `from` to `to`, starting at `velocity`, as Apple specifies one (WWDC23 10158): `duration` (ms) its
 * perceptual length (stiffness (2π/duration)², mass 1), `bounce` between -1 and 1 (0: critically damped, the most
 * versatile; above 0 it overshoots; below 0 it creeps). Closed form: evaluated at any elapsed time. At rest once
 * within `rest` of its end (in the value's units: 0.01, a hundredth of a CSS pixel; a scale or an opacity states its
 * own), and as slow; past then the caller shows `to`.
 * @param {{ from: number, to: number, velocity?: number, duration?: number, bounce?: number, rest?: number }} spring
 */
export function spring({ from, to, velocity = 0, duration = 500, bounce = 0, rest = 0.01 }) {
  if (!(Math.abs(bounce) < 1)) throw new Error(`spring: bounce ${bounce} is outside -1 to 1 (at ±1 it never rests)`);
  if (reduced()) (from = to), (velocity = 0);
  const w0 = (2 * Math.PI) / duration, zeta = bounce >= 0 ? 1 - bounce : 1 / (1 + bounce), x0 = from - to;
  // (Displacement x(t) from `to` and its derivative, for each damping regime: under-, critically, over-damped.)
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
    /** Where it is `t` ms in. @param {number} t */
    at: (t) => to + x(Math.max(0, t)),
    /** Its velocity `t` ms in (units a millisecond). @param {number} t */
    velocity: (t) => v(Math.max(0, t)),
    /** Whether it's at rest `t` ms in (within `rest` of its end, and moving less than that a frame). @param {number} t */
    done: (t) => Math.abs(x(Math.max(0, t))) < rest && Math.abs(v(Math.max(0, t))) * 16 < rest,
  };
}

/**
 * A glide from `from` at `velocity` (units a millisecond), slowing by `rate` (a share of its speed lost a
 * millisecond: -ln of what's kept, as UIScrollView's deceleration 0.998 is -ln 0.998) till under `least` (units a
 * millisecond: 0.02, a CSS pixel in 50 ms): a flick's momentum, in closed form. Its whole travel is velocity / rate;
 * it stops short of it by least / rate.
 * @param {{ from: number, velocity: number, rate: number, least?: number }} glide
 */
export function glide({ from, velocity, rate, least = 0.02 }) {
  if (reduced()) velocity = 0;
  const travel = velocity / rate, duration = Math.abs(velocity) > least ? Math.log(Math.abs(velocity) / least) / rate : 0;
  /** @param {number} t */
  const within = (t) => Math.min(Math.max(0, t), duration);
  return {
    /** Where it would come to rest, never slowing past `least`. */
    to: from + travel,
    /** How long it moves (ms). */
    duration,
    /** Where it is `t` ms in (past its end, where it stopped: short of `to` by least / rate). @param {number} t */
    at: (t) => from + travel * (1 - Math.exp(-rate * within(t))),
    /** @param {number} t */
    velocity: (t) => (t < duration ? velocity * Math.exp(-rate * Math.max(0, t)) : 0),
    /** @param {number} t */
    done: (t) => t >= duration,
  };
}

/**
 * A pointer's velocity along one axis, from its recent moves: the least-squares slope over the last `span` ms, not
 * the last two samples (WWDC18 803), which at iOS's 60 Hz input with 1 ms timestamps are noisy. None (0) once it has
 * stood still `span` ms. Kept in a ring, nothing made a move.
 * @param {number} [span] ms
 */
export function velocityTracker(span = 50) {
  const SIZE = 16, ts = new Float64Array(SIZE), xs = new Float64Array(SIZE);
  let end = 0, size = 0;
  return {
    reset() {
      size = 0;
    },
    /** A move to `x` at `t` (an event's timeStamp). @param {number} t @param {number} x */
    add(t, x) {
      (ts[end] = t), (xs[end] = x), (end = (end + 1) % SIZE), (size = Math.min(SIZE, size + 1));
    },
    /** Its velocity (units a millisecond) as of `now`. @param {number} now */
    velocity(now) {
      const last = (end + SIZE - 1) % SIZE;
      if (size < 2 || now - ts[last] >= span) return 0;
      // (Centered on the last sample, so large timestamps lose no precision.)
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
