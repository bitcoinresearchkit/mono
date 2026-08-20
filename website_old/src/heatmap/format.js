/** @param {number} value */
export function formatCompact(value) {
  if (value >= 1000) return `${formatNumber(value / 1000)}k`;
  return formatNumber(value);
}

/** @param {number} value */
function formatNumber(value) {
  if (value >= 100) return String(Math.round(value));
  if (value >= 10) return trimNumber(value.toFixed(1));
  return trimNumber(value.toFixed(2));
}

/** @param {string} value */
function trimNumber(value) {
  return value.replace(/\.?0+$/, "");
}
