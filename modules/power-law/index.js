// @ts-check
// Harold Christopher Burger's power law ("Bitcoin's natural long-term power-law corridor of growth", 2019):
// log10(price) = intercept + slope · log10(days since the genesis day). His support line keeps the slope and sits
// `support` under it in log10, about 0.45 as read from his figure (his text gives no number), which puts it past
// $100,000 after 2028 and $1,000,000 after 2037, as his text says.
export const POWER_LAW = Object.freeze({ intercept: -17.01593313, slope: 5.84509376, support: 0.45 });
