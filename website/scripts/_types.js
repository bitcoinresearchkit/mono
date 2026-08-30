/**
 * @import { IChartApi, ISeriesApi as _ISeriesApi, SeriesDefinition, SingleValueData as _SingleValueData, CandlestickData as _CandlestickData, BaselineData as _BaselineData, HistogramData as _HistogramData, SeriesType as LCSeriesType, IPaneApi, LineSeriesPartialOptions as _LineSeriesPartialOptions, HistogramSeriesPartialOptions as _HistogramSeriesPartialOptions, BaselineSeriesPartialOptions as _BaselineSeriesPartialOptions, CandlestickSeriesPartialOptions as _CandlestickSeriesPartialOptions, WhitespaceData, DeepPartial, ChartOptions, Time, LineData as _LineData, createChart as CreateLCChart, LineStyle, createSeriesMarkers as CreateSeriesMarkers, SeriesMarker, ISeriesMarkersPluginApi } from './modules/lightweight-charts/5.2.1/dist/typings.js'
 *
 * @import * as Bitview from "./modules/bitview-client/index.js"
 * @import { BitviewClient, Index, SeriesData, Urpd } from "./modules/bitview-client/index.js"
 *
 * @import { Options } from './options/full.js'
 *
 * @import { PersistedValue } from './utils/persisted.js'
 *
 * @import { SingleValueData, CandlestickData, Series, AnySeries, ISeries, HistogramData, LineData, BaselineData, LineSeriesPartialOptions, BaselineSeriesPartialOptions, HistogramSeriesPartialOptions, CandlestickSeriesPartialOptions, Chart, Legend } from "./utils/chart/index.js"
 *
 * @import { Color } from "./utils/colors.js"
 *
 * @import { HeatmapAxis, HeatmapAxisChoice, HeatmapDefaults, HeatmapGrid, HeatmapGridFactory, HeatmapPoints, HeatmapRange, HeatmapPointSource, HeatmapColorFn, HeatmapTooltipFn } from "../src/heatmap/types.js"
 *
 * @import { Option, PartialChartOption, ChartOption, AnyPartialOption, ProcessedOptionAddons, OptionsTree, AnySeriesBlueprint, SeriesType, AnyFetchedSeriesBlueprint, ExplorerOption, UrlOption, PartialOptionsGroup, OptionsGroup, PartialOptionsTree, UtxoCohortObject, AddrCohortObject, CohortObject, CohortGroupObject, FetchedLineSeriesBlueprint, FetchedBaselineSeriesBlueprint, FetchedHistogramSeriesBlueprint, FetchedDotsBaselineSeriesBlueprint, PatternAll, PatternFull, PatternCore, PatternWithPercentiles, PatternBasic, PatternBasicWithMarketCap, PatternBasicWithoutMarketCap, PatternWithoutRelative, CohortAll, CohortFull, CohortCore, CohortWithPercentiles, CohortBasic, CohortBasicWithMarketCap, CohortBasicWithoutMarketCap, CohortWithoutRelative, CohortAddr, CohortLongTerm, CohortAgeRange, CohortAgeRangeWithMatured, CohortGroupFull, CohortGroupCore, CohortGroupWithPercentiles, CohortGroupLongTerm, CohortGroupAgeRange, CohortGroupBasic, CohortGroupBasicWithMarketCap, CohortGroupBasicWithoutMarketCap, CohortGroupWithoutRelative, CohortGroupAddr, UtxoCohortGroupObject, AddrCohortGroupObject, FetchedDotsSeriesBlueprint, PartialHeatmapOption, HeatmapOption, FetchedCandlestickSeriesBlueprint, FetchedPriceSeriesBlueprint, AnyPricePattern, AnyValuePattern } from "./options/partial.js"
 *
 *
 * @import { UnitObject as Unit } from "./utils/units.js"
 *
 * @import { ChartableIndex, IndexLabel } from "./utils/serde.js";
 */

// import uFuzzy = require("./modules/leeoniya-ufuzzy/1.0.19/dist/uFuzzy.d.ts");

