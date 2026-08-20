import { dark } from "./theme.js";

/** @type {Map<string, string>} */
const rgbaCache = new Map();

/**
 * Convert oklch to rgba with caching
 * @param {string} color - oklch color string
 */
function toRgba(color) {
  if (color === "transparent") return color;
  const cached = rgbaCache.get(color);
  if (cached) return cached;
  const rgba = oklchToRgba(color);
  rgbaCache.set(color, rgba);
  return rgba;
}

/**
 * Reduce color opacity to 50% for dimming effect
 * @param {string} color - oklch color string
 */
function tameColor(color) {
  if (color === "transparent") return color;
  return `${color.slice(0, -1)} / 25%)`;
}

/**
 * @typedef {Object} ColorMethods
 * @property {() => string} tame - Returns tamed (50% opacity) version
 * @property {(highlighted: boolean) => string} highlight - Returns normal if highlighted, tamed otherwise
 */

/**
 * @typedef {(() => string) & ColorMethods} Color
 */

/**
 * Creates a Color object that is callable and has utility methods
 * @param {() => string} getter
 * @returns {Color}
 */
function createColor(getter) {
  const color = /** @type {Color} */ (() => toRgba(getter()));
  color.tame = () => toRgba(tameColor(getter()));
  color.highlight = (highlighted) =>
    highlighted ? toRgba(getter()) : toRgba(tameColor(getter()));
  return color;
}

const globalComputedStyle = getComputedStyle(window.document.documentElement);

/**
 * Resolve a light-dark() value based on current theme
 * @param {string} value
 */
function resolveLightDark(value) {
  if (value.startsWith("light-dark(")) {
    const [light, _dark] = value.slice(11, -1).split(", ");
    return dark ? _dark : light;
  }
  return value;
}

/**
 * @param {string} name
 */
function getColor(name) {
  return globalComputedStyle.getPropertyValue(`--${name}`).trim();
}

/**
 * @param {string} property
 */
function getLightDarkValue(property) {
  return resolveLightDark(
    globalComputedStyle.getPropertyValue(property).trim(),
  );
}

const palette = {
  red: createColor(() => getColor("red")),
  orange: createColor(() => getColor("orange")),
  amber: createColor(() => getColor("amber")),
  yellow: createColor(() => getColor("yellow")),
  avocado: createColor(() => getColor("avocado")),
  lime: createColor(() => getColor("lime")),
  green: createColor(() => getColor("green")),
  emerald: createColor(() => getColor("emerald")),
  teal: createColor(() => getColor("teal")),
  cyan: createColor(() => getColor("cyan")),
  sky: createColor(() => getColor("sky")),
  blue: createColor(() => getColor("blue")),
  indigo: createColor(() => getColor("indigo")),
  violet: createColor(() => getColor("violet")),
  purple: createColor(() => getColor("purple")),
  fuchsia: createColor(() => getColor("fuchsia")),
  pink: createColor(() => getColor("pink")),
  rose: createColor(() => getColor("rose")),
};

const paletteArr = Object.values(palette);

/**
 * Get a palette color by index, spreading small groups for better separation
 * @param {number} index
 * @param {number} [length]
 */
function at(index, length) {
  const n = paletteArr.length;
  if (length && length <= n / 2) {
    return paletteArr[Math.round((index * n) / length) % n];
  }
  return paletteArr[index % n];
}

/**
 * Build a named color map from keys, using position-based palette assignment
 * @param {readonly string[]} keys
 */
function seq(keys) {
  return Object.fromEntries(keys.map((key, i) => [key, at(i, keys.length)]));
}

