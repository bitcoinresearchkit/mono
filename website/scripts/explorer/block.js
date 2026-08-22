import { brk } from "../utils/client.js";
import { createPersistedValue } from "../utils/persisted.js";
import { createRow, formatFeeRate, formatHeightPrefix, renderTx, showPanel, hidePanel, TX_PAGE_SIZE } from "./render.js";

/** @typedef {[string, (b: BlockInfoV1) => string | null, ((b: BlockInfoV1) => string | null)?]} RowDef */

/** @param {(x: BlockInfoV1["extras"]) => string | null} fn @returns {(b: BlockInfoV1) => string | null} */
const ext = (fn) => (b) => fn(b.extras);

/** @type {RowDef[]} */
const ROW_DEFS = [
  ["Hash", (b) => b.id, (b) => `/block/${b.id}`],
  ["Previous Hash", (b) => b.previousblockhash, (b) => `/block/${b.previousblockhash}`],
  ["Merkle Root", (b) => b.merkleRoot],
  ["Timestamp", (b) => new Date(b.timestamp * 1000).toUTCString()],
  ["Median Time", (b) => new Date(b.mediantime * 1000).toUTCString()],
  ["Version", (b) => `0x${b.version.toString(16)}`],
  ["Bits", (b) => b.bits.toString(16)],
  ["Nonce", (b) => b.nonce.toLocaleString()],
  ["Difficulty", (b) => Number(b.difficulty).toLocaleString()],
  ["Size", (b) => `${(b.size / 1_000_000).toFixed(2)} MB`],
  ["Weight", (b) => `${(b.weight / 1_000_000).toFixed(2)} MWU`],
  ["Transactions", (b) => b.txCount.toLocaleString()],
  ["Price", ext((x) => `$${x.price.toLocaleString()}`)],
  ["Pool", ext((x) => x.pool.name)],
  ["Pool ID", ext((x) => x.pool.id.toString())],
  ["Pool Slug", ext((x) => x.pool.slug)],
  ["Miner Names", ext((x) => x.pool.minerNames?.join(", ") || null)],
  ["Reward", ext((x) => `${(x.reward / 1e8).toFixed(8)} BTC`)],
  ["Total Fees", ext((x) => `${(x.totalFees / 1e8).toFixed(8)} BTC`)],
  ["Median Fee Rate", ext((x) => `${formatFeeRate(x.medianFee)} sat/vB`)],
  ["Avg Fee Rate", ext((x) => `${formatFeeRate(x.avgFeeRate)} sat/vB`)],
  ["Avg Fee", ext((x) => `${x.avgFee.toLocaleString()} sat`)],
  ["Median Fee", ext((x) => `${x.medianFeeAmt.toLocaleString()} sat`)],
  ["Fee Range", ext((x) => x.feeRange.map(formatFeeRate).join(", ") + " sat/vB")],
  ["Fee Percentiles", ext((x) => x.feePercentiles.map((f) => f.toLocaleString()).join(", ") + " sat")],
  ["Avg Tx Size", ext((x) => `${x.avgTxSize.toLocaleString()} B`)],
  ["Virtual Size", ext((x) => `${x.virtualSize.toLocaleString()} vB`)],
  ["Inputs", ext((x) => x.totalInputs.toLocaleString())],
  ["Outputs", ext((x) => x.totalOutputs.toLocaleString())],
  ["Total Input Amount", ext((x) => `${(x.totalInputAmt / 1e8).toFixed(8)} BTC`)],
  ["Total Output Amount", ext((x) => `${(x.totalOutputAmt / 1e8).toFixed(8)} BTC`)],
  ["UTXO Set Change", ext((x) => x.utxoSetChange.toLocaleString())],
  ["UTXO Set Size", ext((x) => x.utxoSetSize.toLocaleString())],
  ["SegWit Txs", ext((x) => x.segwitTotalTxs.toLocaleString())],
  ["SegWit Size", ext((x) => `${x.segwitTotalSize.toLocaleString()} B`)],
  ["SegWit Weight", ext((x) => `${x.segwitTotalWeight.toLocaleString()} WU`)],
  ["Coinbase Address", ext((x) => x.coinbaseAddress || null)],
  ["Coinbase Addresses", ext((x) => x.coinbaseAddresses.join(", ") || null)],
  ["Coinbase Raw", ext((x) => x.coinbaseRaw)],
  ["Coinbase Signature", ext((x) => x.coinbaseSignature)],
  ["Coinbase Signature ASCII", ext((x) => x.coinbaseSignatureAscii)],
  ["Header", ext((x) => x.header)],
];

/** @typedef {{ first: HTMLButtonElement, prev: HTMLButtonElement, label: HTMLSpanElement, next: HTMLButtonElement, last: HTMLButtonElement }} TxNav */

