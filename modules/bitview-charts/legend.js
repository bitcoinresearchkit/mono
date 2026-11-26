// @ts-check

/** @typedef {{ id: string, elements?: HTMLElement[], series?: any[], label?: string, hidden?: boolean }} LegendItem */

/** Under the page's rules (`:where`): any of its own wins. */
let sheet = /** @type {CSSStyleSheet | null} */ (null);
function adopt() {
  if (sheet || typeof CSSStyleSheet === "undefined") return;
  sheet = new CSSStyleSheet();
  sheet.replaceSync(`
    :where(.bc-legend) { position: absolute; top: 4px; left: 8px; display: flex; align-items: center; gap: 2px; height: 22px; overflow: auto hidden; scrollbar-width: none; pointer-events: none; font: var(--bc-font); white-space: nowrap }
    .bc-legend[hidden], .bc-legend > [hidden] { display: none !important }
    :where(.bc-legend-item) { display: flex; flex-shrink: 0; align-items: center; gap: 6px; padding: 2px 6px 2px 4px; color: inherit }
    :where(.bc-legend-summary) { flex-shrink: 0; margin-right: 6px; opacity: 0.7 }
  `);
  document.adoptedStyleSheets = [sheet, ...document.adoptedStyleSheets];
}

export class Legend {
  /** @param {HTMLElement} frame */
  constructor(frame) {
    adopt();
    this.element = document.createElement("div");
    this.element.className = "bc-legend";
    frame.append(this.element);
    /** @type {LegendItem[]} */
    this.items = [];
    /** @type {HTMLElement[][]} */
    this.laid = [];
    this.pointed = /** @type {string | null} */ (null);
    this.summary = Object.assign(document.createElement("span"), { className: "bc-legend-summary" });
    this.element.addEventListener("focusin", ({ target }) => {
      const item = [...this.element.children].find((child) => child.contains(/** @type {Node} */ (target)));
      if (item) this.#reveal(/** @type {HTMLElement} */ (item));
    });
  }

  /** @param {LegendItem[]} items @param {string | HTMLElement} [summary] */
  set(items, summary = "") {
    this.items = items;
    if (typeof summary !== "string") this.summary.firstChild !== summary && this.summary.replaceChildren(summary);
    else if (this.summary.textContent !== summary || this.summary.children.length) this.summary.textContent = summary;
    this.summary.hidden = !this.summary.textContent && !this.summary.children.length;
    this.laid = items.map((item) => item.elements ?? [made(item)]);
    const all = [this.summary, ...this.laid.flat()];
    // Moved only where not in place already: a move drops a focus in it.
    for (const child of [...this.element.children]) if (!all.includes(/** @type {HTMLElement} */ (child))) child.remove();
    if (all.some((element, k) => this.element.children[k] !== element)) for (const element of all) this.element.append(element);
    this.#show();
  }

  /** @param {any} series */
  itemOf(series) {
    if (!series) return null;
    for (const item of this.items) if (item.series?.includes(series)) return item.id;
    return null;
  }

  /** @param {string | null} id */
  point(id) {
    if (id === this.pointed) return;
    this.pointed = id;
    const first = this.laid[this.items.findIndex((item) => item.id === id)]?.[0];
    if (first && !first.hidden) this.#reveal(first);
  }

  /** Clear of the page's edge fades (`scroll-padding-inline`). @param {HTMLElement} element */
  #reveal(element) {
    // The row is their offset parent: positions are within it whatever its scroll.
    const row = this.element, style = getComputedStyle(row);
    const room = Math.max(0, row.clientWidth - element.offsetWidth) / 2;
    const left = element.offsetLeft - Math.min(room, parseFloat(style.scrollPaddingInlineStart) || 0), right = element.offsetLeft + element.offsetWidth + Math.min(room, parseFloat(style.scrollPaddingInlineEnd) || 0);
    if (left < row.scrollLeft) row.scrollLeft = left;
    else if (right > row.scrollLeft + row.clientWidth) row.scrollLeft = right - row.clientWidth;
  }

  /**
   * @param {number} index NaN: none
   * @param {(series: any, value: number) => string} write @param {(series: any) => number} rest
   */
  read(index, write, rest) {
    for (const [k, item] of this.items.entries()) {
      if (item.elements) continue;
      const series = item.series?.[0], value = /** @type {HTMLElement | null} */ (this.laid[k][0].querySelector(".bc-legend-value"));
      if (!series?.values || !value) continue;
      const past = !(index === index && index < series.values.length), at = past ? rest(series) : index;
      const v = series.values[at], text = v === v && v !== undefined ? write(series, v) : "";
      if (value.textContent !== text) value.textContent = text;
    }
  }

  #show() {
    const first = this.items.findIndex((item) => !item.hidden);
    let refocus = false;
    for (const [n, item] of this.items.entries()) {
      const off = Boolean(item.hidden);
      if (off && this.laid[n].some((element) => element.contains(document.activeElement))) refocus = true;
      for (const element of this.laid[n]) setHidden(element, off);
    }
    if (refocus) this.laid[first]?.[0]?.focus({ preventScroll: true });
  }

  remove() {
    this.element.remove();
  }
}

/** @param {HTMLElement} element @param {boolean} hidden */
const setHidden = (element, hidden) => void (element.hidden !== hidden && (element.hidden = hidden));

/** @param {LegendItem} item */
function made(item) {
  const element = document.createElement("span"), color = item.series?.[0]?.color ?? "currentColor";
  element.className = "bc-legend-item";
  const key = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  key.setAttribute("viewBox", "0 0 16 12"), key.setAttribute("width", "16"), key.setAttribute("height", "12"), key.setAttribute("class", "bc-legend-key");
  key.innerHTML = `<path d="M1 9 L5 5 L9 7 L15 2" fill="none" stroke="${String(color).replace(/"/g, "")}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>`;
  const name = Object.assign(document.createElement("span"), { className: "bc-legend-name", textContent: item.label ?? "" });
  const value = Object.assign(document.createElement("span"), { className: "bc-legend-value" });
  element.append(key, name, value);
  return element;
}
