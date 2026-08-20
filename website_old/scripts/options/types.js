/**
 * @typedef {Object} BaseSeriesBlueprint
 * @property {string} title
 * @property {string} [key] - Optional key for persistence (derived from title if not provided)
 * @property {boolean} [defaultActive]
 *
 * @typedef {Object} BaselineSeriesBlueprintSpecific
 * @property {"Baseline"} type
 * @property {Color} [color]
 * @property {[Color, Color]} [colors]
 * @property {BaselineSeriesPartialOptions} [options]
 * @typedef {BaseSeriesBlueprint & BaselineSeriesBlueprintSpecific} BaselineSeriesBlueprint
 *
 * @typedef {Object} CandlestickSeriesBlueprintSpecific
 * @property {"Candlestick"} type
 * @property {[Color, Color]} [colors]
 * @property {CandlestickSeriesPartialOptions} [options]
 * @typedef {BaseSeriesBlueprint & CandlestickSeriesBlueprintSpecific} CandlestickSeriesBlueprint
 *
 * @typedef {Object} LineSeriesBlueprintSpecific
 * @property {"Line"} [type]
 * @property {Color} [color]
 * @property {(value: number) => Color} [colorFn]
 * @property {LineSeriesPartialOptions} [options]
 * @typedef {BaseSeriesBlueprint & LineSeriesBlueprintSpecific} LineSeriesBlueprint
 *
 * @typedef {Object} HistogramSeriesBlueprintSpecific
 * @property {"Histogram"} type
 * @property {Color | [Color, Color]} [color] - Single color or [positive, negative] colors (defaults to green/red)
 * @property {(value: number) => Color} [colorFn]
 * @property {HistogramSeriesPartialOptions} [options]
 * @typedef {BaseSeriesBlueprint & HistogramSeriesBlueprintSpecific} HistogramSeriesBlueprint
 *
 * @typedef {Object} DotsSeriesBlueprintSpecific
 * @property {"Dots"} type
 * @property {Color} [color]
 * @property {LineSeriesPartialOptions} [options]
 * @typedef {BaseSeriesBlueprint & DotsSeriesBlueprintSpecific} DotsSeriesBlueprint
 *
 * @typedef {Object} DotsBaselineSeriesBlueprintSpecific
 * @property {"DotsBaseline"} type
 * @property {Color} [color]
 * @property {[Color, Color]} [colors]
 * @property {BaselineSeriesPartialOptions} [options]
 * @typedef {BaseSeriesBlueprint & DotsBaselineSeriesBlueprintSpecific} DotsBaselineSeriesBlueprint
 *
 * @typedef {Object} PriceSeriesBlueprintSpecific
 * @property {"Price"} type
 * @property {AnySeriesPattern} ohlcSeries - OHLC series for candlestick (>= 1h indexes)
 * @property {[Color, Color]} [colors]
 * @property {CandlestickSeriesPartialOptions} [options]
 * @typedef {BaseSeriesBlueprint & PriceSeriesBlueprintSpecific} PriceSeriesBlueprint
 *
 * @typedef {BaselineSeriesBlueprint | CandlestickSeriesBlueprint | LineSeriesBlueprint | HistogramSeriesBlueprint | DotsSeriesBlueprint | DotsBaselineSeriesBlueprint | PriceSeriesBlueprint} AnySeriesBlueprint
 *
 * @typedef {AnySeriesBlueprint["type"]} SeriesType
 *
 * @typedef {{ series: AnySeriesPattern, unit?: Unit }} FetchedAnySeriesOptions
 *
 * @typedef {BaselineSeriesBlueprint & FetchedAnySeriesOptions} FetchedBaselineSeriesBlueprint
 * @typedef {CandlestickSeriesBlueprint & FetchedAnySeriesOptions} FetchedCandlestickSeriesBlueprint
 * @typedef {LineSeriesBlueprint & FetchedAnySeriesOptions} FetchedLineSeriesBlueprint
 * @typedef {HistogramSeriesBlueprint & FetchedAnySeriesOptions} FetchedHistogramSeriesBlueprint
 * @typedef {DotsSeriesBlueprint & FetchedAnySeriesOptions} FetchedDotsSeriesBlueprint
 * @typedef {DotsBaselineSeriesBlueprint & FetchedAnySeriesOptions} FetchedDotsBaselineSeriesBlueprint
 * @typedef {AnySeriesBlueprint & FetchedAnySeriesOptions} AnyFetchedSeriesBlueprint
 *
 * Any pattern with usd and sats sub-series (auto-expands to USD + sats)
 * @typedef {{ usd: AnySeriesPattern, sats: AnySeriesPattern }} AnyPricePattern
 *
 * Any pattern with sats, btc, and usd sub-series (value patterns like stack)
 * @typedef {{ sats: AnySeriesPattern, btc: AnySeriesPattern, usd: AnySeriesPattern }} AnyValuePattern
 *
 * Top pane price series - requires a price pattern with usd/sats, auto-expands to USD + sats
 * @typedef {{ series: AnyPricePattern }} FetchedPriceSeriesOptions
 * @typedef {LineSeriesBlueprint & FetchedPriceSeriesOptions} FetchedPriceSeriesBlueprint
 *
 * @typedef {Object} PartialOption
 * @property {string} name
 *
 * @typedef {Object} ProcessedOptionAddons
 * @property {string} title
 * @property {string[]} path
 *
 * @typedef {Object} PartialExplorerOptionSpecific
 * @property {"explorer"} kind
 * @property {string} title
 *
 * @typedef {PartialOption & PartialExplorerOptionSpecific} PartialExplorerOption
 *
 * @typedef {Required<PartialExplorerOption> & ProcessedOptionAddons} ExplorerOption
 *
 * @typedef {Object} PartialChartOptionSpecific
 * @property {"chart"} [kind]
 * @property {string} title
 * @property {(FetchedPriceSeriesBlueprint | AnyFetchedSeriesBlueprint)[]} [top]
 * @property {AnyFetchedSeriesBlueprint[]} [bottom]
 *
 * @typedef {PartialOption & PartialChartOptionSpecific} PartialChartOption
 *
 * @typedef {Object} ProcessedChartOptionAddons
 * @property {() => Map<Unit, AnyFetchedSeriesBlueprint[]>} top
 * @property {() => Map<Unit, AnyFetchedSeriesBlueprint[]>} bottom
 *
 * @typedef {Required<Omit<PartialChartOption, "top" | "bottom">> & ProcessedChartOptionAddons & ProcessedOptionAddons} ChartOption
 *
 * @typedef {Object} PartialHeatmapOptionSpecific
 * @property {"heatmap"} kind
 * @property {string} title
 * @property {HeatmapPointSource} points
 * @property {HeatmapGridFactory} grid
 * @property {HeatmapColorFn} color
 * @property {HeatmapAxis} [axis]
 * @property {HeatmapDefaults} [defaults]
 * @property {HeatmapTooltipFn} [tooltip]
 *
 * @typedef {PartialOption & PartialHeatmapOptionSpecific} PartialHeatmapOption
 *
 * @typedef {PartialHeatmapOption & ProcessedOptionAddons} HeatmapOption
 *
 * @typedef {Object} PartialUrlOptionSpecific
 * @property {"link"} [kind]
 * @property {() => string} url
 * @property {string} title
 * @property {boolean} [qrcode]
 *
 * @typedef {PartialOption & PartialUrlOptionSpecific} PartialUrlOption
 *
 * @typedef {Required<PartialUrlOption> & ProcessedOptionAddons} UrlOption
 *
 * @typedef {PartialExplorerOption | PartialChartOption | PartialUrlOption | PartialHeatmapOption} AnyPartialOption
 *
 * @typedef {ExplorerOption | ChartOption | UrlOption | HeatmapOption} Option
 *
 * @typedef {(AnyPartialOption | PartialOptionsGroup)[]} PartialOptionsTree
 *
 * @typedef {Object} PartialOptionsGroup
 * @property {string} name
 * @property {PartialOptionsTree} tree
 *
 * @typedef {Object} OptionsGroup
 * @property {string} name
 * @property {OptionsTree} tree
 *
 * @typedef {(Option | OptionsGroup)[]} OptionsTree
 *
 * @typedef {Object} UtxoCohortObject
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {UtxoCohortPattern} tree
 *
 * ============================================================================
 * UTXO Cohort Pattern Types (based on Bitview client patterns)
 * ============================================================================
 *
 * Patterns with adjustedSopr + percentiles + RelToMarketCap:
 *   - ShortTermPattern (term.short)
 * @typedef {ShortTermPattern} PatternFull
 *
 * The "All" pattern is special - has adjustedSopr + percentiles but NO RelToMarketCap
 * @typedef {AllUtxoPattern} PatternAll
 *
 * Core UTXO cohort shape:
 *   - Entry, under/over age, epoch, and class cohorts
 * @typedef {BasicUtxoPattern} PatternCore
 *
 * Patterns with percentiles only (RealizedPattern2, CostBasisPattern2):
 *   - LongTermPattern (term.long)
 *   - AgeRangePattern (ageRange.*)
 * @typedef {LongTermPattern | AgeRangePattern} PatternWithPercentiles
 *
 * Patterns with RelToMarketCap in relative (geAmount.*, ltAmount.*):
 * @typedef {UtxoAmountPattern} PatternBasicWithMarketCap
 *
 * Patterns without RelToMarketCap in relative:
 *   - EpochPattern (epoch.*, year.*)
 *   - UtxoAmountPattern (amountRange.*)
 *   - OutputsRealizedSupplyUnrealizedPattern2 (addressable type.*)
 * @typedef {EpochPattern | UtxoAmountPattern | EmptyPattern} PatternBasicWithoutMarketCap
 *
 * Patterns without relative section entirely (edge case output types):
 *   - EmptyPattern (type.empty, type.p2ms, type.unknown)
 * @typedef {EmptyPattern} PatternWithoutRelative
 *
 * Union of basic patterns (for backwards compat)
 * @typedef {PatternBasicWithMarketCap | PatternBasicWithoutMarketCap} PatternBasic
 *
 * ============================================================================
 * Cohort Object Types (by capability)
 * ============================================================================
 *
 * All cohort: adjustedSopr + percentiles but NO RelToMarketCap (special)
 * @typedef {Object} CohortAll
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {PatternAll} tree
 * @property {AddrCountPattern} addressCount
 * @property {AvgAmountPattern} avgAmount
 *
 * Full cohort: adjustedSopr + percentiles + RelToMarketCap (term.short)
 * @typedef {Object} CohortFull
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {PatternFull} tree
 *
 * Core UTXO cohort.
 * @typedef {Object} CohortCore
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {PatternCore} tree
 *
 * Cohort with percentiles only (term.long, ageRange.*)
 * @typedef {Object} CohortWithPercentiles
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {PatternWithPercentiles} tree
 *
 * Long term cohort (term.long) - has nupl via RelativePattern5
 * @typedef {Object} CohortLongTerm
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {LongTermPattern} tree
 *
 * Age range cohort (ageRange.*) - no nupl via RelativePattern2
 * @typedef {Object} CohortAgeRange
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {AgeRangePattern} tree
 *
 * Age range cohort with matured supply
 * @typedef {CohortAgeRange & { matured: FullValuePattern }} CohortAgeRangeWithMatured
 *
 * Basic cohort WITH RelToMarketCap (geAmount.*, ltAmount.*)
 * @typedef {Object} CohortBasicWithMarketCap
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {PatternBasicWithMarketCap} tree
 *
 * Basic cohort WITHOUT RelToMarketCap (epoch.*, amountRange.*, year.*, type.*)
 * @typedef {Object} CohortBasicWithoutMarketCap
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {PatternBasicWithoutMarketCap} tree
 *
 * Cohort without relative section (edge case types: empty, p2ms, unknown)
 * @typedef {Object} CohortWithoutRelative
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {PatternWithoutRelative} tree
 *
 * Union of basic cohort types
 * @typedef {CohortBasicWithMarketCap | CohortBasicWithoutMarketCap} CohortBasic
 *
 * ============================================================================
 * Extended Cohort Types (with address count)
 * ============================================================================
 *
 * Addressable cohort with address count (for "type" cohorts - uses OutputsRealizedSupplyUnrealizedPattern2)
 * @typedef {{ name: string, key: AddressableType, title: string, color: Color, tree: EmptyPattern, addressCount: AddrCountPattern, avgAmount: AvgAmountPattern, exposed: ExposedTree, reused: ReusedTree, respent: RespentTree }} CohortAddr
 *
 * ============================================================================
 * Cohort Group Types (by capability)
 * ============================================================================
 *
 * @typedef {Object} CohortGroupFull
 * @property {string} name
 * @property {string} title
 * @property {readonly CohortFull[]} list
 * @property {CohortAll} all
 *
 * @typedef {Object} CohortGroupCore
 * @property {string} name
 * @property {string} title
 * @property {readonly CohortCore[]} list
 * @property {CohortAll} all
 *
 * @typedef {Object} CohortGroupWithPercentiles
 * @property {string} name
 * @property {string} title
 * @property {readonly CohortWithPercentiles[]} list
 * @property {CohortAll} all
 *
 * @typedef {Object} CohortGroupLongTerm
 * @property {string} name
 * @property {string} title
 * @property {readonly CohortLongTerm[]} list
 * @property {CohortAll} all
 *
 * @typedef {Object} CohortGroupAgeRange
 * @property {string} name
 * @property {string} title
 * @property {readonly CohortAgeRange[]} list
 * @property {CohortAll} all
 *
 * @typedef {Object} CohortGroupBasicWithMarketCap
 * @property {string} name
 * @property {string} title
 * @property {readonly CohortBasicWithMarketCap[]} list
 * @property {CohortAll} all
 *
 * @typedef {Object} CohortGroupBasicWithoutMarketCap
 * @property {string} name
 * @property {string} title
 * @property {readonly CohortBasicWithoutMarketCap[]} list
 * @property {CohortAll} all
 *
 * @typedef {Object} CohortGroupWithoutRelative
 * @property {string} name
 * @property {string} title
 * @property {readonly CohortWithoutRelative[]} list
 * @property {CohortAll} all
 *
 * Union of basic cohort group types
 * @typedef {CohortGroupBasicWithMarketCap | CohortGroupBasicWithoutMarketCap} CohortGroupBasic
 *
 * @typedef {Object} UtxoCohortGroupObject
 * @property {string} name
 * @property {string} title
 * @property {readonly UtxoCohortObject[]} list
 * @property {CohortAll} all
 *
 * @typedef {Object} AddrCohortObject
 * @property {string} name
 * @property {string} title
 * @property {Color} color
 * @property {AddrCohortPattern} tree
 * @property {AddrCountPattern} addressCount
 *
 * @typedef {UtxoCohortObject | AddrCohortObject | CohortWithoutRelative} CohortObject
 *
 * @typedef {Object} AddrCohortGroupObject
 * @property {string} name
 * @property {string} title
 * @property {readonly AddrCohortObject[]} list
 * @property {CohortAll} all
 *
 * @typedef {UtxoCohortGroupObject | AddrCohortGroupObject} CohortGroupObject
 *
 * @typedef {Object} CohortGroupAddr
 * @property {string} name
 * @property {string} title
 * @property {readonly CohortAddr[]} list
 * @property {CohortAll} all
 */

// Re-export for type consumers
export {};
