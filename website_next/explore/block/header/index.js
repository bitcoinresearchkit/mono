import { createUsdAmount, renderUsdAmount } from "../../../usd/index.js";
import { formatDateAndAge } from "../format.js";
import { createBlockTitle } from "../title/index.js";

/** @typedef {import("../../../modules/bitview-client/index.js").BlockInfoV1} Block */

/** @param {string} hash */
function createHashElement(hash) {
  const element = document.createElement("span");
  const prefix = document.createElement("span");
  const value = document.createElement("span");
  const firstNonZero = hash.search(/[^0]/);
  const visibleStart = firstNonZero === -1 ? hash.length : firstNonZero;

  element.dataset.blockHash = "";
  prefix.dataset.dim = "";
  prefix.textContent = hash.slice(0, visibleStart);
  value.textContent = hash.slice(visibleStart);
  element.append(prefix, value);

  return element;
}

/** @param {Node[]} [actions] */
export function createBlockHeader(actions = []) {
  const element = document.createElement("header");
  const title = document.createElement("h1");
  const actionList = document.createElement("div");
  const date = document.createElement("time");
  const hash = document.createElement("p");
  const price = createUsdAmount("output", 0, {
    size: "title",
    tone: "positive",
  });
  let timestamp = 0;
  let dateTimer = 0;

  element.dataset.blockHeader = "";
  title.dataset.blockTitleText = "";
  actionList.append(...actions);
  element.append(title, price, hash, actionList, date);

  function refreshDate() {
    date.textContent = formatDateAndAge(timestamp);
    dateTimer = window.setTimeout(refreshDate, 60_000);
  }

  /** @param {Block} block */
  function update(block) {
    title.replaceChildren(...createBlockTitle(block.height));
    date.dateTime = new Date(block.timestamp * 1_000).toISOString();
    timestamp = block.timestamp;
    window.clearTimeout(dateTimer);
    refreshDate();
    hash.replaceChildren(createHashElement(block.id));
    renderUsdAmount(price, block.extras.price, {
      size: "title",
      tone: "positive",
    });
  }

  return /** @type {const} */ ({
    element,
    update,
  });
}
