// Who uses Bitview, each linking to its own website, its logo in this folder: the one list the README's "Used by",
// the project page's strip (/dev/#used-by) and the API page's line are written from, by scripts/home.mjs, which
// also checks this list against the folder's files.
//
// Each logo fetched once, 2026-11-18, from the org's own site, trimmed to its ink and kept as published unless noted.
// `dark` is for dark backgrounds (the README's dark mode); /dev draws `logo`'s shape alone, in the page's ink.
//   sparrowwallet    no wordmark anywhere (its site sets the name as text): its bird from the app's own repository,
//                    github.com/sparrowwallet/sparrow, src/main/resources/image/sparrow-black-small@3x.png, and
//                    sparrow-white-small@3x.png for dark, both untrimmed (one size)
//   lookintobitcoin  https://www.lookintobitcoin.com/svg/oneline_blue-black_transparent_1246w%201.svg (the PNG inside it);
//                    no dark one published: dark is its black made white, its blue kept
//   checkonchain     https://www.checkonchain.com/ (its header's Checkonchain_Logo_2026_WHITE.svg, cdn.prod.website-files.com)
//                    is dark; no light one published: light is its white made ink
//   newhedge         https://newhedge.io/assets/newhedge_logo_fit-1e8c778f….svg (its header's; viewBox added, editor ids
//                    dropped); no dark one published: dark is it filled white
//   thebitcoinlayer  https://thebitcoinlayer.com/logos/tbl-white-edgeless.png is dark; light is a copy of it in black,
//                    as its own site's light mode shows it (there by a filter)
//   qfilabs          no wordmark anywhere (Substack sets the name as text): its mark from its og:image,
//                    substack-post-media/…/66937bef-…_1024x1024.png, its black ground made transparent; reads on both
//   papanglabs       no wordmark anywhere (its header is the mark alone): https://papanglabs.com/images/logo-encre.svg,
//                    and logo-creme.svg for dark, viewBox trimmed to the mark; light with its cream eye stripe cut out
//                    of the head instead, so the ground shows there (alike on a light ground) and /dev's shape keeps
//                    the eye
//   atreidis         its header's inline SVG at https://atreidis.app/, the four bars without its "atreidis.app" text (set
//                    in Geist, which isn't ours); reads on both

/**
 * @typedef {object} User
 * @property {string} name
 * @property {string} url its website
 * @property {string} logo the file's name in this folder, for light backgrounds (and its shape for /dev)
 * @property {string} [dark] the file's name for dark backgrounds, when `logo` doesn't read on them
 * @property {true} [mark] the logo is a mark alone: the name is set beside it, in the page's own font
 * @property {number} [scale] the logo's height against the others' (1), so all read at one optical height
 */

/** In the order shown. @type {User[]} */
export const USERS = [
  { name: "Sparrow Wallet", url: "https://sparrowwallet.com/", logo: "sparrowwallet.png", dark: "sparrowwallet-dark.png", mark: true, scale: 1.1 },
  { name: "Look Into Bitcoin", url: "https://www.lookintobitcoin.com/", logo: "lookintobitcoin.png", dark: "lookintobitcoin-dark.png", scale: 0.85 },
  { name: "Checkonchain", url: "https://www.checkonchain.com/", logo: "checkonchain.svg", dark: "checkonchain-dark.svg", scale: 1.15 },
  { name: "Newhedge", url: "https://newhedge.io/", logo: "newhedge.svg", dark: "newhedge-dark.svg", scale: 0.85 },
  { name: "The Bitcoin Layer", url: "https://thebitcoinlayer.com/", logo: "thebitcoinlayer.png", dark: "thebitcoinlayer-dark.png", scale: 1.3 },
  { name: "QFI Labs", url: "https://qfilabs.substack.com/", logo: "qfilabs.png", mark: true },
  { name: "Papang Labs", url: "https://papanglabs.com/", logo: "papanglabs.svg", dark: "papanglabs-dark.svg", mark: true, scale: 0.85 },
  { name: "Atreidis", url: "https://atreidis.app/", logo: "atreidis.svg", mark: true },
];
