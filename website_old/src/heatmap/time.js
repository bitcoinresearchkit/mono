const DAY_MS = 86_400_000;
export const GENESIS_DATE = "2009-01-03";

/**
 * @param {Date} date
 */
export function toISODate(date) {
  return date.toISOString().slice(0, 10);
}

export function todayISODate() {
  return toISODate(new Date());
}

/**
 * Inclusive UTC date range.
 *
 * @param {string} from
 * @param {string} to
 */
export function dateRange(from, to) {
  const dates = [];
  for (
    let time = Date.parse(`${from}T00:00:00Z`),
      end = Date.parse(`${to}T00:00:00Z`);
    time <= end;
    time += DAY_MS
  ) {
    dates.push(toISODate(new Date(time)));
  }
  return dates;
}
