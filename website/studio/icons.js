// @ts-check
// Studio's icons, and the small drawings of chart types, line shapes and marks, as SVG.

/**
 * The style table every icon in the site is drawn to (icons-check.mjs measures all of them against it).
 * One grid: an icon is drawn at the size it is shown, 16 units for 16 px, so a unit is a pixel and nothing is scaled.
 * A 1.5 stroke has edges on whole pixels at 2× only when its centre sits on .25 or .75, so every straight edge does:
 * a drawing of lines alone is centred on 7.75, a framed one (FRAME) on 8, a quarter unit apart, the price of crisp edges.
 */
const STYLE = {
  grid: 16,
  stroke: 1.5,
  /** Every point of a drawing is a multiple of this. */
  snap: 0.25,
  /** Ink stays this far from the edge of the grid. */
  pad: 0.5,
  /** Corner radii of rects and rounded paths: small parts, frames. */
  radii: [1, 2],
  /** Clear space between the ink of two separate parts, so they don't merge at 16 px (parts that touch are one). */
  minGap: 1,
  /** Straight lines run at these degrees from the horizontal: whole pixel steps, 1:1 and 2:1. */
  angles: [0, 26.565, 45, 63.435, 90],
  /** The keyline circle's outer diameter; a drawing's longer side is at least `min`. */
  keyline: { circle: 13, min: 9 },
  /** Ink as a multiple of the keyline circle's: past these an icon looks darker or fainter than its neighbours. A frame with three lines inside (table) is the heaviest, at 1.8; the next is 1.61. */
  weight: [0.15, 1.8],
};
/** Rules an icon is allowed to break, each with its reason. */
const ICON_EXEMPT = /** @type {Record<string, { rules: string[], why: string }>} */ ({
  weight: { rules: ["stroke"], why: "it draws three stroke widths" },
  star: { rules: ["angle", "gap"], why: "a five-point star's edges run at 36° and 72°, its tips narrower than 45°" },
  bolt: { rules: ["angle", "gap"], why: "a lightning bolt's diagonals lean 56° and its stroke turns back sharply at both tips; at the table's 45° or 63° it reads as a shuriken or a stack of parallels" },
});

