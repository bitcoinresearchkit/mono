import { colors } from "../../utils/colors.js";
import { entries } from "../../utils/array.js";
import { bitview } from "../../utils/client.js";
import { ageRanges } from "../age-ranges.js";
import { AGE_CUTOFFS } from "../age-cutoffs.js";
import { lazy } from "../lazy.js";
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
  const cohorts = bitview.series.cohorts;
  const { addrs } = bitview.series;
  const {
    TERM_NAMES,
    EPOCH_NAMES,
    AMOUNT_RANGE_NAMES,
    SPENDABLE_TYPE_NAMES,
    CLASS_NAMES,
  } = bitview;

  const cohortAll = lazy(() => ({
    name: "",
    title: "",
    color: colors.bitcoin,
    tree: bitview.series.distributionAggregated.cohorts.all,
    addressCount: {
      base: addrs.funded.all,
      delta: addrs.delta.all,
    },
    avgAmount: {
      utxo: cohorts.outputs.avgAmount.all,
      addr: addrs.avgBalance.all,
    },
  }));

  const shortNames = TERM_NAMES.short;
  const termShort = lazy(() => ({
    name: shortNames.short,
    title: shortNames.long,
    color: colors.term.short,
    tree: bitview.series.distributionAggregated.cohorts.sth,
  }));

  const longNames = TERM_NAMES.long;
  const termLong = lazy(() => ({
    name: longNames.short,
    title: longNames.long,
    color: colors.term.long,
    tree: bitview.series.distributionAggregated.cohorts.lth,
  }));

  const ageCutoff = lazy(() =>
    AGE_CUTOFFS.map(({ key, name }, i, all) => ({
      name,
      title: `UTXOs ${name}`,
      color: colors.at(i, all.length),
      tree: bitview.series.distributionAggregated.cohorts[key],
    })),
  );

  const ageRange = lazy(() =>
    ageRanges.map(({ key, ...range }) => ({
      ...range,
      tree: selectCohortTree({ tree: cohorts, path: `age.${key}` }),
      matured: cohorts.supply.matured[key],
    })),
  );

  const epoch = lazy(() =>
    entries(EPOCH_NAMES).map(([key, names], i, arr) => ({
      name: names.short,
      title: names.long,
      color: colors.at(i, arr.length),
      tree: selectCohortTree({ tree: cohorts, path: `epoch.${key}` }),
    })),
  );

  const utxosAmountRange = lazy(() =>
    entries(AMOUNT_RANGE_NAMES).map(([key, names], i, arr) => ({
      name: names.short,
      title: `UTXOs ${names.long}`,
      color: colors.at(i, arr.length),
      tree: selectCohortTree({
        tree: cohorts,
        path: `utxoAmount.${key}`,
      }),
    })),
  );

  const addressesAmountRange = lazy(() =>
    entries(AMOUNT_RANGE_NAMES).map(([key, names], i, arr) => {
      const cohort = addressBalanceTree(key);
      return {
        name: names.short,
        title: `Addresses ${names.long}`,
        color: colors.at(i, arr.length),
        tree: cohort,
        addressCount: addrs.funded.balance[key],
      };
    }),
  );

  const typeAddressable = lazy(() =>
    ADDRESSABLE_TYPES.map((key) => {
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
          utxo: cohorts.outputs.avgAmount.byType[key],
          addr: addrs.avgBalance[key],
        },
        exposed: addrs.exposed,
        reused: addrs.reused,
        respent: addrs.respent,
      };
    }),
  );

  const typeOther = lazy(() =>
    entries(SPENDABLE_TYPE_NAMES)
      .filter(([key]) => !isAddressable(key))
      .map(([key, names]) => ({
        key,
        name: names.short,
        title: names.short,
        color: colors.scriptType[key],
        tree: selectCohortTree({ tree: cohorts, path: `type.${key}` }),
        avgUtxoAmount: cohorts.outputs.avgAmount.byType[key],
      })),
  );

  const class_ = lazy(() =>
    entries(CLASS_NAMES)
      .reverse()
      .map(([key, names], i, arr) => ({
        name: names.short,
        title: names.long,
        color: colors.at(i, arr.length),
        tree: selectCohortTree({ tree: cohorts, path: `class.${key}` }),
      })),
  );

  return {
    get cohortAll() {
      return cohortAll();
    },
    get termShort() {
      return termShort();
    },
    get termLong() {
      return termLong();
    },
    get ageCutoff() {
      return ageCutoff();
    },
    get ageRange() {
      return ageRange();
    },
    get epoch() {
      return epoch();
    },
    get utxosAmountRange() {
      return utxosAmountRange();
    },
    get addressesAmountRange() {
      return addressesAmountRange();
    },
    get typeAddressable() {
      return typeAddressable();
    },
    get typeOther() {
      return typeOther();
    },
    get class() {
      return class_();
    },
  };
}

/** @param {keyof Bitview.SeriesTree["addrs"]["byBalance"]["supply"]} key */
export function addressBalanceTree(key) {
  const { byBalance } = bitview.series.addrs;
  return {
    supply: byBalance.supply[key],
    outputs: { unspentCount: byBalance.utxoCount[key] },
    activity: { transferVolume: byBalance.transferVolume[key] },
    realized: {
      cap: byBalance.realizedCap[key],
      profit: byBalance.realizedProfit[key],
      loss: byBalance.realizedLoss[key],
    },
  };
}
