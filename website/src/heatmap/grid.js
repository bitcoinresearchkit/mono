/**
 * Generic date/y binning with average merge semantics.
 *
 * @param {Object} args
 * @param {number} args.yMin
 * @param {number} args.yMax
 * @param {number} [args.minCellSize]
 * @param {number} [args.maxCols]
 * @param {number} [args.nativeRows]
 * @param {"bottom" | "top"} [args.yOrigin]
 * @returns {HeatmapGridFactory}
 */
export function createAverageGrid({
  yMin: defaultYMin,
  yMax: defaultYMax,
  minCellSize = 1,
  maxCols = Number.POSITIVE_INFINITY,
  nativeRows = Number.POSITIVE_INFINITY,
  yOrigin = "bottom",
}) {
  return {
    create({ dates, width, height, yMin = defaultYMin, yMax = defaultYMax }) {
      const cols = Math.max(
        1,
        Math.min(
          dates.length || 1,
          maxCols,
          Math.floor(width / minCellSize) || 1,
        ),
      );
      const rows = Math.max(
        1,
        Math.min(nativeRows, Math.floor(height / minCellSize) || 1),
      );
      const sums = new Float64Array(cols * rows);
      const counts = new Uint32Array(cols * rows);
      const maxByCol = new Float64Array(cols);
      const magnitudeMaxByCol = new Float64Array(cols);
      let maxValue = 0;
      let magnitudeMaxValue = 0;
      const ySpan = yMax - yMin;

      /** @param {number} dateIndex */
      function toCol(dateIndex) {
        if (dateIndex < 0 || dateIndex >= dates.length) return undefined;
        return clamp(
          Math.floor((dateIndex * cols) / dates.length),
          0,
          cols - 1,
        );
      }

      /** @param {number} y */
      function toRow(y) {
        if (!Number.isFinite(y) || !Number.isFinite(ySpan) || ySpan <= 0) {
          return undefined;
        }
        const t = (y - yMin) / ySpan;
        if (t < 0 || t > 1) return undefined;
        const row = clamp(Math.floor(t * rows), 0, rows - 1);
        return yOrigin === "top" ? row : rows - 1 - row;
      }

      /**
       * @param {number} col
       * @param {number} y
       * @param {number} value
       */
      function addValue(col, y, value) {
        if (!Number.isFinite(value)) return false;
        const row = toRow(y);
        if (row === undefined) return false;
        const index = row * cols + col;
        sums[index] += value;
        counts[index] += 1;
        return true;
      }

      /** @param {number} col */
      function updateColumnMax(col) {
        let max = 0;
        let magnitudeMax = 0;
        for (let row = 0; row < rows; row++) {
          const index = row * cols + col;
          if (counts[index]) {
            const value = sums[index] / counts[index];
            max = Math.max(max, value);
            magnitudeMax = Math.max(magnitudeMax, Math.abs(value));
          }
        }
        maxByCol[col] = max;
        magnitudeMaxByCol[col] = magnitudeMax;
        maxValue = 0;
        magnitudeMaxValue = 0;
        for (let c = 0; c < cols; c++) {
          maxValue = Math.max(maxValue, maxByCol[c]);
          magnitudeMaxValue = Math.max(magnitudeMaxValue, magnitudeMaxByCol[c]);
        }
      }

      /** @type {HeatmapGrid} */
      const grid = {
        dates,
        cols,
        rows,
        add(dateIndex, points) {
          const col = toCol(dateIndex);
          if (col === undefined) return undefined;
          let dirty = false;
          if (points.kind === "implicit") {
            for (let i = 0; i < points.values.length; i++) {
              if (
                addValue(
                  col,
                  points.yStart + i * points.yStep,
                  points.values[i],
                )
              ) {
                dirty = true;
              }
            }
          } else {
            const length = Math.min(points.y.length, points.values.length);
            for (let i = 0; i < length; i++) {
              if (addValue(col, points.y[i], points.values[i])) dirty = true;
            }
          }
          if (!dirty) return undefined;
          const previousMax = maxValue;
          const previousMagnitudeMax = magnitudeMaxValue;
          updateColumnMax(col);
          return {
            col,
            maxChanged:
              maxValue !== previousMax ||
              magnitudeMaxValue !== previousMagnitudeMax,
          };
        },
        getValue(col, row) {
          if (col < 0 || col >= cols || row < 0 || row >= rows) {
            return Number.NaN;
          }
          const index = row * cols + col;
          return counts[index] ? sums[index] / counts[index] : Number.NaN;
        },
        getCount(col, row) {
          if (col < 0 || col >= cols || row < 0 || row >= rows) return 0;
          return counts[row * cols + col];
        },
        getMaxValue(col) {
          if (col === undefined) return maxValue;
          if (col < 0 || col >= cols) return 0;
          return maxByCol[col];
        },
        getMagnitudeMaxValue(col) {
          if (col === undefined) return magnitudeMaxValue;
          if (col < 0 || col >= cols) return 0;
          return magnitudeMaxByCol[col];
        },
        getDateIndexRange(col) {
          if (col < 0 || col >= cols || dates.length === 0) {
            return emptyRange();
          }
          const start = Math.ceil((col * dates.length) / cols);
          const end = Math.floor(((col + 1) * dates.length - 1) / cols);
          return { start, end: clamp(end, start, dates.length - 1) };
        },
        getYRange(row) {
          if (row < 0 || row >= rows || ySpan <= 0) return emptyRange();
          const index = yOrigin === "top" ? row : rows - row - 1;
          const start = yMin + (index / rows) * ySpan;
          const end = yMin + ((index + 1) / rows) * ySpan;
          return { start, end };
        },
      };

      return grid;
    },
  };
}

/**
 * @param {number} value
 * @param {number} min
 * @param {number} max
 */
function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value));
}

/** @returns {HeatmapRange} */
function emptyRange() {
  return { start: Number.NaN, end: Number.NaN };
}
