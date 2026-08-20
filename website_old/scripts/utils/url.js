/**
 * @param {string | string[]} [pathname]
 */
function processPathname(pathname) {
  pathname ||= window.location.pathname;
  const result = Array.isArray(pathname) ? pathname.join("/") : pathname;
  // Strip leading slash to avoid double slashes when prepending /
  return result.startsWith("/") ? result.slice(1) : result;
}

const chartParamsWhitelist = ["range"];

/**
 * @param {string | string[]} [pathname]
 * @param {URLSearchParams} [urlParams]
 */
function buildUrl(pathname, urlParams) {
  const path = processPathname(pathname);
  const query = (urlParams ?? new URLSearchParams(window.location.search)).toString();
  return `/${path}${query ? `?${query}` : ""}`;
}

/**
 * @param {string | string[]} pathname
 */
export function pushHistory(pathname) {
  try {
    window.history.pushState(null, "", buildUrl(pathname));
  } catch (_) {}
}

/**
 * @param {Object} args
 * @param {URLSearchParams} [args.urlParams]
 * @param {string | string[]} [args.pathname]
 */
export function replaceHistory({ urlParams, pathname }) {
  try {
    window.history.replaceState(null, "", buildUrl(pathname, urlParams));
  } catch (_) {}
}

/**
 * @param {Option} option
 */
export function resetParams(option) {
  const urlParams = new URLSearchParams();
  if (option.kind === "chart") {
    [...new URLSearchParams(window.location.search).entries()]
      .filter(([key, _]) => chartParamsWhitelist.includes(key))
      .forEach(([key, value]) => {
        urlParams.set(key, value);
      });
  }
  replaceHistory({ urlParams, pathname: option.path.join("/") });
}

/**
 * @param {string} key
 * @param {string | boolean | null | undefined} value
 */
export function writeParam(key, value) {
  const urlParams = new URLSearchParams(window.location.search);

  if (value !== null && value !== undefined) {
    urlParams.set(key, String(value));
  } else {
    urlParams.delete(key);
  }

  replaceHistory({ urlParams });
}

/**
 *
 * @param {string} key
 * @returns {string | null}
 */
export function readParam(key) {
  const params = new URLSearchParams(window.location.search);
  return params.get(key);
}
