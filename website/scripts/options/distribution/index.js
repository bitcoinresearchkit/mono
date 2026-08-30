/**
 * Cohort module - exports all cohort-related functionality
 *
 * Folder builders compose sections from building blocks:
 * - holdings.js: Supply, UTXO Count, Address Count
 * - valuation.js: Realized Cap, Market Cap, MVRV
 * - prices.js: Realized Price, ratios
 * - cost-basis.js: Cost basis percentiles
 * - profitability.js: Unrealized/Realized P&L, Invested Capital
 * - activity.js: SOPR, Volume, Lifespan
 */

import {
  formatCohortTitle,
  amountBaseline,
  satsBtcUsd,
  satsBtcUsdFullTree,
  avgHoldingsSubtree,
  exposedSubtree,
  reusedSubtree,
} from "../shared.js";
import {
  ROLLING_WINDOWS,
  line,
  percentRatio,
  amountSumsTreeBaseline,
  rollingPercentRatioTree,
} from "../series.js";
import { Unit } from "../../utils/units.js";
import { colors } from "../../utils/colors.js";
import { bitview } from "../../utils/client.js";
import { lazyGroup } from "../lazy.js";

// Section builders
import {
  createHoldingsSection,
  createHoldingsSectionAll,
  createHoldingsSectionAddress,
  createHoldingsSectionAddressAmount,
  createHoldingsSectionWithProfitLoss,
  createHoldingsSectionWithRelative,
  createHoldingsSectionWithOwnSupply,
  createGroupedHoldingsSection,
  createGroupedHoldingsSectionAddress,
  createGroupedHoldingsSectionAddressAmount,
  createGroupedHoldingsSectionWithRelative,
  createGroupedHoldingsSectionWithOwnSupply,
} from "./holdings.js";
import {
  createValuationSection,
  createValuationSectionBase,
  createValuationSectionFull,
  createGroupedValuationSection,
  createGroupedValuationSectionBase,
  createGroupedValuationSectionWithOwnMarketCap,
} from "./valuation.js";
import {
  createPricesSectionFull,
  createPricesSectionBasic,
  createGroupedPricesSection,
  createGroupedPricesSectionFull,
} from "./prices.js";
import {
  createCostBasisSectionWithPercentiles,
  createGroupedCostBasisSectionWithPercentiles,
} from "./cost-basis.js";
import {
  createProfitabilitySection,
  createProfitabilitySectionRealized,
  createProfitabilitySectionAll,
  createProfitabilitySectionFull,
  createProfitabilitySectionWithProfitLoss,
  createProfitabilitySectionWithInvestedCapitalPct,
  createProfitabilitySectionLongTerm,
  createGroupedProfitabilitySection,
  createGroupedProfitabilitySectionRealized,
  createGroupedProfitabilitySectionWithProfitLoss,
  createGroupedProfitabilitySectionWithNupl,
  createGroupedProfitabilitySectionWithInvestedCapitalPct,
} from "./profitability.js";
import {
  createActivitySection,
  createActivitySectionWithAdjusted,
  createActivitySectionWithActivity,
  createGroupedActivitySection,
  createGroupedActivitySectionWithActivity,
  createActivitySectionMinimal,
  createGroupedActivitySectionMinimal,
} from "./activity.js";

// Re-export data builder
export { buildCohortData } from "./data.js";

// ============================================================================
// Single Cohort Folder Builders
// ============================================================================

/**
 * All folder: for the special "All" cohort
 * @param {CohortAll} cohort
 * @returns {PartialOptionsGroup}
 */
