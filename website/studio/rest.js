// @ts-check
// The rest (see later.js): the whole client first, then ⌘K, whose imports keep the others' order. Lilex isn't on a
// first screen: asked for, not waited on (its fallback fits its metrics).
import "/studio/client.js";
export * as command from "/studio/command.js";
export * as pen from "/studio/drawing.js";
export * as grid from "/studio/grid.js";
export * as heatmaps from "/studio/heatmaps.js";
export * as inspector from "/studio/inspector.js";
export * as menus from "/studio/menus.js";
export * as presetsEdit from "/studio/presets-edit.js";
export * as gestures from "/studio/gestures.js";
export * as xkcd from "/modules/xkcd-colors/index.js";

document.fonts.load("590 10px Lilex").catch(() => {});