export const colors = {
  transparent: createColor(() => "transparent"),
  default: createColor(() => getLightDarkValue("--color")),
  background: createColor(() => getLightDarkValue("--background-color")),
  gray: createColor(() => getColor("gray")),
  border: createColor(() => getLightDarkValue("--border-color")),
  offBorder: createColor(() => getLightDarkValue("--off-border-color")),

  // Directional
  profit: palette.green,
  loss: palette.red,
  bitcoin: palette.orange,
  usd: palette.green,

  // Bi-color pairs for baselines (spaced by 2 in palette)
  bi: {
    /** @type {[Color, Color]} */
    p1: [palette.green, palette.red],
    /** @type {[Color, Color]} */
    p2: [palette.teal, palette.amber],
    /** @type {[Color, Color]} */
    p3: [palette.sky, palette.avocado],
  },

  // Cointime economics
  liveliness: palette.pink,
  vaulted: palette.lime,
  active: palette.rose,
  awake: palette.rose,
  dormant: palette.lime,
  activity: palette.purple,
  cointime: palette.yellow,
  coinflow: palette.purple,
  mobile: palette.rose,
  immobile: palette.lime,
  capitalSentiment: {
    bear: palette.red,
    limbo: palette.orange,
    cautiousBull: palette.yellow,
    bull: palette.green,
  },
  destroyed: palette.red,
  created: palette.orange,
  stored: palette.green,
  transfer: palette.cyan,
  balanced: palette.indigo,
  terminal: palette.fuchsia,
  delta: palette.violet,

  // Valuations
  realized: palette.orange,
  investor: palette.fuchsia,
  capitalized: palette.green,
  thermo: palette.emerald,
  trueMarketMean: palette.blue,
  vocdd: palette.purple,
  hodlBank: palette.blue,
  reserveRisk: palette.orange,

  // Comparisons (base vs adjusted)
  base: palette.orange,
  adjusted: palette.purple,
  adjustedCreated: palette.lime,
  adjustedDestroyed: palette.pink,

  // Realized P&L
  gross: palette.yellow,
  regret: palette.pink,

  // Ratios
  plRatio: palette.yellow,

  // Mining
  mining: seq(["coinbase", "subsidy", "fee"]),

  // Network
  segwit: palette.cyan,

  // Entity (transactions, inputs, outputs)
  entity: seq(["tx", "input", "output"]),

  // Technical indicators
  indicator: {
    main: palette.indigo,
    fast: palette.blue,
    slow: palette.orange,
    upper: palette.green,
    lower: palette.red,
    mid: palette.yellow,
  },

  stat: {
    sum: palette.blue,
    cumulative: palette.indigo,
    avg: palette.orange,
    max: palette.green,
    pct90: palette.cyan,
    pct75: palette.blue,
    median: palette.yellow,
    pct25: palette.violet,
    pct10: palette.fuchsia,
    min: palette.red,
  },

  // Ratio percentile bands (extreme values)
  ratioPct: {
    _99_9: palette.rose,
    _99_5: palette.red,
    _99: palette.orange,
    _98: palette.amber,
    _95: palette.yellow,
    _90: palette.avocado,
    _80: palette.lime,
    _70: palette.green,
    _60: palette.emerald,
    _50: palette.green,
    _40: palette.teal,
    _30: palette.cyan,
    _20: palette.sky,
    _10: palette.blue,
    _5: palette.cyan,
    _2: palette.sky,
    _1: palette.blue,
    _0_5: palette.indigo,
    _0_1: palette.purple,
  },

  bedrock: {
    levels: [
      palette.purple,
      palette.violet,
      palette.indigo,
      palette.blue,
      palette.sky,
      palette.cyan,
      palette.teal,
      palette.emerald,
      palette.green,
      palette.lime,
    ],
    percentiles: [
      palette.yellow,
      palette.amber,
      palette.orange,
      palette.rose,
      palette.red,
    ],
  },

  time: {
    _24h: palette.red,
    _1w: palette.yellow,
    _1m: palette.green,
    _1y: palette.blue,
    all: palette.purple,
  },

  term: {
    short: palette.yellow,
    long: palette.fuchsia,
  },

  scriptType: {
    p2pk65: palette.rose,
    p2pk33: palette.pink,
    p2pkh: palette.orange,
    p2ms: palette.teal,
    p2sh: palette.green,
    p2wpkh: palette.red,
    p2wsh: palette.yellow,
    p2tr: palette.cyan,
    p2a: palette.indigo,
    opReturn: palette.purple,
    unknown: palette.violet,
    empty: palette.fuchsia,
  },

  arr: paletteArr,

  at,
};

// ---
// oklch
// ---

