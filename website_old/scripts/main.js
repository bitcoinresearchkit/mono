import { initPrice, onPrice } from "./utils/price.js";
import { brk } from "./utils/client.js";
import { onFirstIntersection, getElementById, isHidden } from "./utils/dom.js";
import { initOptions } from "./options/full.js";
import {
  init as initChart,
  setOption as setChartOption,
} from "./panes/chart.js";
import { init as initExplorer } from "./explorer/index.js";
import { init as initSearch } from "./panes/search.js";
import { setOption as setHeatmapOption } from "../src/heatmap/index.js";
import { readStored, removeStored, writeToStorage } from "./utils/storage.js";
import {
  asideElement,
  asideLabelElement,
  chartElement,
  explorerElement,
  frameSelectorsElement,
  mainElement,
  navElement,
  navLabelElement,
  searchElement,
  layoutButtonElement,
  heatmapElement,
  style,
} from "./utils/elements.js";
import { idle } from "./utils/timing.js";

const DESKTOP_QUERY = window.matchMedia("(min-width: 768px)");

const SPLIT = "split";

function updateLayout() {
  const pref = readStored("split-view") !== "false";
  const wasSplit = isSplit();
  document.documentElement.dataset.layout =
    DESKTOP_QUERY.matches && pref ? "split" : "full";
  if (isSplit() !== wasSplit) syncFrame();
}

function isSplit() {
  return document.documentElement.dataset.layout === SPLIT;
}

function syncFrame() {
  if (isSplit()) navLabelElement.click();
  else asideLabelElement.click();
}

DESKTOP_QUERY.addEventListener("change", updateLayout);
updateLayout();

layoutButtonElement.addEventListener("click", () => {
  writeToStorage("split-view", String(!isSplit()));
  updateLayout();
});

function initFrameSelectors() {
  const children = Array.from(frameSelectorsElement.children);

  /** @type {HTMLElement | undefined} */
  let focusedFrame = undefined;

  for (let i = 0; i < children.length; i++) {
    const element = children[i];

    switch (element.tagName) {
      case "LABEL": {
        element.addEventListener("click", () => {
          const inputId = element.getAttribute("for");

          if (!inputId) {
            console.log(element, element.getAttribute("for"));
            throw "Input id in label not found";
          }

          const input = window.document.getElementById(inputId);

          if (!input || !("value" in input)) {
            throw "Not input or no value";
          }

          const frame = window.document.getElementById(
            /** @type {string} */ (input.value),
          );

          if (!frame) {
            console.log(input.value);
            throw "Frame element doesn't exist";
          }

          if (frame === focusedFrame) {
            return;
          }

          frame.hidden = false;
          if (focusedFrame) {
            focusedFrame.hidden = true;
          }
          focusedFrame = frame;
        });
        break;
      }
    }
  }

  syncFrame();
}
initFrameSelectors();

initPrice(brk);

onPrice((price) => {
  console.log("close:", price);
  window.document.title = `${price.toLocaleString("en-us")} | bitview`;
});

const options = initOptions();

window.addEventListener("popstate", () => options.resolveUrl());

function initSelected() {
  let firstRun = true;
  function initSelectedFrame() {
    if (!firstRun) throw Error("Unreachable");
    firstRun = false;

    let previousElement = /** @type {HTMLElement | undefined} */ (undefined);
    let firstTimeLoadingChart = true;
    let firstTimeLoadingExplorer = true;

    options.selected.onChange((option) => {
      /** @type {HTMLElement | undefined} */
      let element;

      console.log(option);

      switch (option.kind) {
        case "explorer": {
          element = explorerElement;

          if (firstTimeLoadingExplorer) {
            initExplorer(options.selected);
          }
          firstTimeLoadingExplorer = false;

          break;
        }
        case "heatmap": {
          element = heatmapElement;

          setHeatmapOption(option);

          break;
        }
        case "chart": {
          element = chartElement;

          if (firstTimeLoadingChart) {
            initChart();
          }
          firstTimeLoadingChart = false;

          setChartOption(option);

          break;
        }
        case "link": {
          return;
        }
      }

      if (!element) throw "Element should be set";

      if (element !== previousElement) {
        if (previousElement) previousElement.hidden = true;
        element.hidden = false;
      }

      previousElement = element;
    });
  }

  let firstMobileSwitch = true;
  options.selected.onChange(() => {
    if (!firstMobileSwitch && !isHidden(asideLabelElement)) {
      asideLabelElement.click();
    }
    firstMobileSwitch = false;
  });

  onFirstIntersection(asideElement, initSelectedFrame);
}
initSelected();

idle(() => options.setParent(navElement));

onFirstIntersection(navElement, () => {
  options.setParent(navElement);

  navElement
    .querySelector(`a[href="${window.document.location.pathname}"]`)
    ?.scrollIntoView({
      behavior: "instant",
      block: "center",
    });
});

function initResizeBar() {
  const bar = getElementById("resize-bar");
  const key = "bar-width";
  const root = document.documentElement;
  const max = () =>
    (parseFloat(style.getPropertyValue("--max-main-width")) / 100) *
    window.innerWidth;

  const saved = readStored(key);
  if (saved) root.style.setProperty("--sidebar-width", `${saved}px`);

  /** @param {number | null} width */
  function setWidth(width) {
    if (width != null) {
      const clamped = Math.min(width, max());
      root.style.setProperty("--sidebar-width", `${clamped}px`);
      writeToStorage(key, String(clamped));
    } else {
      root.style.removeProperty("--sidebar-width");
      removeStored(key);
    }
  }

  bar.addEventListener("pointerdown", (e) => {
    e.preventDefault();
    bar.setPointerCapture(e.pointerId);
    const startX = e.clientX;
    const startW = mainElement.clientWidth;
    document.documentElement.dataset.resize = "";

    /** @param {PointerEvent} e */
    function onMove(e) {
      setWidth(startW + (e.clientX - startX));
    }

    function onUp() {
      delete document.documentElement.dataset.resize;
      bar.removeEventListener("pointermove", onMove);
      bar.removeEventListener("pointerup", onUp);
      bar.removeEventListener("pointercancel", onUp);
    }

    bar.addEventListener("pointermove", onMove);
    bar.addEventListener("pointerup", onUp);
    bar.addEventListener("pointercancel", onUp);
  });

  bar.addEventListener("dblclick", () => setWidth(null));
}
initResizeBar();

onFirstIntersection(searchElement, () => {
  initSearch(options);
});
