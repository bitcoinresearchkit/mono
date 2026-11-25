// @ts-check
// A pane's legend, over its plot's top left (`pane.setLegend`): one row of its items in their order, each its series'
// key, name and value, scrolling sideways where they don't fit (the page makes it reachable and fades its edges); the
// item pointed at (`point`: the crosshair's series, or the page's) is brought into view. Hidden items are left to the
// page's list of them, unless none is shown (each is then named, so the way to show one again is there). Items the
// page's own elements (a key, a name, a value it writes), or made here from a series (its color's key, its name, its
// value at the crosshair, else its last in view, as its tag writes it). Looked as the page's stylesheet says, over a few
// rules of its own (`:where`, so any of the page's wins).

/**
 * @typedef {{ id: string, elements?: HTMLElement[], series?: any[], label?: string, hidden?: boolean }} LegendItem
 * An item: its `elements` (the page's: the first its key and name, the rest beside it), else made from its first
 * series and `label`; the `series` its key is for (the crosshair on one of them points at it).
 */

/** The legend's own rules, laid under the page's (made once). */
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
  /** @param {HTMLElement} frame its pane's frame, over its plot */
  constructor(frame) {
    adopt();
    this.element = document.createElement("div");
    this.element.className = "bc-legend";
    frame.append(this.element);
    /** @type {LegendItem[]} */
    this.items = [];
    /** Each item's elements, as last laid. @type {HTMLElement[][]} */
    this.laid = [];
    /** The item pointed at (its id), or null. */
    this.pointed = /** @type {string | null} */ (null);
    /** Optional context kept before the items (the page's words, or its own element). */
    this.summary = Object.assign(document.createElement("span"), { className: "bc-legend-summary" });
    // (A focused item (a Tab onto one cut by the row's edge) comes into view whole, as a pointed one does.)
    this.element.addEventListener("focusin", ({ target }) => {
      const item = [...this.element.children].find((child) => child.contains(/** @type {Node} */ (target)));
      if (item) this.#reveal(/** @type {HTMLElement} */ (item));
    });
  }

  /** Its items (the page's elements put in it in their order, in place of what it held). @param {LegendItem[]} items @param {string | HTMLElement} [summary] context before the items (words, or the page's own element) */
  set(items, summary = "") {
    this.items = items;
    if (typeof summary !== "string") this.summary.firstChild !== summary && this.summary.replaceChildren(summary);
    else if (this.summary.textContent !== summary || this.summary.children.length) this.summary.textContent = summary;
    this.summary.hidden = !this.summary.textContent && !this.summary.children.length;
    this.laid = items.map((item) => item.elements ?? [made(item)]);
    const all = [this.summary, ...this.laid.flat()];
    // (What it holds now, in order: its summary, then the elements given; moved only where they're not so already (a
    // move drops a focus in it).)
    for (const child of [...this.element.children]) if (!all.includes(/** @type {HTMLElement} */ (child))) child.remove();
    if (all.some((element, k) => this.element.children[k] !== element)) for (const element of all) this.element.append(element);
    this.#show();
  }

  /** The item a series is in (its id), or null. @param {any} series */
  itemOf(series) {
    if (!series) return null;
    for (const item of this.items) if (item.series?.includes(series)) return item.id;
    return null;
  }

  /** The item pointed at (null: none), brought into view where the row scrolls. @param {string | null} id */
  point(id) {
    if (id === this.pointed) return;
    this.pointed = id;
    const first = this.laid[this.items.findIndex((item) => item.id === id)]?.[0];
    if (first && !first.hidden) this.#reveal(first);
  }

  /** Scrolls the row so one of its elements shows whole, clear of what the page fades at its edges (its `scroll-padding-inline`) where there's room. @param {HTMLElement} element */
  #reveal(element) {
    // (Its row is their offset parent: positions are within it, whatever it is scrolled to.)
    const row = this.element, style = getComputedStyle(row);
    // (An element too wide for that room beside it is centered whole instead: its edges under the fade, never cut.)
    const room = Math.max(0, row.clientWidth - element.offsetWidth) / 2;
    const left = element.offsetLeft - Math.min(room, parseFloat(style.scrollPaddingInlineStart) || 0), right = element.offsetLeft + element.offsetWidth + Math.min(room, parseFloat(style.scrollPaddingInlineEnd) || 0);
    if (left < row.scrollLeft) row.scrollLeft = left;
    else if (right > row.scrollLeft + row.clientWidth) row.scrollLeft = right - row.clientWidth;
  }

  /**
   * The values of the items made here (their series' at the crosshair's point, else their last in view), written as
   * their tags write them. @param {number} index the crosshair's (NaN: none)
   * @param {(series: any, value: number) => string} write @param {(series: any) => number} rest a series' point at rest (see Pane.restOf)
   */
  read(index, write, rest) {
    for (const [k, item] of this.items.entries()) {
      if (item.elements) continue;
      const series = item.series?.[0], value = /** @type {HTMLElement | null} */ (this.laid[k][0].querySelector(".bc-legend-value"));
      if (!series?.values || !value) continue;
      // (Past its last point (the crosshair after the data), read as at rest: its last in view.)
      const past = !(index === index && index < series.values.length), at = past ? rest(series) : index;
      const v = series.values[at], text = v === v && v !== undefined ? write(series, v) : "";
      if (value.textContent !== text) value.textContent = text;
    }
  }

  /** The shown items; hidden ones only where none is shown (each named). */
  #show() {
    const shown = this.items.filter((item) => !item.hidden);
    let refocus = false;
    for (const [n, item] of this.items.entries()) {
      const off = shown.length > 0 && Boolean(item.hidden);
      if (off && this.laid[n].some((element) => element.contains(document.activeElement))) refocus = true;
      for (const element of this.laid[n]) setHidden(element, off);
    }
    if (refocus) this.laid[this.items.indexOf(/** @type {LegendItem} */ (shown[0]))]?.[0]?.focus({ preventScroll: true });
  }

  /** Gone with its pane. */
  remove() {
    this.element.remove();
  }
}

/** @param {HTMLElement} element @param {boolean} hidden */
const setHidden = (element, hidden) => void (element.hidden !== hidden && (element.hidden = hidden));

/** An item made from its first series: its color's key (a line), its name, its value (see read). @param {LegendItem} item */
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
