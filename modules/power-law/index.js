// @ts-check
// Harold Christopher Burger's power law ("Bitcoin's natural long-term power-law corridor of growth", 2019):
// log10(price) = intercept + slope · log10(days since the genesis day). His support line keeps the slope and sits
// `support` under it in log10, as read from his figure (0.443 to 0.457 across it), which also puts it at $100,000 in
// February 2028 and $1,000,000 in April 2037, as his text says.
export const POWER_LAW = Object.freeze({ intercept: -17.01593313, slope: 5.84509376, support: 0.45 });
