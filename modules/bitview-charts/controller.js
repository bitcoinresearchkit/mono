// @ts-check
// How a view moves, for any page on a Timeline: the wheel (a mouse's notches, a trackpad's scroll and its momentum),
// a drag's pan and a flick gliding on once let go, a pinch, and a move eased from where the view is to where a page
// sends it (a range button, a slider, play, the keys' zoom). The pointers' arbitration (what a press means: a
// crosshair, an axis, a separator) is the page's; it tells the controller what moved the view.
//
//   const controller = new Controller(element, time, { move: () => draw() });
//   controller.viewTo({ from: 400, to: 800 }, 250);   // eased there, over 250 ms
//   controller.range;                                  // where the view is going (a move from it starts there)
//
// Every move reports through `move` (the page draws on its next frame): its own frames' (an eased move's, a flick's)
// and a wheel's. A flick is stepped by the page's frame (`step`), so it's drawn in the frame that moved it.

import { Timeline } from "./timeline.js";

/** A flick's speed kept each millisecond: a chart's glide, two thirds of iOS's lists' (0.998), not years past what it was flicked for. */
const SLOWING = 0.997;
/** A flick's speed (CSS pixels a millisecond) as its finger's over its last moments (this many milliseconds); slower, none (a drag let go where it stopped). */
const FLICK_SPAN = 50, FLICK_LEAST = 0.25;
/** The fingers' least distance (CSS pixels) a pinch zooms by: nearer, it only pans. */
const PINCH_LEAST = 8;

export class Controller {
  /** A flick gliding on: its speed (CSS pixels a millisecond; 0: none), its last frame's time, its speed kept each millisecond (SLOWING; more where it'd pass an end: it eases into it, see #fling), and its way left to that end. */
  #speed = 0;
  #at = 0;
  #slowing = SLOWING;
  #left = Infinity;
  /** A drag's last moves (its x and when): its speed as it's let go (see FLICK_SPAN). */
  #trail = new Float64Array(16);
  #trailAt = new Float64Array(16);
  #trailEnd = 0;
  #trailSize = 0;
  /** An eased move: where it goes (null: none), its last step (kept a frame: the chart takes it by then), its frame's request. @type {{ from: number, to: number } | null} */
  #to = null;
  /** @type {{ from: number, to: number } | null} */
  #last = null;
  #frame = 0;
  #listening = new AbortController();

