// @ts-check

const SUFFIXES = ["", "K", "M", "B", "T"];

/** @param {number} step */
function decimalsOf(step) {
  if (!(step > 0 && step < Infinity)) return 0;
  for (let p = 0; p < 20; p++) {
    const scaled = step * 10 ** p;
    if (Math.abs(scaled - Math.round(scaled)) <= scaled * 1e-9) return p;
  }
  return 20;
}

/** @param {number} v @param {number} [step] */
export function compact(v, step = NaN) {
  const a = Math.abs(v);
  if (step > 0 && a < 1e15) {
    // (Rounded to nothing: unsigned, as zero is.)
    const k = a >= 1000 ? Math.min(SUFFIXES.length - 1, Math.floor(Math.log10(a) / 3)) : 0, unit = 1000 ** k, text = (a / unit).toFixed(Math.min(20, decimalsOf(step / unit) + 1));
    return `${v < 0 && Number(text) ? "-" : ""}${text}${SUFFIXES[k]}`;
  }
  if (v === 0) return "0";
  if (a < 1e-4) return v.toExponential(1);
  if (a < 1000) return String(Number(v.toPrecision(4)));
  if (a >= 1e15) return v.toExponential(2);
  const k = Math.min(SUFFIXES.length - 1, Math.floor(Math.log10(a) / 3));
  return `${Number((v / 1000 ** k).toPrecision(3))}${SUFFIXES[k]}`;
}

/**
 * A scale's ticks come back again and again as it moves, so each frame writes none anew.
 * @param {(v: number, step?: number) => string} write @param {number} [most]
 */
export function kept(write, most = 512) {
  const written = new Map();
  return (/** @type {number} */ v, step = NaN) => {
    const key = step === step ? `${v}|${step}` : v;
    let text = written.get(key);
    if (text === undefined) {
      if (written.size >= most) written.clear();
      written.set(key, (text = write(v, step)));
    }
    return text;
  };
}
