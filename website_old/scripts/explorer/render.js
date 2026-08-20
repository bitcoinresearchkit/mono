export const TX_PAGE_SIZE = 25;

/** @param {HTMLElement} el */
export function showPanel(el) {
  el.hidden = false;
  el.scrollTop = 0;
}

/** @param {HTMLElement} el */
export function hidePanel(el) {
  el.hidden = true;
}

/** @param {number} sats */
export function formatBtc(sats) {
  return (sats / 1e8).toFixed(8);
}

/** @param {number} rate */
export function formatFeeRate(rate) {
  if (rate >= 1_000_000) return `${(rate / 1_000_000).toFixed(1)}M`;
  if (rate >= 100_000) return `${Math.round(rate / 1_000)}k`;
  if (rate >= 1_000) return `${(rate / 1_000).toFixed(1)}k`;
  if (rate >= 100) return Math.round(rate).toLocaleString();
  if (rate >= 10) return rate.toFixed(1);
  return rate.toFixed(2);
}

/** @param {string} text @param {HTMLElement} el */
export function setAddrContent(text, el) {
  el.textContent = "";
  if (text.length <= 6) {
    el.textContent = text;
    return;
  }
  const head = document.createElement("span");
  head.classList.add("addr-head");
  head.textContent = text.slice(0, -6);
  const tail = document.createElement("span");
  tail.classList.add("addr-tail");
  tail.textContent = text.slice(-6);
  el.append(head, tail);
}

/** @param {number} height */
export function formatHeightPrefix(height) {
  return "#" + "0".repeat(Math.max(0, 7 - String(height).length));
}

/** @param {number} height */
export function createHeightElement(height) {
  const container = document.createElement("span");
  const str = height.toString();
  const prefix = document.createElement("span");
  prefix.classList.add("dim");
  prefix.style.userSelect = "none";
  prefix.textContent = formatHeightPrefix(height);
  const num = document.createElement("span");
  num.textContent = str;
  container.append(prefix, num);
  return container;
}

/**
 * @param {string} label
 * @param {boolean} [isLink]
 * @returns {{ row: HTMLDivElement, valueEl: HTMLElement }}
 */
export function createRow(label, isLink = false) {
  const row = document.createElement("div");
  row.classList.add("row");
  const labelEl = document.createElement("span");
  labelEl.classList.add("label");
  labelEl.textContent = label;
  const valueEl = document.createElement(isLink ? "a" : "span");
  valueEl.classList.add("value");
  row.append(labelEl, valueEl);
  return { row, valueEl };
}

/**
 * @param {[string, string, (string | null)?][]} rows
 * @param {HTMLElement} parent
 */
export function renderRows(rows, parent) {
  for (const [label, value, href] of rows) {
    const { row, valueEl } = createRow(label, Boolean(href));
    valueEl.textContent = value;
    if (href) /** @type {HTMLAnchorElement} */ (valueEl).href = href;
    parent.append(row);
  }
}

/**
 * @param {TxIn} vin
 * @param {string} [coinbaseAscii]
 */
function renderInput(vin, coinbaseAscii) {
  const row = document.createElement("div");
  row.classList.add("tx-io");
  const addr = document.createElement("span");
  addr.classList.add("addr");
  if (vin.isCoinbase) {
    addr.textContent = "Coinbase";
    addr.classList.add("coinbase");
    if (coinbaseAscii) {
      const sig = document.createElement("div");
      sig.classList.add("coinbase-sig");
      sig.textContent = coinbaseAscii;
      row.append(sig);
    }
  } else {
    const addrStr = /** @type {string | undefined} */ (
      /** @type {any} */ (vin.prevout)?.scriptpubkey_address
    );
    if (addrStr) {
      const link = document.createElement("a");
      link.href = `/address/${addrStr}`;
      setAddrContent(addrStr, link);
      addr.append(link);
    } else {
      addr.textContent = "Unknown";
    }
  }
  const amt = document.createElement("span");
  amt.classList.add("amount");
  amt.textContent = vin.prevout ? `${formatBtc(vin.prevout.value)} BTC` : "";
  row.append(addr, amt);
  return row;
}

/** @param {TxOut} vout */
function renderOutput(vout) {
  const row = document.createElement("div");
  row.classList.add("tx-io");
  const addr = document.createElement("span");
  addr.classList.add("addr");
  const type = /** @type {string | undefined} */ (
    /** @type {any} */ (vout).scriptpubkey_type
  );
  const a = /** @type {string | undefined} */ (
    /** @type {any} */ (vout).scriptpubkey_address
  );
  if (type === "op_return") {
    addr.textContent = "OP_RETURN";
    addr.classList.add("op-return");
  } else if (a) {
    const link = document.createElement("a");
    link.href = `/address/${a}`;
    setAddrContent(a, link);
    addr.append(link);
  } else {
    setAddrContent(vout.scriptpubkey, addr);
  }
  const amt = document.createElement("span");
  amt.classList.add("amount");
  amt.textContent = `${formatBtc(vout.value)} BTC`;
  row.append(addr, amt);
  return row;
}

/**
 * @template T
 * @param {T[]} items
 * @param {(item: T) => HTMLElement} render
 * @param {HTMLElement} container
 */
function renderCapped(items, render, container, max = 10) {
  const limit = Math.min(items.length, max);
  for (let i = 0; i < limit; i++) container.append(render(items[i]));
  if (items.length > max) {
    const btn = document.createElement("button");
    btn.classList.add("show-more");
    btn.textContent = `Show ${items.length - max} more`;
    btn.addEventListener("click", () => {
      btn.remove();
      for (let i = max; i < items.length; i++) container.append(render(items[i]));
    });
    container.append(btn);
  }
}

/** @param {Transaction} tx @param {string} [coinbaseAscii] */
export function renderTx(tx, coinbaseAscii) {
  const el = document.createElement("div");
  el.classList.add("tx");

  const head = document.createElement("div");
  head.classList.add("tx-head");
  const txidEl = document.createElement("a");
  txidEl.classList.add("txid");
  txidEl.textContent = tx.txid;
  txidEl.href = `/tx/${tx.txid}`;
  head.append(txidEl);
  if (tx.status?.blockTime) {
    const time = document.createElement("span");
    time.classList.add("tx-time");
    time.textContent = new Date(tx.status.blockTime * 1000).toLocaleString();
    head.append(time);
  }
  el.append(head);

  const body = document.createElement("div");
  body.classList.add("tx-body");

  const inputs = document.createElement("div");
  inputs.classList.add("tx-inputs");
  renderCapped(tx.vin, (vin) => renderInput(vin, coinbaseAscii), inputs);

  const outputs = document.createElement("div");
  outputs.classList.add("tx-outputs");
  renderCapped(tx.vout, renderOutput, outputs);

  const totalOut = tx.vout.reduce((s, v) => s + v.value, 0);

  body.append(inputs, outputs);
  el.append(body);

  const foot = document.createElement("div");
  foot.classList.add("tx-foot");
  const feeInfo = document.createElement("span");
  const vsize = Math.ceil(tx.weight / 4);
  const feeRate = vsize > 0 ? tx.fee / vsize : 0;
  feeInfo.textContent = `${formatFeeRate(feeRate)} sat/vB \u2013 ${tx.fee.toLocaleString()} sats`;
  const total = document.createElement("span");
  total.classList.add("amount", "total");
  total.textContent = `${formatBtc(totalOut)} BTC`;
  foot.append(feeInfo, total);
  el.append(foot);

  return el;
}
