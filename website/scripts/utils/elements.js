import { getElementById } from "./dom.js";

export const style = getComputedStyle(window.document.documentElement);

export const headElement = window.document.getElementsByTagName("head")[0];
export const bodyElement = window.document.body;

export const mainElement = getElementById("main");
export const asideElement = getElementById("aside");
export const searchElement = getElementById("search");
export const navElement = getElementById("nav");
export const chartElement = getElementById("chart");
export const explorerElement = getElementById("explorer");
export const heatmapElement = getElementById("heatmap");

export const asideLabelElement = getElementById("aside-selector-label");
export const navLabelElement = getElementById(`nav-selector-label`);
export const searchLabelElement = getElementById(`search-selector-label`);
export const searchInput = /** @type {HTMLInputElement} */ (
  getElementById("search-input")
);
export const searchResultsElement = getElementById("search-results");
export const frameSelectorsElement = getElementById("frame-selectors");
export const layoutButtonElement = getElementById("layout-button");
