import { bitview } from "../../utils/client.js";
import { colors } from "../../utils/colors.js";
import { Unit } from "../../utils/units.js";
import {
  baseline,
  dotted,
  histogram,
  line,
  percentRatio,
  price,
} from "../series.js";
import { percentileBands, priceBands } from "../shared.js";

import { createRarityMeterComponents } from "./rarity-meter-components.js";
import { createRarityMeterVariants } from "./rarity-meter-variants.js";

/**
 * Create Rarity Meter model section.
 * @returns {PartialOptionsGroup}
 */
export function createRarityMeterSection() {
  const { rarityMeter, distributionAggregated } = bitview.series;
  const all = distributionAggregated.cohorts.all;
  const extremes = /** @type {const} */ ([
    {
      key: "coinsInLoss",
      name: "Coins in Loss",
      source: all.supply.inLoss.btc,
      unit: Unit.btc,
      color: colors.loss,
    },
    {
      key: "profitTaking",
      name: "Profit Taking",
      source: all.realized.profit.sum._24h.usd,
      unit: Unit.usd,
      color: colors.bitcoin,
    },
    {
      key: "capitulation",
      name: "Capitulation",
      source: all.realized.loss.sum._24h.usd,
      unit: Unit.usd,
      color: colors.loss,
    },
    {
      key: "peakRegret",
      name: "Peak Regret",
      source: all.realized.peakRegret.sum._24h.usd,
      unit: Unit.usd,
      color: colors.regret,
    },
    {
      key: "sellerExhaustion",
      name: "Seller Exhaustion",
      source: all.ratios.sellSideRiskRatio._24h.percent,
      unit: Unit.percentage,
      color: colors.profit,
    },
  ]);

  return {
    name: "Rarity Meter",
    tree: [
      ...createRarityMeterVariants().map((version) => ({
        name: version.name,
        tree: version.variants.map(({ name, meter, sources }) => {
          const title = `Bitcoin Rarity Meter ${version.name}: ${name}`;
          return {
            name,
            tree: [
              {
                name: "Meter",
                title,
                top: priceBands(percentileBands(meter)),
                bottom: [
                  histogram({
                    series: meter.index,
                    name: "Index",
                    unit: Unit.count,
                    colorFn: (value) =>
                      /** @type {const} */ ([
                        colors.ratioPct._0_1,
                        colors.ratioPct._0_5,
                        colors.ratioPct._1,
                        colors.ratioPct._2,
                        colors.ratioPct._5,
                        colors.transparent,
                        colors.ratioPct._95,
                        colors.ratioPct._98,
                        colors.ratioPct._99,
                        colors.ratioPct._99_5,
                        colors.ratioPct._99_9,
                      ])[value + 5],
                  }),
                  baseline({
                    series: meter.score,
                    name: "Score",
                    unit: Unit.count,
                    color: [colors.ratioPct._99_9, colors.ratioPct._0_1],
                    defaultActive: false,
                  }),
                ],
              },
              {
                name: "Sources",
                title: `${title} Sources`,
                top: sources.map((source, index) =>
                  price({
                    ...source,
                    color: colors.at(index, sources.length),
                  }),
                ),
              },
            ],
          };
        }),
      })),
      {
        name: "Extremes",
        tree: extremes.map(({ key, name, source, unit, color }) => {
          const extreme = rarityMeter.extremes[key];
          return {
            name,
            tree: [
              {
                name: "Value",
                title: `Bitcoin Rarity Meter: ${name}`,
                bottom: [
                  line({
                    series: source,
                    name,
                    color,
                    unit,
                  }),
                  dotted({
                    series: extreme.thresholdPct01,
                    name: "0.1%",
                    color: colors.ratioPct._95,
                    unit,
                  }),
                  dotted({
                    series: extreme.thresholdPct005,
                    name: "0.05%",
                    color: colors.ratioPct._99,
                    unit,
                  }),
                  dotted({
                    series: extreme.thresholdPct0025,
                    name: "0.025%",
                    color: colors.ratioPct._99_9,
                    unit,
                  }),
                ],
              },
              {
                name: "Tail",
                title: `Bitcoin Rarity Meter: ${name} Historical Tail`,
                bottom: percentRatio({
                  pattern: extreme.tail,
                  name: "Historical Tail",
                  color,
                }),
              },
              {
                name: "Rank",
                title: `Bitcoin Rarity Meter: ${name} Extreme Rank`,
                bottom: [
                  histogram({
                    series: extreme.rank,
                    name: "Rank",
                    unit: Unit.count,
                    colorFn: (rank) =>
                      rank >= 3
                        ? color
                        : rank === 2
                          ? colors.ratioPct._99
                          : rank === 1
                            ? colors.ratioPct._95
                            : colors.transparent,
                  }),
                ],
              },
            ],
          };
        }),
      },
      createRarityMeterComponents(),
    ],
  };
}
