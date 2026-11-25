// @ts-check
// How a Chart's view moves: the wheel (a mouse's notches, a trackpad's scroll and its momentum), a drag's pan and a
// flick gliding on once let go, a pinch, and a move eased from where the view is to where a page sends it (a range
// button, a slider, play, the keys' zoom). The chart's pointers' arbitration (what a press means: a crosshair, an axis,
// a separator) is the chart's; it tells the controller what moved the view. A page reaches it as `chart.controller`.
//
//   chart.controller.viewTo({ from: 400, to: 800 }, 250);   // eased there, over 250 ms
//   chart.controller.range;                                   // where the view is (a move from it starts there)
//
// Every move reports through `move` (the chart draws on its next frame). A flick and an eased move are stepped by the
// chart's own frame (`step`), so each is drawn in the frame that moved it.

import { Timeline } from "./timeline.js";
import { motionClock, motionTime } from "../motion/clock.js";
import { glide, velocityTracker } from "../motion/physics.js";
import { reduced } from "../motion/reduced.js";

/** A flick's speed kept each millisecond: a chart's glide, two thirds of iOS's lists' (0.998), not years past what it was flicked for; its rate of slowing. */
const SLOWING = 0.997, RATE = -Math.log(SLOWING);
/** A flick's speed (CSS pixels a millisecond) as its finger's over its last moments (this many milliseconds); slower, none (a drag let go where it stopped). */
const FLICK_SPAN = 50, FLICK_LEAST = 0.25;
/** The fingers' least distance (CSS pixels) a pinch zooms by: nearer, it only pans. */
const PINCH_LEAST = 8;

export class Controller {
  #time;
  #move;
  /**
   * A flick gliding on (null: none): its glide in CSS pixels from where it was let go, read at its time since (its
   * clock's, from the release's); how far it's moved the view; whether it eases into an end (see #fling).
   * @type {{ glide: ReturnType<typeof glide>, clock: (now?: number) => number, start: number, moved: number, toEnd: boolean } | null}
   */
  #flick = null;
  /** A drag's speed, from its last moves (see FLICK_SPAN). */
  #velocity = velocityTracker(FLICK_SPAN);
  /** An eased move: where it goes (null: none), from where, over how long, and its time since it began. @type {{ from: number, to: number } | null} */
  #to = null;
  /** @type {{ from: number, to: number }} */
  #from = { from: 0, to: 0 };
  #duration = 0;
  #since = motionTime();

  /**
   * @param {HTMLElement} element where the wheel is heard
   * @param {Timeline} time the view it moves
   * @param {{ move: () => void, left: () => number, signal?: AbortSignal, embedded?: boolean }} options `move`: the view
   * moved, to be drawn; `left`: CSS pixels from the element's left to its plot (a wheel zooms about the pointer there);
   * `signal`: its wheel heard till it aborts (none: as long as the page); `embedded`, a chart in a page that scrolls: a mostly vertical wheel scrolls
   * the page, a pinch (a trackpad's, a ctrl wheel) zooms, a sideways scroll pans
   */
  constructor(element, time, { move, left, signal, embedded = false }) {
    this.#time = time;
    this.#move = move;
    element.addEventListener("wheel", (event) => {
      if (embedded && !event.ctrlKey && Math.abs(event.deltaX) <= Math.abs(event.deltaY)) return;
      event.preventDefault();
      const box = element.getBoundingClientRect(), scale = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? box.height : 1;
      // (A pinch zooms alone; a scroll both at once, a trackpad's diagonal swipe panning and zooming together; embedded,
      // a sideways scroll only pans.)
      if (event.deltaY && (event.ctrlKey || !embedded)) time.zoom(event.clientX - box.left - left(), Math.exp(event.deltaY * scale * (event.ctrlKey ? 0.01 : 0.001)));
      if (event.deltaX && !event.ctrlKey) time.pan(-event.deltaX * scale);
      this.stop();
      move();
    }, { passive: false, signal });
  }

  get range() {
    return this.#time.length ? { from: this.#time.from, to: this.#time.to } : null;
  }

  /** Where an eased move is going; null: none. */
  get target() {
    return this.#to;
  }

