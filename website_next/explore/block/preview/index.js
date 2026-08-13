import { loadBlockPreview } from "./data.js";
import { createBlockPreviewFilterData } from "./filters/data.js";
import {
  createPendingPreviewFilters,
  createPreviewFilters,
} from "./filters/index.js";
import { createBlockPreviewHeatmap } from "./heatmap/index.js";
import { createBlockPreviewInspector } from "./inspector.js";

function noop() {}

/**
 * @param {HTMLElement} body
 * @param {HTMLElement} filters
 * @param {HTMLElement} [inspector]
 */
function createFigure(body, filters, inspector) {
  const figure = document.createElement("figure");
  const caption = document.createElement("figcaption");

  figure.dataset.blockPreviewFigure = "";
  caption.dataset.blockPreviewLegend = "";
  caption.append(filters);
  figure.append(caption, body);
  if (inspector) figure.append(inspector);

  return figure;
}

/**
 * @param {BlockPreviewData} data
 * @param {number} height
 * @param {AbortSignal} signal
 */
function createPreview(data, height, signal) {
  const filterData = createBlockPreviewFilterData(height, data.range, signal);
  const inspector = createBlockPreviewInspector(signal, filterData);
  const heatmap = createBlockPreviewHeatmap(data, {
    onInspect: inspector.inspect,
  });
  const filters = createPreviewFilters(filterData, heatmap);

  return {
    destroy() {
      inspector.destroy();
      filters.destroy();
      heatmap.destroy();
    },
    element: createFigure(heatmap.element, filters.element, inspector.element),
  };
}

/**
 * @param {HTMLElement} content
 * @param {string} status
 */
function renderStatus(content, status) {
  const p = document.createElement("p");

  p.dataset.blockPreviewStatus = status;
  p.textContent = status;
  content.replaceChildren(createFigure(p, createPendingPreviewFilters()));
}

/**
 * @param {import("../../../modules/brk-client/index.js").BlockInfoV1} block
 */
export function createBlockPreviewPane(block) {
  const content = document.createElement("div");
  const controller = new AbortController();
  /** @type {ReturnType<typeof loadBlockPreview> | null} */
  let data = null;
  let destroyHeatmap = noop;
  let live = true;

  content.dataset.blockPreview = "";
  renderStatus(content, "Loading");

  function load() {
    if (data) return data;

    data = loadBlockPreview(block, controller.signal);

    void data
      .then((data) => {
        if (!live) return;
        const preview = createPreview(data, block.height, controller.signal);

        destroyHeatmap = preview.destroy;
        content.replaceChildren(preview.element);
      })
      .catch((error) => {
        if (!live) return;
        console.error(error);
        renderStatus(content, "Unavailable");
      });

    return data;
  }

  return {
    element: content,
    load,
    destroy() {
      live = false;
      controller.abort();
      destroyHeatmap();
      destroyHeatmap = noop;
    },
  };
}

/** @typedef {import("./data.js").BlockPreviewData} BlockPreviewData */

/**
 * @typedef {Object} BlockPreview
 * @property {() => void} destroy
 * @property {HTMLElement} element
 */
