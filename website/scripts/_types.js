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

/**
 * @typedef {[number, number, number, number]} OHLCTuple
 *
 * Lightweight Charts markers
 * @typedef {ISeriesMarkersPluginApi<Time>} SeriesMarkersPlugin
 * @typedef {SeriesMarker<Time>} TimeSeriesMarker
 *
 * Bitview tree types (stable across regenerations)
 * @typedef {Bitview.SeriesTree["cohorts"]} UtxoCohortTree
 * @typedef {Bitview.SeriesTree["cohorts"]} AddrCohortTree
 * @typedef {Bitview.SeriesTree["distributionAggregated"]["cohorts"]["all"]} AllUtxoPattern
 * @typedef {Bitview.SeriesTree["distributionAggregated"]["cohorts"]["sth"]} ShortTermPattern
 * @typedef {Bitview.SeriesTree["distributionAggregated"]["cohorts"]["lth"]} LongTermPattern
 * @typedef {AllUtxoPattern["unrealized"]} AllRelativePattern
 * @typedef {keyof Bitview.SeriesTree["supply"]["circulating"]} BtcSatsUsdKey
 * @typedef {Bitview.SeriesTree["supply"]["circulating"]} SupplyPattern
 * @typedef {Bitview.SeriesTree["blocks"]["vbytes"]} BlockSizePattern
 * @typedef {keyof Bitview.SeriesTree["cohorts"]["supply"]["total"]["type"]} SpendableType
 * @typedef {AllUtxoPattern["outputs"]} OutputsPattern
 * @typedef {keyof Bitview.SeriesTree["addrs"]["raw"]} AddressableType
 *
 * Bitview pattern types (using new pattern names)
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree["cohorts"], "age.under1h">} AgeRangePattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree["cohorts"], "utxoAmount._0sats">} UtxoAmountPattern
 * @typedef {ReturnType<typeof import("./options/distribution/data.js").addressBalanceTree>} AddrAmountPattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree["cohorts"], "epoch._0">} BasicUtxoPattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree["cohorts"], "epoch._0">} EpochPattern
 * @typedef {import("./options/distribution/cohort-tree-types.js").ProjectCohortPath<Bitview.SeriesTree["cohorts"], "type.empty">} EmptyPattern
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
 * @typedef {Bitview.SeriesTree["transactions"]["volume"]["transferVolume"]} FullValuePattern
 * RollingWindowSlot: a single rolling window with stats (pct10, pct25, median, pct75, pct90, max, min) per unit
 * @typedef {Bitview.SeriesTree["transactions"]["size"]["weight"]["block"]} RollingWindowSlot
 * @typedef {Bitview.AnySeriesPattern} AnySeriesPattern
 * @typedef {Bitview.SeriesTree["rarityMeter"]["full"]["pct01"]} ActivePricePattern
 * @typedef {Bitview.AnySeriesEndpoint} AnySeriesEndpoint
 * @typedef {Bitview.AnySeriesData} AnySeriesData
 * Relative patterns by capability:
 * Unrealized patterns by capability level
 * @typedef {BasicUtxoPattern["unrealized"]} BasicRelativePattern
 * @typedef {ShortTermPattern["unrealized"]} FullRelativePattern
 *
 * Aggregate realized metrics (capitalization, prices and P&L)
 * @typedef {AllUtxoPattern["realized"]} RealizedPattern
 * @typedef {Pick<RealizedPattern, "profit" | "loss" | "netPnl" | "grossPnl" | "peakRegret">} FullRealizedProfitabilityPattern
 *
 * Transfer volume pattern (block + cumulative + sum + average windows)
 * @typedef {AllUtxoPattern["activity"]["transferVolume"]} TransferVolumePattern
 *
 * Realized profit/loss pattern (block + cumulative + sum windows, cents/usd)
 * @typedef {Bitview.SeriesTree["addrs"]["byBalance"]["realizedProfit"]["_0sats"]} RealizedProfitLossPattern
 *
 * Aggregate activity pattern (coindays, coinyears and transfer volume)
 * @typedef {AllUtxoPattern["activity"]} FullActivityPattern
 *
 *
 * PPM + percent + ratio pattern
 * @typedef {Bitview.SeriesTree["indicators"]["gini"]} PercentRatioPattern
 *
 * Percent + ratio per window + cumulative (mirrors CountPattern but for percent)
 * @typedef {Bitview.SeriesTree["opReturn"]["total"]["feeShare"]} PercentRatioCumulativePattern
 *
 * PPM + ratio pattern (for NUPL and similar)
 * @typedef {Bitview.SeriesTree["cointime"]["cap"]["aviv"]} NuplPattern
 *
 * Net PnL pattern with change (base + change + cumulative + delta + rel + sum)
 * @typedef {AllUtxoPattern["realized"]["netPnl"]} NetPnlFullPattern
 *
 * Net PnL basic pattern (base + cumulative + delta + sum)
 * @typedef {Bitview.SeriesTree["cohorts"]["realized"]["netPnl"]["age"]["under1h"]} NetPnlBasicPattern
 *
 * Realized profitability shared by Core and AgeRange cohorts
 * @typedef {Pick<AgeRangePattern["realized"], "profit" | "loss" | "netPnl">} MidRealizedPattern
 *
 * Basic realized pattern (cap + loss + MVRV + price + profit, no net/sopr)
 * @typedef {BasicUtxoPattern["realized"]} BasicRealizedPattern
 * @typedef {Pick<UtxoAmountPattern["realized"], "profit" | "loss">} BasicRealizedProfitabilityPattern
 *
 * Moving average price ratio pattern (ppm + cents + ratio + sats + usd)
 * @typedef {Bitview.SeriesTree["cointime"]["under4mAwakePrice"]} MaPriceRatioPattern
 *
 * Address count pattern (base + delta with absolute + rate)
 * @typedef {Bitview.SeriesTree["addrs"]["byBalance"]["utxoCount"]["_0sats"]} AddrCountPattern
 * @typedef {{
 *   utxo: Bitview.SeriesTree["cohorts"]["outputs"]["avgAmount"]["all"],
 *   addr: Bitview.SeriesTree["addrs"]["avgBalance"]["all"],
 * }} AvgAmountPattern
 * @typedef {Bitview.SeriesTree["addrs"]["exposed"]} ExposedTree
 * @typedef {Bitview.SeriesTree["addrs"]["reused"]} ReusedTree
 * @typedef {Bitview.SeriesTree["addrs"]["respent"]} RespentTree
 */

