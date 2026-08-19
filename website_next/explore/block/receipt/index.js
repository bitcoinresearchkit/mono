import { createBrand } from "../../../brand/index.js";
import { openDialog } from "../../../dialog/index.js";
import { getCoinbaseMessage } from "../format.js";
import { createBlockUrl, getReceiptSections } from "./data.js";
import { createReceiptQr } from "./qr.js";
import { createBlockTitle } from "../title/index.js";

/** @typedef {import("../../../modules/bitview-client/index.js").BlockInfoV1} Block */
/** @typedef {import("./data.js").ReceiptRow} ReceiptRow */
/** @typedef {import("./data.js").ReceiptSection} ReceiptSection */

/** @param {ReceiptRow} data */
function createReceiptRow(data) {
  const row = document.createElement("div");
  const name = document.createElement("span");
  const amount = document.createElement("strong");

  row.dataset.receiptRow = data.variant ?? (data.detail ? "amount" : "");
  name.textContent = data.label;
  amount.textContent = data.value;
  if (data.detail) {
    const detail = document.createElement("small");

    detail.textContent = data.detail;
    amount.append(detail);
  }
  row.append(name, amount);

  return row;
}

/** @param {ReceiptSection} data */
function createReceiptSection(data) {
  const section = document.createElement("section");
  const heading = document.createElement("h3");

  section.dataset.receiptSection = "";
  heading.textContent = data.title;
  section.append(heading, ...data.rows.map(createReceiptRow));

  return section;
}

/** @param {string} message */
function createCoinbaseNote(message) {
  const note = document.createElement("p");

  note.dataset.receiptNote = "";
  note.textContent = message;

  return note;
}

/** @param {Block} block */
function createReceiptHead(block) {
  const head = document.createElement("header");
  const title = document.createElement("h2");

  head.dataset.receiptHead = "";
  title.dataset.blockTitleText = "";
  title.append(...createBlockTitle(block.height));
  head.append(title);

  return head;
}

function createReceiptBrand() {
  const section = document.createElement("section");

  section.dataset.receiptBrand = "";
  section.append(createBrand({ tone: "receipt", fill: 1 }));

  return section;
}

/** @param {Block} block */
function openReceiptDialog(block) {
  const dialog = document.createElement("dialog");
  const content = document.createElement("main");
  const paper = document.createElement("article");
  const controls = document.createElement("footer");
  const print = document.createElement("button");
  const closeForm = document.createElement("form");
  const close = document.createElement("button");
  const url = createBlockUrl(block);
  const coinbase = getCoinbaseMessage(block.extras.coinbaseSignatureAscii);

  dialog.dataset.blockReceipt = "";
  dialog.tabIndex = -1;
  paper.dataset.receiptPaper = "";
  controls.dataset.receiptControls = "";
  print.type = "button";
  print.dataset.receiptAction = "print";
  print.textContent = "Print";
  closeForm.method = "dialog";
  close.type = "submit";
  close.dataset.receiptAction = "close";
  close.textContent = "Close";
  closeForm.append(close);
  controls.append(closeForm, print);
  paper.append(
    createReceiptHead(block),
    ...getReceiptSections(block).map(createReceiptSection),
    ...(coinbase ? [createCoinbaseNote(coinbase)] : []),
    createReceiptQr(block, url),
    createReceiptBrand(),
  );
  content.append(paper);
  dialog.append(content, controls);

  print.addEventListener("click", () => {
    window.print();
  });
  openDialog(dialog, document.body);
  dialog.focus({ preventScroll: true });
  content.scrollTop = 0;
}

export function createBlockReceipt() {
  const button = document.createElement("button");
  /** @type {Block | null} */
  let block = null;

  button.type = "button";
  button.disabled = true;
  button.dataset.receiptButton = "";
  button.textContent = "Receipt";
  button.addEventListener("click", () => {
    if (block) openReceiptDialog(block);
  });

  /** @param {Block} nextBlock */
  function update(nextBlock) {
    block = nextBlock;
    button.disabled = false;
  }

  return /** @type {const} */ ({
    button,
    update,
  });
}