/**
 * @param {readonly [number, number, number, number, number, number, number, number, number]} A
 * @param {readonly [number, number, number]} B
 */
function multiplyMatrices(A, B) {
  return /** @type {const} */ ([
    A[0] * B[0] + A[1] * B[1] + A[2] * B[2],
    A[3] * B[0] + A[4] * B[1] + A[5] * B[2],
    A[6] * B[0] + A[7] * B[1] + A[8] * B[2],
  ]);
}

/** @param {readonly [number, number, number]} param0 */
function oklch2oklab([l, c, h]) {
  return /** @type {const} */ ([
    l,
    isNaN(h) ? 0 : c * Math.cos((h * Math.PI) / 180),
    isNaN(h) ? 0 : c * Math.sin((h * Math.PI) / 180),
  ]);
}

/** @param {readonly [number, number, number]} rgb */
function srgbLinear2rgb(rgb) {
  return rgb.map((c) =>
    Math.abs(c) > 0.0031308
      ? (c < 0 ? -1 : 1) * (1.055 * Math.abs(c) ** (1 / 2.4) - 0.055)
      : 12.92 * c,
  );
}

/** @param {readonly [number, number, number]} lab */
function oklab2xyz(lab) {
  const LMSg = multiplyMatrices(
    [
      1, 0.3963377773761749, 0.2158037573099136, 1, -0.1055613458156586,
      -0.0638541728258133, 1, -0.0894841775298119, -1.2914855480194092,
    ],
    lab,
  );
  const LMS = /** @type {[number, number, number]} */ (
    LMSg.map((val) => val ** 3)
  );
  return multiplyMatrices(
    [
      1.2268798758459243, -0.5578149944602171, 0.2813910456659647,
      -0.0405757452148008, 1.112286803280317, -0.0717110580655164,
      -0.0763729366746601, -0.4214933324022432, 1.5869240198367816,
    ],
    LMS,
  );
}

/** @param {readonly [number, number, number]} xyz */
function xyz2rgbLinear(xyz) {
  return multiplyMatrices(
    [
      3.2409699419045226, -1.537383177570094, -0.4986107602930034,
      -0.9692436362808796, 1.8759675015077202, 0.04155505740717559,
      0.05563007969699366, -0.20397695888897652, 1.0569715142428786,
    ],
    xyz,
  );
}

/** @type {Map<string, [number, number, number, number]>} */
const conversionCache = new Map();

/**
 * Parse oklch string and return rgba tuple
 * @param {string} oklch
 * @returns {[number, number, number, number] | null}
 */
function parseOklch(oklch) {
  if (!oklch.startsWith("oklch(")) return null;

  const cached = conversionCache.get(oklch);
  if (cached) return cached;

  let str = oklch.slice(6, -1); // remove "oklch(" and ")"
  let alpha = 1;

  const slashIdx = str.indexOf(" / ");
  if (slashIdx !== -1) {
    const alphaPart = str.slice(slashIdx + 3);
    alpha = alphaPart.includes("%")
      ? Number(alphaPart.replace("%", "")) / 100
      : Number(alphaPart);
    str = str.slice(0, slashIdx);
  }

  const parts = str.split(" ");
  const l = parts[0].includes("%")
    ? Number(parts[0].replace("%", "")) / 100
    : Number(parts[0]);
  const c = Number(parts[1]);
  const h = Number(parts[2]);

  const rgb = srgbLinear2rgb(
    xyz2rgbLinear(oklab2xyz(oklch2oklab([l, c, h]))),
  ).map((v) => Math.max(Math.min(Math.round(v * 255), 255), 0));

  const result = /** @type {[number, number, number, number]} */ ([
    ...rgb,
    alpha,
  ]);
  conversionCache.set(oklch, result);
  return result;
}

/**
 * Convert oklch string to rgba string
 * @param {string} oklch
 * @returns {string}
 */
export function oklchToRgba(oklch) {
  const result = parseOklch(oklch);
  if (!result) return oklch;
  const [r, g, b, a] = result;
  return a === 1 ? `rgb(${r}, ${g}, ${b})` : `rgba(${r}, ${g}, ${b}, ${a})`;
}