/**
 * @template T
 * @typedef {Bitview.SeriesEndpoint<T>} SeriesEndpoint
 */
/**
 * Rolling windows pattern (24h, 1w, 1m, 1y)
 * @typedef {Bitview.SeriesTree["inputs"]["perSec"]} RollingWindowPattern
 */
/**
 * Sell side risk rolling windows pattern
 * @typedef {Bitview.SeriesTree["cohorts"]["realized"]["valueDestroyed"]["age"]["under1h"]["average"]} SellSideRiskPattern
 */
/**
 * Stats pattern: min, max, median, percentiles
 * @typedef {Bitview.SeriesTree["transactions"]["size"]["weight"]["block"]} StatsPattern
 */
/**
 * Full stats pattern: cumulative, sum, average, min, max, percentiles + rolling
 * @typedef {Bitview.SeriesTree["blocks"]["vbytes"]} FullStatsPattern
 */
/**
 * Aggregated pattern: cumulative + rolling (with distribution stats) + sum (no base)
 * @typedef {Bitview.SeriesTree["inputs"]["count"]} AggregatedPattern
 */
/**
 * Count pattern: height, cumulative, and rolling sum windows. Typed from a nullable instance of
 * the shared shape (`?Float64` values), which every count series is assignable to.
 * @typedef {Bitview.SeriesTree["cointime"]["value"]["destroyed"]} CountPattern
 */