const svg = (body, attrs = `width="${STYLE.grid}" height="${STYLE.grid}"`) => `<svg viewBox="0 0 ${STYLE.grid} ${STYLE.grid}" ${attrs} fill="none" stroke="currentColor" stroke-width="${STYLE.stroke}" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${body}</svg>`;
// Drawn once, shared: the frame of every panel, the lens of every search, the dot of every grip.
const FRAME = '<rect x="2.25" y="2.75" width="11.5" height="10.5" rx="2"/>';
const LENS = '<circle cx="7.25" cy="7.25" r="4.5"/><path d="m10.5 10.5 3.25 3.25"/>';
const dot = (cx, cy, r = 1) => `<circle cx="${cx}" cy="${cy}" r="${r}" fill="currentColor" stroke="none"/>`;
const CHEVRON_LEFT = svg('<path d="M9.75 3.25 5.25 7.75l4.5 4.5"/>');
const CHEVRON_RIGHT = svg('<path d="m6.25 3.25 4.5 4.5-4.5 4.5"/>');
const ARROW = {
  up: '<path d="M7.75 13.25v-10.5M3.75 6.75l4-4 4 4"/>',
  down: '<path d="M7.75 2.75v10.5M3.75 9.25l4 4 4-4"/>',
  right: '<path d="M2.75 7.75h10.5M9.25 3.75l4 4-4 4"/>',
  left: '<path d="M13.25 7.75h-10.5M6.75 3.75l-4 4 4 4"/>',
};
const CLOCK = svg('<circle cx="7.75" cy="7.75" r="5.5"/><path d="M7.75 4.75v3l2 2"/>');
const FOLDER_CUT = "M7.25 13.25h-4.5a1 1 0 0 1-1-1v-8.5a1 1 0 0 1 1-1h3.5l1.5 1.5h5.5a1 1 0 0 1 1 1v2.5";
const ICONS = {
  plus: svg('<path d="M7.75 2.75v10M2.75 7.75h10"/>'),
  computer: svg('<rect x="2.25" y="2.75" width="11.5" height="7.5" rx="2"/><path d="M7.75 10.25v3M5.25 13.25h5"/>'),
  sun: svg('<circle cx="7.75" cy="7.75" r="3"/><path d="M7.75 1.25v1M7.75 13.25v1M1.25 7.75h1M13.25 7.75h1M2.75 2.75l1 1M11.75 11.75l1 1M2.75 12.75l1-1M11.75 3.75l1-1"/>'),
  moon: svg('<path d="M7.75 2.25A6 6 0 1 0 13.75 8.75 4.5 4.5 0 0 1 7.75 2.25Z"/>'),
  formula: svg('<path d="M11.75 2.75h-2a2 2 0 0 0-2 2v6.5a2 2 0 0 1-2 2h-2M5.25 7.25h5"/>'),
  charts: svg(`${FRAME}<path d="m4.75 10.75 2.5-2.5 1.5 1.5 2.5-2.5"/>`),
  grid: svg('<rect x="2.25" y="2.25" width="4" height="4" rx="1"/><rect x="9.75" y="2.25" width="4" height="4" rx="1"/><rect x="2.25" y="9.75" width="4" height="4" rx="1"/><rect x="9.75" y="9.75" width="4" height="4" rx="1"/>'),
  browse: svg(LENS),
  keyboard: svg(`<rect x="2.25" y="3.75" width="11.5" height="8.5" rx="2"/>${dot(5, 6.5)}${dot(8, 6.5)}${dot(11, 6.5)}<path d="M5.25 9.75h5.5"/>`),
  sidebar: svg(`${FRAME}<path d="M6.25 2.75v10.5"/>`),
  arrowUp: svg(ARROW.up),
  arrowDown: svg(ARROW.down),
  // (Its page: a row's, View's wherever it's offered.)
  arrowRight: svg(ARROW.right),
  axis: svg('<path d="M12.75 2.75v10.5M12.75 5.25h1.5M12.75 8.25h1.5M12.75 11.25h1.5"/><path d="m2.25 10.75 3-3 2 2 3-3"/>'),
  swap: svg('<path d="M2.75 5.25h10.5M10.75 2.75l2.5 2.5-2.5 2.5"/><path d="M13.25 10.75H2.75M5.25 8.25l-2.5 2.5 2.5 2.5"/>'),
  panel: svg(`${FRAME}<path d="M9.75 2.75v10.5"/>`),
  pane: svg(`${FRAME}<path d="M2.25 8.75h11.5"/>`),
  more: svg(`${dot(3.5, 8, 1.5)}${dot(8, 8, 1.5)}${dot(12.5, 8, 1.5)}`),
  next: CHEVRON_RIGHT,
  back: CHEVRON_LEFT,
  left: CHEVRON_LEFT,
  presets: svg('<path d="M4.25 2.75h7.5"/><rect x="2.25" y="5.75" width="11.5" height="7.5" rx="2"/><path d="m4.75 10.75 2.5-2.5 2 2 1.5-1.5"/>'),
  right: CHEVRON_RIGHT,
  pencil: svg('<path d="M2.75 13.25v-3l7.5-7.5 3 3-7.5 7.5Z"/><path d="m7.25 5.75 3 3"/>'),
  sliders: svg('<path d="M2.25 5.25h6M12.25 5.25h1.5M2.25 10.75h1M7.25 10.75h6.5"/><circle cx="10.25" cy="5.25" r="2"/><circle cx="5.25" cy="10.75" r="2"/>'),
  undo: svg('<path d="M5.25 3.25 2.25 6.25l3 3"/><path d="M2.25 6.25h7.5a3.5 3.5 0 0 1 0 7h-3.5"/>'),
  redo: svg('<path d="m10.25 3.25 3 3-3 3"/><path d="M13.25 6.25h-7.5a3.5 3.5 0 0 0 0 7h3.5"/>'),
  halvings: svg('<path d="M2.25 3.75h3.5v4h4v4h4"/>'),
  eye: svg('<path d="M1.75 7.75C3.25 5 5.25 3.25 7.75 3.25s4.5 1.75 6 4.5c-1.5 2.75-3.5 4.5-6 4.5s-4.5-1.75-6-4.5Z"/><circle cx="7.75" cy="7.75" r="2"/>'),
  eyeOff: svg('<path d="M1.75 7.75C3.25 5 5.25 3.25 7.75 3.25s4.5 1.75 6 4.5c-1.5 2.75-3.5 4.5-6 4.5s-4.5-1.75-6-4.5Z"/><path d="m2.75 2.75 10.5 10.5"/>'),
  close: svg('<path d="m4.25 4.25 7.5 7.5M11.75 4.25l-7.5 7.5"/>'),
  copy: svg('<rect x="5.25" y="5.25" width="9" height="9" rx="2"/><path d="M9.75 1.75h-6a2 2 0 0 0-2 2v6"/>'),
  duplicate: svg('<rect x="5.25" y="5.25" width="9" height="9" rx="2"/><path d="M9.75 1.75h-6a2 2 0 0 0-2 2v6M9.75 8.25v3M8.25 9.75h3"/>'),
  chain: svg('<path d="M6.25 5.25h-2a2.75 2.75 0 0 0 0 5.5h2M9.75 5.25h2a2.75 2.75 0 0 1 0 5.5h-2M5.75 7.75h4.5"/>'),
  link: svg('<path d="M9.75 2.25h4v4M13.75 2.25l-6 6M6.25 3.25h-2a2 2 0 0 0-2 2v6.5a2 2 0 0 0 2 2h6.5a2 2 0 0 0 2-2v-2"/>'),
  trash: svg('<path d="M2.25 4.25h11.5M6.25 4.25v-1a1 1 0 0 1 1-1h1.5a1 1 0 0 1 1 1v1M3.75 4.25v8a2 2 0 0 0 2 2h4.5a2 2 0 0 0 2-2v-8"/>'),
  // (Flipped: a shape and its mirror across a dotted axis, left for right, top for bottom.)
  flipX: svg('<path d="M1.75 4.25v7l3.5-3.5ZM13.75 4.25v7l-3.5-3.5ZM7.75 2.25v1M7.75 5.75v1M7.75 9.25v1M7.75 12.75v1"/>'),
  flipY: svg('<path d="M4.25 1.75h7l-3.5 3.5ZM4.25 13.75h7l-3.5-3.5ZM2.25 7.75h1M5.75 7.75h1M9.25 7.75h1M12.75 7.75h1"/>'),
  folder: svg('<path d="M2.75 2.75h3.5l1.5 1.5h5.5a1 1 0 0 1 1 1v7a1 1 0 0 1-1 1h-10.5a1 1 0 0 1-1-1v-8.5a1 1 0 0 1 1-1Z"/>'),
  folderOpen: svg('<path d="M13.25 7.25v-2a1 1 0 0 0-1-1H7.75L6.25 2.75h-3a1 1 0 0 0-1 1v3.5"/><path d="M2.25 13.25h8l3-6h-8Z"/>'),
  folderOut: svg(`<path d="${FOLDER_CUT}"/><path d="M12.25 14.25v-4.5M10.25 11.75l2-2 2 2"/>`),
  // (A preset section's own, and a folder's to pick: see FOLDER_ICONS.)
  candles: svg('<path d="M5.25 2.25v2.5M5.25 10.75v3M10.75 3.25v3M10.75 11.75v2"/><rect x="3.75" y="4.75" width="3" height="6" rx="1"/><rect x="9.25" y="6.25" width="3" height="5.5" rx="1"/>'),
  scale: svg('<path d="M7.75 2.25v11.5M4.75 13.75h6M2.25 4.25h11M3.25 4.25l-2 4h4zM12.25 4.25l-2 4h4z"/>'),
  coins: svg('<path d="M2.75 4.25C2.75 3.15 5 2.25 7.75 2.25s5 .9 5 2-2.25 2-5 2-5-.9-5-2ZM2.75 4.25v7c0 1.1 2.25 2 5 2s5-.9 5-2v-7M2.75 7.75c0 1.1 2.25 2 5 2s5-.9 5-2"/>'),
  people: svg('<circle cx="5.75" cy="4.75" r="2.5"/><path d="M2.25 13.75v-1a2 2 0 0 1 2-2h3a2 2 0 0 1 2 2v1M10.75 2.75a2.5 2.5 0 0 1 0 5M13.75 13.75v-1a2 2 0 0 0-2-2"/>'),
  hourglass: svg('<path d="M3.75 2.25h8M3.75 13.25h8M4.75 2.25c0 2.5 1.75 3.25 1.75 5.5s-1.75 3-1.75 5.5M10.75 2.25c0 2.5-1.75 3.25-1.75 5.5s1.75 3 1.75 5.5"/>'),
  percent: svg('<path d="M13.25 2.75 2.75 13.25"/><circle cx="4.5" cy="4.5" r="1.75"/><circle cx="11.5" cy="11.5" r="1.75"/>'),
  cube: svg('<path d="M7.75 2.25 13.75 5.25v5.5L7.75 13.75 1.75 10.75v-5.5ZM1.75 5.25l6 3 6-3M7.75 8.25v5.5"/>'),
  dollar: svg('<path d="M7.75 1.75v12.5M11.25 4.25h-5a2 2 0 0 0 0 4h3a2 2 0 0 1 0 4h-5"/>'),
  key: svg('<circle cx="5.25" cy="10.75" r="3"/><path d="M7.5 8.5 13.75 2.25M11.25 4.75l2 2M9.25 6.75l2 2"/>'),
  pickaxe: svg('<path d="M5.25 2.75a8 8 0 0 1 8 8M2.75 13.25l8.5-8.5"/>'),
  nodes: svg(`<path d="M7.75 3.75l-4 8h8z"/>${dot(7.75, 3.75, 1.75)}${dot(3.75, 11.75, 1.75)}${dot(11.75, 11.75, 1.75)}`),
  clock: CLOCK,
  layers: svg('<path d="M7.75 2.25 13.75 5.25l-6 3-6-3ZM1.75 8.25l6 3 6-3M1.75 11.25l6 3 6-3"/>'),
  star: svg('<path d="M7.75 1.75 9.25 6l4.5.25-3.5 2.75 1.25 4.25-3.75-2.5-3.75 2.5L5.25 9l-3.5-2.75L6.25 6Z"/>'),
  flag: svg('<path d="M3.75 13.75v-11.5M3.75 3.25h8l-2.5 2.5 2.5 2.5h-8"/>'),
  bolt: svg('<path d="M9.25 1.75 4.25 9.25h4l-1.5 5 5-7.5h-4Z"/>'),
  target: svg('<circle cx="7.75" cy="7.75" r="5.5"/><circle cx="7.75" cy="7.75" r="2.5"/>'),
  folderAdd: svg(`<path d="${FOLDER_CUT}"/><path d="M12.25 10.75v4M10.25 12.75h4"/>`),
  series: svg('<path d="m1.75 12.25 4.5-4.5 3 3 4.5-4.5"/>'),
  recent: CLOCK,
  calendar: svg('<rect x="2.25" y="3.25" width="11.5" height="10.5" rx="2"/><path d="M2.25 6.75h11.5M5.25 1.75v1.5M10.25 1.75v1.5"/>'),
  share: svg('<path d="M7.75 10.25v-8M5.25 4.75l2.5-2.5 2.5 2.5"/><path d="M5.25 7.75h-.5a2 2 0 0 0-2 2v2.5a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2v-2.5a2 2 0 0 0-2-2h-.5"/>'),
  upload: svg('<path d="M7.75 10.25v-7.5M4.75 5.75l3-3 3 3"/><path d="M2.75 10.25v2.5a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1v-2.5"/>'),
  download: svg('<path d="M7.75 2.75v7.5M4.75 7.25l3 3 3-3"/><path d="M2.75 10.25v2.5a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1v-2.5"/>'),
  image: svg(`${FRAME}${dot(5.5, 6.5)}<path d="M4.25 13.25 9.75 7.75l4 4"/>`),
  table: svg(`${FRAME}<path d="M2.25 6.25h11.5M2.25 9.75h11.5M6.25 6.25v7"/>`),
  file: svg('<path d="M5.25 1.75h3l4 4v6.5a2 2 0 0 1-2 2h-5a2 2 0 0 1-2-2v-8.5a2 2 0 0 1 2-2ZM8.25 1.75v4h4"/>'),
  check: svg('<path d="m3.25 8.25 3.5 3.5 6-6"/>'),
  ruler: svg('<path d="m2.25 11.25 9-9 3 3-9 9ZM4.75 8.75l1 1M7.25 6.25l1 1M9.75 3.75l1 1"/>'),
  magnifier: svg(`${LENS}<path d="M5.25 7.25h4M7.25 5.25v4"/>`),
  // (Zoom out: the magnifier's lens, its plus a minus.)
  zoomOut: svg(`${LENS}<path d="M5.25 7.25h4"/>`),
  draw: svg('<path d="M7.75 14.25 2.75 9.25v-4l2-4h6l2 4v4Z"/><circle cx="7.75" cy="7.75" r="1.5"/><path d="M7.75 9.25v5"/>'),
  ellipse: svg('<ellipse cx="7.75" cy="7.75" rx="6" ry="4.5"/>'),
  arrow: svg('<path d="m3.25 12.75 9.5-9.5M7.25 3.25h5.5v5.5"/>'),
  label: svg('<path d="M1.75 7.75h3M10.75 7.75h3"/><rect x="4.75" y="4.75" width="6" height="6" rx="1"/>'),
  legend: svg('<path d="M2.25 4.75h2.5M7.25 4.75h6.5M2.25 10.75h2.5M7.25 10.75h4.5"/>'),
  keep: svg('<path d="M5.25 2.25h5a1 1 0 0 1 1 1v10.5l-3.5-3.5-3.5 3.5V3.25a1 1 0 0 1 1-1Z"/>'),
  home: svg('<path d="m2.25 7.75 5.5-5.5 5.5 5.5"/><path d="M3.75 6.25v6.5a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1v-6.5M6.25 13.75v-2.5a1.5 1.5 0 0 1 3 0v2.5"/>'),
  checkbox: svg('<rect x="2.25" y="2.25" width="11.5" height="11.5" rx="2"/><path d="m5.25 8.25 2 2 3.5-3.5"/>'),
  expand: svg('<path d="M2.25 6.25v-4h4M9.25 2.25h4v4M13.25 9.25v4h-4M6.25 13.25h-4v-4"/>'),
  shrink: svg('<path d="M6.25 2.25v4h-4M13.25 6.25h-4v-4M9.25 13.25v-4h4M2.25 9.25h4v4"/>'),
  select: svg('<path d="M4.75 2.25h-1.5a1 1 0 0 0-1 1v1.5M7.25 2.25h1M10.75 2.25h1.5a1 1 0 0 1 1 1v1.5M13.25 7.25v1M13.25 10.75v1.5a1 1 0 0 1-1 1h-1.5M7.25 13.25h1M4.75 13.25h-1.5a1 1 0 0 1-1-1v-1.5M2.25 7.25v1"/>'),
  cursor: svg('<path d="M3.75 2.25v11l3-3h5Z"/><path d="m7.75 10.25 2 4"/>'),
  opacity: svg('<circle cx="8" cy="8" r="5.75"/><path d="M8 4.5a3.5 3.5 0 0 1 0 7Z" fill="currentColor"/>'),
  weight: svg('<path d="M2.75 3.5h10.5" stroke-width="1"/><path d="M2.75 7h10.5" stroke-width="2"/><path d="M2.75 12h10.5" stroke-width="3"/>'),
  grip: svg(`${dot(6, 4)}${dot(10, 4)}${dot(6, 8)}${dot(10, 8)}${dot(6, 12)}${dot(10, 12)}`),
  trend: svg('<path d="M4.75 11.25 11.25 4.75"/><circle cx="3.5" cy="12.5" r="1.75"/><circle cx="12.5" cy="3.5" r="1.75"/>'),
  path: svg('<path d="m4.5 10 2-4M9 5.5l2 2"/><circle cx="3.5" cy="12" r="1.75"/><circle cx="7.5" cy="4" r="1.75"/><circle cx="12.5" cy="9" r="1.75"/>'),
  hline: svg('<path d="M1.75 7.75h4M9.75 7.75h4"/><circle cx="7.75" cy="7.75" r="2"/>'),
  vline: svg('<path d="M7.75 1.75v4M7.75 9.75v4"/><circle cx="7.75" cy="7.75" r="2"/>'),
  rect: svg('<rect x="2.25" y="3.25" width="11.5" height="9.5" rx="1"/>'),
  text: svg('<path d="M3.25 3.25h9M7.75 3.25v9.5M5.75 12.75h4"/>'),
  toTop: svg('<path d="M2.75 2.75h10M7.75 13.25v-7.5M4.25 9.25l3.5-3.5 3.5 3.5"/>'),
  toBottom: svg('<path d="M2.75 13.25h10M7.75 2.75v7.5M4.25 6.75l3.5 3.5 3.5-3.5"/>'),
  profile: svg('<g fill="currentColor" stroke="none"><rect x="8" y="1.5" width="6" height="2.5" rx="1"/><rect x="4" y="5" width="10" height="2.5" rx="1"/><rect x="2" y="8.5" width="12" height="2.5" rx="1" fill-opacity=".5"/><rect x="9" y="12" width="5" height="2.5" rx="1" fill-opacity=".5"/></g>'),
  heatmap: svg('<g fill="currentColor" stroke="none"><rect x="2" y="2" width="3" height="3" rx="1" fill-opacity=".25"/><rect x="6.5" y="2" width="3" height="3" rx="1" fill-opacity=".55"/><rect x="11" y="2" width="3" height="3" rx="1" fill-opacity=".9"/><rect x="2" y="6.5" width="3" height="3" rx="1" fill-opacity=".55"/><rect x="6.5" y="6.5" width="3" height="3" rx="1"/><rect x="11" y="6.5" width="3" height="3" rx="1" fill-opacity=".4"/><rect x="2" y="11" width="3" height="3" rx="1" fill-opacity=".9"/><rect x="6.5" y="11" width="3" height="3" rx="1" fill-opacity=".4"/><rect x="11" y="11" width="3" height="3" rx="1" fill-opacity=".2"/></g>'),
  cloud: svg('<path d="M1.5 6.25c2-2 4-2.25 6.25-1.25s4 1 6.25-1.25V8.75c-2.25 2.25-4 2.25-6.25 1.25s-4-.5-6.25 1.25Z" fill="currentColor" fill-opacity=".4" stroke="none"/><path d="M1.5 6.25c2-2 4-2.25 6.25-1.25s4 1 6.25-1.25M1.5 11.25c2-2 4-2.25 6.25-1.25s4 1 6.25-1.25"/>'),
  // (A signal: a mark pointing down onto a line.)
  signal: svg('<path d="M1.5 13.25c2.5-1.5 4-3.5 6.25-3.5s4.25 1.75 6.25.75" stroke-opacity=".45"/><path d="M7.75 2.25v5M5.25 4.75l2.5 2.5 2.5-2.5"/>'),
  riding: svg('<path d="M1.5 6.25c2-2 4-2.25 6.25-1.25s4 1 6.25-1.25M1.5 11.25c2-2 4-2.25 6.25-1.25s4 1 6.25-1.25" stroke-opacity=".35"/><path d="M1.5 11.25c2-2 4-2.25 6.25-1.25M7.75 5c2.25 1 4 1 6.25-1.25"/>'),
  brush: svg('<path d="M11.75 2.25 13.75 4.25 9.25 8.75 7.25 6.75Z"/><path d="M8.25 7.75C6.5 7.25 4.75 8.25 4.5 10 4.25 11.25 3.75 12.5 2.5 13.25c2.5 1 5.5.25 6-1.75.25-1.25 0-2.5-.75-3.75Z"/>'),
};
/**
 * A key's symbols as keycaps draw them, one family with the icons (their stroke, their ends): a font draws each its
 * own way (⌘ thin, ⇧ and ↵ uneven, some from another font), unlike each other and the icons beside them. Sized in em
 * with the key's own text, so shown a little under 16 px.
 */
