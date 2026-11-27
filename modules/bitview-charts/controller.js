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
  #velocity = velocityTracker(FLICK_SPAN);
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
    this.#flick = null;
  }

  stop() {
    this.halt();
    this.#to = null;
  }

  grab() {
    this.stop();
    this.#velocity.reset();
  }

  forget() {
    this.#velocity.reset();
  }

  /** @param {number} dx @param {number} x @param {number} at */
  drag(dx, x, at) {
    this.#time.pan(dx);
    this.#velocity.add(at, x);
  }

  /** @param {number} x @param {number} was @param {number} now @param {number} dx */
  pinch(x, was, now, dx) {
    if (was > PINCH_LEAST && now > PINCH_LEAST) this.#time.zoom(x, was / now);
    this.#time.pan(dx);
  }

  /** @param {number} at */
  flick(at) {
    const speed = this.#velocity.velocity(at);
    this.#velocity.reset();
    if (Math.abs(speed) >= FLICK_LEAST && !reduced()) this.#fling(speed, at);
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
