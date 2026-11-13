// @ts-check
// Shared histogram geometry, in CSS pixels: gaps grow once bars have room; narrow bars have square ends.

/** Studio's bar and candle corners: none when crowded, then gently larger as their spacing grows. @param {number} spacing */
export const barCorner = (spacing) => spacing < 4 ? 0 : Math.min(4, Math.max(1, spacing / 10));

/** @param {number} spacing @param {number} [corner] */
export function barStyle(spacing, corner = barCorner(spacing)) {
  const width = Math.max(1, spacing - (spacing > 8 ? 2 : 1));
  return { width, radius: spacing >= 3 && corner > 0 ? Math.min(corner, width / 2) : 0 };
}
