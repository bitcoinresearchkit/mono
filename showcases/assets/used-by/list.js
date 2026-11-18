// Who uses Bitview, each linking to its own website, its logo in this folder: the one list the README's "Used by",
// the project page's row (/dev/#used-by) and the API page's line are written from, by showcases/home.mjs, which
// also checks this list against the folder's files.
//
// Each logo fetched once, 2026-11-18, from the org's own site, and kept as an opaque square of 128 px (or SVG):
//   checkonchain     https://www.checkonchain.com/ (its 512 px icon, cdn.prod.website-files.com/…_C_logo_small_dark.jpeg)
//   newhedge         https://newhedge.io/apple-touch-icon.png
//   qfilabs          https://qfilabs.substack.com/ (its og:image, substack-post-media/…/66937bef-…_1024x1024.png)
//   thebitcoinlayer  https://thebitcoinlayer.com/lovable-uploads/9f953670-6683-46f0-b3ac-043854f31b3a.png, set on white
//   papanglabs       https://papanglabs.com/images/favicon.svg, set on white (its dark-mode swap dropped for it)
//   atreidis         https://atreidis.app/apple-touch-icon.png
//   sparrowwallet    https://sparrowwallet.com/assets/images/sparrow.png, set on white
//   lookintobitcoin  https://storage.googleapis.com/lookintobitcoin/static/apple-touch-icon.18f6aafd6cf2.png (its site's)

/** @typedef {{ name: string, url: string, logo: string }} User */

/** In the order shown. `logo`: the file's name in this folder. @type {User[]} */
export const USERS = [
  { name: "Checkonchain", url: "https://www.checkonchain.com/", logo: "checkonchain.png" },
  { name: "Newhedge", url: "https://newhedge.io/", logo: "newhedge.png" },
  { name: "QFI Labs", url: "https://qfilabs.substack.com/", logo: "qfilabs.png" },
  { name: "The Bitcoin Layer", url: "https://thebitcoinlayer.com/", logo: "thebitcoinlayer.png" },
  { name: "Papang Labs", url: "https://papanglabs.com/", logo: "papanglabs.svg" },
  { name: "Atreidis", url: "https://atreidis.app/", logo: "atreidis.png" },
  { name: "Sparrow Wallet", url: "https://sparrowwallet.com/", logo: "sparrowwallet.png" },
  { name: "LookIntoBitcoin", url: "https://www.lookintobitcoin.com/", logo: "lookintobitcoin.png" },
];
