/**
 * @param {number} value
 * @param {number} [digits]
 * @param {Intl.NumberFormatOptions} [options]
 */
function numberToUSNumber(value, digits, options) {
  return value.toLocaleString("en-us", {
    ...options,
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  });
}

/**
 * @param {number} value
 * @param {0 | 2} [digits]
 */
export function numberToShortUSFormat(value, digits) {
  const absoluteValue = Math.abs(value);

  if (isNaN(value) || !isFinite(value)) {
    return "";
  } else if (absoluteValue < 10) {
    return numberToUSNumber(value, Math.min(3, digits || 10));
  } else if (absoluteValue < 1_000) {
    return numberToUSNumber(value, Math.min(2, digits || 10));
  } else if (absoluteValue < 10_000) {
    return numberToUSNumber(value, Math.min(1, digits || 10));
  } else if (absoluteValue < 1_000_000) {
    return numberToUSNumber(value, 0);
  } else if (absoluteValue >= 1e27) {
    return "Inf.";
  }

  const log = Math.floor(Math.log10(absoluteValue) - 6);

  const suffices = ["M", "B", "T", "P", "E", "Z", "Y"];
  const letterIndex = Math.floor(log / 3);
  const letter = suffices[letterIndex];

  const modulused = log % 3;

  if (modulused === 0) {
    return `${numberToUSNumber(
      value / (1_000_000 * 1_000 ** letterIndex),
      3,
    )}${letter}`;
  } else if (modulused === 1) {
    return `${numberToUSNumber(
      value / (1_000_000 * 1_000 ** letterIndex),
      2,
    )}${letter}`;
  } else {
    return `${numberToUSNumber(
      value / (1_000_000 * 1_000 ** letterIndex),
      1,
    )}${letter}`;
  }
}

/**
 * @param {string} s
 */
export function stringToId(s) {
  return s
    .trim()
    .replace(/[ /]+/g, "-")
    .toLowerCase()
    .replace(/%/g, "%25");
}
