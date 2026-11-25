// @ts-check
// Time on the page: work put off to idle moments and done in slices.

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

export { inSlices, whenIdle };