/**
 * Full per-block pattern: height, cumulative, sum, and distribution stats (all flat)
 * FullPerBlockPattern: cumulative + sum + average + distribution stats (used by chartsFromFull)
 * Note: some callers also have .block but the function doesn't use it
 * @typedef {Omit<Bitview.SeriesTree["blocks"]["vbytes"], 'block'>} FullPerBlockPattern
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
 * @typedef {Bitview.SeriesTree["opReturn"]["total"]["feeShare"]} DominancePattern
 */

/**
 *
 * @typedef {InstanceType<typeof BitviewClient>["INDEXES"]} Indexes
 * @typedef {Indexes[number]} IndexName
 * @typedef {InstanceType<typeof BitviewClient>["POOL_ID_TO_POOL_NAME"]} PoolIdToPoolName
 * @typedef {keyof PoolIdToPoolName} PoolId
 *
 * Tree branch types
 * @typedef {Bitview.SeriesTree["market"]} Market
 * @typedef {Bitview.SeriesTree["market"]["movingAverage"]} MarketMovingAverage
 * @typedef {FullStatsPattern} AnyFullStatsPattern
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
 * @typedef {ShortTermPattern | LongTermPattern | AgeRangePattern | BasicUtxoPattern} PatternWithNupl
 * @typedef {AllUtxoPattern | AgeRangePattern | UtxoAmountPattern} PatternWithCostBasis
 * @typedef {AllUtxoPattern | AgeRangePattern | UtxoAmountPattern} PatternWithActivity
 * @typedef {AllUtxoPattern | AgeRangePattern} PatternWithCostBasisPercentiles
 * @typedef {Bitview.SeriesTree["cointime"]["urpd"]["all"]["costBasis"]["perCoin"]} PercentilesPattern
 *
 * Cohort objects with specific pattern capabilities
 * @typedef {{ name: string, title: string, color: Color, tree: { realized: { price: AnyPricePattern } } }} CohortWithRealizedPrice
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithFullRealized }} CohortWithFullRealized
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithNupl }} CohortWithNupl
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithCostBasis }} CohortWithCostBasis
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithActivity }} CohortWithActivity
 * @typedef {{ name: string, title: string, color: Color, tree: PatternWithCostBasisPercentiles }} CohortWithCostBasisPercentiles
 * @typedef {{ name: string, title: string, color: Color, tree: { realized: BasicRealizedProfitabilityPattern } }} CohortWithRealizedProfitLoss
 * @typedef {{ name: string, title: string, color: Color, tree: { realized: { cap: { usd: AnySeriesPattern, delta?: FiatDeltaPattern } } } }} CohortWithRealizedCap
 *
 * Cohorts with full NUPL and cost-basis percentiles.
 * @typedef {CohortFull | CohortLongTerm} CohortWithNuplPercentiles
 * @typedef {{ name: string, title: string, list: readonly CohortWithNuplPercentiles[], all: CohortAll }} CohortGroupWithNuplPercentiles
 *
 * Delta patterns with absolute + rate rolling windows
 * @typedef {Bitview.SeriesTree["addrs"]["delta"]["all"]} DeltaPattern
 * @typedef {AllUtxoPattern["realized"]["cap"]["delta"]} FiatDeltaPattern
 * @typedef {AllUtxoPattern["supply"]["delta"]} AmountDeltaPattern
 * @typedef {Bitview.SeriesTree["addrs"]["byBalance"]["supply"]["_0sats"]["delta"]["absolute"]["_1m"]} AmountPattern
 *
 * Generic tree node type for walking
 * @typedef {null | undefined | string | number | boolean | bigint | symbol} TreePrimitive
 * @typedef {(...args: never[]) => void} TreeFunction
 * @typedef {{ [key: string]: TreeNode }} TreeBranch
 * @typedef {TreePrimitive | TreeFunction | AnySeriesPattern | TreeBranch} TreeNode
 */
