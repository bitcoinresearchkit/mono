import { createHeader } from "../../scripts/utils/dom.js";
import { heatmapElement } from "../../scripts/utils/elements.js";
import { debounce, next } from "../../scripts/utils/timing.js";
import { createHeatmapControls } from "./controls/index.js";
import { createHeatmapLoader } from "./loader.js";
import { createRenderer } from "./renderer.js";
import { createTooltipView } from "./tooltip/view.js";

/** @type {ReturnType<typeof createRenderer> | undefined} */
let renderer;
/** @type {HTMLCanvasElement | undefined} */
let canvas;
/** @type {ReturnType<typeof createTooltipView> | undefined} */
let tooltipView;
/** @type {ReturnType<typeof createHeatmapControls> | undefined} */
let controls;
/** @type {ReturnType<typeof createHeatmapLoader> | undefined} */
let loader;
/** @type {HTMLHeadingElement | undefined} */
let headingElement;
/** @type {HeatmapOption | undefined} */
let currentOption;
/** @type {HeatmapGrid | undefined} */
let currentGrid;
const dirtyCols = new Set();
let paintScheduled = false;
let initialized = false;
let from = "";
let to = "";
/** @type {number | undefined} */
let yMin;
/** @type {number | undefined} */
let yMax;

/**
 * Initializes the heatmap pane once for the app lifetime.
 */
export function init() {
  if (initialized) return;
  initialized = true;

  const header = createHeader();
  headingElement = header.headingElement;
  const { headerElement } = header;
  controls = createHeatmapControls({
    onRangeChange(range) {
      from = range.from;
      to = range.to;
      hideTooltip();
      loadRange();
    },
    onYRangeChange(range) {
      yMin = range.yMin;
      yMax = range.yMax;
      hideTooltip();
      rebuildAndLoadVisibleDates();
    },
  });

  heatmapElement.append(headerElement);
  heatmapElement.append(controls.element);

  canvas = document.createElement("canvas");
  heatmapElement.append(canvas);
  renderer = createRenderer(canvas);
  loader = createHeatmapLoader({ addDateToGrid, rebuildGrid, paint });
  tooltipView = createTooltipView(heatmapElement);

  canvas.addEventListener("pointermove", updateHoverTooltip);
  canvas.addEventListener("pointerdown", updateTapTooltip);
  canvas.addEventListener("pointerleave", hideHoverTooltip);
  canvas.addEventListener("pointercancel", hideTooltip);

  void next().then(resizeAndRebuild);

  new ResizeObserver(
    debounce(() => {
      resizeAndRebuild();
    }, 250),
  ).observe(heatmapElement);
}

/** @param {HeatmapOption} option */
export function setOption(option) {
  init();
  if (currentOption !== option) {
    currentOption = option;
    loader?.reset();
    const selection = controls?.setOption(option);
    if (selection) {
      from = selection.from;
      to = selection.to;
      yMin = selection.yMin;
      yMax = selection.yMax;
    }
    if (headingElement) headingElement.textContent = option.title;
    hideTooltip();
  }
  loadRange();
}

function resizeAndRebuild() {
  if (!canvas || !renderer) return;
  const { width, height } = canvas.getBoundingClientRect();
  if (renderer.resize(width, height)) rebuildAndLoadVisibleDates();
}

function loadRange() {
  if (!currentOption || !loader) return;
  loader.setRange({ option: currentOption, from, to });
  rebuildAndLoadVisibleDates();
}

function rebuildAndLoadVisibleDates() {
  rebuildGrid();
  loadVisibleDates();
}

function rebuildGrid() {
  const dates = loader?.dates;
  if (
    !currentOption ||
    !renderer ||
    !loader ||
    !dates?.length ||
    renderer.width < 1 ||
    renderer.height < 1
  ) {
    currentGrid = undefined;
    return;
  }

  currentGrid = currentOption.grid.create({
    dates,
    width: renderer.width,
    height: renderer.height,
    yMin,
    yMax,
  });

  for (const dateIndex of getVisibleDateIndexes(currentGrid)) {
    const points = loader.getPoint(dates[dateIndex]);
    if (points) currentGrid.add(dateIndex, points);
  }

  paint();
}

function loadVisibleDates() {
  if (!currentOption || !loader || !currentGrid) return;
  loader.load({
    option: currentOption,
    dateIndexes: getVisibleDateIndexes(currentGrid),
  });
}

/**
 * @param {HeatmapGrid} grid
 * @returns {number[]}
 */
function getVisibleDateIndexes(grid) {
  /** @type {number[]} */
  const indexes = [];
  let previousDateIndex = -1;
  for (let col = 0; col < grid.cols; col++) {
    const dateIndex = grid.getDateIndexRange(col).end;
    if (!Number.isInteger(dateIndex) || dateIndex === previousDateIndex) {
      continue;
    }
    previousDateIndex = dateIndex;
    indexes.push(dateIndex);
  }
  return indexes;
}

/**
 * @param {number} dateIndex
 * @param {HeatmapPoints} points
 */
function addDateToGrid(dateIndex, points) {
  if (!currentGrid) return;
  const result = currentGrid.add(dateIndex, points);
  if (!result) return;
  if (result.maxChanged) {
    paint();
  } else {
    schedulePaint(result.col);
  }
}

/** @param {number} col */
function schedulePaint(col) {
  dirtyCols.add(col);
  if (paintScheduled) return;
  paintScheduled = true;
  requestAnimationFrame(() => {
    paintScheduled = false;
    if (!dirtyCols.size) return;
    paint(dirtyCols);
    dirtyCols.clear();
  });
}

/** @param {Iterable<number>} [dirty] */
function paint(dirty) {
  if (!renderer || !currentGrid || !currentOption) return;
  const grid = currentGrid;
  const option = currentOption;
  renderer.paint(
    grid.cols,
    grid.rows,
    (col, row) => option.color(grid.getValue(col, row), { grid, col, row }),
    dirty,
  );
}

/** @param {PointerEvent} event */
function updateHoverTooltip(event) {
  if (event.pointerType !== "mouse") return;
  updateTooltip(event, "auto");
}

/** @param {PointerEvent} event */
function updateTapTooltip(event) {
  if (event.pointerType === "mouse") return;
  updateTooltip(event, "above");
}

/** @param {PointerEvent} event */
function hideHoverTooltip(event) {
  if (event.pointerType === "mouse") hideTooltip();
}

/**
 * @param {PointerEvent} event
 * @param {"auto" | "above"} placement
 */
function updateTooltip(event, placement) {
  if (!canvas || !currentGrid || !currentOption?.tooltip || !tooltipView) {
    hideTooltip();
    return;
  }
  const rect = canvas.getBoundingClientRect();
  const col = Math.floor(
    ((event.clientX - rect.left) * currentGrid.cols) / rect.width,
  );
  const row = Math.floor(
    ((event.clientY - rect.top) * currentGrid.rows) / rect.height,
  );
  if (
    col < 0 ||
    col >= currentGrid.cols ||
    row < 0 ||
    row >= currentGrid.rows
  ) {
    hideTooltip();
    return;
  }
  if (currentGrid.getCount(col, row) === 0) {
    hideTooltip();
    return;
  }
  tooltipView.show(
    event,
    currentOption.tooltip({
      option: currentOption,
      grid: currentGrid,
      col,
      row,
    }),
    { placement },
  );
}

function hideTooltip() {
  tooltipView?.hide();
}
