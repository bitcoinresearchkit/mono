import { createQrDataUrl } from "../../../qr/index.js";
import { formatDisplayUrl } from "./data.js";

/** @typedef {import("../../../modules/bitview-client/index.js").BlockInfoV1} Block */

/**
 * @param {Block} block
 * @param {string} url
 */
export function createReceiptQr(block, url) {
  const section = document.createElement("section");
  const label = document.createElement("span");
  const image = document.createElement("img");
  const link = document.createElement("a");

  section.dataset.receiptQr = "";
  label.textContent = "Scan to verify";
  image.alt = `QR code for block ${block.height}`;
  image.src = createQrDataUrl(url, { scale: 8 });
  link.href = url;
  link.textContent = formatDisplayUrl(url);
  section.append(label, image, link);

  return section;
}
