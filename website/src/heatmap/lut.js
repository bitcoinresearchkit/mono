const INFERNO_STOPS = [
  [0, 0, 0, 0],
  [0.13, 40, 11, 84],
  [0.25, 101, 21, 110],
  [0.38, 159, 42, 99],
  [0.5, 212, 72, 66],
  [0.63, 245, 125, 21],
  [0.75, 250, 193, 39],
  [0.88, 252, 243, 105],
  [1, 252, 255, 164],
];

export const INFERNO_LUT = createColorLut(INFERNO_STOPS);

const DIVERGING_NEGATIVE_STOPS = [
  [0, 0, 0, 0],
  [0.25, 60, 0, 0],
  [0.5, 140, 10, 0],
  [0.75, 200, 30, 10],
  [1, 240, 60, 20],
];

const DIVERGING_POSITIVE_STOPS = [
  [0, 0, 0, 0],
  [0.25, 0, 40, 0],
  [0.5, 0, 110, 10],
  [0.75, 10, 180, 20],
  [1, 30, 230, 50],
];

export const DIVERGING_NEGATIVE_LUT = createColorLut(DIVERGING_NEGATIVE_STOPS);
export const DIVERGING_POSITIVE_LUT = createColorLut(DIVERGING_POSITIVE_STOPS);

/**
 * @param {ArrayLike<number>} lut
 * @returns {HeatmapColorFn}
 */
export function logIntensityColor(lut) {
  return (value, context) => {
    if (!Number.isFinite(value) || value <= 0) return 0x00000000;
    const max = context.grid.getMaxValue(context.col);
    if (max <= 0) return 0x00000000;
    const t = Math.log2(value + 1) / Math.log2(max + 1);
    const index = Math.min(255, Math.max(0, Math.round(t * 255)));
    return lut[index] ?? 0x00000000;
  };
}

/**
 * @param {ArrayLike<number>} lut
 * @param {number} [exponent]
 * @returns {HeatmapColorFn}
 */
export function powerIntensityColor(lut, exponent = 0.4) {
  return (value, context) => {
    if (!Number.isFinite(value) || value <= 0) return 0x00000000;
    const cap = context.grid.getMaxValue(context.col);
    if (cap <= 0) return 0x00000000;
    const t = Math.pow(Math.min(1, value / cap), exponent);
    const index = Math.min(255, Math.max(0, Math.round(t * 255)));
    return lut[index] ?? 0x00000000;
  };
}

/**
 * @param {ArrayLike<number>} negativeLut
 * @param {ArrayLike<number>} positiveLut
 * @param {number} [exponent]
 * @returns {HeatmapColorFn}
 */
export function divergingPowerIntensityColor(
  negativeLut,
  positiveLut,
  exponent = 0.4,
) {
  return (value, context) => {
    if (!Number.isFinite(value) || value === 0) return 0x00000000;
    const cap = context.grid.getMagnitudeMaxValue(context.col);
    if (cap <= 0) return 0x00000000;
    const t = Math.pow(Math.min(1, Math.abs(value) / cap), exponent);
    const index = Math.min(255, Math.max(0, Math.round(t * 255)));
    const lut = value < 0 ? negativeLut : positiveLut;
    return lut[index] ?? 0x00000000;
  };
}

/**
 * @param {number[][]} stops - Tuples of [position, red, green, blue].
 */
export function createColorLut(stops) {
  const lut = new Uint32Array(256);
  for (let i = 0; i < lut.length; i++) {
    const t = i / 255;
    let a = stops[0];
    let b = stops[stops.length - 1];
    for (let j = 0; j < stops.length - 1; j++) {
      if (t >= stops[j][0] && t <= stops[j + 1][0]) {
        a = stops[j];
        b = stops[j + 1];
        break;
      }
    }
    const f = a[0] === b[0] ? 0 : (t - a[0]) / (b[0] - a[0]);
    const r = (a[1] + f * (b[1] - a[1]) + 0.5) | 0;
    const g = (a[2] + f * (b[2] - a[2]) + 0.5) | 0;
    const blue = (a[3] + f * (b[3] - a[3]) + 0.5) | 0;
    lut[i] = 0xff000000 | (blue << 16) | (g << 8) | r;
  }
  return lut;
}
