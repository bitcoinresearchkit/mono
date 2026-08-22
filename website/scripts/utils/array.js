/**
 * Typed Object.entries that preserves key types
 * @template {object} T
 * @param {T} obj
 * @returns {[keyof T & string, T[keyof T & string]][]}
 */
export const entries = (obj) => /** @type {[keyof T & string, T[keyof T & string]][]} */ (Object.entries(obj));

/**
 * Typed Object.fromEntries that preserves key/value types
 * @template {string} K
 * @template V
 * @param {Iterable<readonly [K, V]>} pairs
 * @returns {Record<K, V>}
 */
export const fromEntries = (pairs) => /** @type {Record<K, V>} */ (Object.fromEntries(pairs));

/**
 * Type-safe includes that narrows the value type
 * @template T
 * @template V
 * @param {readonly T[]} arr
 * @param {V} value
 * @returns {value is V & T}
 */
export const includes = (arr, value) => arr.some((item) => Object.is(item, value));

/**
 * @template T
 * @param {readonly T[]} arr
 * @returns {T}
 */
export function randomFromArray(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}