/**
 * @typedef {[number, number, number, number]} OHLCTuple
 *
 * Lightweight Charts markers
 * @typedef {ISeriesMarkersPluginApi<Time>} SeriesMarkersPlugin
 * @typedef {SeriesMarker<Time>} TimeSeriesMarker
 *
 * Bitview tree types (stable across regenerations)
 * @typedef {Bitview.SeriesTree_Cohorts} UtxoCohortTree
 * @typedef {Bitview.SeriesTree_Cohorts} AddrCohortTree
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree_Cohorts, "all">} AllUtxoPattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree_Cohorts, "term.short">} ShortTermPattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree_Cohorts, "term.long">} LongTermPattern
 * @typedef {AllUtxoPattern["unrealized"]} AllRelativePattern
 * @typedef {keyof Bitview.BtcCentsSatsUsdPattern} BtcSatsUsdKey
 * @typedef {Bitview.BtcCentsSatsUsdPattern} SupplyPattern
 * @typedef {Bitview.AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern} BlockSizePattern
 * @typedef {keyof Bitview.SeriesTree_Cohorts_Supply_Total["type"]} SpendableType
 * @typedef {Bitview.SpentUnspentPattern} OutputsPattern
 * @typedef {keyof Bitview.SeriesTree_Addrs_Raw} AddressableType
 *
 * Bitview pattern types (using new pattern names)
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree_Cohorts, "age.over._1d">} MaxAgePattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree_Cohorts, "age.range.under1h">} AgeRangePattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree_Cohorts, "utxoAmount.range._0sats">} UtxoAmountPattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree_Cohorts, "addrBalance.range._0sats">} AddrAmountPattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree_Cohorts, "entry.discount">} BasicUtxoPattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree_Cohorts, "epoch._0">} EpochPattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree_Cohorts, "type.empty">} EmptyPattern
 * @typedef {Bitview.Dollars} Dollars
 * @typedef {Bitview.BlockInfo} BlockInfo
 * @typedef {Bitview.Height} Height
 * @typedef {Bitview.BlockHash} BlockHash
 * @typedef {Bitview.BlockInfoV1} BlockInfoV1
 * @typedef {Bitview.Transaction} Transaction
 * @typedef {Bitview.Txid} Txid
 * @typedef {Bitview.TxIndex} TxIndex
 * @typedef {Bitview.AddrStats} AddrStats
 * @typedef {Bitview.TxIn} TxIn
 * @typedef {Bitview.TxOut} TxOut
 * @typedef {Bitview.BlockTemplate} BlockTemplate
 * @typedef {Bitview.MempoolBlock} MempoolBlock
 * @typedef {Bitview.NextBlockHash} NextBlockHash
 * AnyRatioPattern: price pattern with a ratio
 * @typedef {AnyPricePattern & { ratio: AnySeriesPattern }} AnyRatioPattern
 * FullValuePattern: block + cumulative + sum + average rolling windows (sats/btc/cents/usd)
 * @typedef {Bitview.AverageBlockCumulativeSumPattern2} FullValuePattern
 * RollingWindowSlot: a single rolling window with stats (pct10, pct25, median, pct75, pct90, max, min) per unit
 * @typedef {Bitview.MaxMedianMinPct10Pct25Pct75Pct90Pattern<number>} RollingWindowSlot
 * @typedef {Bitview.AnySeriesPattern} AnySeriesPattern
 * @typedef {Bitview.CentsSatsUsdPattern} ActivePricePattern
 * @typedef {Bitview.AnySeriesEndpoint} AnySeriesEndpoint
 * @typedef {Bitview.AnySeriesData} AnySeriesData
 * Relative patterns by capability:
 * Unrealized patterns by capability level
 * @typedef {Bitview.LossNetNuplProfitPattern} BasicRelativePattern
 * @typedef {Bitview.CapitalizedGrossInvestedLossNetNuplProfitSentimentPattern2} FullRelativePattern
 *
 * Profitability bucket pattern (supply + realized_cap + unrealized_pnl + nupl)
 * @typedef {Bitview.NuplRealizedSupplyUnrealizedPattern} RealizedSupplyPattern
 *
 * Realized pattern (full: cap + gross + capitalized + loss + mvrv + net + peak + price + profit + sell + sopr)
 * @typedef {Bitview.CapCapitalizedGrossLossMvrvNetPeakPriceProfitSellSoprPattern} RealizedPattern
 * @typedef {Omit<RealizedPattern, "sopr">} FullRealizedProfitabilityPattern
 *
 * Transfer volume pattern (block + cumulative + inProfit/inLoss + sum windows)
 * @typedef {Bitview.AverageBlockCumulativeInSumPattern} TransferVolumePattern
 *
 * Realized profit/loss pattern (block + cumulative + sum windows, cents/usd)
 * @typedef {Bitview.BlockCumulativeSumPattern} RealizedProfitLossPattern
 *
 * Full activity pattern (coindays, coinyears, dormancy, transfer volume)
 * @typedef {Bitview.CoindaysCoinyearsDormancyTransferPattern} FullActivityPattern
 *
 *
 * PPM + percent + ratio pattern
 * @typedef {Bitview.PercentPpmRatioPattern2} PercentRatioPattern
 *
 * Percent + ratio per window + cumulative (mirrors CountPattern but for percent)
 * @typedef {Bitview._1m1w1y24hPercentPpmRatioPattern} PercentRatioCumulativePattern
 *
 * PPM + ratio pattern (for NUPL and similar)
 * @typedef {Bitview.PpmRatioPattern} NuplPattern
 *
 * Net PnL pattern with change (base + change + cumulative + delta + rel + sum)
 * @typedef {Bitview.BlockChangeCumulativeDeltaSumPattern} NetPnlFullPattern
 *
 * Net PnL basic pattern (base + cumulative + delta + sum)
 * @typedef {Bitview.BlockCumulativeDeltaSumPattern} NetPnlBasicPattern
 *
 * Mid realized pattern (cap + loss + MVRV + net + price + profit + SOPR)
 * @typedef {Bitview.CapLossMvrvNetPriceProfitSoprPattern} MidRealizedPattern
 *
 * Basic realized pattern (cap + loss + MVRV + price + profit, no net/sopr)
 * @typedef {Bitview.CapLossMvrvPriceProfitPattern} BasicRealizedPattern
 * @typedef {Pick<Bitview.CapLossProfitPattern, "profit" | "loss">} BasicRealizedProfitabilityPattern
 *
 * Moving average price ratio pattern (ppm + cents + ratio + sats + usd)
 * @typedef {Bitview.CentsPpmRatioSatsUsdPattern} MaPriceRatioPattern
 *
 * Address count pattern (base + delta with absolute + rate)
 * @typedef {Bitview.BaseDeltaPattern} AddrCountPattern
 * @typedef {{
 *   utxo: Bitview.SeriesTree_Addrs_AvgAmount["utxo"]["all"],
 *   addr: Bitview.SeriesTree_Addrs_AvgAmount["addr"]["all"],
 * }} AvgAmountPattern
 * @typedef {Bitview.SeriesTree_Addrs_Exposed} ExposedTree
 * @typedef {Bitview.SeriesTree_Addrs_Reused} ReusedTree
 * @typedef {Bitview.SeriesTree_Addrs_Respent} RespentTree
 */