const KEY_GLYPHS = /** @type {Record<string, string>} */ ({
  "⌘": '<path d="M9.25 4.5V11a1.75 1.75 0 1 0 1.75-1.75H4.5A1.75 1.75 0 1 0 6.25 11V4.5A1.75 1.75 0 1 0 4.5 6.25H11A1.75 1.75 0 1 0 9.25 4.5"/>',
  "⇧": '<path d="m7.75 2.75-5 5h3v5.5h4v-5.5h3Z"/>',
  "⌥": '<path d="M2.75 4.25h3.5l3.5 7h3.5M9.75 4.25h3.5"/>',
  "⌃": '<path d="m3.75 9.75 4-4 4 4"/>',
  "↵": '<path d="M12.75 3.25v4.5a2 2 0 0 1-2 2h-7.5M6.25 6.75l-3 3 3 3"/>',
  "⇥": '<path d="M2.75 7.75h8M7.75 4.75l3 3-3 3M13.25 3.75v8"/>',
  "⌫": '<path d="M5.75 3.75h6.5a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1h-6.5l-4-4ZM8 6.5l2.5 2.5M10.5 6.5 8 9"/>',
  "←": ARROW.left,
  "→": ARROW.right,
  "↑": ARROW.up,
  "↓": ARROW.down,
});
/** A key's symbol, drawn: an svg as wide as the key's text is tall. */
const keyGlyph = (/** @type {string} */ glyph) => (KEY_GLYPHS[glyph] ? svg(KEY_GLYPHS[glyph], 'class="key"') : "");
/** The caret of a control that opens a list. */
const CHEVRON = `<svg viewBox="0 0 10 6" aria-hidden="true"><path d="M1.25 1.25 5 5l3.75-3.75" fill="none" stroke="currentColor" stroke-width="${STYLE.stroke}" stroke-linecap="round" stroke-linejoin="round"/></svg>`;
const glyphSvg = (/** @type {string} */ body) => `<svg class="glyph" viewBox="0 0 16 12" aria-hidden="true">${body}</svg>`;
/** Each chart type, drawn on whole pixels at 16×12: crisp at any pixel density. Candles as on the chart: filled when falling, hollow when rising. */
const glyph = (type) => {
  const body = {
    line: '<path d="M1.5 9.5l4-4 3 3 6-6" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>',
    dots: '<circle cx="2.5" cy="9.5" r="1.5"/><circle cx="5.5" cy="6.5" r="1.5"/><circle cx="8.5" cy="8.5" r="1.5"/><circle cx="11.5" cy="5.5" r="1.5"/><circle cx="14.5" cy="2.5" r="1.5"/>'.replaceAll("/>", ' fill="currentColor"/>'),
    area: '<path d="M1.5 9.5l4-4 3 3 6-6V11h-13z" fill="currentColor" opacity=".25"/><path d="M1.5 9.5l4-4 3 3 6-6" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>',
    histogram: '<rect x="1" y="6" width="2" height="5" rx=".5" fill="currentColor"/><rect x="5" y="2" width="2" height="9" rx=".5" fill="currentColor"/><rect x="9" y="5" width="2" height="6" rx=".5" fill="currentColor"/><rect x="13" y="3" width="2" height="8" rx=".5" fill="currentColor"/>',
    baseline: '<path d="M1 6.5h14" stroke="currentColor" stroke-dasharray="2 2" opacity=".5"/><path d="M1.5 4.5l3-2.5 4 5.5 2.5 2 3.5-1.5" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>',
    candles: '<path d="M4.5 3v8M11.5 1v2.5M11.5 8.5V10" stroke="currentColor"/><rect x="2" y="5" width="5" height="5" rx=".5" fill="currentColor"/><rect x="9.5" y="3.5" width="4" height="5" rx=".5" fill="none" stroke="currentColor"/>',
    bubbles: '<circle cx="4.5" cy="7.5" r="2.5" fill="currentColor" fill-opacity=".35" stroke="currentColor"/><circle cx="11" cy="5.5" r="4" fill="currentColor" fill-opacity=".35" stroke="currentColor"/>',
  }[type] ?? (type === "signal" ? markBody("above", "currentColor") : "");
  return glyphSvg(body);
};
/** A signal's mark, drawn at 16×12 as the chart has it: a dot resting on its layer (a faint line) from above, hanging under it, or on it; shaded, a band. */
const markBody = (/** @type {string | undefined} */ place, /** @type {string} */ color) => {
  if (place === "shade") return `<rect x="4.5" y="1" width="7" height="10" rx="1" fill="${color}" fill-opacity=".4"/>`;
  const [line, dot] = place === "on" ? [6, 6] : place === "below" ? [2.5, 6.8] : [9.5, 5.2];
  return `<path d="M1.5 ${line}h13" stroke="${color}" stroke-opacity=".4" stroke-width="1.4"/><circle cx="8" cy="${dot}" r="3" fill="${color}"/>`;
};
/** A line shape, drawn on whole pixels at 16×12. */
const shapeGlyph = (shape) => glyphSvg(`<path d="${{ straight: "M1.5 9.5l4-4 3 3 6-6", steps: "M1.5 9.5h3v-4h4v3h3v-6h3", curved: "M1.5 9.5C4 9.5 4 5 6.5 5s2.5 3.5 5 3.5 2-6 3-6" }[shape]}" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>`);
/** An outline: a line over its band. */
const OUTLINE_GLYPH = glyphSvg('<path d="M1.5 9.5l4-4 3 3 6-6" stroke="currentColor" stroke-width="4" opacity=".3" fill="none" stroke-linecap="round" stroke-linejoin="round"/><path d="M1.5 9.5l4-4 3 3 6-6" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>');
/** A line style, its dashes drawn on whole pixels at 16×12. */
const dashGlyph = (dash, gap) => glyphSvg(`${Array.from({ length: Math.floor((14 + gap) / (dash + gap)) }, (_, k) => `<rect x="${1 + k * (dash + gap)}" y="5" width="${dash}" height="2" fill="currentColor"/>`).join("")}`);

export { CHEVRON, dashGlyph, glyph, glyphSvg, ICON_EXEMPT, ICONS, KEY_GLYPHS, keyGlyph, markBody, OUTLINE_GLYPH, shapeGlyph, STYLE };