  /**
   * @param {HTMLElement} element where the wheel is heard
   * @param {Timeline} time the view it moves
   * @param {{ move: () => void, left?: () => number, clock?: () => (now?: number) => number }} options `move`: the view moved,
   * to be drawn; `left`: CSS pixels from the element's left to its plot (a wheel zooms about the pointer there); `clock`:
   * a page's own, a motion's time since it began from each frame's (the first counted a step on)
   */
  constructor(element, time, { move, left = () => 0, clock = since }) {
    this.time = time;
    this.clock = clock;
    this.move = move;
    element.addEventListener("wheel", (event) => {
      event.preventDefault();
      const scale = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? element.getBoundingClientRect().height : 1;
      // (A pinch zooms alone; a scroll both at once, a trackpad's diagonal swipe panning and zooming together.)
      if (event.deltaY) time.zoom(event.clientX - element.getBoundingClientRect().left - left(), Math.exp(event.deltaY * scale * (event.ctrlKey ? 0.01 : 0.001)));
      if (event.deltaX && !event.ctrlKey) time.pan(-event.deltaX * scale);
      this.stop();
      move();
    }, { passive: false, signal: this.#listening.signal });
  }

  /** The view as it's going: an eased move's last step, else the timeline's own (none without data). */
  get range() {
    return this.#last ?? (this.time.length ? { from: this.time.from, to: this.time.to } : null);
  }

  /** Where an eased move is going; null: none. */
  get target() {
    return this.#to;
  }

  /**
   * The view moved to show from point `from` to `to`, within the zoom's bounds (narrower, centred at the closest). As
   * wide as it is (an eased move's step, a navigator's drag), moved as a pan is, by whole pixels (the rest kept); `exact`,
   * at its endpoints as given (a view restored, dates gone to). An effective change ends a flick.
   * @param {number} from @param {number} to @param {{ exact?: boolean }} [options]
   */
  setRange(from, to, options) {
    const exact = options?.exact === true, time = this.time, span = to - from, wasFrom = time.from, wasTo = time.to;
    // (From where it truly is: its kept part of a pixel counted once.)
    if (!exact && time.length && Math.abs(span - (time.to - time.from)) <= span * 1e-9) time.pan((time.from - from) * time.spacing - time.rest);
    else time.setRange(from, to);
    if (time.from !== wasFrom || time.to !== wasTo) this.halt();
    this.move();
  }

  /**
   * The view eased to `to` over `duration` ms (cubic, out), from where it's going now (see range), another move ending.
   * At once with no duration (a motion reduced), or with no view to start from (its data being swapped).
   * @param {{ from: number, to: number }} to @param {number} duration
   */
  viewTo(to, duration) {
    const from = this.range;
    this.#unease();
    if (!(to.from < to.to)) return;
    if (!from || !(from.from < from.to) || !duration) return this.setRange(to.from, to.to);
    this.#to = to;
    const since = this.clock();
    const step = (/** @type {number} */ now) => {
      const t = Math.min(1, since(now) / duration), eased = 1 - (1 - t) ** 3;
      const next = { from: from.from + (to.from - from.from) * eased, to: from.to + (to.to - from.to) * eased };
      if (!(next.from < next.to)) return this.#unease();
      this.setRange(next.from, next.to);
      // (Kept till the chart's taken it, a frame on.)
      this.#last = next;
      requestAnimationFrame(() => this.#last === next && (this.#last = null));
      if (t < 1) this.#frame = requestAnimationFrame(step);
      else this.#to = null;
    };
    this.#frame = requestAnimationFrame(step);
  }

  /** A flick's end: the view stays where it is. */
  halt() {
    this.#speed = 0;
  }

  /** Every motion's end: a flick and an eased move. */
  stop() {
    this.halt();
    this.#unease();
  }

  #unease() {
    cancelAnimationFrame(this.#frame);
    this.#to = this.#last = null;
  }

  /** A press took the view: its motion ends, and a drag's trail begins anew. */
  grab() {
    this.stop();
    this.#trailSize = 0;
  }

  /** The drag's speed no longer counts (an axis dragged, a click, a finger let go that doesn't flick). */
  forget() {
    this.#trailSize = 0;
  }

  /** A drag's move: the view panned `dx` CSS pixels, the pointer now at `x` at time `at` (an event's timeStamp). @param {number} dx @param {number} x @param {number} at */
  drag(dx, x, at) {
    this.time.pan(dx);
    this.#trail[this.#trailEnd] = x;
    this.#trailAt[this.#trailEnd] = at;
    this.#trailEnd = (this.#trailEnd + 1) % this.#trail.length;
    this.#trailSize = Math.min(this.#trail.length, this.#trailSize + 1);
  }

  /**
   * Two pointers' move: zoomed about `x` (CSS pixels across the plot) as their distance went from `was` to `now`, and panned
   * `dx`, so what's under each stays under it.
   * @param {number} x @param {number} was @param {number} now @param {number} dx
   */
  pinch(x, was, now, dx) {
    if (was > PINCH_LEAST && now > PINCH_LEAST) this.time.zoom(x, was / now);
    this.time.pan(dx);
  }

  /**
   * A drag let go at time `at` (an event's timeStamp): it glides on at its speed over its last moves (a finger slowing to a
   * stop lets go slow), if it was still moving then (held still a moment, none) and not too slowly.
   * @param {number} at
   */
  flick(at) {
    if (this.#trailSize < 2) return void (this.#trailSize = 0);
    const trail = this.#trail, trailAt = this.#trailAt, last = (this.#trailEnd + trail.length - 1) % trail.length;
    let first = last;
    for (let k = 1; k < this.#trailSize; k++) {
      const i = (last - k + trail.length) % trail.length;
      if (trailAt[last] - trailAt[i] > FLICK_SPAN) break;
      first = i;
    }
    const ms = trailAt[last] - trailAt[first], speed = ms > 0 ? (trail[last] - trail[first]) / ms : 0, stale = at - trailAt[last] >= FLICK_SPAN;
    this.#trailSize = 0;
    if (!stale && Math.abs(speed) >= FLICK_LEAST) this.#fling(speed);
  }

  /**
   * A flick let go at `speed` (CSS pixels a millisecond): it glides on, slowing by SLOWING a millisecond (its travel `speed /
   * -ln(SLOWING)`); where that would pass an end, slowing faster to come to rest there (held at it, it stopped dead).
   * @param {number} speed
   */
  #fling(speed) {
    const room = this.time.room(speed), passes = Math.abs(speed) / -Math.log(SLOWING) > room;
    this.#slowing = passes ? Math.exp(-Math.abs(speed) / Math.max(room, 1e-9)) : SLOWING;
    this.#left = passes ? room : Infinity;
    this.#speed = speed;
    this.#at = performance.now();
    this.move();
  }

  /** A flick's frame, the page's own before it draws: on, slowing, till it rests or meets an end. @param {number} now */
  step(now) {
    if (!this.#speed) return;
    // (Its move over the frame as its speed falls through it (not at its end's speed); come to its least speed, a flick
    // easing into an end moves the rest of its way there.)
    const dt = Math.min(64, now - this.#at), decay = this.#slowing ** dt, ln = -Math.log(this.#slowing);
    this.#at = now;
    let by = ln < Infinity ? (this.#speed * (1 - decay)) / ln : 0;
    this.#speed *= decay;
    const ends = Math.abs(this.#speed) < 0.02;
    if (ends && this.#left < Infinity) by = Math.sign(by || this.#speed) * this.#left;
    this.#left -= Math.abs(by);
    if (!this.time.pan(by) || ends) this.halt();
    else this.move();
    if (ends && by) this.move();
  }

  /** Its wheel no longer heard, its motion ended. */
  destroy() {
    this.stop();
    this.#listening.abort();
  }
}

/** A motion's time since it began, from each frame's: its first counted a step on (shown moved, never where it began). @returns {(now?: number) => number} */
function since() {
  let start = -1;
  return (now = performance.now()) => ((start < 0 && (start = now - 1000 / 120)), now - start);
}
