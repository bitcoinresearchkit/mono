/** Unit definitions for chart series */

/**
 * Unit enum with id (for serialization) and name (for display)
 */
export const Unit = /** @type {const} */ ({
  // Value units
  sats: { id: "sats", name: "Satoshis" },
  btc: { id: "btc", name: "Bitcoin" },
  usd: { id: "usd", name: "US Dollars" },

  // Ratios & percentages
  percentage: { id: "percentage", name: "Percentage" },
  cagr: { id: "cagr", name: "CAGR (%/year)" },
  ratio: { id: "ratio", name: "Ratio" },
  index: { id: "index", name: "Index" },
  state: { id: "state", name: "State" },

  // Relative percentages
  pctSupply: { id: "pct-supply", name: "% of circulating" },
  pctOwn: { id: "pct-own", name: "% of Own" },

  // Time
  days: { id: "days", name: "Days" },
  years: { id: "years", name: "Years" },
  secs: { id: "secs", name: "Seconds" },

  // Counts
  count: { id: "count", name: "Count" },
  blocks: { id: "blocks", name: "Blocks" },
  sigopCost: { id: "sigop-cost", name: "Sigop Cost" },

  // Size
  bytes: { id: "bytes", name: "Bytes" },
  vb: { id: "vb", name: "Virtual Bytes" },
  wu: { id: "wu", name: "Weight Units" },

  // Mining
  hashRate: { id: "hashrate", name: "Hash Rate" },
  difficulty: { id: "difficulty", name: "Difficulty" },
  epoch: { id: "epoch", name: "Epoch" },

  // Fees
  feeRate: { id: "feerate", name: "Sat/vByte" },

  // Rates
  perSec: { id: "per-sec", name: "Per Second" },

  // Cointime
  coinblocks: { id: "coinblocks", name: "Coinblocks" },
  coindays: { id: "coindays", name: "Coindays" },
  satblocks: { id: "satblocks", name: "Satblocks" },
  satdays: { id: "satdays", name: "Satdays" },

  // Hash price/value
  usdPerThsPerDay: { id: "usd-ths-day", name: "USD/TH/s/Day" },
  usdPerPhsPerDay: { id: "usd-phs-day", name: "USD/PH/s/Day" },
  satsPerThsPerDay: { id: "sats-ths-day", name: "Sats/TH/s/Day" },
  satsPerPhsPerDay: { id: "sats-phs-day", name: "Sats/PH/s/Day" },
});

/** @typedef {keyof typeof Unit} UnitKey */
/** @typedef {typeof Unit[UnitKey]} UnitObject */
