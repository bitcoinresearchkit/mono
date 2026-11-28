// @ts-check

import { Timeline } from "./timeline.js";
import { motionClock, motionTime } from "../motion/clock.js";
import { glide, velocityTracker } from "../motion/physics.js";
import { reduced } from "../motion/reduced.js";

/** Two thirds of iOS's list glide (0.998): not years past what it was flicked for. */
const SLOWING = 0.997, RATE = -Math.log(SLOWING);
const FLICK_SPAN = 50, FLICK_LEAST = 0.25;
const PINCH_LEAST = 8;

export class Controller {
  #time;
  #move;
  /** @type {{ glide: ReturnType<typeof glide>, clock: (now?: number) => number, start: number, moved: number, toEnd: boolean } | null} */
  #flick = null;
  /** @type {{ glide: ReturnType<typeof glide>, clock: (now?: number) => number, start: number, moved: number, x: number, width: number } | null} */
  #zoom = null;
  #velocity = velocityTracker(FLICK_SPAN);
  /** The log of the pinching fingers' distance. */
  #spread = velocityTracker(FLICK_SPAN);
  /** Where the pinch was, the zoom's glide held about it. */
  #pinchX = 0;
  /** @type {{ from: number, to: number } | null} */
  #to = null;
  /** @type {{ from: number, to: number }} */
  #from = { from: 0, to: 0 };
  #duration = 0;
  #since = motionTime();

  /**
   * @param {HTMLElement} element @param {Timeline} time
   * @param {{ move: () => void, left: () => number, signal?: AbortSignal, embedded?: boolean }} options `embedded`: a chart in a scrolling page
   */
  constructor(element, time, { move, left, signal, embedded = false }) {
    this.#time = time;
    this.#move = move;
    element.addEventListener("wheel", (event) => {
      if (embedded && !event.ctrlKey && Math.abs(event.deltaX) <= Math.abs(event.deltaY)) return;
      event.preventDefault();
      const box = element.getBoundingClientRect(), scale = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? box.height : 1;
      if (event.deltaY && (event.ctrlKey || !embedded)) time.zoom(event.clientX - box.left - left(), Math.exp(event.deltaY * scale * (event.ctrlKey ? 0.01 : 0.001)));
      if (event.deltaX && !event.ctrlKey) time.pan(-event.deltaX * scale);
      this.stop();
      move();
    }, { passive: false, signal });
  }

  get range() {
    return this.#time.length ? { from: this.#time.from, to: this.#time.to } : null;
  }

  get target() {
    return this.#to;
  }

  /** @param {{ from: number, to: number }} to @param {number} duration */
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
    this.#flick = this.#zoom = null;
  }

  stop() {
    this.halt();
    this.#to = null;
  }

  grab() {
    this.stop();
    this.#velocity.reset();
    this.#spread.reset();
  }

  forget() {
    this.#velocity.reset();
  }

  /** @param {number} dx @param {number} x @param {number} at */
  drag(dx, x, at) {
    this.#time.pan(dx);
    this.#velocity.add(at, x);
  }

  /** @param {number} x @param {number} was @param {number} now @param {number} dx @param {number} at */
  pinch(x, was, now, dx, at) {
    if (was > PINCH_LEAST && now > PINCH_LEAST) this.#time.zoom(x, was / now), this.#spread.add(at, Math.log(now)), (this.#pinchX = x);
    this.#time.pan(dx);
  }

  /**
   * After a pinch, its zoom glides on if both fingers left within the tracker's span, never a pan: the finger left last
   * moves apart from the other, not the view.
   * @param {number} at @param {boolean} [pinched]
   */
  flick(at, pinched = false) {
    // A zoom's speed as the plot's edge moves, in the flick's own pixels a millisecond.
    const speed = pinched ? 0 : this.#velocity.velocity(at), width = this.#time.width, edge = this.#spread.velocity(at) * width;
    this.#velocity.reset();
    this.#spread.reset();
    if (reduced()) return;
    if (Math.abs(speed) >= FLICK_LEAST) this.#fling(speed, at);
    else if (Math.abs(edge) >= FLICK_LEAST) {
      const clock = motionClock();
      this.#zoom = { glide: glide({ from: 0, velocity: edge, rate: RATE }), clock, start: clock(at), moved: 0, x: this.#pinchX, width };
      this.#move();
    }
  }

  /** @param {number} speed @param {number} at */
  #fling(speed, at) {
    // A flick that would pass an end slows faster to rest there: held at it, it would stop dead.
    const room = this.#time.room(speed), toEnd = Math.abs(speed) / RATE > room, clock = motionClock();
    this.#flick = { glide: glide({ from: 0, velocity: speed, rate: toEnd ? Math.abs(speed) / Math.max(room, 1e-9) : RATE }), clock, start: clock(at), moved: 0, toEnd };
    this.#move();
  }

  /** @param {number} now */
  step(now) {
    const zoom = this.#zoom;
    if (zoom) {
      const t = zoom.clock(now) - zoom.start, to = zoom.glide.at(t), span = this.#time.to - this.#time.from;
      // (A frame stamped at or before the lift asks for no zoom: that isn't the limit.)
      const asked = to !== zoom.moved;
      if (asked) this.#time.zoom(zoom.x, Math.exp(-(to - zoom.moved) / zoom.width));
      zoom.moved = to;
      // Held at the least or most it zooms to: no more to glide.
      if (zoom.glide.done(t) || (asked && this.#time.to - this.#time.from === span)) this.#zoom = null;
      this.#move();
    }
    if (this.#to) {
      const t = Math.min(1, this.#since(now) / this.#duration), eased = 1 - (1 - t) ** 3, from = this.#from, to = this.#to;
      this.#set(from.from + (to.from - from.from) * eased, from.to + (to.to - from.to) * eased);
      if (t === 1) this.#to = null;
    }
    const flick = this.#flick;
    if (!flick) return;
    // Closed form, not stepped per frame: any frame rate or a late frame reads alike.
    const t = flick.clock(now) - flick.start, ends = flick.glide.done(t), to = ends && flick.toEnd ? flick.glide.to : flick.glide.at(t), by = to - flick.moved;
    flick.moved = to;
    if (!this.#time.pan(by) || ends) this.halt();
    else this.#move();
    if (ends && by) this.#move();
  }
}
