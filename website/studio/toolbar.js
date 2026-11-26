// @ts-check
// The chart's toolbar as it shows: its title field, and its interval, window and ranges.
import { ICONS, CHEVRON } from "/studio/icons.js";
import { focusKey, $, restoreFocus, escapeHtml } from "/studio/dom.js";
import { chart, chartWindow, measure, phone } from "/studio/app.js";
import { capitalized, windowShort, windowSaid } from "/studio/words.js";
import { presetFits, periodOf, presetShown, shownRange } from "/studio/chart.js";
import { RANGES, INTERVAL_LABEL, CADENCE } from "/studio/intervals.js";
/** @import { Chart } from "/studio/vocabulary.js" */
import { chartPlace, stepsTo } from "/studio/preset-charts.js";

function renderRanges() {
  const html = RANGES.map(([id, text], i) => {
    const fits = presetFits(id), tip = fits ? capitalized(periodOf(id)) : `More than fits at ${INTERVAL_LABEL[chart.index]}`;
    return `<button type="button" data-range="${id}" aria-pressed="${presetShown && shownRange() === id}" aria-label="${text}: ${tip.toLowerCase()}" data-tip="${tip}" ${fits ? `data-kbd="${i + 1}"` : 'aria-disabled="true"'}>${text}</button>`;
  }).join("");
  if (html === rangesShown) return;
  const focus = focusKey($("ranges"));
  $("ranges").innerHTML = rangesShown = html;
  restoreFocus($("ranges"), focus);
}
let rangesShown = "";
function renderWindow() {
  const found = chart.layers.length ? chartWindow(chart) : null;
  $("window").hidden = !found;
  if (!found) return;
  $("window").innerHTML = `${ICONS.hourglass}${windowShort(found.window)}${CHEVRON}`;
  $("window").setAttribute("aria-label", `${windowSaid(found.window)} window`);
}
function renderInterval() {
  $("interval").innerHTML = `${ICONS.recent}${INTERVAL_LABEL[chart.index] ?? chart.index}${CHEVRON}`;
  $("interval").setAttribute("aria-label", `${INTERVAL_LABEL[chart.index] ?? chart.index}, ${(CADENCE[chart.index] ?? chart.index).toLowerCase()} interval`);
}
// Safari and Firefox lack field-sizing.
function fitTitle() {
  if (CSS.supports("field-sizing", "content")) return;
  const field = /** @type {HTMLInputElement} */ ($("title")), style = getComputedStyle(field), text = field.value || field.placeholder;
  measure.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
  const spacing = parseFloat(style.letterSpacing) || 0;
  field.style.width = `${Math.ceil(measure.measureText(text).width + spacing * text.length + parseFloat(style.paddingLeft) + parseFloat(style.paddingRight)) + 2}px`;
}
const titleReads = () => (/** @type {HTMLInputElement} */ ($("title")).readOnly = phone.matches);
const typedFrom = new WeakMap();
let chartPathDrawn = "";
/**
 * By identity, not id: a preview saved again takes a new id.
 * @type {Chart | null}
 */
let chartPathOf = null;
function renderChartPath() {
  const place = chartPlace(), steps = place ? stepsTo(place) : [], parent = steps.at(-1), path = $("chart-path");
  const html = parent ? `<button type="button" id="chart-place" dir="auto" aria-haspopup="menu" aria-label="In ${escapeHtml(steps.map((step) => step.name).join(", "))}" data-tip="In ${escapeHtml(steps.map((step) => step.name).join(" › "))}"><span>${escapeHtml(parent.name)}</span></button><span class="sep" aria-hidden="true">›</span>` : "";
  if (chart !== chartPathOf) path.style.minWidth = "";
  else if (html !== chartPathDrawn && chartPathDrawn) path.style.minWidth = `${path.offsetWidth}px`;
  chartPathOf = chart;
  path.hidden = !html;
  if (html !== chartPathDrawn) path.innerHTML = chartPathDrawn = html;
}
export { renderInterval, renderRanges, rangesShown, renderWindow, titleReads, fitTitle, typedFrom, renderChartPath, chartPathOf, chartPathDrawn
};
