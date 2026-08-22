import { brk } from "../utils/client.js";
import { latestPrice } from "../utils/price.js";
import { createRow, formatBtc, renderTx, showPanel, hidePanel, TX_PAGE_SIZE } from "./render.js";

/** @type {HTMLDivElement} */ let el;
/** @type {HTMLElement[]} */ let valueEls;
/** @type {HTMLDivElement} */ let txSection;
/** @type {string} */ let currentAddr = "";

const ROW_LABELS = [
  "Address",
  "Confirmed Balance",
  "Pending",
  "Confirmed UTXOs",
  "Pending UTXOs",
  "Total Received",
  "Tx Count",
  "Type",
  "Avg Cost Basis",
];

/** @param {HTMLElement} parent @param {(e: MouseEvent) => void} linkHandler */
export function initAddrDetails(parent, linkHandler) {
  el = document.createElement("div");
  el.id = "addr-details";
  el.hidden = true;
  parent.append(el);
  el.addEventListener("click", linkHandler);

  const title = document.createElement("h1");
  title.textContent = "Address";
  el.append(title);

  valueEls = ROW_LABELS.map((label) => {
    const { row, valueEl } = createRow(label);
    el.append(row);
    return valueEl;
  });

  txSection = document.createElement("div");
  txSection.classList.add("transactions");
  const heading = document.createElement("h2");
  heading.textContent = "Transactions";
  txSection.append(heading);
  el.append(txSection);
}

/**
 * @param {string} address
 * @param {AbortSignal} signal
 */
export async function update(address, signal) {
  currentAddr = address;
  valueEls[0].textContent = address;
  for (let i = 1; i < valueEls.length; i++) {
    valueEls[i].textContent = "...";
    valueEls[i].classList.add("dim");
  }
  while (txSection.children.length > 1) txSection.lastChild?.remove();

  try {
    const stats = await brk.getAddress(address, { signal });
    if (signal.aborted || currentAddr !== address) return;

    const chain = stats.chainStats;
    const balance = chain.fundedTxoSum - chain.spentTxoSum;
    const mempool = stats.mempoolStats;
    const pending = mempool ? mempool.fundedTxoSum - mempool.spentTxoSum : 0;
    const pendingUtxos = mempool
      ? mempool.fundedTxoCount - mempool.spentTxoCount
      : 0;
    const confirmedUtxos = chain.fundedTxoCount - chain.spentTxoCount;
    const price = latestPrice();
    const fmtUsd = (/** @type {number} */ sats) =>
      price
        ? ` $${((sats / 1e8) * price).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`
        : "";

    const values = [
      address,
      `${formatBtc(balance)} BTC${fmtUsd(balance)}`,
      `${pending >= 0 ? "+" : ""}${formatBtc(pending)} BTC${fmtUsd(pending)}`,
      confirmedUtxos.toLocaleString(),
      pendingUtxos.toLocaleString(),
      `${formatBtc(chain.fundedTxoSum)} BTC`,
      chain.txCount.toLocaleString(),
      stats.addrType.replace(/^v\d+_/, "").toUpperCase(),
      chain.realizedPrice
        ? `$${Number(chain.realizedPrice).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`
        : "N/A",
    ];

    for (let i = 0; i < valueEls.length; i++) {
      valueEls[i].textContent = values[i];
      valueEls[i].classList.remove("dim");
    }

    let loading = false;
    let pageIndex = 0;
    /** @type {string | undefined} */
    let afterTxid;

    const observer = new IntersectionObserver((entries) => {
      if (
        entries[0].isIntersecting &&
        !loading &&
        pageIndex * TX_PAGE_SIZE < chain.txCount
      )
        loadMore();
    });

    async function loadMore() {
      if (currentAddr !== address) return;
      loading = true;
      try {
        const txs = afterTxid
          ? await brk.getAddressConfirmedTxsAfter(address, afterTxid, { signal })
          : await brk.getAddressTxs(address, { signal });
        if (currentAddr !== address) return;
        for (const tx of txs) txSection.append(renderTx(tx));
        pageIndex++;
        if (txs.length) {
          afterTxid = txs[txs.length - 1].txid;
          observer.disconnect();
          const last = txSection.lastElementChild;
          if (last) observer.observe(last);
        }
      } catch (e) {
        if (!signal.aborted) console.error("explorer addr txs:", e);
        pageIndex = chain.txCount;
      }
      loading = false;
    }

    await loadMore();
  } catch (e) {
    if (!signal.aborted) console.error("explorer addr:", e);
  }
}

export function show() { showPanel(el); }
export function hide() { hidePanel(el); }
