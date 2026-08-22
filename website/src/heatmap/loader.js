import { dateRange } from "./time.js";

const MAX_PARALLEL_FETCHES = 8;

/**
 * @param {Object} args
 * @param {(dateIndex: number, points: HeatmapPoints) => void} args.addDateToGrid
 * @param {() => void} args.rebuildGrid
 * @param {() => void} args.paint
 */
export function createHeatmapLoader({ addDateToGrid, rebuildGrid, paint }) {
  /** @type {string[]} */
  let dates = [];
  /** @type {Map<string, HeatmapPoints>} */
  let pointsByDate = new Map();
  /** @type {AbortController | undefined} */
  let abortController;
  /** @type {HeatmapOption | undefined} */
  let activeOption;
  let generation = 0;

  return {
    get dates() {
      return dates;
    },
    /** @param {string} date */
    getPoint(date) {
      return pointsByDate.get(date);
    },
    reset() {
      abortController?.abort();
      pointsByDate = new Map();
    },
    /**
     * @param {Object} args
     * @param {HeatmapOption} args.option
     * @param {string} args.from
     * @param {string} args.to
     */
    setRange({ option, from, to }) {
      abortController?.abort();
      generation += 1;
      activeOption = option;
      dates = dateRange(from, to);
    },
    /**
     * @param {Object} args
     * @param {HeatmapOption} args.option
     * @param {readonly number[]} args.dateIndexes
     */
    load({ option, dateIndexes }) {
      abortController?.abort();
      const controller = new AbortController();
      const currentGeneration = ++generation;
      activeOption = option;
      abortController = controller;

      /** @type {{ date: string, dateIndex: number }[]} */
      const missing = [];
      let previousDateIndex = -1;
      for (const dateIndex of dateIndexes) {
        if (
          dateIndex === previousDateIndex ||
          dateIndex < 0 ||
          dateIndex >= dates.length
        ) {
          continue;
        }
        previousDateIndex = dateIndex;
        const date = dates[dateIndex];
        if (!pointsByDate.has(date)) missing.push({ date, dateIndex });
      }

      if (!missing.length) {
        abortController = undefined;
        return;
      }

      let cursor = 0;
      let needsRebuild = false;
      const workers = Array.from({
        length: Math.min(MAX_PARALLEL_FETCHES, missing.length),
      }).map(async () => {
        let index = nextMissingIndex();
        while (index !== undefined) {
          const entry = missing[index];
          try {
            const points = await option.points.fetch(
              entry.date,
              controller.signal,
              (points) => {
                if (isCurrentLoad(option, controller, currentGeneration)) {
                  setPoints(entry, points);
                }
              },
            );
            if (isCurrentLoad(option, controller, currentGeneration)) {
              setPoints(entry, points);
            }
          } catch (error) {
            if (controller.signal.aborted) return;
            console.error(
              `Failed to fetch heatmap points for ${entry.date}`,
              error,
            );
          }
          index = nextMissingIndex();
        }
      });

      void Promise.all(workers).then(() => {
        if (isCurrentLoad(option, controller, currentGeneration)) {
          if (needsRebuild) {
            rebuildGrid();
          } else {
            paint();
          }
        }
      });

      function nextMissingIndex() {
        if (cursor >= missing.length) return undefined;
        const index = cursor;
        cursor += 1;
        return index;
      }

      /**
       * @param {{ date: string, dateIndex: number }} entry
       * @param {HeatmapPoints} points
       */
      function setPoints(entry, points) {
        const previous = pointsByDate.get(entry.date);
        if (previous && samePoints(previous, points)) return;
        pointsByDate.set(entry.date, points);
        if (previous) {
          needsRebuild = true;
        } else {
          addDateToGrid(entry.dateIndex, points);
        }
      }
    },
  };

  /**
   * @param {HeatmapOption} option
   * @param {AbortController} controller
   * @param {number} currentGeneration
   */
  function isCurrentLoad(option, controller, currentGeneration) {
    return (
      activeOption === option &&
      abortController === controller &&
      generation === currentGeneration &&
      !controller.signal.aborted
    );
  }
}

/**
 * @param {HeatmapPoints} a
 * @param {HeatmapPoints} b
 */
function samePoints(a, b) {
  if (a === b) return true;
  if (a.kind !== b.kind || a.values !== b.values) return false;
  if (a.kind === "implicit" && b.kind === "implicit") {
    return a.yStart === b.yStart && a.yStep === b.yStep;
  }
  if (a.kind === "explicit" && b.kind === "explicit") return a.y === b.y;
  return false;
}
