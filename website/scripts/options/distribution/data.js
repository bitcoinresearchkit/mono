import { colors } from "../../utils/colors.js";
import { entries } from "../../utils/array.js";
import { brk } from "../../utils/client.js";
import { ageRanges } from "../age-ranges.js";
import { selectCohortTree } from "./cohort-tree.js";

/** @type {readonly AddressableType[]} */
const ADDRESSABLE_TYPES = [
  "p2a",
  "p2tr",
  "p2wsh",
  "p2wpkh",
  "p2sh",
  "p2pkh",
  "p2pk33",
  "p2pk65",
];

/**
 * @param {SpendableType} key
 * @returns {key is AddressableType}
 */
function isAddressable(key) {
  return /** @type {readonly string[]} */ (ADDRESSABLE_TYPES).includes(key);
}

export function buildCohortData() {
  const cohorts = brk.series.cohorts;
  const { addrs } = brk.series;
  const {
    TERM_NAMES,
    EPOCH_NAMES,
    UNDER_AGE_NAMES,
    OVER_AGE_NAMES,
    OVER_AMOUNT_NAMES,
    UNDER_AMOUNT_NAMES,
    AMOUNT_RANGE_NAMES,
    SPENDABLE_TYPE_NAMES,
    CLASS_NAMES,
    ENTRY_NAMES,
    PROFITABILITY_RANGE_NAMES,
    PROFIT_NAMES,
    LOSS_NAMES,
  } = brk;

  const cohortAll = {
    name: "",
    title: "",
    color: colors.bitcoin,
    tree: selectCohortTree({ tree: cohorts, path: "all" }),
    addressCount: {
      base: addrs.funded.all,
      delta: addrs.delta.all,
    },
    avgAmount: {
      utxo: addrs.avgAmount.utxo.all,
      addr: addrs.avgAmount.addr.all,
    },
  };

  const shortNames = TERM_NAMES.short;
  const termShort = {
    name: shortNames.short,
    title: shortNames.long,
    color: colors.term.short,
    tree: selectCohortTree({ tree: cohorts, path: "term.short" }),
  };

  const longNames = TERM_NAMES.long;
  const termLong = {
    name: longNames.short,
    title: longNames.long,
    color: colors.term.long,
    tree: selectCohortTree({ tree: cohorts, path: "term.long" }),
  };

  // Under age cohorts
  const underAge = entries(UNDER_AGE_NAMES).map(([key, names], i, arr) => ({
    name: names.short,
    title: `UTXOs ${names.long}`,
    color: colors.at(i, arr.length),
    tree: selectCohortTree({ tree: cohorts, path: `age.under.${key}` }),
  }));

  // Over age cohorts
  const overAge = entries(OVER_AGE_NAMES).map(([key, names], i, arr) => ({
    name: names.short,
    title: `UTXOs ${names.long}`,
    color: colors.at(i, arr.length),
    tree: selectCohortTree({ tree: cohorts, path: `age.over.${key}` }),
  }));

  const ageRange = ageRanges.map(({ key, ...range }) => ({
    ...range,
    tree: selectCohortTree({ tree: cohorts, path: `age.range.${key}` }),
    matured: cohorts.supply.matured[key],
  }));

  const epoch = entries(EPOCH_NAMES).map(([key, names], i, arr) => ({
    name: names.short,
    title: names.long,
    color: colors.at(i, arr.length),
    tree: selectCohortTree({ tree: cohorts, path: `epoch.${key}` }),
  }));

  const utxosOverAmount = entries(OVER_AMOUNT_NAMES).map(
    ([key, names], i, arr) => ({
      name: names.short,
      title: `UTXOs ${names.long}`,
      color: colors.at(i, arr.length),
      tree: selectCohortTree({ tree: cohorts, path: `utxoAmount.over.${key}` }),
    }),
  );

  const addressesOverAmount = entries(OVER_AMOUNT_NAMES).map(
    ([key, names], i, arr) => {
      const cohort = selectCohortTree({
        tree: cohorts,
        path: `addrBalance.over.${key}`,
      });
      return {
        name: names.short,
        title: `Addresses ${names.long}`,
        color: colors.at(i, arr.length),
        tree: cohort,
        addressCount: addrs.funded.balance.over[key],
      };
    },
  );

  const utxosUnderAmount = entries(UNDER_AMOUNT_NAMES).map(
    ([key, names], i, arr) => ({
      name: names.short,
      title: `UTXOs ${names.long}`,
      color: colors.at(i, arr.length),
      tree: selectCohortTree({ tree: cohorts, path: `utxoAmount.under.${key}` }),
    }),
  );

  const addressesUnderAmount = entries(UNDER_AMOUNT_NAMES).map(
    ([key, names], i, arr) => {
      const cohort = selectCohortTree({
        tree: cohorts,
        path: `addrBalance.under.${key}`,
      });
      return {
        name: names.short,
        title: `Addresses ${names.long}`,
        color: colors.at(i, arr.length),
        tree: cohort,
        addressCount: addrs.funded.balance.under[key],
      };
    },
  );

  const utxosAmountRange = entries(AMOUNT_RANGE_NAMES).map(
    ([key, names], i, arr) => ({
      name: names.short,
      title: `UTXOs ${names.long}`,
      color: colors.at(i, arr.length),
      tree: selectCohortTree({ tree: cohorts, path: `utxoAmount.range.${key}` }),
    }),
  );

  const addressesAmountRange = entries(AMOUNT_RANGE_NAMES).map(
    ([key, names], i, arr) => {
      const cohort = selectCohortTree({
        tree: cohorts,
        path: `addrBalance.range.${key}`,
      });
      return {
        name: names.short,
        title: `Addresses ${names.long}`,
        color: colors.at(i, arr.length),
        tree: cohort,
        addressCount: addrs.funded.balance.range[key],
      };
    },
  );

  const typeAddressable = ADDRESSABLE_TYPES.map((key) => {
    const names = SPENDABLE_TYPE_NAMES[key];
    return {
      key,
      name: names.short,
      title: names.short,
      color: colors.scriptType[key],
      tree: selectCohortTree({ tree: cohorts, path: `type.${key}` }),
      addressCount: {
        base: addrs.funded[key],
        delta: addrs.delta[key],
      },
      avgAmount: {
        utxo: addrs.avgAmount.utxo[key],
        addr: addrs.avgAmount.addr[key],
      },
      exposed: addrs.exposed,
      reused: addrs.reused,
      respent: addrs.respent,
    };
  });

  const typeOther = entries(SPENDABLE_TYPE_NAMES)
    .filter(([key]) => !isAddressable(key))
    .map(([key, names]) => ({
      key,
      name: names.short,
      title: names.short,
      color: colors.scriptType[key],
      tree: selectCohortTree({ tree: cohorts, path: `type.${key}` }),
    }));

  const class_ = entries(CLASS_NAMES)
    .reverse()
    .map(([key, names], i, arr) => ({
      name: names.short,
      title: names.long,
      color: colors.at(i, arr.length),
      tree: selectCohortTree({ tree: cohorts, path: `class.${key}` }),
    }));

  const entryColors = {
    discount: colors.arr[11],
    premium: colors.arr[0],
  };

  const entry = entries(ENTRY_NAMES).map(([key, names]) => ({
    name: names.short,
    title: `UTXOs ${names.long}`,
    color: entryColors[key],
    tree: selectCohortTree({ tree: cohorts, path: `entry.${key}` }),
  }));

  const profitability = cohorts.profitability;
  /** @param {keyof typeof profitability.supply.range} key */
  const profitabilityRangePattern = (key) => ({
    supply: profitability.supply.range[key],
    realizedCap: profitability.realizedCap.range[key],
    unrealizedPnl: profitability.unrealizedPnl.range[key],
    nupl: profitability.nupl.range[key],
  });

  /** @param {keyof typeof profitability.supply.profit} key */
  const profitabilityProfitPattern = (key) => ({
    supply: profitability.supply.profit[key],
    realizedCap: profitability.realizedCap.profit[key],
    unrealizedPnl: profitability.unrealizedPnl.profit[key],
    nupl: profitability.nupl.profit[key],
  });

  /** @param {keyof typeof profitability.supply.loss} key */
  const profitabilityLossPattern = (key) => ({
    supply: profitability.supply.loss[key],
    realizedCap: profitability.realizedCap.loss[key],
    unrealizedPnl: profitability.unrealizedPnl.loss[key],
    nupl: profitability.nupl.loss[key],
  });

  const profitabilityRange = entries(PROFITABILITY_RANGE_NAMES).map(
    ([key, names], i, arr) => ({
      name: names.short,
      color: colors.at(i, arr.length),
      pattern: profitabilityRangePattern(key),
    }),
  );

  const profitabilityProfit = entries(PROFIT_NAMES).map(
    ([key, names], i, arr) => ({
      name: names.short,
      color: colors.at(i, arr.length),
      pattern: profitabilityProfitPattern(key),
    }),
  );

  const profitabilityLoss = entries(LOSS_NAMES).map(([key, names], i, arr) => ({
    name: names.short,
    color: colors.at(i, arr.length),
    pattern: profitabilityLossPattern(key),
  }));

  return {
    cohortAll,
    termShort,
    termLong,
    underAge,
    overAge,
    ageRange,
    epoch,
    utxosOverAmount,
    addressesOverAmount,
    utxosUnderAmount,
    addressesUnderAmount,
    utxosAmountRange,
    addressesAmountRange,
    typeAddressable,
    typeOther,
    class: class_,
    entry,
    profitabilityRange,
    profitabilityProfit,
    profitabilityLoss,
  };
}
