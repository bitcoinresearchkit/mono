// @ts-check
// How a scale writes its values unless told otherwise: four significant digits at most, thousands as K, M, B and T.

const SUFFIXES = ["", "K", "M", "B", "T"];

/** @param {number} v */
export function compact(v) {
  if (v === 0) return "0";
  const a = Math.abs(v);
  if (a < 1e-4) return v.toExponential(1);
  if (a < 1000) return String(Number(v.toPrecision(4)));
  const k = Math.min(SUFFIXES.length - 1, Math.floor(Math.log10(a) / 3));
  return `${Number((v / 1000 ** k).toPrecision(3))}${SUFFIXES[k]}`;
}

/**
 * A writer that writes each value once: a scale's ticks come back again and again as it moves (round numbers), so
 * each frame writes none anew. Forgets all past `most`.
 * @param {(v: number) => string} write @param {number} [most]
 */
export function kept(write, most = 512) {
  const written = new Map();
  return (/** @type {number} */ v) => {
    let text = written.get(v);
    if (text === undefined) {
      if (written.size >= most) written.clear();
      written.set(v, (text = write(v)));
    }
    return text;
  };
}
