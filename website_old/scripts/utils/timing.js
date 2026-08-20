/**
 * @param {number} ms
 */
export function sleep(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

export function next() {
  return sleep(0);
}

/**
 * @param {() => void} callback
 */
export function idle(callback) {
  ("requestIdleCallback" in window ? requestIdleCallback : setTimeout)(
    callback,
  );
}

/**
 *
 * @template {(...args: never[]) => unknown} F
 * @param {F} callback
 * @param {number} [wait]
 */
export function throttle(callback, wait = 1000) {
  /** @type {number | null} */
  let timeoutId = null;
  /** @type {Parameters<F>} */
  let latestArgs;
  let hasTrailing = false;

  return (/** @type {Parameters<F>} */ ...args) => {
    latestArgs = args;
    if (timeoutId) {
      hasTrailing = true;
      return;
    }
    callback(...latestArgs);
    timeoutId = setTimeout(() => {
      timeoutId = null;
      if (hasTrailing) {
        hasTrailing = false;
        callback(...latestArgs);
      }
    }, wait);
  };
}

/**
 * @template {(...args: never[]) => unknown} F
 * @param {F} callback
 * @param {number} [wait]
 * @returns {((...args: Parameters<F>) => void) & { cancel: () => void }}
 */
export function debounce(callback, wait = 1000) {
  /** @type {number | null} */
  let timeoutId = null;

  const fn = (/** @type {Parameters<F>} */ ...args) => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    timeoutId = setTimeout(() => {
      callback(...args);
      timeoutId = null;
    }, wait);
  };

  fn.cancel = () => {
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
  };

  return fn;
}
