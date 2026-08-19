import { createPoolLogo } from "../../../pools/index.js";
import { formatPoolBlockNumber, getCoinbaseMessage } from "../format.js";

/** @typedef {import("../../../modules/bitview-client/index.js").BlockInfoV1} Block */

/** @param {string} raw */
function createCoinbaseMessage(raw) {
  const message = getCoinbaseMessage(raw);

  if (!message) return null;

  const element = document.createElement("p");

  element.dataset.coinbaseMessage = "";
  element.textContent = message;

  return element;
}

/** @param {Block} block */
export function createMinerPane(block) {
  const { pool } = block.extras;
  const pane = document.createElement("div");
  const head = document.createElement("div");
  const identity = document.createElement("div");
  const title = document.createElement("div");
  const name = document.createElement("strong");
  const blockNumber = document.createElement("span");
  const slug = document.createElement("span");
  const logo = createPoolLogo(pool);
  const coinbaseMessage = createCoinbaseMessage(block.extras.coinbaseSignatureAscii);

  pane.dataset.minerPane = "";

  name.textContent = pool.name;
  blockNumber.textContent = formatPoolBlockNumber(pool.blockNumber);
  slug.textContent = pool.slug;
  title.append(name, blockNumber);
  identity.append(title, slug);
  head.append(identity, logo);
  pane.append(head, ...(coinbaseMessage ? [coinbaseMessage] : []));

  return pane;
}
