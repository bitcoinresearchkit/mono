// @ts-check
// Time on the page: a motion's clock and each frame's time, work put off to idle moments and done in slices.

/**
 * A motion's clock, called once a frame (its own frames, timed here: the view's glides, the logo, …): the frame's time,
 * unless the browser rounds it (resisting fingerprinting: 16.67 ms; two frames of a 120 Hz screen sharing one, the
 * motion would stand still in one, leap in the next): then on from the last by the frames' own pace, never past the
 * clock's next tick. @returns {(now?: number) => number}
 */
function motionClock() {
  let first = -1, last = 0, frames = 0, seen = -1, rounded = false;
  return (now = performance.now()) => {
    // (Rounded, as two frames given one time show; else the frame's own time, always.)
    rounded ||= now === seen;
    seen = now;
    // (Its first frame, or one after a while without (a still logo waking, a tab back in sight): its pace measured
    // anew from there.)
    if (first < 0 || now - last > 100) return ((first = last = now), (frames = 0), now);
    frames++;
    if (!rounded) return (last = now);
    // (On by the frames' own pace, within the tick the rounded time stands for: behind it by more, the clock's own
    // taken again (a rounded time ahead, jumped to at once, leaves the next frame next to standing still); ahead,
    // held at its end, never back.)
    const pace = now > first ? Math.min(1000 / 60, (now - first) / frames) : 1000 / 120, next = last + pace;
    return (last = next < now - 1000 / 60 ? now : Math.min(next, now + 1000 / 60));
  };
}
/**
 * A motion's time since it began, by its clock: its first frame counted a step on (shown moved, never where it began:
 * a time rounded, or one before the motion was asked for, would stand it still a frame; one motion taking over
 * another, halt it). @returns {(now?: number) => number}
 */
function motionTime() {
  const clock = motionClock();
  let start = -1;
  return (now) => {
    now = clock(now);
    if (start < 0) start = now - 1000 / 120;
    return now - start;
  };
}
/**
 * Work for a moment the page is idle, given how long it has: at the latest after `timeout` ms. Without idle
 * callbacks (Safari), after `settle` ms, with a slice of 8.
 * @param {(deadline: { timeRemaining: () => number }) => void} work
 */
const whenIdle = (work, { timeout = 1000, settle = 40 } = {}) => void ("requestIdleCallback" in window ? requestIdleCallback(work, { timeout }) : setTimeout(() => {
  const end = performance.now() + 8;
  work({ timeRemaining: () => Math.max(0, end - performance.now()) });
}, settle));
/**
 * Work in steps, a slice of them in an idle moment (`deadline`): as long as its time lasts, 2 to 8 ms (one step at
 * least: a callback timed out has none left), none of it a long task (a phone's engine without a JIT), the rest
 * asked for (`again`) in the next; none (asked now), all at once. `step` does one, true once there's none left.
 * @param {{ timeRemaining: () => number } | undefined} deadline @param {() => boolean} step
 * @param {(deadline: { timeRemaining: () => number }) => void} again
 */
function inSlices(deadline, step, again) {
  const until = deadline ? performance.now() + Math.min(8, Math.max(2, deadline.timeRemaining())) : Infinity;
  while (!step()) if (performance.now() >= until) return void whenIdle(again);
}

export { inSlices, motionClock, motionTime, whenIdle };
