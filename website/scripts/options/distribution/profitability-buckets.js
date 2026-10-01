import { colors } from "../../utils/colors.js";
import { Unit } from "../../utils/units.js";
import {
  ROLLING_WINDOWS,
  line,
  percentRatio,
  amountSumsTreeBaseline,
  rollingPercentRatioTree,
} from "../series.js";
import { amountBaseline, formatCohortTitle, satsBtcUsd } from "../shared.js";

const CAPITAL_METRICS = /** @type {const} */ ([
  { key: "realizedCap", name: "Realized Cap" },
  { key: "unrealizedPnl", name: "Unrealized PnL" },
]);

const TERMS = /** @type {const} */ ([
  { key: "all", name: "All", color: colors.default },
  { key: "sth", name: "STH", color: colors.term.short },
  { key: "lth", name: "LTH", color: colors.term.long },
]);

/**
 * @param {RealizedSupplyPattern["supply"]["all"]} supply
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function supplyChanges(supply, title) {
  return [
    {
      ...amountSumsTreeBaseline({
        windows: supply.delta.absolute,
        title,
        metric: "Supply Change",
        legend: "Change",
      }),
      name: "Change",
    },
    {
      ...rollingPercentRatioTree({
        windows: supply.delta.rate,
        title,
        metric: "Supply Growth Rate",
      }),
      name: "Growth Rate",
    },
  ];
}

/**
 * @param {{ name: string, color: Color, pattern: RealizedSupplyPattern }} bucket
 * @returns {PartialOptionsGroup}
 */
function singleBucketFolder({ name, color, pattern }) {
  const title = formatCohortTitle(name);
  return {
    name,
    tree: [
      {
        name: "Supply",
        tree: [
          {
            name: "Total",
            title: title("Supply"),
            bottom: TERMS.flatMap((term) =>
              satsBtcUsd({
                pattern: pattern.supply[term.key],
                name: term.name,
                color: term.color,
              }),
            ),
          },
          ...supplyChanges(pattern.supply.all, title),
          ...TERMS.slice(1).map((term) => ({
            name: term.name,
            tree: supplyChanges(
              pattern.supply[term.key],
              formatCohortTitle(`${name} ${term.name}`),
            ),
          })),
        ],
      },
      ...CAPITAL_METRICS.map((metric) => ({
        name: metric.name,
        title: title(metric.name),
        bottom: TERMS.map((term) =>
          line({
            series: pattern[metric.key][term.key].usd,
            name: term.name,
            color: term.color,
            unit: Unit.usd,
          }),
        ),
      })),
      {
        name: "NUPL",
        title: title("NUPL"),
        bottom: [
          line({ series: pattern.nupl.ratio, name, color, unit: Unit.ratio }),
        ],
      },
    ],
  };
}

/**
 * @param {{ name: string, color: Color, pattern: RealizedSupplyPattern }[]} list
 * @param {"all" | "sth" | "lth"} term
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedSupplyChanges(list, term, title) {
  return [
    {
      name: "Change",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} Supply Change`),
        bottom: list.flatMap(({ name, color, pattern }) =>
          amountBaseline({
            pattern: pattern.supply[term].delta.absolute[w.key],
            name,
            color,
          }),
        ),
      })),
    },
    {
      name: "Growth Rate",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} Supply Growth Rate`),
        bottom: list.flatMap(({ name, color, pattern }) =>
          percentRatio({
            pattern: pattern.supply[term].delta.rate[w.key],
            name,
            color,
          }),
        ),
      })),
    },
  ];
}

/**
 * @param {{ name: string, color: Color, pattern: RealizedSupplyPattern }[]} list
 * @returns {PartialOptionsTree}
 */
function groupedBucketCharts(list) {
  const title = formatCohortTitle("Profitability Range");
  return [
    {
      name: "Supply",
      tree: [
        ...TERMS.map((term) => ({
          name: term.name,
          title: title(`${term.key === "all" ? "" : `${term.name} `}Supply`),
          bottom: list.flatMap(({ name, color, pattern }) =>
            satsBtcUsd({ pattern: pattern.supply[term.key], name, color }),
          ),
        })),
        ...groupedSupplyChanges(list, "all", title),
        ...TERMS.slice(1).map((term) => ({
          name: `${term.name} Changes`,
          tree: groupedSupplyChanges(
            list,
            term.key,
            formatCohortTitle(`Profitability Range ${term.name}`),
          ),
        })),
      ],
    },
    ...CAPITAL_METRICS.map((metric) => ({
      name: metric.name,
      tree: TERMS.map((term) => ({
        name: term.name,
        title: title(
          `${term.key === "all" ? "" : `${term.name} `}${metric.name}`,
        ),
        bottom: list.map(({ name, color, pattern }) =>
          line({
            series: pattern[metric.key][term.key].usd,
            name,
            color,
            unit: Unit.usd,
          }),
        ),
      })),
    })),
    {
      name: "NUPL",
      title: title("NUPL"),
      bottom: list.map(({ name, color, pattern }) =>
        line({ series: pattern.nupl.ratio, name, color, unit: Unit.ratio }),
      ),
    },
  ];
}

/**
 * @param {{ range: { name: string, color: Color, pattern: RealizedSupplyPattern }[] }} args
 * @returns {PartialOptionsGroup}
 */
export function createUtxoProfitabilitySection({ range }) {
  return {
    name: "UTXO Profitability",
    tree: [
      { name: "Compare", tree: groupedBucketCharts(range) },
      ...range.map(singleBucketFolder),
    ],
  };
}
