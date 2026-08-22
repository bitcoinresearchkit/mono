/**
 * @typedef {Object} HeatmapImplicitPoints
 * @property {"implicit"} kind
 * @property {number} yStart
 * @property {number} yStep
 * @property {ArrayLike<number>} values
 *
 * @typedef {Object} HeatmapExplicitPoints
 * @property {"explicit"} kind
 * @property {ArrayLike<number>} y
 * @property {ArrayLike<number>} values
 *
 * @typedef {HeatmapImplicitPoints | HeatmapExplicitPoints} HeatmapPoints
 *
 * @typedef {Object} HeatmapPointSource
 * @property {(date: string, signal: AbortSignal, onPoints?: (points: HeatmapPoints) => void) => Promise<HeatmapPoints>} fetch
 *
 * @typedef {Object} HeatmapRange
 * @property {number} start
 * @property {number} end
 *
 * @typedef {"genesis" | "today" | `${number}`} HeatmapDateKey
 *
 * @typedef {Object} HeatmapDefaults
 * @property {HeatmapDateKey} [from]
 * @property {HeatmapDateKey} [to]
 * @property {number} [yMin]
 * @property {number} [yMax]
 *
 * @typedef {Object} HeatmapGridAddResult
 * @property {number} col
 * @property {boolean} maxChanged
 *
 * @typedef {Object} HeatmapGrid
 * @property {readonly string[]} dates
 * @property {number} cols
 * @property {number} rows
 * @property {(dateIndex: number, points: HeatmapPoints) => HeatmapGridAddResult | undefined} add
 * @property {(col: number, row: number) => number} getValue
 * @property {(col: number, row: number) => number} getCount
 * @property {(col?: number) => number} getMaxValue
 * @property {(col?: number) => number} getMagnitudeMaxValue
 * @property {(col: number) => HeatmapRange} getDateIndexRange
 * @property {(row: number) => HeatmapRange} getYRange
 *
 * @typedef {Object} HeatmapGridFactory
 * @property {(args: { dates: readonly string[], width: number, height: number, yMin?: number, yMax?: number }) => HeatmapGrid} create
 *
 * @typedef {Object} HeatmapAxisChoice
 * @property {string} label
 * @property {string} [key]
 * @property {number} value
 *
 * @typedef {Object} HeatmapAxis
 * @property {{ label: string, choices?: HeatmapAxisChoice[], format?: (value: number) => string }} [y]
 *
 * @typedef {(value: number, context: { grid: HeatmapGrid, col: number, row: number }) => number} HeatmapColorFn
 * @typedef {(context: { option: { axis?: HeatmapAxis }, grid: HeatmapGrid, col: number, row: number }) => string} HeatmapTooltipFn
 */

export {};
