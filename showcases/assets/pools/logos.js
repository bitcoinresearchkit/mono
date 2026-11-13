// The pool logos in this folder, for every page that draws a pool: a page asks only for a logo there is, the default one
// for a pool without (or its own stand-in, its initials). The two lists are written from the files here by
// crates/bitview_primitives/pools.sh, and checked by showcases/home.mjs.

/** The pools with a logo (`<slug>.svg`). */
export const LOGOS = new Set([
  "1thash", "antpool", "arkpool", "binancepool", "bitcoincom", "bitfufupool", "bitfury", "braiinspool",
  "braiinssolo", "btclab", "btctop", "cloverpool", "default", "dmnd", "emcdpool", "est3lar", "f2pool", "foundryusa",
  "gdpool", "huobipool", "innopolistech", "kucoinpool", "luxor", "marapool", "maxipool", "minerium", "miningsquared",
  "mononaut", "neopool", "nicehash", "noderunners", "ocean", "okexpool", "okkong", "parasite", "pegapool", "phoenix",
  "poolin", "publicpool", "rawpool", "sbicrypto", "secpool", "sigmapoolcom", "slushpool", "solopoolcom",
  "spiderpool", "terrapool", "titan", "trustpool", "ultimuspool", "unknown", "viabtc", "wayicn", "whitepool", "wiz",
]);
/** Those with one of their own for a light ground (`<slug>.light.svg`), a dark ground's not showing on it. */
export const LIGHT = new Set([
  "default", "foundryusa", "ultimuspool", "unknown", "whitepool",
]);

/**
 * A pool's logo for the ground it's drawn on, its file's address: the default one for a pool without.
 * @param {string} slug
 * @param {boolean} light
 */
export const logo = (slug, light) => {
  const file = LOGOS.has(slug) ? slug : "default";
  return `/assets/pools/${file}${light && LIGHT.has(file) ? ".light" : ""}.svg`;
};