export function createCohortFolderAll(cohort) {
  const title = formatCohortTitle(cohort.title);
  return {
    name: cohort.name || "all",
    tree: [
      ...createHoldingsSectionAll({ cohort, title }),
      lazyGroup("Capitalization", () =>
        createValuationSectionFull({ cohort, title }),
      ),
      lazyGroup("Prices", () => createPricesSectionFull({ cohort, title })),
      lazyGroup("Cost Basis", () =>
        createCostBasisSectionWithPercentiles({ cohort, title }),
      ),
      lazyGroup("Profitability", () =>
        createProfitabilitySectionAll({ cohort, title }),
      ),
      lazyGroup("Activity", () =>
        createActivitySectionWithAdjusted({ cohort, title }),
      ),
      lazyGroup("Average Holdings", () =>
        avgHoldingsSubtree(cohort.avgAmount, title),
      ),
    ],
  };
}

/**
 * Full folder: adjustedSopr + percentiles + RelToMarketCap
 * @param {CohortFull} cohort
 * @returns {PartialOptionsGroup}
 */
export function createCohortFolderFull(cohort) {
  const title = formatCohortTitle(cohort.title);
  return {
    name: cohort.name || "all",
    tree: [
      ...createHoldingsSectionWithRelative({ cohort, title }),
      lazyGroup("Capitalization", () =>
        createValuationSectionFull({ cohort, title }),
      ),
      lazyGroup("Prices", () => createPricesSectionFull({ cohort, title })),
      lazyGroup("Cost Basis", () =>
        createCostBasisSectionWithPercentiles({ cohort, title }),
      ),
      lazyGroup("Profitability", () =>
        createProfitabilitySectionFull({ cohort, title }),
      ),
      lazyGroup("Activity", () =>
        createActivitySectionWithAdjusted({ cohort, title }),
      ),
    ],
  };
}

/**
 * Core cohort folder.
 * @param {CohortCore} cohort
 * @returns {PartialOptionsGroup}
 */
export function createCohortFolderCore(cohort) {
  const title = formatCohortTitle(cohort.title);
  return {
    name: cohort.name || "all",
    tree: [
      ...createHoldingsSectionWithOwnSupply({ cohort, title }),
      lazyGroup("Capitalization", () =>
        createValuationSection({ cohort, title }),
      ),
      lazyGroup("Prices", () => createPricesSectionBasic({ cohort, title })),
      lazyGroup("Profitability", () =>
        createProfitabilitySectionWithInvestedCapitalPct({ cohort, title }),
      ),
      lazyGroup("Activity", () =>
        createActivitySectionWithActivity({ cohort, title }),
      ),
    ],
  };
}

/**
 * LongTerm folder: has own market cap + NUPL + peak regret + P/L ratio
 * @param {CohortLongTerm} cohort
 * @returns {PartialOptionsGroup}
 */
export function createCohortFolderLongTerm(cohort) {
  const title = formatCohortTitle(cohort.title);
  return {
    name: cohort.name || "all",
    tree: [
      ...createHoldingsSectionWithRelative({ cohort, title }),
      lazyGroup("Capitalization", () =>
        createValuationSectionFull({ cohort, title }),
      ),
      lazyGroup("Prices", () => createPricesSectionFull({ cohort, title })),
      lazyGroup("Cost Basis", () =>
        createCostBasisSectionWithPercentiles({ cohort, title }),
      ),
      lazyGroup("Profitability", () =>
        createProfitabilitySectionLongTerm({ cohort, title }),
      ),
      lazyGroup("Activity", () => createActivitySection({ cohort, title })),
    ],
  };
}

/**
 * Age range folder: no nupl
 * @param {CohortAgeRange} cohort
 * @returns {PartialOptionsGroup}
 */
export function createCohortFolderAgeRange(cohort) {
  const title = formatCohortTitle(cohort.title);
  return {
    name: cohort.name || "all",
    tree: [
      ...createHoldingsSectionWithOwnSupply({ cohort, title }),
      lazyGroup("Capitalization", () =>
        createValuationSection({ cohort, title }),
      ),
      lazyGroup("Prices", () => createPricesSectionBasic({ cohort, title })),
      lazyGroup("Profitability", () =>
        createProfitabilitySectionWithInvestedCapitalPct({ cohort, title }),
      ),
      lazyGroup("Activity", () =>
        createActivitySectionWithActivity({ cohort, title }),
      ),
    ],
  };
}

