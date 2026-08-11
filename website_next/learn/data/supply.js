import { createCohortSeries } from "./cohort-series.js";
import { colors } from "../../utils/colors.js";

export const circulatingSupplySeries = createCohortSeries([
  {
    label: "Circulating",
    color: colors.orange,
    metric: (client) => client.series.supply.circulating.btc,
  },
]);

export const supplyProfitabilitySeries = createCohortSeries([
  {
    label: "In profit",
    color: colors.green,
    metric: (client) =>
      client.series.cohorts.utxo.profitability.supply.profit.all.all.btc,
  },
  {
    label: "In loss",
    color: colors.red,
    metric: (client) =>
      client.series.cohorts.utxo.profitability.supply.loss.all.all.btc,
  },
]);
