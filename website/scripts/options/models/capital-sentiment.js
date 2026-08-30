import { bitview } from "../../utils/client.js";
import { colors } from "../../utils/colors.js";
import { Unit } from "../../utils/units.js";
import { histogram, price } from "../series.js";

/**
 * Create Capital Sentiment model section.
 * @returns {PartialOptionsGroup}
 */
export function createCapitalSentimentSection() {
  const { capitalSentiment, cohorts } = bitview.series;
  const sma = bitview.series.market.movingAverage.sma._1y;
  const references = () => [
    price({
      series: cohorts.realized.capitalizedPrice.all,
      name: "All",
      color: colors.capitalized,
    }),
    price({
      series: cohorts.realized.capitalizedPrice.sth,
      name: "STH",
      color: colors.term.short,
    }),
    price({
      series: cohorts.realized.capitalizedPrice.lth,
      name: "LTH",
      color: colors.term.long,
    }),
    price({
      series: sma,
      name: "1Y SMA",
      color: colors.time._1y,
    }),
  ];

  return {
    name: "Capital Sentiment",
    tree: [
      {
        name: "Score",
        title: "Capital Sentiment Score",
        top: references(),
        bottom: [
          histogram({
            series: capitalSentiment.score,
            name: "Score",
            unit: Unit.count,
            colorFn: (score) =>
              score === 2
                ? colors.capitalSentiment.bull
                : score === 1
                  ? colors.capitalSentiment.cautiousBull
                  : score === -1
                    ? colors.capitalSentiment.limbo
                    : colors.capitalSentiment.bear,
          }),
        ],
      },
      {
        name: "Position",
        title: "Capital Sentiment Position",
        top: references(),
        bottom: [
          histogram({
            series: capitalSentiment.isLong,
            name: "Long",
            unit: Unit.state,
            color: colors.capitalSentiment.bull,
          }),
          histogram({
            series: capitalSentiment.isShort,
            name: "Short",
            unit: Unit.state,
            color: colors.capitalSentiment.bear,
          }),
        ],
      },
    ],
  };
}
