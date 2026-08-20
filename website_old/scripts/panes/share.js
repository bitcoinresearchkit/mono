import { getElementById } from "../utils/dom.js";
import * as leanQr from "../modules/lean-qr/2.7.1/index.mjs";

const shareDiv = getElementById("share-div");
const shareContentDiv = getElementById("share-content-div");
const shareButton = getElementById("share-button");
const imgQrcode = /** @type {HTMLImageElement} */ (getElementById("share-img"));
const anchor = /** @type {HTMLAnchorElement} */ (
  getElementById("share-anchor")
);

/** @param {string | null} url */
export function setQr(url) {
  if (!url) {
    shareDiv.hidden = true;
    return;
  }

  anchor.href = url;
  anchor.innerText =
    (url.startsWith("http") ? url.split("//").at(-1) : url.split(":").at(-1)) ||
    "";

  imgQrcode.src =
    // @ts-ignore — lean-qr types don't resolve for file path import
    leanQr.generate(url)?.toDataURL({ padX: 0, padY: 0 }) || "";

  shareDiv.hidden = false;
}

shareButton.addEventListener("click", () => {
  setQr(window.location.href);
});

shareDiv.addEventListener("click", () => {
  setQr(null);
});

shareContentDiv.addEventListener("click", (event) => {
  event.stopPropagation();
  event.preventDefault();
});
