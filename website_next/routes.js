const routes = /** @type {const} */ ({
  "/": async () => (await import("./home/index.js")).createHomePage(),
  "/ask": async () => (await import("./ask/index.js")).createAskPage(),
  "/explore": async () =>
    (await import("./explore/index.js")).createExplorePage(),
  "/learn": async () => (await import("./learn/index.js")).createLearnPage(),
  "/build": async () => (await import("./build/index.js")).createBuildPage(),
  "/wallets": async () =>
    (await import("./wallets/index.js")).createWalletsPage(),
});

/** @typedef {keyof typeof routes} RoutePath */

/** @param {string} pathname */
function canonicalPath(pathname) {
  return pathname !== "/" && pathname.endsWith("/")
    ? pathname.slice(0, -1)
    : pathname;
}

/**
 * @param {string} pathname
 * @returns {RoutePath | undefined}
 */
export function resolvePath(pathname) {
  const path = canonicalPath(pathname);

  return path in routes ? /** @type {RoutePath} */ (path) : undefined;
}

/**
 * @param {string} pathname
 * @returns {RoutePath}
 */
export function normalizePath(pathname) {
  return resolvePath(pathname) ?? "/";
}

/**
 * @param {RoutePath} pathname
 * @returns {Promise<HTMLElement>}
 */
export function createRoutePage(pathname) {
  return routes[pathname]();
}