/**
 * Age range folder with matured supply
 * @param {CohortAgeRangeWithMatured} cohort
 * @returns {PartialOptionsGroup}
 */
export function createCohortFolderAgeRangeWithMatured(cohort) {
  const folder = createCohortFolderAgeRange(cohort);
  const title = formatCohortTitle(cohort.title);
  folder.tree.push(
    lazyGroup("Matured", () => ({
      name: "Matured",
      tree: satsBtcUsdFullTree({
        pattern: cohort.matured,
        title,
        metric: "Matured Supply",
      }),
    })),
  );
  return folder;
}

/**
 * Basic folder WITH RelToMarketCap
 * @param {CohortBasicWithMarketCap} cohort
 * @returns {PartialOptionsGroup}
 */
export function createCohortFolderBasicWithMarketCap(cohort) {
  const title = formatCohortTitle(cohort.title);
  return {
    name: cohort.name || "all",
    tree: [
      ...createHoldingsSection({ cohort, title }),
      lazyGroup("Capitalization", () =>
        createValuationSection({ cohort, title }),
      ),
      lazyGroup("Prices", () => createPricesSectionBasic({ cohort, title })),
      lazyGroup("Profitability", () =>
        createProfitabilitySection({ cohort, title }),
      ),
      lazyGroup("Activity", () =>
        createActivitySectionMinimal({ cohort, title }),
      ),
    ],
  };
}

/**
 * Address folder: like basic but with address count
 * @param {CohortAddr} cohort
 * @returns {PartialOptionsGroup}
 */
export function createCohortFolderAddress(cohort) {
  const title = formatCohortTitle(cohort.title);
  return {
    name: cohort.name || "all",
    tree: [
      ...createHoldingsSectionAddress({ cohort, title }),
      lazyGroup("Capitalization", () =>
        createValuationSection({ cohort, title }),
      ),
      lazyGroup("Prices", () => createPricesSectionBasic({ cohort, title })),
      lazyGroup("Profitability", () =>
        createProfitabilitySectionWithProfitLoss({ cohort, title }),
      ),
      lazyGroup("Activity", () =>
        createActivitySectionMinimal({ cohort, title }),
      ),
      lazyGroup("Average Holdings", () =>
        avgHoldingsSubtree(cohort.avgAmount, title),
      ),
      lazyGroup("Reused", () =>
        reusedSubtree(cohort.reused, cohort.respent, cohort.key, title),
      ),
      lazyGroup("Exposed", () =>
        exposedSubtree(cohort.exposed, cohort.key, title),
      ),
    ],
  };
}

/**
 * Folder for cohorts WITHOUT relative section
 * @param {CohortWithoutRelative} cohort
 * @returns {PartialOptionsGroup}
 */
export function createCohortFolderWithoutRelative(cohort) {
  const title = formatCohortTitle(cohort.title);
  return {
    name: cohort.name || "all",
    tree: [
      ...createHoldingsSectionWithProfitLoss({ cohort, title }),
      lazyGroup("Capitalization", () =>
        createValuationSection({ cohort, title }),
      ),
      lazyGroup("Prices", () => createPricesSectionBasic({ cohort, title })),
      lazyGroup("Profitability", () =>
        createProfitabilitySectionWithProfitLoss({ cohort, title }),
      ),
      lazyGroup("Activity", () =>
        createActivitySectionMinimal({ cohort, title }),
      ),
    ],
  };
}

/**
 * Address-balance cohort folder.
 * @param {AddrCohortObject} cohort
 * @returns {PartialOptionsGroup}
 */
export function createAddressCohortFolder(cohort) {
  const title = formatCohortTitle(cohort.title);
  return {
    name: cohort.name || "all",
    tree: [
      ...createHoldingsSectionAddressAmount({ cohort, title }),
      lazyGroup("Capitalization", () =>
        createValuationSectionBase({ cohort, title }),
      ),
      lazyGroup("Profitability", () =>
        createProfitabilitySectionRealized({ cohort, title }),
      ),
      lazyGroup("Activity", () =>
        createActivitySectionMinimal({ cohort, title }),
      ),
    ],
  };
}

