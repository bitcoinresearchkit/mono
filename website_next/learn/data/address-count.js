import {
  createCohortSeries,
  createCohortSeriesFromKeys,
} from "./cohort-series.js";
import { addressableTypes, amountRanges } from "./groups.js";
import { createRollingWindowSeries } from "./rolling-windows.js";
import { colors } from "../../utils/colors.js";

export const fundedSeries = createCohortSeries([
  {
    label: "Funded",
    color: colors.orange,
    metric: (client) => client.series.addrs.funded.all,
  },
]);

export const newSeries = createRollingWindowSeries(
  (key) => (client) => client.series.addrs.new.all.sum[key],
);

export const changeSeries = createRollingWindowSeries(
  (key) => (client) => client.series.addrs.delta.all.absolute[key],
);

export const growthRateSeries = createRollingWindowSeries(
  (key) => (client) => client.series.addrs.delta.all.rate[key].percent,
);

export const activeSeries = createRollingWindowSeries(
  (key) => (client) => client.series.addrs.activity.all.active[key],
);

export const sendingSeries = createRollingWindowSeries(
  (key) => (client) => client.series.addrs.activity.all.sending[key],
);

export const receivingSeries = createRollingWindowSeries(
  (key) => (client) => client.series.addrs.activity.all.receiving[key],
);

export const bidirectionalSeries = createRollingWindowSeries(
  (key) => (client) => client.series.addrs.activity.all.bidirectional[key],
);

export const reactivatedSeries = createRollingWindowSeries(
  (key) => (client) => client.series.addrs.activity.all.reactivated[key],
);

export const stateSeries = createCohortSeries([
  {
    label: "Funded",
    color: colors.green,
    metric: (client) => client.series.addrs.funded.all,
  },
  {
    label: "Empty",
    color: colors.red,
    metric: (client) => client.series.addrs.empty.all,
  },
  {
    label: "Total",
    color: colors.orange,
    metric: (client) => client.series.addrs.total.all,
  },
]);

export const balanceSeries = createCohortSeriesFromKeys(
  amountRanges,
  (key) => (client) => client.series.addrs.funded.balance.range[key].base,
);

export const typeSeries = createCohortSeriesFromKeys(
  addressableTypes,
  (key) => (client) => client.series.addrs.funded[key],
);

export const reuseSeries = createCohortSeries([
  {
    label: "Reused",
    color: colors.yellow,
    metric: (client) => client.series.addrs.reused.count.funded.all,
  },
  {
    label: "Respent",
    color: colors.fuchsia,
    metric: (client) => client.series.addrs.respent.count.funded.all,
  },
  {
    label: "Exposed",
    color: colors.orange,
    metric: (client) => client.series.addrs.exposed.count.funded.all,
  },
]);