/** @type {HTMLDivElement} */ let el;
/** @type {HTMLSpanElement} */ let heightPrefix;
/** @type {HTMLSpanElement} */ let heightNum;
/** @type {{ row: HTMLDivElement, valueEl: HTMLElement }[]} */ let detailRows;
/** @type {HTMLDivElement} */ let txList;
/** @type {HTMLDivElement} */ let txSection;
/** @type {IntersectionObserver} */ let txObserver;
/** @type {TxNav[]} */ const txNavs = [];
/** @type {BlockInfoV1 | null} */ let txBlock = null;
let txTotalPages = 0;
let txLoading = false;
let txLoaded = false;

const txPageParam = createPersistedValue({
  defaultValue: 0,
  urlKey: "page",
  serialize: (v) => String(v + 1),
  deserialize: (s) => Math.max(0, Number(s) - 1),
});

/** @param {HTMLElement} parent @param {(e: MouseEvent) => void} linkHandler */
export function initBlockDetails(parent, linkHandler) {
  el = document.createElement("div");
  el.id = "block-details";
  parent.append(el);

  const title = document.createElement("h1");
  title.textContent = "Block ";
  const code = document.createElement("code");
  const container = document.createElement("span");
  heightPrefix = document.createElement("span");
  heightPrefix.classList.add("dim");
  heightPrefix.style.userSelect = "none";
  heightNum = document.createElement("span");
  container.append(heightPrefix, heightNum);
  code.append(container);
  title.append(code);
  el.append(title);

  el.addEventListener("click", linkHandler);

  detailRows = ROW_DEFS.map(([label, , linkFn]) => {
    const { row, valueEl } = createRow(label, Boolean(linkFn));
    el.append(row);
    return { row, valueEl };
  });

  txSection = document.createElement("div");
  txSection.classList.add("transactions");
  el.append(txSection);

  const txHeader = document.createElement("div");
  txHeader.classList.add("tx-header");
  const heading = document.createElement("h2");
  heading.textContent = "Transactions";
  txHeader.append(heading, createTxNav());
  txSection.append(txHeader);

  txList = document.createElement("div");
  txList.classList.add("tx-list");
  txSection.append(txList, createTxNav());

  txObserver = new IntersectionObserver((entries) => {
    if (entries[0].isIntersecting && !txLoaded) {
      loadTxPage(txPageParam.value, false);
    }
  });
  txObserver.observe(txSection);
}

function createTxNav() {
  const nav = document.createElement("div");
  nav.classList.add("pagination");
  const first = document.createElement("button");
  first.textContent = "\u00AB";
  const prev = document.createElement("button");
  prev.textContent = "\u2190";
  const label = document.createElement("span");
  const next = document.createElement("button");
  next.textContent = "\u2192";
  const last = document.createElement("button");
  last.textContent = "\u00BB";
  nav.append(first, prev, label, next, last);
  first.addEventListener("click", () => loadTxPage(0));
  prev.addEventListener("click", () => loadTxPage(txPageParam.value - 1));
  next.addEventListener("click", () => loadTxPage(txPageParam.value + 1));
  last.addEventListener("click", () => loadTxPage(txTotalPages - 1));
  txNavs.push({ first, prev, label, next, last });
  return nav;
}

/** @param {number} page */
function updateTxNavs(page) {
  const atFirst = page <= 0;
  const atLast = page >= txTotalPages - 1;
  for (const n of txNavs) {
    n.label.textContent = `${page + 1} / ${txTotalPages}`;
    n.first.disabled = atFirst;
    n.prev.disabled = atFirst;
    n.next.disabled = atLast;
    n.last.disabled = atLast;
  }
}

/** @param {BlockInfoV1} block */
export function update(block) {
  heightPrefix.textContent = formatHeightPrefix(block.height);
  heightNum.textContent = block.height.toString();

  ROW_DEFS.forEach(([, getter, linkFn], i) => {
    const value = getter(block);
    const { row, valueEl } = detailRows[i];
    if (value !== null) {
      valueEl.textContent = value;
      if (linkFn)
        /** @type {HTMLAnchorElement} */ (valueEl).href = linkFn(block) ?? "";
      row.hidden = false;
    } else {
      row.hidden = true;
    }
  });

  txBlock = block;
  txTotalPages = Math.ceil(block.txCount / TX_PAGE_SIZE);
  if (txLoaded) txPageParam.setImmediate(0);
  txLoaded = false;
  updateTxNavs(txPageParam.value);
  txList.innerHTML = "";
  txObserver.disconnect();
  txObserver.observe(txSection);
}

export function show() { showPanel(el); }
export function hide() { hidePanel(el); }

/** @param {number} page @param {boolean} [pushUrl] */
async function loadTxPage(page, pushUrl = true) {
  const block = txBlock;
  if (txLoading || !block || page < 0 || page >= txTotalPages) return;
  txLoading = true;
  txLoaded = true;
  if (pushUrl) txPageParam.setImmediate(page);
  updateTxNavs(page);
  try {
    const txs = await brk.getBlockTxsFromIndex(block.id, page * TX_PAGE_SIZE);
    txList.innerHTML = "";
    const ascii = block.extras.coinbaseSignatureAscii;
    for (const tx of txs) txList.append(renderTx(tx, ascii));
  } catch (e) {
    console.error("explorer txs:", e);
  }
  txLoading = false;
}
