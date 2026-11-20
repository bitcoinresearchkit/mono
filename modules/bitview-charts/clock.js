// @ts-check
// A motion's time: each frame's, made steady where the browser rounds it.

/**
 * A motion's clock, called once a frame (its own frames, timed here: a flick, an eased move, a page's own): the frame's time,
 * unless the browser rounds it (resisting fingerprinting: 16.67 ms; two frames of a 120 Hz screen sharing one, the
 * motion would stand still in one, leap in the next): then on from the last by the frames' own pace, never past the
 * clock's next tick. @returns {(now?: number) => number}
 */
export function motionClock() {
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
export function motionTime() {
  const clock = motionClock();
  let start = -1;
  return (now) => {
    now = clock(now);
    if (start < 0) start = now - 1000 / 120;
    return now - start;
  };
}