  /**
   * The view eased to `to` over `duration` ms (cubic, out), from where it is, another motion ending. At once with no
   * duration, under reduced motion, or with no view to start from (its data being swapped).
   * @param {{ from: number, to: number }} to @param {number} duration
   */
  viewTo(to, duration) {
    const from = this.range;
    this.stop();
    if (!(to.from < to.to)) return;
    if (!from || !(from.from < from.to) || !duration || reduced()) return this.#set(to.from, to.to);
    this.#to = to;
    this.#from = from;
    this.#duration = duration;
    this.#since = motionTime();
    this.#move();
  }

  /** @param {number} from @param {number} to */
  #set(from, to) {
    this.#time.slide(from, to);
    this.#move();
  }

  halt() {
    this.#flick = null;
  }

  /** Every motion's end: a flick and an eased move. */
  stop() {
    this.halt();
    this.#to = null;
  }

  /** A press took the view: its motion ends, and a drag's trail begins anew. */
  grab() {
    this.stop();
    this.#velocity.reset();
  }

  /** The drag's speed no longer counts (an axis dragged, a click, a finger let go that doesn't flick). */
  forget() {
    this.#velocity.reset();
  }

  /** A drag's move: the view panned `dx` CSS pixels, the pointer now at `x` at time `at` (an event's timeStamp). @param {number} dx @param {number} x @param {number} at */
  drag(dx, x, at) {
    this.#time.pan(dx);
    this.#velocity.add(at, x);
  }

  /**
   * Two pointers' move: zoomed about `x` (CSS pixels across the plot) as their distance went from `was` to `now`, and panned
   * `dx`, so what's under each stays under it.
   * @param {number} x @param {number} was @param {number} now @param {number} dx
   */
  pinch(x, was, now, dx) {
    if (was > PINCH_LEAST && now > PINCH_LEAST) this.#time.zoom(x, was / now);
    this.#time.pan(dx);
  }

  /**
   * A drag let go at time `at` (an event's timeStamp): it glides on at its speed over its last moves (a finger slowing to a
   * stop lets go slow), if it was still moving then (held still a moment, none) and not too slowly; under reduced motion,
   * none.
   * @param {number} at
   */
  flick(at) {
    const speed = this.#velocity.velocity(at);
    this.#velocity.reset();
    if (Math.abs(speed) >= FLICK_LEAST && !reduced()) this.#fling(speed, at);
  }

  /**
   * A flick let go at `speed` (CSS pixels a millisecond): it glides on, slowing by SLOWING a millisecond (its travel `speed /
   * -ln(SLOWING)`); where that would pass an end, slowing faster to come to rest there (held at it, it stopped dead).
   * @param {number} speed @param {number} at the release's time
   */
  #fling(speed, at) {
    const room = this.#time.room(speed), toEnd = Math.abs(speed) / RATE > room, clock = motionClock();
    // (Its travel at that rate is room exactly: its end, where it stops short by least / rate, taken whole.)
    this.#flick = { glide: glide({ from: 0, velocity: speed, rate: toEnd ? Math.abs(speed) / Math.max(room, 1e-9) : RATE }), clock, start: clock(at), moved: 0, toEnd };
    this.#move();
  }

  /** A frame of the motion, the chart's own before it draws: an eased move's step; a flick on, slowing, till it rests or meets an end. @param {number} now */
  step(now) {
    if (this.#to) {
      const t = Math.min(1, this.#since(now) / this.#duration), eased = 1 - (1 - t) ** 3, from = this.#from, to = this.#to;
      this.#set(from.from + (to.from - from.from) * eased, from.to + (to.to - from.to) * eased);
      if (t === 1) this.#to = null;
    }
    const flick = this.#flick;
    if (!flick) return;
    // (Where it is at this frame's time, by its closed form (never stepped a frame at a time: any frame rate, a late
    // frame, all read it alike; a frame timed before the lift, none moved yet); at its end, a flick easing into an end
    // moved the rest of its way there.)
    const t = flick.clock(now) - flick.start, ends = flick.glide.done(t), to = ends && flick.toEnd ? flick.glide.to : flick.glide.at(t), by = to - flick.moved;
    flick.moved = to;
    if (!this.#time.pan(by) || ends) this.halt();
    else this.#move();
    if (ends && by) this.#move();
  }
}
