import { createBtcAmount, satsToUsd } from "../../../btc/index.js";
import {
  appendLegendListItem,
  createLegendItem,
  createLegendList,
} from "../../../legend/index.js";
import { createUsdAmount } from "../../../usd/index.js";

/** @typedef {import("../../../modules/bitview-client/index.js").BlockExtras} BlockExtras */

const REWARD_COLORS = /** @type {const} */ ({
  subsidy: "var(--orange)",
  fees: "var(--green)",
});

/** @typedef {keyof typeof REWARD_COLORS} RewardType */

/**
 * @param {number} sats
 * @param {number} total
 */
function formatShare(sats, total) {
  return `${((sats / total) * 100).toFixed(2)}%`;
}

/**
 * @param {number} sats
 * @param {number} price
 */
function createSatsUsdAmount(sats, price) {
  return createUsdAmount("span", satsToUsd(sats, price));
}

/**
 * @param {number} sats
 * @param {number} total
 * @param {number} price
 */
function createRewardDetail(sats, total, price) {
  const detail = document.createDocumentFragment();

  detail.append(createSatsUsdAmount(sats, price), " · ", formatShare(sats, total));

  return detail;
}

/**
 * @param {RewardType} type
 * @param {number} sats
 * @param {number} total
 */
function createRewardSegment(type, sats, total) {
  const segment = document.createElement("span");

  segment.dataset.rewardSegment = type;
  segment.dataset.rewardKey = type;
  segment.style.setProperty("--share", `${(sats / total) * 100}%`);

  return segment;
}

/**
 * @param {RewardType} type
 * @param {string} label
 * @param {number} sats
 * @param {number} total
 * @param {number} price
 */
function createRewardPart(type, label, sats, total, price) {
  const { button: part, value } = createLegendItem({
    label,
    color: REWARD_COLORS[type],
    ariaLabel: `Highlight ${label}`,
    detail: createRewardDetail(sats, total, price),
  });
  const amount = createBtcAmount("span", sats);

  part.dataset.rewardPart = type;
  part.dataset.rewardKey = type;
  value.replaceChildren(amount);

  return part;
}

/**
 * @param {string} label
 * @param {number} sats
 * @param {number} price
 */
function createRewardTotal(label, sats, price) {
  const total = document.createElement("div");
  const name = document.createElement("span");
  const amount = createBtcAmount("strong", sats);
  const usd = createSatsUsdAmount(sats, price);

  total.dataset.rewardTotal = "";
  name.textContent = label;
  total.append(name, amount, usd);

  return total;
}

/** @param {EventTarget | null} target */
function getRewardKey(target) {
  if (!(target instanceof HTMLElement)) return null;

  return target.closest("[data-reward-key]")?.getAttribute("data-reward-key") ?? null;
}

/**
 * @param {HTMLElement[]} elements
 * @param {string | null} activeKey
 */
function setRewardPreview(elements, activeKey) {
  for (const element of elements) {
    if (element.dataset.rewardKey === activeKey) {
      element.dataset.preview = "";
      delete element.dataset.muted;
    } else if (activeKey) {
      element.dataset.muted = "";
      delete element.dataset.preview;
    } else {
      delete element.dataset.muted;
      delete element.dataset.preview;
    }
  }
}

/** @param {BlockExtras} extras */
export function createRewardsPane(extras) {
  const subsidy = extras.reward - extras.totalFees;
  const rewards = document.createElement("div");
  const bar = document.createElement("div");
  const split = createLegendList({ fill: true });
  const subsidyPart = createRewardPart(
    "subsidy",
    "Subsidy",
    subsidy,
    extras.reward,
    extras.price,
  );
  const feesPart = createRewardPart(
    "fees",
    "Fees",
    extras.totalFees,
    extras.reward,
    extras.price,
  );
  const subsidySegment = createRewardSegment("subsidy", subsidy, extras.reward);
  const feesSegment = createRewardSegment("fees", extras.totalFees, extras.reward);
  const previewElements = [subsidyPart, feesPart, subsidySegment, feesSegment];

  rewards.dataset.rewardsPane = "";
  appendLegendListItem(split, subsidyPart);
  appendLegendListItem(split, feesPart);
  bar.dataset.rewardBar = "";
  bar.append(subsidySegment, feesSegment);
  rewards.append(createRewardTotal("Rewards", extras.reward, extras.price), bar, split);

  rewards.addEventListener("pointerenter", (event) => {
    setRewardPreview(previewElements, getRewardKey(event.target));
  }, true);
  rewards.addEventListener("pointerleave", () => setRewardPreview(previewElements, null));
  rewards.addEventListener("pointerdown", (event) => {
    setRewardPreview(previewElements, getRewardKey(event.target));
  });
  rewards.addEventListener("pointerup", () => setRewardPreview(previewElements, null));
  rewards.addEventListener("pointercancel", () => {
    setRewardPreview(previewElements, null);
  });

  return rewards;
}
