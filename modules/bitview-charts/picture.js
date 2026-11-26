// @ts-check
// The chart's canvases in one, drawn whole first so no pan's seams are in it.

/**
 * @param {import("./index.js").Chart} chart @param {{ crosshair?: boolean }} [options]
 * @returns {HTMLCanvasElement}
 */
export function picture(chart, { crosshair = false } = {}) {
  chart.render(true);
  const dpr = chart.dpr, canvas = document.createElement("canvas");
  canvas.width = Math.round(chart.width * dpr);
  canvas.height = Math.round(chart.height * dpr);
  const g = /** @type {CanvasRenderingContext2D} */ (canvas.getContext("2d"));
  g.fillStyle = chart.theme.background;
  g.fillRect(0, 0, canvas.width, canvas.height);
  const left = Math.round(chart.left * dpr);
  for (const pane of chart.panes) {
    if (!pane.height) continue;
    g.drawImage(pane.canvas, left, Math.round(pane.top * dpr));
    if (pane.axisCanvas.width) g.drawImage(pane.axisCanvas, left + Math.round(pane.plot * dpr), Math.round(pane.top * dpr));
    if (pane.leftCanvas && left) g.drawImage(pane.leftCanvas, 0, Math.round(pane.top * dpr));
  }
  g.drawImage(chart.timeAxis.canvas, left, Math.round((chart.height - chart.timeHeight) * dpr));
  if (crosshair) g.drawImage(chart.crosshair.canvas, 0, 0);
  for (const pane of chart.panes) if (pane.height && pane.overCanvas) g.drawImage(pane.overCanvas, left, Math.round(pane.top * dpr));
  return canvas;
}
