// Studio's holder groups' names, one table wherever they're named (the presets' tree, titles and legends, Compass's
// among them; the page's heatmap and profile pickers), by the keys the series tree names them by.

/**
 * The holder groups (and weighted cohorts), by their keys, in the order the page lists them (all coins, then each
 * complementary pair: the terms, under 4 months and at least 4, under 6 and at least 6):
 * - `name`: as their places keep it;
 * - `label`: as shown, where the place's name is false (the groups at least 4 and 6 months old hold their edge, their
 *   places kept from when they said "over");
 * - `brief`: in a list that already says what they are ("Short-term", "At least 4 months");
 * - `who`: in titles; `adds`: what they add to some coins ("Awake coins under 4 months old"); `short`: an abbreviation;
 * - `urpd`: their URPD cohort, where they have one (the weighted ones don't).
 * @type {Readonly<Record<string, Readonly<{ name: string, label?: string, brief: string, who: string, adds: string, short?: string, urpd?: string }>>>}
 */
export const HOLDERS = Object.freeze({
  all: { name: "All coins", brief: "All coins", who: "All coins", adds: "", urpd: "all" },
  sth: { name: "Short-term holders", brief: "Short-term", who: "Short-term holders", adds: " of short-term holders", short: "STH", urpd: "sth" },
  lth: { name: "Long-term holders", brief: "Long-term", who: "Long-term holders", adds: " of long-term holders", short: "LTH", urpd: "lth" },
  under4m: { name: "Under 4 months old", brief: "Under 4 months", who: "Coins under 4 months old", adds: " under 4 months old", urpd: "age_under_4m" },
  over4m: { name: "Over 4 months old", label: "At least 4 months old", brief: "At least 4 months", who: "Coins at least 4 months old", adds: " at least 4 months old", urpd: "age_over_4m" },
  under6m: { name: "Under 6 months old", brief: "Under 6 months", who: "Coins under 6 months old", adds: " under 6 months old", urpd: "age_under_6m" },
  over6m: { name: "Over 6 months old", label: "At least 6 months old", brief: "At least 6 months", who: "Coins at least 6 months old", adds: " at least 6 months old", urpd: "age_over_6m" },
  awake: { name: "Awake coins", brief: "Awake", who: "Awake coins", adds: "" },
  mobile: { name: "Mobile coins", brief: "Mobile", who: "Mobile coins", adds: "" },
});
