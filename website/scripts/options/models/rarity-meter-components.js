import { bitview } from "../../utils/client.js";
import { colors } from "../../utils/colors.js";
import { priceRatioPercentilesTree } from "../shared.js";

/**
 * @param {Parameters<typeof priceRatioPercentilesTree>[0] & { name: string }} component
 * @returns {PartialChartOption}
 */
function componentChart(component) {
  const [chart] = priceRatioPercentilesTree({
    ...component,
    title: `Bitcoin Rarity Meter: ${component.title}`,
  });
  return { ...chart, name: component.name };
}

/** @returns {PartialOptionsGroup} */
export function createRarityMeterComponents() {
  const { rarityMeter, cointime, coinflow, cohorts } = bitview.series;
  const components = rarityMeter.components;
  const medians = [
    {
      name: "All",
      btc: components.medianPriceBtcWeighted,
      usd: components.medianPriceUsdWeighted,
    },
    {
      name: "STH",
      btc: components.sthMedianPriceBtcWeighted,
      usd: components.sthMedianPriceUsdWeighted,
    },
    {
      name: "LTH",
      btc: components.lthMedianPriceBtcWeighted,
      usd: components.lthMedianPriceUsdWeighted,
    },
    {
      name: "Cointime",
      btc: components.cointimeMedianPriceBtcWeighted,
      usd: components.cointimeMedianPriceUsdWeighted,
      color: colors.cointime,
    },
    {
      name: "STH Cointime",
      btc: components.sthCointimeMedianPriceBtcWeighted,
      usd: components.sthCointimeMedianPriceUsdWeighted,
      color: colors.cointime,
    },
    {
      name: "LTH Cointime",
      btc: components.lthCointimeMedianPriceBtcWeighted,
      usd: components.lthCointimeMedianPriceUsdWeighted,
      color: colors.cointime,
    },
    {
      name: "Coinflow",
      btc: components.coinflowMedianPriceBtcWeighted,
      usd: components.coinflowMedianPriceUsdWeighted,
      color: colors.coinflow,
    },
    {
      name: "STH Coinflow",
      btc: components.sthCoinflowMedianPriceBtcWeighted,
      usd: components.sthCoinflowMedianPriceUsdWeighted,
      color: colors.coinflow,
    },
    {
      name: "LTH Coinflow",
      btc: components.lthCoinflowMedianPriceBtcWeighted,
      usd: components.lthCoinflowMedianPriceUsdWeighted,
      color: colors.coinflow,
    },
  ];

  return {
    name: "Components",
    tree: [
      {
        name: "Realized Price",
        tree: [
          {
            name: "All",
            title: "Realized Price",
            pattern: cohorts.realized.price.all,
            percentiles: components.realizedPrice,
            legend: "RP",
            color: colors.realized,
          },
          {
            name: "STH",
            title: "STH Realized Price",
            pattern: cohorts.realized.price.term.short,
            percentiles: components.sthRealizedPrice,
            legend: "STH RP",
            color: colors.realized,
          },
          {
            name: "LTH",
            title: "LTH Realized Price",
            pattern: cohorts.realized.price.term.long,
            percentiles: components.lthRealizedPrice,
            legend: "LTH RP",
            color: colors.realized,
          },
          {
            name: "<4M",
            title: "<4M Realized Price",
            pattern: rarityMeter.referencePrices.under4m,
            percentiles: components.under4mRealizedPrice,
            legend: "<4M RP",
            color: colors.realized,
          },
          {
            name: "<6M",
            title: "<6M Realized Price",
            pattern: rarityMeter.referencePrices.under6m,
            percentiles: components.under6mRealizedPrice,
            legend: "<6M RP",
            color: colors.realized,
          },
          {
            name: ">4M",
            title: ">4M Realized Price",
            pattern: rarityMeter.referencePrices.over4m,
            percentiles: components.over4mRealizedPrice,
            legend: ">4M RP",
            color: colors.realized,
          },
          {
            name: ">6M",
            title: ">6M Realized Price",
            pattern: rarityMeter.referencePrices.over6m,
            percentiles: components.over6mRealizedPrice,
            legend: ">6M RP",
            color: colors.realized,
          },
        ].map(componentChart),
      },
      {
        name: "Capitalized Price",
        tree: [
          {
            name: "All",
            title: "Capitalized Price",
            pattern: cohorts.realized.capitalizedPrice.all,
            percentiles: components.capitalizedPrice,
            legend: "CP",
            color: colors.capitalized,
          },
          {
            name: "STH",
            title: "STH Capitalized Price",
            pattern: cohorts.realized.capitalizedPrice.sth,
            percentiles: components.sthCapitalizedPrice,
            legend: "STH CP",
            color: colors.capitalized,
          },
          {
            name: "LTH",
            title: "LTH Capitalized Price",
            pattern: cohorts.realized.capitalizedPrice.lth,
            percentiles: components.lthCapitalizedPrice,
            legend: "LTH CP",
            color: colors.capitalized,
          },
          {
            name: "<4M",
            title: "<4M Capitalized Price",
            pattern: rarityMeter.referencePrices.under4mCapitalizedPrice,
            percentiles: components.under4mCapitalizedPrice,
            legend: "<4M CP",
            color: colors.capitalized,
          },
          {
            name: "<6M",
            title: "<6M Capitalized Price",
            pattern: rarityMeter.referencePrices.under6mCapitalizedPrice,
            percentiles: components.under6mCapitalizedPrice,
            legend: "<6M CP",
            color: colors.capitalized,
          },
        ].map(componentChart),
      },
      {
        name: "Median Price",
        tree: /** @type {const} */ ([
          { name: "BTC Weighted", key: "btc", color: colors.realized },
          { name: "USD Weighted", key: "usd", color: colors.capitalized },
        ]).map((weight) => ({
          name: weight.name,
          tree: medians.map((median) => {
            const legend =
              median.name === "All" ? "Median" : `${median.name} Median`;
            const pattern = median[weight.key];
            return componentChart({
              name: median.name,
              title: `${legend} Price (${weight.name})`,
              pattern,
              percentiles: pattern,
              legend,
              color: median.color ?? weight.color,
            });
          }),
        })),
      },
      {
        name: "Cointime",
        tree: [
          {
            name: "Price",
            title: "Cointime Price",
            pattern: cointime.prices.cointime,
            percentiles: components.cointimePrice,
            legend: "Cointime",
            color: colors.cointime,
          },
          {
            name: "Awake",
            title: "Awake Price",
            pattern: cointime.awake.price,
            percentiles: components.awakePrice,
            legend: "Awake",
            color: colors.awake,
          },
          {
            name: "Active",
            title: "Active Price",
            pattern: cointime.prices.active,
            percentiles: components.activePrice,
            legend: "Active",
            color: colors.active,
          },
          {
            name: "Vaulted",
            title: "Vaulted Price",
            pattern: cointime.prices.vaulted,
            percentiles: components.vaultedPrice,
            legend: "Vaulted",
            color: colors.vaulted,
          },
          {
            name: "True Market Mean",
            title: "True Market Mean",
            pattern: cointime.prices.trueMarketMean,
            percentiles: components.trueMarketMeanPrice,
            legend: "True Market Mean",
            color: colors.trueMarketMean,
          },
        ].map(componentChart),
      },
      componentChart({
        name: "Coinflow Price",
        title: "Coinflow Price",
        pattern: coinflow.price,
        percentiles: components.coinflowPrice,
        legend: "Coinflow",
        color: colors.coinflow,
      }),
    ],
  };
}
