import { getFeeRateColor } from "../../block/fee-rates.js";
import {
  createPreviewFeeRange,
  orderTransactions,
} from "../../block/preview/heatmap/fees.js";

/** @param {import("../../block/preview/data.js").BlockPreviewData} data */
export function createXrayTransactions(data) {
  const order = orderTransactions(data.weights, data.feeRates);
  const ranges = createPreviewFeeRange(data.feeRates, order);

  return Array.from(order, (offset) => {
    const feeRate = data.feeRates[offset];

    return {
      color: getFeeRateColor(feeRate, ranges),
      feeRate,
      txIndex: data.range.start + offset,
      weight: data.weights[offset],
    };
  });
}
