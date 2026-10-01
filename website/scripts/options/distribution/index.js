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
  satsBtcUsd,
  satsBtcUsdFullTree,
  avgHoldingsSubtree,
  exposedSubtree,
  reusedSubtree,
} from "../shared.js";
import { ROLLING_WINDOWS, percentRatio } from "../series.js";
import { colors } from "../../utils/colors.js";
import { bitview } from "../../utils/client.js";
import { lazyGroup } from "../lazy.js";

// Section builders
import {
  createHoldingsSection,
  createHoldingsSectionAll,
  createHoldingsSectionAddress,
  createHoldingsSectionAddressAmount,
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
  createProfitabilitySectionWithInvestedCapitalPct,
  createProfitabilitySectionLongTerm,
  createGroupedProfitabilitySection,
  createGroupedProfitabilitySectionRealized,
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
        createCostBasisSectionWithPercentiles({ cohort, title, ageBounds: bitview.series.cohorts.urpd.ageBounds }),
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
        createProfitabilitySection({ cohort, title }),
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
        createGroupedProfitabilitySection({ list, all, title }),
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

export { createUtxoProfitabilitySection } from "./profitability-buckets.js";

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
