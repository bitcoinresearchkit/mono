import { ios, canShare } from "../env.js";
import { style } from "../elements.js";
import { colors } from "../colors.js";
import { stringToId } from "../format.js";

export const canCapture = !ios || canShare;
const openUrls = new Set();

window.addEventListener("pagehide", () => {
  for (const url of openUrls) {
    URL.revokeObjectURL(url);
  }
  openUrls.clear();
});

/**
 * @typedef {Object} CaptureMetadata
 * @property {string} title
 * @property {string} fileName
 */

/** @param {string} title */
function captureMetadata(title) {
  return {
    title: `${title} | bitview`,
    fileName: `bitview-${stringToId(title)}.png`,
  };
}

/**
 * @typedef {Object} LegendItem
 * @property {string} text
 * @property {string[]} colors
 * @property {boolean} muted
 *
 * @typedef {Object} LegendCapture
 * @property {number} x
 * @property {number} y
 * @property {number} width
 * @property {LegendItem[]} items
 *
 * @typedef {Object} LegendMetrics
 * @property {number} dot
 * @property {number} fontSize
 * @property {number} itemGap
 * @property {number} lineHeight
 * @property {number} rowGap
 * @property {number} textGap
 */

/**
 * @param {string} value
 * @param {number} [fallback]
 */
function cssPx(value, fallback = 0) {
  return Number.parseFloat(value) || fallback;
}

/**
 * @param {CSSStyleDeclaration} computedStyle
 * @param {number} size
 */
