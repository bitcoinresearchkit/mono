import { entries } from "../../utils/array.js";
import { bitview } from "../../utils/client.js";
import { colors } from "../../utils/colors.js";
import { Unit } from "../../utils/units.js";
import {
  averagesArray,
  chartsFromBlockAnd6b,
  chartsFromCount,
  chartsFromCountEntries,
  chartsFromFullPerBlock,
  line,
} from "../series.js";
import { satsBtcUsdFullTree } from "../shared.js";

/**
 * @param {Object} args
 * @param {string} args.name
 * @param {string} args.metric
 * @param {CountPattern<number>} args.pattern
 * @param {Unit} [args.unit]
 * @returns {PartialOptionsGroup}
 */
function createCountFolder({ name, metric, pattern, unit = Unit.count }) {
  return {
    name,
    tree: chartsFromCount({ pattern, metric, unit }),
  };
}

/**
 * Create the transaction section of the Network tree.
 *
 * Common metrics stay shallow. Specialist metrics are grouped by the concept
 * users are looking for, while every leaf remains an independent chart.
 *
 * @returns {PartialOptionsGroup}
 */
export function createTransactionsSection() {
  const { transactions, supply } = bitview.series;
  const featureCount = transactions.features.count;

  return {
    name: "Transactions",
    tree: [
      {
        name: "Count",
        tree: chartsFromFullPerBlock({
          pattern: transactions.count.total,
          metric: "Transaction Count",
          unit: Unit.count,
        }),
      },
      {
        name: "Per Second",
        tree: averagesArray({
          windows: transactions.volume.txPerSec,
          metric: "Transactions per Second",
          unit: Unit.perSec,
        }),
      },
      {
        name: "Volume",
        tree: satsBtcUsdFullTree({
          pattern: transactions.volume.transferVolume,
          metric: "Transaction Volume",
        }),
      },
      {
        name: "Velocity",
        title: "Transaction Velocity",
        bottom: [
          line({
            series: supply.velocity.native,
            name: "BTC",
            unit: Unit.ratio,
          }),
          line({
            series: supply.velocity.fiat,
            name: "USD",
            color: colors.usd,
            unit: Unit.ratio,
          }),
        ],
      },
      {
        name: "Effective Fee Rate",
        tree: chartsFromBlockAnd6b({
          pattern: transactions.fees.effectiveFeeRate,
          metric: "Effective Transaction Fee Rate",
          unit: Unit.feeRate,
        }),
      },
      {
        name: "Fee",
        tree: chartsFromBlockAnd6b({
          pattern: transactions.fees.fee,
          metric: "Transaction Fee",
          unit: Unit.sats,
        }),
      },
      {
        name: "CPFP",
        tree: [
          createCountFolder({
            name: "Parents",
            metric: "Confirmed CPFP Parent Transactions",
            pattern: transactions.fees.count.cpfpParent,
          }),
          createCountFolder({
            name: "Children",
            metric: "Confirmed CPFP Child Transactions",
            pattern: transactions.fees.count.cpfpChild,
          }),
        ],
      },
      {
        name: "Size",
        tree: [
          {
            name: "Weight",
            tree: chartsFromBlockAnd6b({
              pattern: transactions.size.weight,
              metric: "Transaction Weight",
              unit: Unit.wu,
            }),
          },
          {
            name: "Virtual Size",
            tree: chartsFromBlockAnd6b({
              pattern: transactions.size.vsize,
              metric: "Transaction Virtual Size",
              unit: Unit.vb,
            }),
          },
        ],
      },
      createCountFolder({
        name: "Sigops",
        metric: "Total Sigop Cost",
        pattern: transactions.sigops.total,
        unit: Unit.sigopCost,
      }),
      {
        name: "Sighashes",
        tree: [
          createCountFolder({
            name: "ALL",
            metric: "Transactions with SIGHASH_ALL",
            pattern: featureCount.sighashAll,
          }),
          createCountFolder({
            name: "NONE",
            metric: "Transactions with SIGHASH_NONE",
            pattern: featureCount.sighashNone,
          }),
          createCountFolder({
            name: "SINGLE",
            metric: "Transactions with SIGHASH_SINGLE",
            pattern: featureCount.sighashSingle,
          }),
          createCountFolder({
            name: "DEFAULT",
            metric: "Transactions with SIGHASH_DEFAULT",
            pattern: featureCount.sighashDefault,
          }),
          createCountFolder({
            name: "ANYONECANPAY",
            metric: "Transactions with SIGHASH_ANYONECANPAY",
            pattern: featureCount.sighashAnyoneCanPay,
          }),
        ],
      },
      {
        name: "Data",
        tree: [
          createCountFolder({
            name: "Inscriptions",
            metric: "Transactions with Inscriptions",
            pattern: featureCount.inscription,
          }),
          createCountFolder({
            name: "Taproot Annexes",
            metric: "Transactions with Taproot Annexes",
            pattern: featureCount.annex,
          }),
        ],
      },
      {
        name: "Patterns",
        tree: [
          createCountFolder({
            name: "CoinJoins",
            metric: "Detected CoinJoin Transactions",
            pattern: transactions.patterns.count.coinjoin,
          }),
          createCountFolder({
            name: "Consolidations",
            metric: "Detected Consolidation Transactions",
            pattern: transactions.patterns.count.consolidation,
          }),
          createCountFolder({
            name: "Batch Payouts",
            metric: "Detected Batch Payout Transactions",
            pattern: transactions.patterns.count.batchPayout,
          }),
        ],
      },
      {
        name: "Policy",
        tree: [
          createCountFolder({
            name: "Dust Outputs",
            metric: "Transactions with Dust Outputs",
            pattern: featureCount.dustOutput,
          }),
          createCountFolder({
            name: "Nonstandard",
            metric: "Nonstandard Transactions",
            pattern: transactions.policy.count.nonstandard,
          }),
        ],
      },
      {
        name: "Versions",
        tree: chartsFromCountEntries({
          entries: entries(transactions.versions),
          metric: "Transaction Versions",
          unit: Unit.count,
        }),
      },
    ],
  };
}