/**
 * @template T
 * @typedef {Bitview.SeriesEndpoint<T>} SeriesEndpoint
 */
/**
 * Rolling windows pattern (24h, 1w, 1m, 1y)
 * @template T
 * @typedef {Bitview._1m1w1y24hPattern<T>} RollingWindowPattern
 */
/**
 * Sell side risk rolling windows pattern
 * @typedef {Bitview._1m1w1y24hPattern8} SellSideRiskPattern
 */
/**
 * Stats pattern: min, max, median, percentiles
 * @typedef {Bitview.MaxMedianMinPct10Pct25Pct75Pct90Pattern<number>} StatsPattern
 */
/**
 * Full stats pattern: cumulative, sum, average, min, max, percentiles + rolling
 * @typedef {Bitview.AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern} FullStatsPattern
 */
/**
 * Aggregated pattern: cumulative + rolling (with distribution stats) + sum (no base)
 * @typedef {Bitview.CumulativeRollingSumPattern} AggregatedPattern
 */
/**
 * Count pattern: height, cumulative, and rolling sum windows
 * @template T
 * @typedef {Bitview.AverageBlockCumulativeSumPattern<T>} CountPattern
 */
/**
 * Full per-block pattern: height, cumulative, sum, and distribution stats (all flat)
 * FullPerBlockPattern: cumulative + sum + average + distribution stats (used by chartsFromFull)
 * Note: some callers also have .block but the function doesn't use it
 * @typedef {Omit<Bitview.AverageBlockCumulativeMaxMedianMinPct10Pct25Pct75Pct90SumPattern, 'block'>} FullPerBlockPattern
 */
/**
 * Any stats pattern union
 * @typedef {FullStatsPattern} AnyStatsPattern
 */
/**
 * Distribution stats: min, max, median, pct10/25/75/90
 * @typedef {{ min: AnySeriesPattern, max: AnySeriesPattern, median: AnySeriesPattern, pct10: AnySeriesPattern, pct25: AnySeriesPattern, pct75: AnySeriesPattern, pct90: AnySeriesPattern }} DistributionStats
 */
/**
 * Windowed distribution stats: each stat property is a rolling window record
 * @template T
 * @typedef {{ median: Record<string, T>, max: Record<string, T>, min: Record<string, T>, pct75: Record<string, T>, pct25: Record<string, T>, pct90: Record<string, T>, pct10: Record<string, T> }} WindowedStats
 */
/**
 * Dominance pattern: percent/ratio at top level + per rolling window
 * @typedef {Bitview._1m1w1y24hPercentPpmRatioPattern} DominancePattern
 */

