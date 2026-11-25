// @ts-check
// Studio's icons, and the small drawings of chart types, line shapes and marks, as SVG.

const svg = (body, size = 16) => `<svg viewBox="0 0 16 16" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="1.45" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${body}</svg>`;
const ICONS = {
  plus: svg('<path d="M8 3v10M3 8h10"/>'),
  computer: svg('<rect x="1.5" y="2.5" width="13" height="8.5" rx="1.8"/><path d="M5.5 14h5M8 11v3"/>'),
  sun: svg('<circle cx="8" cy="8" r="2.75"/><path d="M8 1.5v1.2M8 13.3v1.2M1.5 8h1.2M13.3 8h1.2M3.4 3.4l.85.85M11.75 11.75l.85.85M3.4 12.6l.85-.85M11.75 4.25l.85-.85"/>'),
  moon: svg('<path d="M7.8 2.3A6 6 0 1 0 13.98 8.84A4.6 4.6 0 0 1 7.8 2.3Z"/>'),
  formula: svg('<path d="M10.5 2.5c-1.7 0-2.3.8-2.6 2.4L6.4 12c-.3 1.5-.9 2-2.4 2"/><path d="M4.8 6.5H10"/>'),
  charts: svg('<rect x="2" y="2.5" width="12" height="11" rx="2"/><path d="m4.5 10.5 2.2-2.6 1.8 1.6 3-3.5"/>'),
  grid: svg('<rect x="2" y="2" width="5" height="5" rx="1.2"/><rect x="9" y="2" width="5" height="5" rx="1.2"/><rect x="2" y="9" width="5" height="5" rx="1.2"/><rect x="9" y="9" width="5" height="5" rx="1.2"/>'),
  browse: svg('<circle cx="7" cy="7" r="4.5"/><path d="m10.5 10.5 3 3"/>'),
  keyboard: svg('<rect x="1.5" y="3.5" width="13" height="9" rx="1.8"/><path d="M4.5 6.5h.01M7 6.5h.01M9.5 6.5h.01M12 6.5h.01M5 9.5h6"/>'),
  sidebar: svg('<rect x="2" y="2.5" width="12" height="11" rx="2"/><path d="M6 2.5v11"/>'),
  arrowUp: svg('<path d="M8 13V3M4 7l4-4 4 4"/>', 14),
  arrowDown: svg('<path d="M8 3v10M4 9l4 4 4-4"/>', 14),
  // (Its page: a row's, View's wherever it's offered.)
  arrowRight: svg('<path d="M3 8h10M9 4l4 4-4 4"/>', 14),
  axis: svg('<path d="M12 2.5v11M12 5h2M12 8h2M12 11h2"/><path d="M2 11 4.6 7.6l2.2 1.9L9.3 5"/>'),
  swap: svg('<path d="M3 5.5h9.5M10 3l2.5 2.5L10 8"/><path d="M13 10.5H3.5M6 8l-2.5 2.5L6 13"/>', 14),
  panel: svg('<rect x="2" y="2.5" width="12" height="11" rx="2"/><path d="M10 2.5v11"/>'),
  pane: svg('<rect x="2" y="2.5" width="12" height="11" rx="2"/><path d="M2 9h12"/>'),
  more: svg('<circle cx="3.5" cy="8" r=".9" fill="currentColor"/><circle cx="8" cy="8" r=".9" fill="currentColor"/><circle cx="12.5" cy="8" r=".9" fill="currentColor"/>', 14),
  next: svg('<path d="M6 3.5 10.5 8 6 12.5"/>', 12),
  back: svg('<path d="M10 3.5 5.5 8l4.5 4.5"/>', 14),
  left: svg('<path d="M10 3.5 5.5 8l4.5 4.5"/>'),
  presets: svg('<rect x="2" y="4.5" width="9.5" height="9" rx="1.8"/><path d="M5 2.5h7.2c1 0 1.8.8 1.8 1.8V11"/><path d="m4.3 11.2 1.9-2.2 1.5 1.3 1.6-2"/>'),
  right: svg('<path d="M6 3.5 10.5 8 6 12.5"/>'),
  pencil: svg('<path d="m10.5 2.5 3 3-8 8H2.5v-3z"/><path d="m8.5 4.5 3 3"/>', 14),
  sliders: svg('<path d="M2.5 5h6.5M12.5 5h1M2.5 11h1M7 11h6.5"/><circle cx="10.75" cy="5" r="1.75"/><circle cx="5.25" cy="11" r="1.75"/>', 14),
  undo: svg('<path d="M5.5 3.5 2.5 6.5l3 3"/><path d="M2.5 6.5h7a4 4 0 0 1 0 8h-2"/>', 14),
  redo: svg('<path d="m10.5 3.5 3 3-3 3"/><path d="M13.5 6.5h-7a4 4 0 0 0 0 8h2"/>', 14),
  halvings: svg('<path d="M2 4h4v4h4v4h4"/>', 14),
  eye: svg('<path d="M1.5 8s2.4-4.5 6.5-4.5S14.5 8 14.5 8 12.1 12.5 8 12.5 1.5 8 1.5 8Z"/><circle cx="8" cy="8" r="2"/>', 14),
  // (The eye's own outline and pupil, cut where the slash covers them.)
  eyeOff: svg('<path d="M1.5 8C1.5 8 2.6 5.93 4.6 4.6M7.11 3.57C7.4 3.53 7.69 3.5 8 3.5C12.1 3.5 14.5 8 14.5 8C14.5 8 14.07 8.8 13.27 9.73M11.4 11.41C10.46 12.04 9.32 12.5 8 12.5C3.9 12.5 1.5 8 1.5 8"/><path d="M9.41 9.41A2 2 0 0 1 6.59 6.59"/><path d="m2 2 12 12"/>', 14),
  close: svg('<path d="m4 4 8 8M12 4l-8 8"/>', 14),
  copy: svg('<rect x="5.5" y="5.5" width="8" height="8" rx="1.5"/><path d="M10.5 3.5V3a1 1 0 0 0-1-1h-6a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h.5"/>', 14),
  duplicate: svg('<rect x="5.5" y="5.5" width="8" height="8" rx="1.5"/><path d="M10.5 3.5V3a1 1 0 0 0-1-1h-6a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h.5M9.5 7.5v4M7.5 9.5h4"/>', 14),
  chain: svg('<path d="M6.6 9.4 9.4 6.6"/><path d="M7.3 4.6 8.4 3.5a2.65 2.65 0 0 1 3.75 3.75L11 8.4"/><path d="M8.7 11.4 7.6 12.5a2.65 2.65 0 0 1-3.75-3.75L5 7.6"/>', 14),
  link: svg('<path d="M9.5 2.5h4v4M13.5 2.5 7.5 8.5M12 9.5v3a1 1 0 0 1-1 1H3.5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h3"/>', 14),
  trash: svg('<path d="M2.5 4.5h11M6 4.5V3a1 1 0 0 1 1-1h2a1 1 0 0 1 1 1v1.5M4 4.5l.7 8.6a1 1 0 0 0 1 .9h4.6a1 1 0 0 0 1-.9l.7-8.6"/>', 14),
  // (Flipped: a shape and its mirror across a dotted axis, left for right, top for bottom.)
  flipX: svg('<path d="M6 4.5 1.5 11.5H6zM10 4.5l4.5 7H10zM8 2v1M8 5.5v1M8 9.5v1M8 13v1"/>', 14),
  flipY: svg('<path d="M4.5 6 11.5 1.5V6zM4.5 10l7 4.5V10zM2 8h1M5.5 8h1M9.5 8h1M13 8h1"/>', 14),
  folder: svg('<path d="M1.5 4a1 1 0 0 1 1-1h3.6l1.6 1.6h5.8a1 1 0 0 1 1 1V12a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1Z"/>', 15),
  folderOpen: svg('<path d="M2.5 13a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h3.6l1.6 1.6h4.8a1 1 0 0 1 1 1V7"/><path d="M2.5 13h9.4a1 1 0 0 0 .93-.63l1.8-4.5A.64.64 0 0 0 14.04 7H5.1a1 1 0 0 0-.93.63L2.5 13Z"/>', 15),
  folderOut: svg('<path d="M7.5 13h-5a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h3.6l1.6 1.6h5.8a1 1 0 0 1 1 1v2.4"/><path d="M12 14.5v-5M9.8 11.7 12 9.5l2.2 2.2"/>', 15),
  // (A preset section's own, and a folder's to pick: see FOLDER_ICONS.)
  candles: svg('<path d="M5 2.5v2M5 11v2.5M11 3.5v2.5M11 11v2M3.5 4.5h3v6.5h-3zM9.5 6h3v5h-3z"/>', 15),
  scale: svg('<path d="M8 2.5v11M5 13.5h6M3 4.5h10M3 4.5 1.5 9h3zM13 4.5 11.5 9h3z"/>', 15),
  coins: svg('<path d="M3 4.5c0-1.1 2.2-2 5-2s5 .9 5 2-2.2 2-5 2-5-.9-5-2zM3 4.5V8c0 1.1 2.2 2 5 2s5-.9 5-2V4.5M3 8v3.5c0 1.1 2.2 2 5 2s5-.9 5-2V8"/>', 15),
  people: svg('<path d="M6 3.3a2.2 2.2 0 1 0 0 4.4 2.2 2.2 0 1 0 0-4.4zM2 13c.4-2.2 2-3.5 4-3.5s3.6 1.3 4 3.5M11.2 4.2a1.8 1.8 0 1 1 0 3.6M11 9.6c1.6.1 2.8 1.3 3.1 3.1"/>', 15),
  hourglass: svg('<path d="M4 2.5h8M4 13.5h8M5 2.5c0 3 6 3.5 6 5.5s-6 2.5-6 5.5M11 2.5c0 3-6 3.5-6 5.5s6 2.5 6 5.5"/>', 15),
  percent: svg('<path d="M12.5 3.5l-9 9M4.5 2.7a1.8 1.8 0 1 0 0 3.6 1.8 1.8 0 1 0 0-3.6zM11.5 9.7a1.8 1.8 0 1 0 0 3.6 1.8 1.8 0 1 0 0-3.6z"/>', 15),
  cube: svg('<path d="M8 2 13.5 5v6L8 14 2.5 11V5zM2.5 5 8 8l5.5-3M8 8v6"/>', 15),
  dollar: svg('<path d="M8 1.5v13M11.2 3.8H6.7a2.1 2.1 0 0 0 0 4.2h2.6a2.1 2.1 0 0 1 0 4.2H4.8"/>', 15),
  key: svg('<path d="M5.5 7.5a3 3 0 1 0 0 6 3 3 0 1 0 0-6zM7.6 8.4 13 3M11 5l1.8 1.8M9.5 6.5 11 8"/>', 15),
  pickaxe: svg('<path d="M2.5 6.2C5 3.2 11 3.2 13.5 6.2M8 4.2v9.3"/>', 15),
  nodes: svg('<path d="M8 1.9a1.6 1.6 0 1 0 0 3.2 1.6 1.6 0 1 0 0-3.2zM3.5 10.4a1.6 1.6 0 1 0 0 3.2 1.6 1.6 0 1 0 0-3.2zM12.5 10.4a1.6 1.6 0 1 0 0 3.2 1.6 1.6 0 1 0 0-3.2zM7.2 5 4.3 10.6M8.8 5l2.9 5.6M5.1 12h5.8"/>', 15),
  clock: svg('<path d="M8 2.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 1 0 0-11zM8 5v3l2 1.5"/>', 15),
  layers: svg('<path d="M8 2.5 14 5.5 8 8.5 2 5.5zM2 8.5l6 3 6-3M2 11.5l6 3 6-3"/>', 15),
  star: svg('<path d="m8 2.2 1.8 3.7 4 .6-2.9 2.8.7 4L8 11.4l-3.6 1.9.7-4-2.9-2.8 4-.6z"/>', 15),
  flag: svg('<path d="M3.5 14V2.5M3.5 3h8l-1.8 3 1.8 3h-8"/>', 15),
  bolt: svg('<path d="M9 1.8 3.8 9h4L7 14.2 12.2 7h-4z"/>', 15),
  target: svg('<path d="M8 2.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 1 0 0-11zM8 5.5a2.5 2.5 0 1 0 0 5 2.5 2.5 0 1 0 0-5z"/>', 15),
  folderAdd: svg('<path d="M7.5 13h-5a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h3.6l1.6 1.6h5.8a1 1 0 0 1 1 1v2.4"/><path d="M12 9.5v5M9.5 12h5"/>', 15),
  series: svg('<path d="M1.5 12.5 6 7l3 3 5.5-6.5"/>', 15),
  recent: svg('<circle cx="8" cy="8" r="6"/><path d="M8 4.5V8l2.5 1.5"/>', 15),
  calendar: svg('<rect x="2" y="3" width="12" height="11" rx="2"/><path d="M2 6.5h12M5.5 1.5v3M10.5 1.5v3"/>', 15),
  share: svg('<path d="M8 9.5V2M5.5 4.5 8 2l2.5 2.5"/><path d="M5.5 7H4.5a1 1 0 0 0-1 1v5a1 1 0 0 0 1 1h7a1 1 0 0 0 1-1V8a1 1 0 0 0-1-1h-1"/>', 15),
  upload: svg('<path d="M8 10V2.5M5 5.5l3-3 3 3"/><path d="M3 11v1.5a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V11"/>', 15),
  download: svg('<path d="M8 2.5V10M5 7l3 3 3-3"/><path d="M3 11v1.5a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V11"/>', 15),
  image: svg('<rect x="2" y="2.5" width="12" height="11" rx="2"/><circle cx="6" cy="6.5" r="1.2"/><path d="m2.5 12 3.5-3.5 2.5 2.5 2-2 3 3"/>', 14),
  table: svg('<rect x="2" y="2.5" width="12" height="11" rx="2"/><path d="M2 6.5h12M2 10h12M6.5 6.5v7"/>', 14),
  file: svg('<path d="M4 1.5h5l3.5 3.5v9a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-11.5a1 1 0 0 1 1-1Z"/><path d="M9 1.5V5h3.5"/>', 14),
  check: svg('<path d="m4 8.4 2.6 2.6L12 5.4"/>', 12),
  ruler: svg('<path d="m2 11 9-9 3 3-9 9Z"/><path d="m5 8 1.5 1.5M7 6l1 1M9 4l1.5 1.5"/>'),
  magnifier: svg('<circle cx="7" cy="7" r="4.75"/><path d="m10.5 10.5 3.5 3.5"/><path d="M5 7h4M7 5v4"/>'),
  // (Zoom out: the magnifier's lens, its plus a minus.)
  zoomOut: svg('<circle cx="7" cy="7" r="4.75"/><path d="m10.5 10.5 3.5 3.5"/><path d="M5 7h4"/>'),
  draw: svg('<path d="M8 13.5 4 8l2-5.5h4L12 8Z"/><path d="M8 13.5v-5"/><circle cx="8" cy="7.5" r="1"/>'),
  ellipse: svg('<ellipse cx="8" cy="8" rx="6" ry="4.5"/>'),
  arrow: svg('<path d="M3 13 12.5 3.5M7 3.5h5.5V9"/>'),
  label: svg('<path d="M2.5 8h2M11.5 8h2"/><rect x="4.5" y="5" width="7" height="6" rx="1.5"/>'),
  legend: svg('<path d="M2.5 5h2.5M7.5 5h6M2.5 11h2.5M7.5 11h4"/>'),
  keep: svg('<path d="M4.5 2.5h7v11L8 10.6l-3.5 2.9Z"/>'),
  home: svg('<path d="M1.8 7.6 8 2.4l6.2 5.2"/><path d="M3.6 6.3v6.5a1.2 1.2 0 0 0 1.2 1.2h6.4a1.2 1.2 0 0 0 1.2-1.2V6.3"/><path d="M6.5 14v-3.2a1.5 1.5 0 0 1 3 0V14"/>'),
  checkbox: svg('<rect x="2.5" y="2.5" width="11" height="11" rx="3"/><path d="m5.4 8.2 1.8 1.8 3.4-3.7"/>'),
  expand: svg('<path d="M2.5 6V2.5H6M10 2.5h3.5V6M13.5 10v3.5H10M6 13.5H2.5V10"/>'),
  shrink: svg('<path d="M6 2.5V6H2.5M13.5 6H10V2.5M10 13.5V10h3.5M2.5 10H6v3.5"/>'),
  select: svg('<path d="M2.5 5V3.5a1 1 0 0 1 1-1H5M8.5 2.5h-1M11 2.5h1.5a1 1 0 0 1 1 1V5M13.5 8.5v-1M13.5 11v1.5a1 1 0 0 1-1 1H11M7.5 13.5h1M5 13.5H3.5a1 1 0 0 1-1-1V11M2.5 7.5v1"/>'),
  cursor: svg('<path d="M4 2.5 12.5 8.6 8.7 9.3 10.7 13.5 9.1 14.2 7.1 10 4 12.7Z"/>'),
  opacity: svg('<circle cx="8" cy="8" r="5.5"/><path d="M8 2.5a5.5 5.5 0 0 1 0 11Z" fill="currentColor"/>'),
  weight: svg('<path d="M3 4h10" stroke-width="1"/><path d="M3 8h10" stroke-width="1.8"/><path d="M3 12.5h10" stroke-width="2.8"/>'),
  grip: svg('<circle cx="6" cy="4" r="1" fill="currentColor" stroke="none"/><circle cx="10" cy="4" r="1" fill="currentColor" stroke="none"/><circle cx="6" cy="8" r="1" fill="currentColor" stroke="none"/><circle cx="10" cy="8" r="1" fill="currentColor" stroke="none"/><circle cx="6" cy="12" r="1" fill="currentColor" stroke="none"/><circle cx="10" cy="12" r="1" fill="currentColor" stroke="none"/>', 14),
  trend: svg('<path d="M4.4 11.6 11.6 4.4"/><circle cx="3.2" cy="12.8" r="1.6"/><circle cx="12.8" cy="3.2" r="1.6"/>'),
  path: svg('<path d="M4.2 10.2 6.6 5.8M8.9 5.6l2.6 3.2"/><circle cx="3.2" cy="11.9" r="1.6"/><circle cx="7.6" cy="4.1" r="1.6"/><circle cx="12.8" cy="10.3" r="1.6"/>'),
  hline: svg('<path d="M1.5 8h4.8M9.7 8h4.8"/><circle cx="8" cy="8" r="1.6"/>'),
  vline: svg('<path d="M8 1.5v4.8M8 9.7v4.8"/><circle cx="8" cy="8" r="1.6"/>'),
  rect: svg('<rect x="2.5" y="3.5" width="11" height="9" rx="1"/>'),
  text: svg('<path d="M3.5 3.5h9M8 3.5v9M6.2 12.5h3.6"/>'),
  toTop: svg('<path d="M3 2.5h10M8 13.5V6M4.5 9.5 8 6l3.5 3.5"/>'),
  toBottom: svg('<path d="M3 13.5h10M8 2.5V10M4.5 6.5 8 10l3.5-3.5"/>'),
  profile: svg('<g fill="currentColor" stroke="none"><rect x="8" y="2" width="6" height="2.2" rx=".6"/><rect x="4" y="5.3" width="10" height="2.2" rx=".6"/><rect x="2" y="8.6" width="12" height="2.2" rx=".6" fill-opacity=".5"/><rect x="9" y="11.9" width="5" height="2.2" rx=".6" fill-opacity=".5"/></g>'),
  heatmap: svg('<g fill="currentColor" stroke="none"><rect x="2.0" y="2.0" width="3.6" height="3.6" rx=".7" fill-opacity="0.25"/><rect x="6.2" y="2.0" width="3.6" height="3.6" rx=".7" fill-opacity="0.55"/><rect x="10.4" y="2.0" width="3.6" height="3.6" rx=".7" fill-opacity="0.9"/><rect x="2.0" y="6.2" width="3.6" height="3.6" rx=".7" fill-opacity="0.55"/><rect x="6.2" y="6.2" width="3.6" height="3.6" rx=".7" fill-opacity="1"/><rect x="10.4" y="6.2" width="3.6" height="3.6" rx=".7" fill-opacity="0.4"/><rect x="2.0" y="10.4" width="3.6" height="3.6" rx=".7" fill-opacity="0.9"/><rect x="6.2" y="10.4" width="3.6" height="3.6" rx=".7" fill-opacity="0.4"/><rect x="10.4" y="10.4" width="3.6" height="3.6" rx=".7" fill-opacity="0.2"/></g>'),
  cloud: svg('<path d="M2 6.2c2.1-1.9 3.9-2.3 6-1.3s3.9 1 6-1.4v5c-2.1 2.4-3.9 2.4-6 1.4s-3.9-.6-6 1.3Z" fill="currentColor" fill-opacity=".4" stroke="none"/><path d="M2 6.2c2.1-1.9 3.9-2.3 6-1.3s3.9 1 6-1.4M2 11.2c2.1-1.9 3.9-2.3 6-1.3s3.9 1 6-1.4"/>'),
  // (A signal: a mark pointing down onto a line.)
  signal: svg('<path d="M1.5 13.5c2.4-1.6 4-3.6 6.5-3.6s4 1.7 6.5.9" stroke-opacity=".45"/><path d="M8 2.2v5M5.9 5.1 8 7.2l2.1-2.1"/>'),
  riding: svg('<path d="M2 6.2c2.1-1.9 3.9-2.3 6-1.3s3.9 1 6-1.4M2 11.2c2.1-1.9 3.9-2.3 6-1.3s3.9 1 6-1.4" stroke-opacity=".35"/><path d="M2 11.2c2.1-1.9 3.9-2.3 6-1.3M8 4.9c2.1 1 3.9 1 6-1.4" stroke-width="1.9"/>'),
  brush: svg('<path d="M13.6 2.4a1.15 1.15 0 0 1 0 1.65L8.9 8.75 7.25 7.1l4.7-4.7a1.15 1.15 0 0 1 1.65 0Z"/><path d="M6.9 9.1C5.3 8.6 3.9 9.4 3.7 11c-.15 1.2-.7 1.9-1.7 2.3 2.4.9 4.9.3 5.4-1.6.2-.9.05-1.8-.5-2.6Z"/>'),
};
/** The caret of a control that opens a list. */
const CHEVRON = '<svg viewBox="0 0 10 6" aria-hidden="true"><path d="M1 1l4 4 4-4" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/></svg>';
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

export { CHEVRON, dashGlyph, glyph, glyphSvg, ICONS, markBody, OUTLINE_GLYPH, shapeGlyph };
