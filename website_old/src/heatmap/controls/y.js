import { createSelect } from "../../../scripts/utils/dom.js";
import { createHeatmapPersistedValue, findChoiceByKey } from "./shared.js";

/**
 * @param {HeatmapOption} option
 * @param {(range: { yMin: number | undefined, yMax: number | undefined }) => void} onChange
 */
export function createYControls(option, onChange) {
  const y = option.axis?.y;
  const choices = y?.choices;
  if (!choices || choices.length < 2) {
    return { elements: [], yMin: undefined, yMax: undefined };
  }

  const fallbackMinChoice = choices[0];
  const fallbackMaxChoice = choices.at(-1) ?? choices[0];
  const defaultMinChoice = findChoiceByKey(
    choices,
    String(option.defaults?.yMin ?? ""),
    fallbackMinChoice,
    axisChoiceValueKey,
  );
  const defaultMaxChoice = findChoiceByKey(
    choices,
    String(option.defaults?.yMax ?? ""),
    fallbackMaxChoice,
    axisChoiceValueKey,
  );
  const persistedMin = createHeatmapPersistedValue(
    option,
    "y-min",
    "min",
    axisChoiceKey(defaultMinChoice),
  );
  const persistedMax = createHeatmapPersistedValue(
    option,
    "y-max",
    "max",
    axisChoiceKey(defaultMaxChoice),
  );

  let minChoice = findChoiceByKey(
    choices,
    persistedMin.value,
    defaultMinChoice,
    axisChoiceKey,
  );
  let maxChoice = findChoiceByKey(
    choices,
    persistedMax.value,
    defaultMaxChoice,
    axisChoiceKey,
  );
  if (minChoice.value > maxChoice.value) {
    maxChoice = minChoice;
  }
  persistYChoices();

  const minSelect = createSelect({
    id: "heatmap-y-min",
    label: "min",
    choices,
    initialValue: minChoice,
    onChange(choice) {
      minChoice = choice;
      if (minChoice.value > maxChoice.value) {
        maxChoice = minChoice;
        maxSelect.set(maxChoice);
      }
      persistYChoices();
      onChange({ yMin: minChoice.value, yMax: maxChoice.value });
    },
    toKey: axisChoiceKey,
    toLabel: axisChoiceLabel,
  });
  const maxSelect = createSelect({
    id: "heatmap-y-max",
    label: "max",
    choices: Array.from(choices).reverse(),
    initialValue: maxChoice,
    onChange(choice) {
      maxChoice = choice;
      if (minChoice.value > maxChoice.value) {
        minChoice = maxChoice;
        minSelect.set(minChoice);
      }
      persistYChoices();
      onChange({ yMin: minChoice.value, yMax: maxChoice.value });
    },
    toKey: axisChoiceKey,
    toLabel: axisChoiceLabel,
  });

  return {
    elements: [minSelect.element, maxSelect.element],
    yMin: minChoice.value,
    yMax: maxChoice.value,
  };

  function persistYChoices() {
    persistedMin.setImmediate(axisChoiceKey(minChoice));
    persistedMax.setImmediate(axisChoiceKey(maxChoice));
  }
}

/** @param {HeatmapAxisChoice} choice */
function axisChoiceKey(choice) {
  return choice.key ?? choice.label;
}

/** @param {HeatmapAxisChoice} choice */
function axisChoiceLabel(choice) {
  return choice.label;
}

/** @param {HeatmapAxisChoice} choice */
function axisChoiceValueKey(choice) {
  return String(choice.value);
}