// ============================================================================
// Grouped Cohort Folder Builders
// ============================================================================

/**
 * @param {CohortGroupCore} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedCohortFolderCore({
  name,
  title: groupTitle,
  list,
  all,
}) {
  const title = formatCohortTitle(groupTitle);
  return {
    name: name || "all",
    tree: [
      ...createGroupedHoldingsSectionWithOwnSupply({ list, all, title }),
      lazyGroup("Capitalization", () =>
        createGroupedValuationSection({ list, all, title }),
      ),
      lazyGroup("Prices", () =>
        createGroupedPricesSection({ list, all, title }),
      ),
      lazyGroup("Profitability", () =>
        createGroupedProfitabilitySectionWithInvestedCapitalPct({
          list,
          all,
          title,
        }),
      ),
      lazyGroup("Activity", () =>
        createGroupedActivitySectionWithActivity({ list, all, title }),
      ),
    ],
  };
}

/**
 * @param {CohortGroupWithNuplPercentiles} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedCohortFolderWithNupl({
  name,
  title: groupTitle,
  list,
  all,
}) {
  const title = formatCohortTitle(groupTitle);
  return {
    name: name || "all",
    tree: [
      ...createGroupedHoldingsSectionWithRelative({ list, all, title }),
      lazyGroup("Capitalization", () =>
        createGroupedValuationSectionWithOwnMarketCap({ list, all, title }),
      ),
      lazyGroup("Prices", () =>
        createGroupedPricesSectionFull({ list, all, title }),
      ),
      lazyGroup("Cost Basis", () =>
        createGroupedCostBasisSectionWithPercentiles({ list, all, title }),
      ),
      lazyGroup("Profitability", () =>
        createGroupedProfitabilitySectionWithNupl({ list, all, title }),
      ),
      lazyGroup("Activity", () =>
        createGroupedActivitySection({ list, all, title }),
      ),
    ],
  };
}

/**
 * @param {CohortGroupAgeRange} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedCohortFolderAgeRange({
  name,
  title: groupTitle,
  list,
  all,
}) {
  const title = formatCohortTitle(groupTitle);
  return {
    name: name || "all",
    tree: [
      ...createGroupedHoldingsSectionWithOwnSupply({ list, all, title }),
      lazyGroup("Capitalization", () =>
        createGroupedValuationSection({ list, all, title }),
      ),
      lazyGroup("Prices", () =>
        createGroupedPricesSection({ list, all, title }),
      ),
      lazyGroup("Profitability", () =>
        createGroupedProfitabilitySectionWithInvestedCapitalPct({
          list,
          all,
          title,
        }),
      ),
      lazyGroup("Activity", () =>
        createGroupedActivitySectionWithActivity({ list, all, title }),
      ),
    ],
  };
}

/**
 * @param {{ name: string, title: string, list: readonly CohortAgeRangeWithMatured[], all: CohortAll }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedCohortFolderAgeRangeWithMatured({
  name,
  title: groupTitle,
  list,
  all,
}) {
  const folder = createGroupedCohortFolderAgeRange({
    name,
    title: groupTitle,
    list,
    all,
  });
  const title = formatCohortTitle(groupTitle);
  folder.tree.push(
    lazyGroup("Matured", () => ({
      name: "Matured",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} Matured Supply`),
        bottom: list.flatMap((cohort) =>
          satsBtcUsd({
            pattern: cohort.matured.sum[w.key],
            name: cohort.name,
            color: cohort.color,
          }),
        ),
      })),
    })),
  );
  return folder;
}

/**
 * @param {CohortGroupBasicWithMarketCap} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedCohortFolderBasicWithMarketCap({
  name,
  title: groupTitle,
  list,
  all,
}) {
  const title = formatCohortTitle(groupTitle);
  return {
    name: name || "all",
    tree: [
      ...createGroupedHoldingsSection({ list, all, title }),
      lazyGroup("Capitalization", () =>
        createGroupedValuationSection({ list, all, title }),
      ),
      lazyGroup("Prices", () =>
        createGroupedPricesSection({ list, all, title }),
      ),
      lazyGroup("Profitability", () =>
        createGroupedProfitabilitySection({ list, all, title }),
      ),
      lazyGroup("Activity", () =>
        createGroupedActivitySectionMinimal({ list, all, title }),
      ),
    ],
  };
}

/**
 * @param {CohortGroupAddr} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedCohortFolderAddress({
  name,
  title: groupTitle,
  list,
  all,
}) {
  const title = formatCohortTitle(groupTitle);
  return {
    name: name || "all",
    tree: [
      ...createGroupedHoldingsSectionAddress({ list, all, title }),
      lazyGroup("Capitalization", () =>
        createGroupedValuationSection({ list, all, title }),
      ),
      lazyGroup("Prices", () =>
        createGroupedPricesSection({ list, all, title }),
      ),
      lazyGroup("Profitability", () =>
        createGroupedProfitabilitySectionWithProfitLoss({
          list,
          all,
          title,
        }),
      ),
      lazyGroup("Activity", () =>
        createGroupedActivitySectionMinimal({ list, all, title }),
      ),
    ],
  };
}

/**
 * @param {AddrCohortGroupObject} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedAddressCohortFolder({
  name,
  title: groupTitle,
  list,
  all,
}) {
  const title = formatCohortTitle(groupTitle);
  return {
    name: name || "all",
    tree: [
      ...createGroupedHoldingsSectionAddressAmount({ list, all, title }),
      lazyGroup("Capitalization", () =>
        createGroupedValuationSectionBase({ list, all, title }),
      ),
      lazyGroup("Profitability", () =>
        createGroupedProfitabilitySectionRealized({ list, all, title }),
      ),
      lazyGroup("Activity", () =>
        createGroupedActivitySectionMinimal({ list, all, title }),
      ),
    ],
  };
}

// ============================================================================
// UTXO Profitability Folder Builders
// ============================================================================

/**
 * @param {{ name: string, color: Color, pattern: RealizedSupplyPattern }} bucket
 * @param {string} [parentName]
 * @returns {PartialOptionsGroup}
 */
