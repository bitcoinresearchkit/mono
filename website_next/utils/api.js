export const BITVIEW_BASE_URL = "https://bitview.space";

const REQUEST_TIMEOUT_MS = 5_000;

/**
 * @template T
 * @param {string} path
 * @param {AbortSignal} [signal]
 * @returns {Promise<T>}
 */
export async function fetchBrkJson(path, signal) {
  const signals = [AbortSignal.timeout(REQUEST_TIMEOUT_MS)];
  if (signal) signals.push(signal);

  const response = await fetch(`${BITVIEW_BASE_URL}${path}`, {
    signal: AbortSignal.any(signals),
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.url}`);
  }

  return response.json();
}

/**
 * @template T
 * @param {string} name
 * @param {string} index
 * @param {number} start
 * @param {number} end
 * @param {AbortSignal} signal
 * @returns {Promise<{ data: T[] }>}
 */
export function fetchBrkSeries(name, index, start, end, signal) {
  const params = new URLSearchParams({
    end: String(end),
    start: String(start),
  });

  return fetchBrkJson(`/api/series/${name}/${index}?${params}`, signal);
}
