import { bitview } from "../../../utils/client.js";
import { colors } from "../../../utils/colors.js";
import { percentRatio } from "../../series.js";

export function createSupplyDensityFolders() {
  const basis = bitview.series.bedrock.costBasis;
  const modes = /** @type {const} */ (["cointime", "coinflow"]);
  const ages = /** @type {const} */ ([
    { key: "under4m", name: "<4m" },
    { key: "under5m", name: "STH (<5m)" },
    { key: "under6m", name: "<6m" },
  ]);
  const bands = /** @type {const} */ ([
    { key: "supplyDensity", band: 5, name: "Supply Density" },
    { key: "supplyDensity10pct", band: 10, name: "Supply Density (±10%)" },
  ]);
  return bands.map(({ key, band, name }) => ({
    name,
    tree: [
      { name: "All", densities: basis[key] },
      ...ages.map(age => ({
        name: age.name,
        densities: {
          cointime: basis.ageDensity[age.key].cointime[key],
          coinflow: basis.ageDensity[age.key].coinflow[key],
        },
      })),
    ].map(({ name: cohort, densities }) => ({
      name: cohort,
      tree: modes.map(mode => {
        const name = mode === "cointime" ? "Cointime" : "Coinflow";
        const density = densities[mode];
        return {
          name,
          title: `Bitcoin ${cohort} ${name}-Weighted Supply Density (±${band}%)`,
          bottom: [
            ...percentRatio({
              pattern: density.total,
              name: `Total (±${band}%)`,
              color: colors.bitcoin,
            }),
            ...percentRatio({
              pattern: density.inProfit,
              name: `In Profit (−${band}% to Spot)`,
              color: colors.profit,
            }),
            ...percentRatio({
              pattern: density.inLoss,
              name: `In Loss (Spot to +${band}%)`,
              color: colors.loss,
            }),
          ],
        };
      }),
    })),
  }));
}