function singleBucketFolder({ name, color, pattern }, parentName) {
  const title = formatCohortTitle(parentName ? `${parentName} ${name}` : name);
  return {
    name,
    tree: [
      {
        name: "Supply",
        tree: [
          {
            name: "Total",
            title: title("Supply"),
            bottom: [
              ...satsBtcUsd({ pattern: pattern.supply.all, name: "Total" }),
              ...satsBtcUsd({
                pattern: pattern.supply.sth,
                name: "STH",
                color: colors.term.short,
              }),
            ],
          },
          {
            ...amountSumsTreeBaseline({
              windows: pattern.supply.all.delta.absolute,
              title,
              metric: "Supply Change",
              legend: "Change",
            }),
            name: "Change",
          },
          {
            ...rollingPercentRatioTree({
              windows: pattern.supply.all.delta.rate,
              title,
              metric: "Supply Growth Rate",
            }),
            name: "Growth Rate",
          },
        ],
      },
      {
        name: "Realized Cap",
        title: title("Realized Cap"),
        bottom: [
          line({
            series: pattern.realizedCap.all,
            name: "Total",
            unit: Unit.usd,
          }),
          line({
            series: pattern.realizedCap.sth,
            name: "STH",
            color: colors.term.short,
            unit: Unit.usd,
          }),
        ],
      },
      {
        name: "Unrealized PnL",
        title: title("Unrealized PnL"),
        bottom: [
          line({
            series: pattern.unrealizedPnl.all,
            name: "Total",
            unit: Unit.usd,
          }),
          line({
            series: pattern.unrealizedPnl.sth,
            name: "STH",
            color: colors.term.short,
            unit: Unit.usd,
          }),
        ],
      },
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
 * @param {string} groupTitle
 * @returns {PartialOptionsTree}
 */
function groupedBucketCharts(list, groupTitle) {
  const title = formatCohortTitle(groupTitle);
  return [
    {
      name: "Supply",
      tree: [
        {
          name: "All",
          title: title("Supply"),
          bottom: list.flatMap(({ name, color, pattern }) =>
            satsBtcUsd({ pattern: pattern.supply.all, name, color }),
          ),
        },
        {
          name: "STH",
          title: title("STH Supply"),
          bottom: list.flatMap(({ name, color, pattern }) =>
            satsBtcUsd({ pattern: pattern.supply.sth, name, color }),
          ),
        },
        {
          name: "Change",
          tree: ROLLING_WINDOWS.map((w) => ({
            name: w.name,
            title: title(`${w.title} Supply Change`),
            bottom: list.flatMap(({ name, color, pattern }) =>
              amountBaseline({
                pattern: pattern.supply.all.delta.absolute[w.key],
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
                pattern: pattern.supply.all.delta.rate[w.key],
                name,
                color,
              }),
            ),
          })),
        },
      ],
    },
    {
      name: "Realized Cap",
      tree: [
        {
          name: "All",
          title: title("Realized Cap"),
          bottom: list.map(({ name, color, pattern }) =>
            line({
              series: pattern.realizedCap.all,
              name,
              color,
              unit: Unit.usd,
            }),
          ),
        },
        {
          name: "STH",
          title: title("STH Realized Cap"),
          bottom: list.map(({ name, color, pattern }) =>
            line({
              series: pattern.realizedCap.sth,
              name,
              color,
              unit: Unit.usd,
            }),
          ),
        },
      ],
    },
    {
      name: "Unrealized PnL",
      tree: [
        {
          name: "All",
          title: title("Unrealized PnL"),
          bottom: list.map(({ name, color, pattern }) =>
            line({
              series: pattern.unrealizedPnl.all,
              name,
              color,
              unit: Unit.usd,
            }),
          ),
        },
        {
          name: "STH",
          title: title("STH Unrealized PnL"),
          bottom: list.map(({ name, color, pattern }) =>
            line({
              series: pattern.unrealizedPnl.sth,
              name,
              color,
              unit: Unit.usd,
            }),
          ),
        },
      ],
    },
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
 * @param {{ range: { name: string, color: Color, pattern: RealizedSupplyPattern }[], profit: { name: string, color: Color, pattern: RealizedSupplyPattern }[], loss: { name: string, color: Color, pattern: RealizedSupplyPattern }[] }} args
 * @returns {PartialOptionsGroup}
 */
export function createUtxoProfitabilitySection({ range, profit, loss }) {
  return {
    name: "UTXO Profitability",
    tree: [
      {
        name: "Range",
        tree: [
          {
            name: "Compare",
            tree: groupedBucketCharts(range, "Profitability Range"),
          },
          ...range.map((bucket) => singleBucketFolder(bucket)),
        ],
      },
      {
        name: "In Profit",
        tree: [
          { name: "Compare", tree: groupedBucketCharts(profit, "In Profit") },
          ...profit.map((bucket) => singleBucketFolder(bucket, "In Profit")),
        ],
      },
      {
        name: "In Loss",
        tree: [
          { name: "Compare", tree: groupedBucketCharts(loss, "In Loss") },
          ...loss.map((bucket) => singleBucketFolder(bucket, "In Loss")),
        ],
      },
    ],
  };
}

/**
 * Gini leaf for Distribution > Address Balance
 * @returns {AnyPartialOption}
 */
export function createAddressBalanceGiniLeaf() {
  return {
    name: "Gini",
    title: "Address Balance Gini Coefficient",
    bottom: percentRatio({
      pattern: bitview.series.indicators.gini,
      name: "Gini",
      color: colors.loss,
    }),
  };
}
