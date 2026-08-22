import { explorerElement } from "../utils/elements.js";
import { brk } from "../utils/client.js";
import {
  initChain,
  goToCube,
  poll,
  selectCube,
  deselectCube,
} from "../../src/explorer/chain/index.js";
import {
  initBlockDetails,
  update as updateBlock,
  show as showBlock,
  hide as hideBlock,
} from "./block.js";
import {
  initTxDetails,
  update as updateTx,
  clear as clearTx,
  show as showTx,
  hide as hideTx,
} from "./tx.js";
import {
  initAddrDetails,
  update as updateAddr,
  show as showAddr,
  hide as hideAddr,
} from "./address.js";

/** @type {number | undefined} */ let pollInterval;
let navController = new AbortController();
let lastLoadedUrl = "";

function navigate() {
  navController.abort();
  navController = new AbortController();
  lastLoadedUrl = window.location.pathname;
  return navController.signal;
}

function showPanel(/** @type {"block" | "tx" | "addr"} */ which) {
  which === "block" ? showBlock() : hideBlock();
  which === "tx" ? showTx() : hideTx();
  which === "addr" ? showAddr() : hideAddr();
}

/** @param {MouseEvent} e */
function handleLinkClick(e) {
  const a = /** @type {HTMLAnchorElement | null} */ (
    /** @type {HTMLElement} */ (e.target).closest("a[href]")
  );
  if (!a) return;
  const m = a.pathname.match(/^\/(block|tx|address)\/(.+)/);
  if (!m) return;
  e.preventDefault();
  history.pushState(null, "", a.href);
  if (m[1] === "block") {
    navigateToBlock(m[2]);
  } else if (m[1] === "tx") {
    navigateToTx(m[2]);
  } else {
    navigateToAddr(m[2]);
  }
}

/** @param {{ onChange: (cb: (option: Option) => void) => void }} selected */
export function init(selected) {
  initChain(explorerElement, {
    onSelect: (block) => {
      updateBlock(block);
      showPanel("block");
    },
    onCubeClick: (cube) => {
      history.pushState(null, "", cube.href);
      navigate();
      selectCube(cube);
    },
    onTip: () => {
      history.pushState(null, "", "/block/tip");
      navigate();
      goToCube(null);
    },
    onGenesis: () => {
      history.pushState(null, "", "/block/0");
      navigate();
      goToCube(0);
    },
  });

  initBlockDetails(explorerElement, handleLinkClick);
  initTxDetails(explorerElement, handleLinkClick);
  initAddrDetails(explorerElement, handleLinkClick);

  new MutationObserver(() => {
    if (explorerElement.hidden) stopPolling();
    else startPolling();
  }).observe(explorerElement, {
    attributes: true,
    attributeFilter: ["hidden"],
  });

  document.addEventListener("visibilitychange", () => {
    if (!document.hidden && !explorerElement.hidden) poll();
  });

  selected.onChange((option) => {
    if (option.kind === "explorer") {
      const url = window.location.pathname;
      if (url !== lastLoadedUrl) load();
    }
  });
}

function startPolling() {
  stopPolling();
  poll();
  pollInterval = setInterval(poll, 1_000);
}

function stopPolling() {
  if (pollInterval !== undefined) {
    clearInterval(pollInterval);
    pollInterval = undefined;
  }
}

async function load() {
  const signal = navigate();
  try {
    const [kind, value] = window.location.pathname.split("/").filter(Boolean);

    if (kind === "tx" && value) {
      const txid = await resolveTxid(value, { signal });
      if (signal.aborted) return;
      const tx = await brk.getTx(txid, { signal });
      if (signal.aborted) return;
      await goToCube(tx.status?.blockHash ?? tx.status?.blockHeight ?? null, { silent: true });
      updateTx(tx);
      showPanel("tx");
      return;
    }

    if (kind === "address" && value) {
      await goToCube(null, { silent: true });
      navigateToAddr(value);
      return;
    }

    await goToCube(kind === "block" ? value : null);
  } catch (e) {
    if (signal.aborted) return;
    console.error("explorer load:", e);
    await goToCube();
    showPanel("block");
  }
}

/** @param {string} hashOrHeight */
async function navigateToBlock(hashOrHeight) {
  navigate();
  await goToCube(hashOrHeight);
}

/** @param {Txid | TxIndex} value @param {{ signal?: AbortSignal }} [options] */
async function resolveTxid(value, { signal } = {}) {
  return typeof value === "number" || /^\d+$/.test(value)
    ? await brk.getTxByIndex(Number(value), { signal })
    : value;
}

/** @param {Txid | TxIndex} txidOrIndex */
async function navigateToTx(txidOrIndex) {
  const signal = navigate();
  clearTx();
  showPanel("tx");
  try {
    const txid = await resolveTxid(txidOrIndex, { signal });
    if (signal.aborted) return;
    const tx = await brk.getTx(txid, { signal });
    if (signal.aborted) return;
    await goToCube(tx.status?.blockHash ?? tx.status?.blockHeight ?? null, { silent: true });
    updateTx(tx);
  } catch (e) {
    if (!signal.aborted) {
      console.error("explorer tx:", e);
      await goToCube();
      showPanel("block");
    }
  }
}

/** @param {string} address */
function navigateToAddr(address) {
  const signal = navigate();
  deselectCube();
  updateAddr(address, signal);
  showPanel("addr");
}
