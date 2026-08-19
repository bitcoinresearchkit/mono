import { createBlockHeader } from "./header/index.js";
import { createMinerPane } from "./miner/index.js";
import { createDifficultyPane } from "./difficulty/index.js";
import { createRewardsPane } from "./rewards/index.js";
import { createTransactionPane } from "./transactions/index.js";
import { createFeeChart } from "./fee-chart/index.js";
import { createBlockPreviewPane } from "./preview/index.js";
import { appendPane } from "./pane.js";
import { createBlockReceipt } from "./receipt/index.js";
import { createBlockXrayPane } from "./xray/index.js";

function noop() {}

/** @param {string} side */
function createColumn(side) {
  const column = document.createElement("div");

  column.dataset.blockColumn = side;

  return column;
}

/** @param {{ onBack?: () => void }} [options] */
export function createBlockDetails({ onBack = () => {} } = {}) {
  const element = document.createElement("section");
  const back = document.createElement("button");
  const receipt = createBlockReceipt();
  const header = createBlockHeader([receipt.button]);
  const content = document.createElement("div");
  let activatePreview = noop;
  let destroyPreview = noop;
  let destroyFeeChart = noop;
  let destroyXray = noop;

  element.id = "block-details";
  back.type = "button";
  back.dataset.blockBack = "";
  back.textContent = "← Chain";
  back.addEventListener("click", onBack);
  element.append(back, header.element, content);

  function clearContent() {
    activatePreview = noop;
    destroyPreview();
    destroyPreview = noop;
    destroyFeeChart();
    destroyFeeChart = noop;
    destroyXray();
    destroyXray = noop;

    content.textContent = "";
  }

  /** @param {import("../../modules/bitview-client/index.js").BlockInfoV1} block */
  function update(block) {
    const extras = block.extras;

    header.update(block);
    receipt.update(block);

    clearContent();

    const preview = createBlockPreviewPane(block);
    const feeChart = createFeeChart(extras.feeRange, extras.avgFeeRate);
    const left = createColumn("main");
    const right = createColumn("side");

    destroyPreview = preview.destroy;
    destroyFeeChart = feeChart.destroy;
    appendPane(left, "preview", [
      createTransactionPane(block),
      preview.element,
    ]);
    appendPane(right, "mining", [createMinerPane(block)]);
    appendPane(right, "rewards", [createRewardsPane(extras)]);
    appendPane(right, "difficulty", [createDifficultyPane(block)]);
    appendPane(right, "fees", [feeChart.element]);
    content.append(left, right);

    activatePreview = () => {
      const xray = createBlockXrayPane(preview.load(), element);

      destroyXray = xray.destroy;
      content.append(xray.element);
    };
  }

  function activate() {
    activatePreview();
    activatePreview = noop;
  }

  return /** @type {const} */ ({
    activate,
    element,
    update,
  });
}
