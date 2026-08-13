import { SATS_PER_BTC } from "../../btc/index.js";

const MONTHS = /** @type {const} */ ([
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
]);

/** @param {string} text */
export function dim(text) {
  const element = document.createElement("span");

  element.dataset.dim = "";
  element.textContent = text;

  return element;
}

/** @param {number} height */
export function createHeightElement(height) {
  const container = document.createElement("span");
  const prefix = document.createElement("span");
  const value = document.createElement("span");
  const [prefixText, valueText] = formatHeight(height);

  prefix.dataset.dim = "";
  prefix.textContent = prefixText;
  value.textContent = valueText;
  container.append(prefix, value);

  return container;
}

/** @param {number} height */
export function formatHeight(height) {
  const value = String(height);

  return [`#${"0".repeat(Math.max(0, 7 - value.length))}`, value];
}

/** @param {number} value */
export function formatNumber(value) {
  return value.toLocaleString();
}

/** @param {number} bytes */
export function formatMegabytes(bytes) {
  return (bytes / 1_000_000).toFixed(2);
}

/** @param {number} seconds */
export function formatInterval(seconds) {
  const total = Math.max(0, Math.round(seconds));
  const hours = Math.floor(total / 3_600);
  const minutes = Math.floor((total % 3_600) / 60);

  if (hours) return `${hours}h ${String(minutes).padStart(2, "0")}m`;

  const remainder = total % 60;
  if (minutes) return `${minutes}m ${String(remainder).padStart(2, "0")}s`;

  return `${remainder}s`;
}

/** @param {number} sats */
export function formatBtc(sats) {
  return (sats / SATS_PER_BTC).toLocaleString(undefined, {
    maximumSignificantDigits: 3,
  });
}

/** @param {number} unixSeconds */
export function formatShortDate(unixSeconds) {
  const date = new Date(unixSeconds * 1_000);

  return `${MONTHS[date.getMonth()]} ${date.getDate()}`;
}

/** @param {number} unixSeconds */
export function formatYear(unixSeconds) {
  return String(new Date(unixSeconds * 1_000).getFullYear());
}

/** @param {number} unixSeconds */
export function formatUtcOffset(unixSeconds) {
  const date = new Date(unixSeconds * 1_000);
  const offsetMinutes = -date.getTimezoneOffset();
  if (offsetMinutes === 0) return "UTC+0";

  const sign = offsetMinutes > 0 ? "+" : "-";
  const absoluteMinutes = Math.abs(offsetMinutes);
  const hours = Math.floor(absoluteMinutes / 60);
  const minutes = absoluteMinutes % 60;
  const minuteSuffix = minutes
    ? `:${String(minutes).padStart(2, "0")}`
    : "";

  return `UTC${sign}${hours}${minuteSuffix}`;
}

/** @param {number} unixSeconds */
export function formatTime(unixSeconds) {
  const date = new Date(unixSeconds * 1_000);

  return [
    String(date.getHours()).padStart(2, "0"),
    String(date.getMinutes()).padStart(2, "0"),
    String(date.getSeconds()).padStart(2, "0"),
  ];
}
