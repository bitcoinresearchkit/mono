// @ts-check

/**
 * Rounded frame times (fingerprinting resistance) would stand a motion still a frame and leap the next: so on from the last by the frames' own pace.
 * @returns {(now?: number) => number}
 */
export function motionClock() {
  let first = -1, last = 0, frames = 0, seen = -1, rounded = false;
  return (now = performance.now()) => {
    rounded ||= now === seen;
    seen = now;
    if (first < 0 || now - last > 100) return ((first = last = now), (frames = 0), now);
    frames++;
    if (!rounded) return (last = now);
    const pace = now > first ? Math.min(1000 / 60, (now - first) / frames) : 1000 / 120, next = last + pace;
    return (last = next < now - 1000 / 60 ? now : Math.min(next, now + 1000 / 60));
  };
}
/** First frame counts a step on: a rounded or earlier time would stand it still a frame. @returns {(now?: number) => number} */
export function motionTime() {
  const clock = motionClock();
  let start = -1;
  return (now) => {
    now = clock(now);
    if (start < 0) start = now - 1000 / 120;
    return now - start;
  };
}
