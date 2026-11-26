// @ts-check
// What a first chart doesn't run, as one module for later.js to load: ⌘K first, whose imports bring in the others in the
// order they've always run in.
export * as command from "/studio/command.js";
export * as pen from "/studio/drawing.js";
export * as grid from "/studio/grid.js";
export * as heatmaps from "/studio/heatmaps.js";
export * as inspector from "/studio/inspector.js";
export * as menus from "/studio/menus.js";
export * as presetsEdit from "/studio/presets-edit.js";
export * as gestures from "/studio/gestures.js";
export * as xkcd from "/modules/xkcd-colors/index.js";