function canvasFont(computedStyle, size) {
  return [
    computedStyle.fontStyle,
    computedStyle.fontWeight,
    `${size}px`,
    computedStyle.fontFamily || style.fontFamily,
  ].join(" ");
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {CSSStyleDeclaration} computedStyle
 * @param {number} size
 * @param {number} [letterSpacing]
 */
function setFont(ctx, computedStyle, size, letterSpacing = 0) {
  ctx.font = canvasFont(computedStyle, size);
  if ("letterSpacing" in ctx) ctx.letterSpacing = `${letterSpacing}px`;
}

/**
 * @param {string} text
 * @param {HTMLElement} element
 */
function transformText(text, element) {
  switch (getComputedStyle(element).textTransform) {
    case "lowercase":
      return text.toLowerCase();
    case "uppercase":
      return text.toUpperCase();
    case "capitalize":
      return text.replace(/\b\w/g, (char) => char.toUpperCase());
    default:
      return text;
  }
}

/** @param {HTMLElement} element */
function visibleText(element) {
  const select = element.querySelector("select");
  if (!(select instanceof HTMLSelectElement)) {
    return transformText(element.textContent?.trim() ?? "", element);
  }

  const selected =
    select.selectedOptions[0]?.textContent?.trim() || select.value.trim();

  return transformText(selected, element);
}

/**
 * @param {string} text
 * @param {Partial<LegendItem>} [item]
 * @returns {LegendItem[]}
 */
function legendText(text, item = {}) {
  return text
    ? [
        {
          text,
          colors: [],
          muted: false,
          ...item,
        },
      ]
    : [];
}

/**
 * @param {HTMLElement} legend
 * @returns {LegendItem[]}
 */
function legendItems(legend) {
  const scroller = legend.firstElementChild;
  if (!(scroller instanceof HTMLElement)) return [];

  const children = Array.from(scroller.children);
  const seriesRoot =
    children.find((child) =>
      child.querySelector('label input[type="checkbox"]'),
    ) ?? children.at(-1);
  const prefix = children[0] !== seriesRoot ? children[0] : null;
  const separator = prefix ? children[1] : null;

  /** @type {LegendItem[]} */
  const seriesItems = [];

  const root = seriesRoot instanceof HTMLElement ? seriesRoot : scroller;
  for (const label of root.querySelectorAll("label")) {
    const input = label.querySelector('input[type="checkbox"]');
    const name = label.querySelector(".name");
    if (
      !(input instanceof HTMLInputElement) ||
      !(name instanceof HTMLElement) ||
      !input.checked
    ) {
      continue;
    }

    const text = transformText(name.textContent?.trim() ?? "", name);
    if (!text) continue;

    seriesItems.push({
      text,
      muted: false,
      colors: Array.from(label.querySelectorAll(".colors span"))
        .map((span) =>
          span instanceof HTMLElement ? span.style.backgroundColor : "",
        )
        .filter(Boolean),
    });
  }

  if (!seriesItems.length) return [];

  const prefixText = prefix instanceof HTMLElement ? visibleText(prefix) : "";
  return [
    ...legendText(prefixText),
    ...(prefixText && separator instanceof HTMLElement
      ? legendText(visibleText(separator), { muted: true })
      : []),
    ...seriesItems,
  ];
}

/**
 * @param {HTMLElement} legend
 * @param {DOMRect} parentRect
 * @param {(value: number) => number} scale
 * @returns {LegendCapture}
 */
function captureLegend(legend, parentRect, scale) {
  const scroller = legend.firstElementChild;
  if (!(scroller instanceof HTMLElement)) {
    return { x: 0, y: 0, width: 0, items: [] };
  }

  const rect = scroller.getBoundingClientRect();
  const computedStyle = getComputedStyle(scroller);
  const left = cssPx(computedStyle.paddingLeft);
  const right = cssPx(computedStyle.paddingRight);

  return {
    x: scale(rect.left - parentRect.left + left),
    y: scale(rect.top - parentRect.top + cssPx(computedStyle.paddingTop, 6)),
    width: Math.max(0, scale(rect.width - left - right)),
    items: legendItems(legend),
  };
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {LegendItem} item
 * @param {LegendMetrics} metrics
 */
function measureItem(ctx, item, metrics) {
  const swatch = item.colors.length ? metrics.dot * 2 + metrics.textGap : 0;
  return swatch + ctx.measureText(item.text).width + metrics.itemGap;
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {LegendItem[]} items
 * @param {number} width
 * @param {LegendMetrics} metrics
 */
function layoutLegend(ctx, items, width, metrics) {
  /** @type {LegendItem[][]} */
  const rows = [];
  /** @type {LegendItem[]} */
  let row = [];
  let rowWidth = 0;

  for (const item of items) {
    const itemWidth = measureItem(ctx, item, metrics);
    if (row.length && rowWidth + itemWidth > width) {
      rows.push(row);
      row = [];
      rowWidth = 0;
    }
    row.push(item);
    rowWidth += itemWidth;
  }

  if (row.length) rows.push(row);
  return rows;
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {string[]} itemColors
 * @param {number} x
 * @param {number} y
 * @param {number} radius
 */
function drawSwatch(ctx, itemColors, x, y, radius) {
  ctx.save();
  ctx.beginPath();
  ctx.arc(x + radius, y, radius, 0, Math.PI * 2);
  ctx.clip();

  itemColors.forEach((color, index) => {
    const h = (radius * 2) / itemColors.length;
    ctx.fillStyle = color;
    ctx.fillRect(x, y - radius + index * h, radius * 2, h);
  });

  ctx.restore();
}

/**
 * @param {CanvasRenderingContext2D} ctx
 * @param {LegendItem[][]} rows
 * @param {number} x
 * @param {number} y
 * @param {LegendMetrics} metrics
 */
function drawLegend(ctx, rows, x, y, metrics) {
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";

  rows.forEach((row, rowIndex) => {
    let itemX = x;
    const itemY =
      y + metrics.lineHeight / 2 + rowIndex * (metrics.lineHeight + metrics.rowGap);

    for (const item of row) {
      if (item.colors.length) {
        drawSwatch(ctx, item.colors, itemX, itemY, metrics.dot);
        itemX += metrics.dot * 2 + metrics.textGap;
      }

      const textWidth = ctx.measureText(item.text).width;
      ctx.fillStyle = item.muted ? colors.gray() : colors.default();
      ctx.fillText(item.text, itemX, itemY);

      itemX += textWidth + metrics.itemGap;
    }
  });
}

/** @param {HTMLCanvasElement} canvas */
function canvasToBlob(canvas) {
  return new Promise((resolve) => canvas.toBlob(resolve, "image/png"));
}

/** @param {HTMLCanvasElement} canvas */
function canvasToBlobSync(canvas) {
  const data = canvas.toDataURL("image/png").split(",")[1];
  const binary = atob(data);
  const bytes = new Uint8Array(binary.length);

  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }

  return new Blob([bytes], { type: "image/png" });
}

/**
 * @param {Blob} blob
 * @param {CaptureMetadata} metadata
 */
async function shareBlob(blob, metadata) {
  if (!canShare || !("share" in navigator) || !("File" in window)) {
    return false;
  }

  const file = new File([blob], metadata.fileName, { type: "image/png" });
  if (!navigator.canShare({ files: [file] })) {
    return false;
  }

  try {
    await navigator.share({
      files: [file],
      title: metadata.title,
    });
    return true;
  } catch (error) {
    return error instanceof DOMException && error.name === "AbortError";
  }
}

/**
 * @param {Blob} blob
 * @param {CaptureMetadata} metadata
 */
function openBlob(blob, metadata) {
  const file =
    "File" in window
      ? new File([blob], metadata.fileName, { type: "image/png" })
      : blob;
  const url = URL.createObjectURL(file);
  openUrls.add(url);

  const win = window.open(url, "_blank");
  if (win) return;

  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = metadata.fileName;
  anchor.click();
}

/**
 * @param {HTMLCanvasElement} canvas
 * @param {CaptureMetadata} metadata
 */
async function openCanvas(canvas, metadata) {
  const blob = ios ? canvasToBlobSync(canvas) : await canvasToBlob(canvas);
  if (!blob) return;

  if (ios && (await shareBlob(blob, metadata))) {
    return;
  }

  openBlob(blob, metadata);
}

/**
 * @param {Object} args
 * @param {HTMLCanvasElement} args.screenshot
 * @param {number} args.chartWidth
 * @param {HTMLElement} args.chartElement
 * @param {HTMLElement} args.parent
 * @param {{ element: HTMLElement }[]} args.legends
 */
export function capture({
  screenshot,
  chartWidth,
  chartElement,
  parent,
  legends,
}) {
  const dpr =
    chartWidth > 0 ? screenshot.width / chartWidth : window.devicePixelRatio;
  const scale = (/** @type {number} */ value) => Math.round(value * dpr);

  const parentRect = parent.getBoundingClientRect();
  const chartRect = chartElement.getBoundingClientRect();
  const parentStyle = getComputedStyle(parent);
  const rootStyle = getComputedStyle(document.documentElement);
  const pad = scale(
    cssPx(
      parentStyle.paddingLeft,
      cssPx(rootStyle.getPropertyValue("--main-padding"), 32),
    ),
  );

  const chartX = scale(chartRect.left - parentRect.left);
  const chartY = scale(chartRect.top - parentRect.top);

  const title = parent.querySelector("h1");
  const titleText = title?.textContent?.trim() ?? "";
  const metadata = captureMetadata(titleText);
  const titleStyle = title
    ? getComputedStyle(title)
    : getComputedStyle(document.documentElement);
  const titleSize = scale(cssPx(titleStyle.fontSize, 32));

  const legendStyle = getComputedStyle(legends[0].element);
  const metrics = {
    dot: scale(5),
    fontSize: scale(cssPx(legendStyle.fontSize, 12)),
    itemGap: scale(16),
    lineHeight: scale(
      cssPx(legendStyle.lineHeight, cssPx(legendStyle.fontSize, 12) * 1.333),
    ),
    rowGap: scale(3),
    textGap: scale(4),
  };

  const top = captureLegend(legends[0].element, parentRect, scale);
  const bottom = captureLegend(legends[1].element, parentRect, scale);

  const measureCtx = document.createElement("canvas").getContext("2d");
  if (!measureCtx) return;
  setFont(measureCtx, legendStyle, metrics.fontSize);
  const topRows = layoutLegend(measureCtx, top.items, top.width, metrics);
  const bottomRows = layoutLegend(measureCtx, bottom.items, bottom.width, metrics);

  const canvas = document.createElement("canvas");
  canvas.width = Math.max(screenshot.width + chartX, scale(parentRect.width));
  canvas.height = Math.max(
    chartY + screenshot.height + pad,
    scale(parentRect.height),
  );

  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const bodyBg = getComputedStyle(document.body).backgroundColor;
  const htmlBg = getComputedStyle(document.documentElement).backgroundColor;
  ctx.fillStyle = bodyBg === "rgba(0, 0, 0, 0)" ? htmlBg : bodyBg;
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  if (titleText && title instanceof HTMLElement) {
    const rect = title.getBoundingClientRect();
    setFont(
      ctx,
      titleStyle,
      titleSize,
      scale(cssPx(titleStyle.letterSpacing)),
    );
    ctx.fillStyle = colors.default();
    ctx.textAlign = "left";
    ctx.textBaseline = "middle";
    ctx.fillText(
      titleText,
      scale(rect.left - parentRect.left),
      scale(rect.top - parentRect.top + rect.height / 2),
    );
  }

  ctx.drawImage(screenshot, chartX, chartY);

  setFont(ctx, legendStyle, metrics.fontSize);
  drawLegend(ctx, topRows, top.x, top.y, metrics);
  drawLegend(ctx, bottomRows, bottom.x, bottom.y, metrics);

  ctx.fillStyle = colors.gray();
  setFont(ctx, legendStyle, metrics.fontSize);
  ctx.textAlign = "right";
  ctx.textBaseline = "bottom";
  ctx.fillText(window.location.host, canvas.width - pad, canvas.height - pad / 2);

  openCanvas(canvas, metadata);
}
