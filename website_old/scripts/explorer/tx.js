import { formatBtc, formatFeeRate, renderRows, renderTx, showPanel, hidePanel } from "./render.js";

/** @type {HTMLDivElement} */ let el;

/** @param {HTMLElement} parent @param {(e: MouseEvent) => void} linkHandler */
export function initTxDetails(parent, linkHandler) {
  el = document.createElement("div");
  el.id = "tx-details";
  el.hidden = true;
  parent.append(el);
  el.addEventListener("click", linkHandler);
}

export function show() { showPanel(el); }
export function hide() { hidePanel(el); }

export function clear() {
  if (el.children.length) {
    el.querySelector(".transactions")?.remove();
    for (const v of el.querySelectorAll(".row .value")) {
      v.classList.add("dim");
    }
  } else {
    const title = document.createElement("h1");
    title.textContent = "Transaction";
    el.append(title);
  }
}

/** @param {Transaction} tx */
export function update(tx) {
  el.innerHTML = "";

  const title = document.createElement("h1");
  title.textContent = "Transaction";
  el.append(title);

  const vsize = Math.ceil(tx.weight / 4);
  const feeRate = vsize > 0 ? tx.fee / vsize : 0;
  const totalIn = tx.vin.reduce((s, v) => s + (v.prevout?.value ?? 0), 0);
  const totalOut = tx.vout.reduce((s, v) => s + v.value, 0);

  renderRows(
    [
      ["TXID", tx.txid],
      [
        "Status",
        tx.status?.confirmed
          ? `Confirmed (block ${tx.status.blockHeight?.toLocaleString()})`
          : "Unconfirmed",
        tx.status?.blockHash ? `/block/${tx.status.blockHash}` : null,
      ],
      [
        "Timestamp",
        tx.status?.blockTime
          ? new Date(tx.status.blockTime * 1000).toUTCString()
          : "Pending",
      ],
      ["Size", `${tx.size.toLocaleString()} B`],
      ["Virtual Size", `${vsize.toLocaleString()} vB`],
      ["Weight", `${tx.weight.toLocaleString()} WU`],
      ["Fee", `${tx.fee.toLocaleString()} sat`],
      ["Fee Rate", `${formatFeeRate(feeRate)} sat/vB`],
      ["Inputs", `${tx.vin.length}`],
      ["Outputs", `${tx.vout.length}`],
      ["Total Input", `${formatBtc(totalIn)} BTC`],
      ["Total Output", `${formatBtc(totalOut)} BTC`],
      ["Version", `${tx.version}`],
      ["Locktime", `${tx.locktime}`],
    ],
    el,
  );

  const section = document.createElement("div");
  section.classList.add("transactions");
  const heading = document.createElement("h2");
  heading.textContent = "Inputs & Outputs";
  section.append(heading, renderTx(tx));
  el.append(section);
}
