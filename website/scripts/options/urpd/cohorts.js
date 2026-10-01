import { AGE_CUTOFFS } from "../age-cutoffs.js";

export const URPD_COHORTS = /** @type {const} */ ([
  { key: "all", name: "All" },
  { key: "sth", name: "STH" },
  { key: "lth", name: "LTH" },
  ...AGE_CUTOFFS,
]);
