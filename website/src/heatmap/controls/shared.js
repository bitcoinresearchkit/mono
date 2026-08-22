import { createPersistedValue } from "../../../scripts/utils/persisted.js";

/**
 * @param {HeatmapOption} option
 * @param {string} key
 * @param {string} urlKey
 * @param {string} defaultValue
 */
export function createHeatmapPersistedValue(option, key, urlKey, defaultValue) {
  return createPersistedValue({
    defaultValue,
    storageKey: `${heatmapStoragePrefix(option)}-${key}`,
    urlKey,
    serialize: (value) => value,
    deserialize: (value) => value,
  });
}

/**
 * @template T
 * @param {readonly T[]} choices
 * @param {string} key
 * @param {T} fallback
 * @param {(choice: T) => string} toKey
 */
export function findChoiceByKey(choices, key, fallback, toKey) {
  return choices.find((candidate) => toKey(candidate) === key) ?? fallback;
}

/** @param {HeatmapOption} option */
function heatmapStoragePrefix(option) {
  return `heatmap-${option.path.join("-")}`;
}