/**
 *
 * @typedef {InstanceType<typeof BitviewClient>["INDEXES"]} Indexes
 * @typedef {Indexes[number]} IndexName
 * @typedef {InstanceType<typeof BitviewClient>["POOL_ID_TO_POOL_NAME"]} PoolIdToPoolName
 * @typedef {keyof PoolIdToPoolName} PoolId
 *
 * Tree branch types
 * @typedef {Bitview.SeriesTree_Market} Market
 * @typedef {Bitview.SeriesTree_Market_MovingAverage} MarketMovingAverage
 * @typedef {Bitview.SeriesTree_Investing} Investing
 * @typedef {Bitview._10y2y3y4y5y6y8yPattern} PeriodCagrPattern
 * @typedef {FullStatsPattern} AnyFullStatsPattern
 *
 * DCA period keys - derived from pattern types
 * @typedef {keyof Bitview._10y2y3y4y5y6y8yPattern} LongPeriodKey
 * @typedef {"_1w" | "_1m" | "_3m" | "_6m" | "_1y"} ShortPeriodKey
 * @typedef {ShortPeriodKey | LongPeriodKey} AllPeriodKey
 *
 * Pattern unions by cohort type
 * @typedef {AllUtxoPattern | AgeRangePattern | UtxoAmountPattern} UtxoCohortPattern
 * @typedef {AddrAmountPattern} AddrCohortPattern
 * @typedef {UtxoCohortPattern | AddrCohortPattern} CohortPattern
 *
 * Relative pattern capability types
 * @typedef {BasicRelativePattern | FullRelativePattern | AllRelativePattern} RelativeWithMarketCap
 * @typedef {FullRelativePattern | AllRelativePattern} RelativeWithOwnMarketCap
 * @typedef {FullRelativePattern | AllRelativePattern} RelativeWithOwnPnl
 * @typedef {BasicRelativePattern | FullRelativePattern | AllRelativePattern} RelativeWithNupl
 * @typedef {BasicRelativePattern | FullRelativePattern | AllRelativePattern} RelativeWithInvestedCapitalPct
 *
 * Realized pattern capability types
 * @typedef {RealizedPattern} AnyRealizedPattern
 *
 * Capability-based pattern groupings (patterns that have specific properties)
 * @typedef {AllUtxoPattern | ShortTermPattern | LongTermPattern | AgeRangePattern | UtxoAmountPattern | BasicUtxoPattern | EmptyPattern} PatternWithRealizedPrice
 * @typedef {AllUtxoPattern} PatternWithFullRealized
 * @typedef {ShortTermPattern | LongTermPattern | MaxAgePattern | BasicUtxoPattern} PatternWithNupl
 * @typedef {AllUtxoPattern | AgeRangePattern | UtxoAmountPattern} PatternWithCostBasis
 * @typedef {AllUtxoPattern | AgeRangePattern | UtxoAmountPattern} PatternWithActivity
 * @typedef {AllUtxoPattern | AgeRangePattern} PatternWithCostBasisPercentiles
 * @typedef {Bitview.Pct05Pct10Pct15Pct20Pct25Pct30Pct35Pct40Pct45Pct50Pct55Pct60Pct65Pct70Pct75Pct80Pct85Pct90Pct95Pattern} PercentilesPattern
 *
 * Cohort objects with specific pattern capabilities
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithRealizedPrice }} CohortWithRealizedPrice
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithFullRealized }} CohortWithFullRealized
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithNupl }} CohortWithNupl
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithCostBasis }} CohortWithCostBasis
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithActivity }} CohortWithActivity
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithCostBasisPercentiles }} CohortWithCostBasisPercentiles
 * @typedef {{ name: string, title: string, color: Color, tree: { realized: BasicRealizedProfitabilityPattern } }} CohortWithRealizedProfitLoss
 * @typedef {{ name: string, title: string, color: Color, tree: { realized: { cap: { usd: AnySeriesPattern, delta: FiatDeltaPattern } } } }} CohortWithRealizedCap
 *
 * Cohorts with full NUPL and cost-basis percentiles.
 * @typedef {CohortFull | CohortLongTerm} CohortWithNuplPercentiles
 * @typedef {{ name: string, title: string, list: readonly CohortWithNuplPercentiles[], all: CohortAll }} CohortGroupWithNuplPercentiles
 *
 * Delta patterns with absolute + rate rolling windows
 * @typedef {Bitview.AbsoluteRatePattern} DeltaPattern
 * @typedef {Bitview.SeriesTree_Cohorts_Realized_Cap["all"]["delta"]} FiatDeltaPattern
 * @typedef {Bitview.SeriesTree_Cohorts_Supply_Delta["all"]} AmountDeltaPattern
 * @typedef {Bitview.BtcSatsPattern} AmountPattern
 *
 * Generic tree node type for walking
 * @typedef {null | undefined | string | number | boolean | bigint | symbol} TreePrimitive
 * @typedef {(...args: never[]) => void} TreeFunction
 * @typedef {{ [key: string]: TreeNode }} TreeBranch
 * @typedef {TreePrimitive | TreeFunction | AnySeriesPattern | TreeBranch} TreeNode
 */
