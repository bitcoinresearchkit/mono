// @ts-check
// Time on the page: work put off to idle moments and done in slices, and waiting for a thing to hold.

/**
 * No idle callbacks in Safari: runs `settle` ms later instead.
 * @param {(deadline: { timeRemaining: () => number }) => void} work
 */
const whenIdle = (work, { timeout = 1000, settle = 40 } = {}) => void ("requestIdleCallback" in window ? requestIdleCallback(work, { timeout }) : setTimeout(() => {
  const end = performance.now() + 8;
  work({ timeRemaining: () => Math.max(0, end - performance.now()) });
}, settle));
/**
 * A timed-out callback has no time left, yet one step runs; slices stay short for a phone's engine without a JIT.
 * @param {{ timeRemaining: () => number } | undefined} deadline
 * @param {() => boolean} step
 * @param {(deadline: { timeRemaining: () => number }) => void} again
 */
function inSlices(deadline, step, again) {
  const stop = deadline ? performance.now() + Math.min(8, Math.max(2, deadline.timeRemaining())) : Infinity;
  while (!step()) if (performance.now() >= stop) return void whenIdle(again);
}

// Timers, not frames: a change of place draws none and a hidden tab still looks.
const until = (/** @type {() => boolean} */ test, /** @type {number} */ ms, step = 16) => new Promise((done) => {
  const end = performance.now() + ms, look = () => (test() || performance.now() > end ? done(undefined) : setTimeout(look, step));
  look();
});

/** @param {() => Promise<void>} pass */
function serial(pass) {
  let running = false, again = false;
  const run = async () => {
    if (running) return void (again = true);
    running = true;
    try {
      await pass();
    } finally {
      running = false;
      if (again) {
        again = false;
        void run();
      }
    }
  };
  return run;
}
export { inSlices, until, whenIdle, serial };
