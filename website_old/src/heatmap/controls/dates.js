import { createSelect } from "../../../scripts/utils/dom.js";
import { GENESIS_DATE, todayISODate, toISODate } from "../time.js";
import { createHeatmapPersistedValue, findChoiceByKey } from "./shared.js";

/**
 * @typedef {Object} RangeChoice
 * @property {string} label
 * @property {string} date
 */

/**
 * @param {HeatmapOption} option
 * @param {(range: { from: string, to: string }) => void} onChange
 */
export function createDateControls(option, onChange) {
  const currentYear = new Date().getUTCFullYear();
  const fromChoices = createFromChoices(currentYear);
  const toChoices = createToChoices(currentYear);
  const fallbackFromChoice = fromChoices.at(-1) ?? fromChoices[0];
  const fallbackToChoice = toChoices[0];
  const defaultFromChoice = findChoiceByKey(
    fromChoices,
    option.defaults?.from ?? "",
    fallbackFromChoice,
    rangeChoiceLabel,
  );
  const defaultToChoice = findChoiceByKey(
    toChoices,
    option.defaults?.to ?? "",
    fallbackToChoice,
    rangeChoiceLabel,
  );

  const persistedFrom = createHeatmapPersistedValue(
    option,
    "from",
    "from",
    rangeChoiceLabel(defaultFromChoice),
  );
  const persistedTo = createHeatmapPersistedValue(
    option,
    "to",
    "to",
    rangeChoiceLabel(defaultToChoice),
  );

  let fromChoice = findChoiceByKey(
    fromChoices,
    persistedFrom.value,
    defaultFromChoice,
    rangeChoiceLabel,
  );
  let toChoice = findChoiceByKey(
    toChoices,
    persistedTo.value,
    defaultToChoice,
    rangeChoiceLabel,
  );

  if (fromChoice.date > toChoice.date) {
    toChoice = findSameLabelChoice(toChoices, fromChoice, defaultToChoice);
  }
  persistDateChoices();

  const fromSelect = createSelect({
    id: "heatmap-from",
    label: "from",
    choices: fromChoices,
    initialValue: fromChoice,
    onChange(choice) {
      fromChoice = choice;
      if (fromChoice.date > toChoice.date) {
        toChoice = findSameLabelChoice(toChoices, fromChoice, defaultToChoice);
        toSelect.set(toChoice);
      }
      persistDateChoices();
      onChange({ from: fromChoice.date, to: toChoice.date });
    },
    toKey: rangeChoiceLabel,
    toLabel: rangeChoiceLabel,
  });
  const toSelect = createSelect({
    id: "heatmap-to",
    label: "to",
    choices: toChoices,
    initialValue: toChoice,
    onChange(choice) {
      toChoice = choice;
      if (fromChoice.date > toChoice.date) {
        fromChoice = findSameLabelChoice(
          fromChoices,
          toChoice,
          defaultFromChoice,
        );
        fromSelect.set(fromChoice);
      }
      persistDateChoices();
      onChange({ from: fromChoice.date, to: toChoice.date });
    },
    toKey: rangeChoiceLabel,
    toLabel: rangeChoiceLabel,
  });

  return {
    elements: [fromSelect.element, toSelect.element],
    from: fromChoice.date,
    to: toChoice.date,
  };

  function persistDateChoices() {
    persistedFrom.setImmediate(rangeChoiceLabel(fromChoice));
    persistedTo.setImmediate(rangeChoiceLabel(toChoice));
  }
}

/**
 * @param {number} currentYear
 * @returns {RangeChoice[]}
 */
function createFromChoices(currentYear) {
  const choices = [{ label: "genesis", date: GENESIS_DATE }];
  for (let year = 2009; year <= currentYear; year++) {
    choices.push({
      label: String(year),
      date: year === 2009 ? GENESIS_DATE : yearStartISODate(year),
    });
  }
  return choices;
}

/**
 * @param {number} currentYear
 * @returns {RangeChoice[]}
 */
function createToChoices(currentYear) {
  const today = todayISODate();
  const todayTime = Date.parse(`${today}T00:00:00Z`);
  const choices = [{ label: "today", date: today }];
  for (let year = currentYear; year >= 2009; year--) {
    choices.push({ label: String(year), date: yearEndISODate(year, todayTime) });
  }
  return choices;
}

/** @param {RangeChoice} choice */
function rangeChoiceLabel(choice) {
  return choice.label;
}

/**
 * @param {readonly RangeChoice[]} choices
 * @param {RangeChoice} choice
 * @param {RangeChoice} fallback
 */
function findSameLabelChoice(choices, choice, fallback) {
  return choices.find((candidate) => candidate.label === choice.label) ?? fallback;
}

/** @param {number} year */
function yearStartISODate(year) {
  return toISODate(new Date(Date.UTC(year, 0, 1)));
}

/**
 * @param {number} year
 * @param {number} todayTime
 */
function yearEndISODate(year, todayTime) {
  return toISODate(new Date(Math.min(Date.UTC(year, 11, 31), todayTime)));
}
