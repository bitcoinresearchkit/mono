import {
  createCohortSeries,
  createCohortSeriesFromKeys,
} from "./cohort-series.js";
import {
  ageRanges,
  amountRanges,
  classes,
  epochs,
  spendableTypes,
} from "./groups.js";
import { colors } from "../../utils/colors.js";

export const capitalizationSeries = createCohortSeries([
  {
    label: "Market cap",
    color: colors.green,
    metric: (client) => client.series.supply.marketCap.usd,
  },
  {
    label: "Realized cap",
    color: colors.orange,
    metric: (client) => client.series.cohorts.utxo.all.realized.cap.usd,
  },
]);

const [marketCap, realizedCap] = capitalizationSeries;

export const marketCapSeries = [marketCap];

export const realizedCapSeries = [realizedCap];

export const marketCapProfitabilitySeries = createCohortSeries([
  {
    label: "In profit",
    color: colors.green,
    metric: (client) =>
      client.series.cohorts.utxo.profitability.supply.profit.all.all.usd,
  },
  {
    label: "In loss",
    color: colors.red,
    metric: (client) =>
      client.series.cohorts.utxo.profitability.supply.loss.all.all.usd,
  },
]);

export const realizedCapProfitabilitySeries = createCohortSeries([
  {
    label: "In profit",
    color: colors.green,
    metric: (client) =>
      client.series.cohorts.utxo.profitability.realizedCap.profit.all.all,
  },
  {
    label: "In loss",
    color: colors.red,
    metric: (client) =>
      client.series.cohorts.utxo.profitability.realizedCap.loss.all.all,
  },
]);

export const marketCapTermSeries = createCohortSeries([
  {
    label: "STH",
    color: colors.yellow,
    metric: (client) => client.series.cohorts.utxo.sth.supply.total.usd,
  },
  {
    label: "LTH",
    color: colors.fuchsia,
    metric: (client) => client.series.cohorts.utxo.lth.supply.total.usd,
  },
]);

export const realizedCapTermSeries = createCohortSeries([
  {
    label: "STH",
    color: colors.yellow,
    metric: (client) => client.series.cohorts.utxo.sth.realized.cap.usd,
  },
  {
    label: "LTH",
    color: colors.fuchsia,
    metric: (client) => client.series.cohorts.utxo.lth.realized.cap.usd,
  },
]);

export const marketCapAgeSeries = createCohortSeriesFromKeys(
  ageRanges,
  (key) => (client) =>
    client.series.cohorts.utxo.ageRange[key].supply.total.usd,
);

export const realizedCapAgeSeries = createCohortSeriesFromKeys(
  ageRanges,
  (key) => (client) =>
    client.series.cohorts.utxo.ageRange[key].realized.cap.usd,
);

export const marketCapUtxoBalanceSeries = createCohortSeriesFromKeys(
  amountRanges,
  (key) => (client) =>
    client.series.cohorts.utxo.amountRange[key].supply.total.usd,
);

export const realizedCapUtxoBalanceSeries = createCohortSeriesFromKeys(
  amountRanges,
  (key) => (client) =>
    client.series.cohorts.utxo.amountRange[key].realized.cap.usd,
);

export const marketCapAddressBalanceSeries = createCohortSeriesFromKeys(
  amountRanges,
  (key) => (client) =>
    client.series.cohorts.addr.amountRange[key].supply.total.usd,
);

export const realizedCapAddressBalanceSeries = createCohortSeriesFromKeys(
  amountRanges,
  (key) => (client) =>
    client.series.cohorts.addr.amountRange[key].realized.cap.usd,
);

export const marketCapTypeSeries = createCohortSeriesFromKeys(
  spendableTypes,
  (key) => (client) => client.series.cohorts.utxo.type[key].supply.total.usd,
);

export const realizedCapTypeSeries = createCohortSeriesFromKeys(
  spendableTypes,
  (key) => (client) => client.series.cohorts.utxo.type[key].realized.cap.usd,
);

export const marketCapEpochSeries = createCohortSeriesFromKeys(
  epochs,
  (key) => (client) => client.series.cohorts.utxo.epoch[key].supply.total.usd,
);

export const realizedCapEpochSeries = createCohortSeriesFromKeys(
  epochs,
  (key) => (client) => client.series.cohorts.utxo.epoch[key].realized.cap.usd,
);

export const marketCapClassSeries = createCohortSeriesFromKeys(
  classes,
  (key) => (client) => client.series.cohorts.utxo.class[key].supply.total.usd,
);

export const realizedCapClassSeries = createCohortSeriesFromKeys(
  classes,
  (key) => (client) => client.series.cohorts.utxo.class[key].realized.cap.usd,
);
