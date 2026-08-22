import { createLabeledInput, createSpan, createSpanName } from "../dom.js";
import { stringToId } from "../format.js";

/** @param {HTMLElement} el */
function captureScroll(el) {
  el.addEventListener("wheel", (e) => e.stopPropagation(), { passive: true });
  el.addEventListener("touchstart", (e) => e.stopPropagation(), {
    passive: true,
  });
  el.addEventListener("touchmove", (e) => e.stopPropagation(), {
    passive: true,
  });
}

/**
 * Creates a `<legend>` with a scrollable `<div>`.
 * Call `setPrefix(el)` to insert a prefix element followed by a `|` separator.
 * Append further content to `scroller`.
 */
export function createLegend() {
  const element = /** @type {HTMLLegendElement} */ (
    window.document.createElement("legend")
  );
  const scroller = /** @type {HTMLDivElement} */ (
    window.document.createElement("div")
  );
  element.append(scroller);
  captureScroll(scroller);

  const separator = createSpan("|");
  captureScroll(separator);
  /** @type {HTMLElement | null} */
  let prefix = null;

  return {
    element,
    scroller,
    /** @param {HTMLElement} el */
    setPrefix(el) {
      prefix ? prefix.replaceWith(el) : scroller.prepend(el, separator);
      prefix = el;
      captureScroll(el);
    },
    clearPrefix() {
      prefix?.remove();
      prefix = null;
      separator.remove();
    },
  };
}

export function createSeriesLegend() {
  const legend = createLegend();
  const items = window.document.createElement("div");
  legend.scroller.append(items);
  captureScroll(items);

  /** @type {AnySeries | null} */
  let hoveredSeries = null;
  /** @type {Map<AnySeries, { span: HTMLSpanElement, color: Color }[]>} */
  const seriesColorSpans = new Map();

  /** @param {AnySeries | null} series */
  function setHovered(series) {
    if (hoveredSeries === series) return;
    hoveredSeries = series;
    for (const [entrySeries, colorSpans] of seriesColorSpans) {
      const shouldHighlight = !hoveredSeries || hoveredSeries === entrySeries;
      shouldHighlight ? entrySeries.highlight() : entrySeries.tame();
      for (const { span, color } of colorSpans) {
        span.style.backgroundColor = color.highlight(shouldHighlight);
      }
    }
  }

  /** @type {HTMLElement[]} */
  const legends = [];

  return {
    element: legend.element,
    setPrefix: legend.setPrefix,
    clearPrefix: legend.clearPrefix,
    /**
     * @param {Object} args
     * @param {AnySeries} args.series
     * @param {string} args.name
     * @param {number} args.order
     * @param {Color[]} args.colors
     */
    addOrReplace({ series, name, colors, order }) {
      const div = window.document.createElement("div");

      const prev = legends[order];
      if (prev) {
        prev.replaceWith(div);
      } else {
        const elementAtOrder = Array.from(items.children).at(order);
        if (elementAtOrder) {
          elementAtOrder.before(div);
        } else {
          items.append(div);
        }
      }
      legends[order] = div;

      const { label } = createLabeledInput({
        inputId: stringToId(`legend-${series.id}`),
        inputName: stringToId(`selected-${series.id}`),
        inputValue: "value",
        title: "Click to toggle",
        inputChecked: series.active.value,
        onClick: () => {
          series.setActive(!series.active.value);
        },
        type: "checkbox",
      });

      const spanMain = window.document.createElement("span");
      spanMain.classList.add("main");
      label.append(spanMain);

      const spanName = createSpanName(name);
      spanMain.append(spanName);

      div.append(label);
      label.addEventListener("mouseover", () => setHovered(series));
      label.addEventListener("mouseleave", () => setHovered(null));

      const spanColors = window.document.createElement("span");
      spanColors.classList.add("colors");
      spanMain.prepend(spanColors);
      /** @type {{ span: HTMLSpanElement, color: Color }[]} */
      const colorSpans = [];
      colors.forEach((color) => {
        const spanColor = window.document.createElement("span");
        spanColor.style.backgroundColor = color.highlight(true);
        spanColors.append(spanColor);
        colorSpans.push({ span: spanColor, color });
      });
      seriesColorSpans.set(series, colorSpans);

      if (series.url) {
        const anchor = window.document.createElement("a");
        anchor.href = series.url;
        anchor.target = "_blank";
        anchor.rel = "noopener noreferrer";
        anchor.title = "Open the series data in a new tab";
        div.append(anchor);
      }
    },
    /**
     * @param {number} start
     */
    removeFrom(start) {
      legends.splice(start).forEach((child) => child.remove());
    },
  };
}
