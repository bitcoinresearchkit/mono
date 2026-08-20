/**
 * @param {string} key
 */
export function readStored(key) {
  try {
    return localStorage.getItem(key);
  } catch (_) {
    return null;
  }
}

/**
 * @param {string} key
 * @param {string | boolean | null | undefined} value
 */
export function writeToStorage(key, value) {
  try {
    value !== undefined && value !== null
      ? localStorage.setItem(key, String(value))
      : localStorage.removeItem(key);
  } catch (_) {}
}

/**
 * @param {string} key
 */
export function removeStored(key) {
  writeToStorage(key, undefined);
}
