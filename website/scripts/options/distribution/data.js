import { colors } from "../../utils/colors.js";
import { entries } from "../../utils/array.js";
import { bitview } from "../../utils/client.js";
import { ageRanges } from "../age-ranges.js";
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
    PROFITABILITY_RANGE_NAMES,
  } = bitview;

  const cohortAll = lazy(() => ({
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
  }));

  const shortNames = TERM_NAMES.short;
  const termShort = lazy(() => ({
    name: shortNames.short,
    title: shortNames.long,
    color: colors.term.short,
    tree: selectCohortTree({ tree: cohorts, path: "term.short" }),
  }));

  const longNames = TERM_NAMES.long;
  const termLong = lazy(() => ({
    name: longNames.short,
    title: longNames.long,
    color: colors.term.long,
    tree: selectCohortTree({ tree: cohorts, path: "term.long" }),
  }));

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
      const cohort = selectCohortTree({
        tree: cohorts,
        path: `addrBalance.${key}`,
      });
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
          utxo: addrs.avgAmount.utxo[key],
          addr: addrs.avgAmount.addr[key],
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

  const profitability = lazy(() => {
    const profitability = cohorts.profitability;
    /** @param {keyof typeof profitability.supply} key */
    const profitabilityRangePattern = (key) => ({
      supply: profitability.supply[key],
      realizedCap: profitability.realizedCap[key],
      unrealizedPnl: profitability.unrealizedPnl[key],
      nupl: profitability.nupl[key],
    });

    const profitabilityRange = entries(PROFITABILITY_RANGE_NAMES).map(
      ([key, names], i, arr) => ({
        name: names.short,
        color: colors.at(i, arr.length),
        pattern: profitabilityRangePattern(key),
      }),
    );

    return profitabilityRange;
  });

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
    get profitabilityRange() {
      return profitability();
    },
  };
}
