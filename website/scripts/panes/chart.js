import { createHeader } from "../utils/dom.js";
import { chartElement } from "../utils/elements.js";
import { INDEX_FROM_LABEL } from "../utils/serde.js";
import { Unit } from "../utils/units.js";
import { createChart } from "../utils/chart/index.js";
import { colors } from "../utils/colors.js";
import { latestPrice, onPrice } from "../utils/price.js";
import { brk } from "../utils/client.js";

const ONE_BTC_IN_SATS = 100_000_000;

/** @type {((opt: ChartOption) => void) | null} */
let _setOption = null;

/**
 * @param {ChartOption} opt
 */
export function setOption(opt) {
  if (!_setOption) throw new Error("Chart not initialized");
  _setOption(opt);
}

export function init() {
  const { headerElement, headingElement } = createHeader();
  chartElement.append(headerElement);

  const chart = createChart({
    parent: chartElement,
    brk,
  });

  const setChoices = chart.setIndexChoices;

  /**
   * Build top blueprints with price series prepended for each unit
   * @param {Map<Unit, AnyFetchedSeriesBlueprint[]>} optionTop
   * @returns {Map<Unit, AnyFetchedSeriesBlueprint[]>}
   */
  function buildTopBlueprints(optionTop) {
    /** @type {Map<Unit, AnyFetchedSeriesBlueprint[]>} */
    const result = new Map();

    const { ohlc, spot } = brk.series.price;

    result.set(Unit.usd, [
      /** @type {AnyFetchedSeriesBlueprint} */ ({
        type: "Price",
        title: "Price",
        series: spot.usd,
        ohlcSeries: ohlc.usd,
      }),
      ...(optionTop.get(Unit.usd) ?? []),
    ]);

    result.set(Unit.sats, [
      /** @type {AnyFetchedSeriesBlueprint} */ ({
        type: "Price",
        title: "Price",
        series: spot.sats,
        ohlcSeries: ohlc.sats,
        colors: /** @type {const} */ ([colors.default, colors.background]),
      }),
      ...(optionTop.get(Unit.sats) ?? []),
    ]);

    return result;
  }

  function updatePriceWithLatest() {
    const latest = latestPrice();
    if (latest === null) return;

    const priceSeries = chart.panes[0].series[0];
    const unit = chart.panes[0].unit;
    if (!priceSeries?.hasData() || !unit) return;

    const last = priceSeries.getData().at(-1);
    if (!last) return;

    // Convert to sats if needed
    const close =
      unit === Unit.sats ? Math.floor(ONE_BTC_IN_SATS / latest) : latest;

    if ("close" in last) {
      // Candlestick data
      priceSeries.update({ ...last, close });
    } else {
      // Line data
      priceSeries.update({ ...last, value: close });
    }
  }

  // Set up the setOption function
  _setOption = (opt) => {
    headingElement.innerHTML = opt.title;

    // Set blueprints first so storageId is correct before any index change
    chart.setBlueprints({
      name: opt.title,
      top: buildTopBlueprints(opt.top()),
      bottom: opt.bottom(),
      onDataLoaded: updatePriceWithLatest,
    });

    // Update index choices (may trigger rebuild if index changes)
    setChoices(computeChoices(opt));
  };

  // Live price update listener
  onPrice(updatePriceWithLatest);
}

/** @type {{ label: string, items: IndexLabel[] }[]} */
const ALL_GROUPS = [
  {
    label: "Time",
    items: [
      "10mn",
      "30mn",
      "1h",
      "4h",
      "12h",
      "1d",
      "3d",
      "1w",
      "1m",
      "3m",
      "6m",
      "1y",
      "10y",
    ],
  },
  { label: "Block", items: ["blk", "epch", "halv"] },
];

const ALL_CHOICES = /** @satisfies {IndexLabel[]} */ (
  ALL_GROUPS.flatMap((g) => g.items)
);

/**
 * @param {ChartOption} opt
 * @returns {{ choices: IndexLabel[], groups: { label: string, items: IndexLabel[] }[] }}
 */
function computeChoices(opt) {
  if (!opt.top().size && !opt.bottom().size) {
    return { choices: [...ALL_CHOICES], groups: ALL_GROUPS };
  }
  const rawIndexes = new Set(
    [Array.from(opt.top().values()), Array.from(opt.bottom().values())]
      .flat(2)
      .filter((blueprint) => {
        const path = Object.values(blueprint.series.by)[0]?.path ?? "";
        return !path.includes("constant_");
      })
      .flatMap((blueprint) => blueprint.series.indexes()),
  );

  const groups = ALL_GROUPS.map(({ label, items }) => ({
    label,
    items: items.filter((choice) => rawIndexes.has(INDEX_FROM_LABEL[choice])),
  })).filter(({ items }) => items.length > 0);

  return {
    choices: groups.flatMap((g) => g.items),
    groups,
  };
}
