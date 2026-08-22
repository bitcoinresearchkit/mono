import { createDateControls } from "./dates.js";
import { createYControls } from "./y.js";

/**
 * @typedef {Object} HeatmapControlSelection
 * @property {string} from
 * @property {string} to
 * @property {number | undefined} yMin
 * @property {number | undefined} yMax
 */

/**
 * @param {Object} args
 * @param {(range: { from: string, to: string }) => void} args.onRangeChange
 * @param {(range: { yMin: number | undefined, yMax: number | undefined }) => void} args.onYRangeChange
 */
export function createHeatmapControls({ onRangeChange, onYRangeChange }) {
  const element = document.createElement("fieldset");

  return {
    element,
    /**
     * @param {HeatmapOption} option
     * @returns {HeatmapControlSelection}
     */
    setOption(option) {
      const dates = createDateControls(option, onRangeChange);
      const y = createYControls(option, onYRangeChange);
      element.replaceChildren(...dates.elements, ...y.elements);
      return {
        from: dates.from,
        to: dates.to,
        yMin: y.yMin,
        yMax: y.yMax,
      };
    },
  };
}
