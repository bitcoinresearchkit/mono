// @ts-check
// A pane's legend, over its plot's top left (`pane.setLegend`): its items in their order, each its series' key, name
// and value; one shown, named; more, a chip: their first four keys stacked and how many (a press on it told the page:
// `onPress`); pointing at one of them (`point`: the crosshair's series, or the page's), the chip names it alone. Its
// shown items counted (a hidden one's left to the page's list of them; none shown, each named). Items the page's own
// elements (a key, a name, a value it writes), or made here from a series (its color's key, its name, its value at the
// crosshair, else its last in view, as its tag writes it). Looked as the page's stylesheet says, over a few rules of
// its own (`:where`, so any of the page's wins); for a picture, every item named (`whole`).

/**
 * @typedef {{ id: string, elements?: HTMLElement[], series?: any[], label?: string, hidden?: boolean }} LegendItem
 * An item: its `elements` (the page's: the first its key and name, the rest beside it, gone in a chip), else made from
 * its first series and `label`; the `series` its key is for (the crosshair on one of them points at it).
 */

/** The legend's own rules, laid under the page's (made once). */
let sheet = /** @type {CSSStyleSheet | null} */ (null);
function adopt() {
  if (sheet || typeof CSSStyleSheet === "undefined") return;
  sheet = new CSSStyleSheet();
  sheet.replaceSync(`
    :where(.bc-legend) { position: absolute; top: 4px; left: 8px; display: flex; align-items: center; gap: 2px; height: 22px; overflow: hidden; pointer-events: none; font: 12px system-ui, sans-serif; white-space: nowrap }
    .bc-legend[hidden], .bc-legend > [hidden] { display: none !important }
    :where(.bc-legend[data-chip]) { pointer-events: auto; cursor: pointer; gap: 0 }
    :where(.bc-legend[data-chip] > * + *) { margin-left: -6px }
    :where(.bc-legend[data-chip]:not([data-named]) .bc-legend-name, .bc-legend[data-chip]:not([data-named]) .bc-legend-value) { display: none }
    :where(.bc-legend[data-named] > :not([data-pointed]), .bc-legend[data-named] > .bc-legend-count) { display: none }
    :where(.bc-legend-item) { display: flex; align-items: center; gap: 6px; padding: 2px 6px 2px 4px; color: inherit }
    :where(.bc-legend-summary) { margin-left: 8px; padding-left: 8px; border-left: 1px solid currentColor }
    :where(.bc-legend-count) { margin-left: 6px; opacity: 0.7 }
  `);
  document.adoptedStyleSheets = [sheet, ...document.adoptedStyleSheets];
}

export class Legend {
  /** @param {HTMLElement} frame its pane's frame, over its plot @param {{ onPress?: (() => void) | null }} [options] */
  constructor(frame, { onPress = null } = {}) {
    adopt();
    this.element = document.createElement("div");
    this.element.className = "bc-legend";
    frame.append(this.element);
    this.onPress = onPress;
    /** @type {LegendItem[]} */
    this.items = [];
    /** Each item's elements, as last laid. @type {HTMLElement[][]} */
    this.laid = [];
    /** The item pointed at (its id), or null. */
    this.pointed = /** @type {string | null} */ (null);
    /** Every item named (a picture's). */
    this.whole = false;
    /** Optional context kept visible beside the count when items fold. */
    this.summary = Object.assign(document.createElement("span"), { className: "bc-legend-summary" });
    this.count = Object.assign(document.createElement("span"), { className: "bc-legend-count" });
    this.element.addEventListener("click", (event) => {
      if (!this.element.hasAttribute("data-chip") || !this.onPress) return;
      event.stopPropagation();
      this.onPress();
    });
  }

  /** Its items (the page's elements put in it in their order, in place of what it held). @param {LegendItem[]} items @param {string} [summary] context beside the folded count */
  set(items, summary = "") {
    this.items = items;
    if (this.summary.textContent !== summary) this.summary.textContent = summary;
    this.laid = items.map((item) => item.elements ?? [made(item)]);
    const all = [...this.laid.flat(), this.count, this.summary];
    // (What it holds now, in order: the elements given, then its count and summary; moved only where they're not so already (a
    // move drops a focus in it).)
    for (const child of [...this.element.children]) if (!all.includes(/** @type {HTMLElement} */ (child))) child.remove();
    if (all.some((element, k) => this.element.children[k] !== element)) for (const element of all) this.element.append(element);
    this.#fold();
  }

  /** The item a series is in (its id), or null. @param {any} series */
  itemOf(series) {
    if (!series) return null;
    for (const item of this.items) if (item.series?.includes(series)) return item.id;
    return null;
  }

  /** The item pointed at (null: none), named in a chip. @param {string | null} id */
  point(id) {
    if (id === this.pointed) return;
    this.pointed = id;
    this.#fold();
  }

  /** Every item named, at rest (a picture's), or back as it shows. @param {boolean} on */
  setWhole(on) {
    if (on === this.whole) return;
    this.whole = on;
    this.#fold();
  }

  /**
   * The values of the items made here (their series' at the crosshair's point, else their last in view), written as
   * their tags write them. @param {number} index the crosshair's (NaN: none) @param {import("./timeline.js").Timeline} time
   * @param {(series: any, value: number) => string} write
   */
  read(index, time, write) {
    for (const [k, item] of this.items.entries()) {
      if (item.elements) continue;
      const series = item.series?.[0], value = /** @type {HTMLElement | null} */ (this.laid[k][0].querySelector(".bc-legend-value"));
      if (!series?.values || !value) continue;
      // (Past its last point (the crosshair after the data), read as at rest: its last in view.)
      const past = !(index === index && index < series.values.length), at = past ? time.lastIn(series.values) : index;
      const v = series.values[at], text = v === v && v !== undefined ? write(series, v) : "";
      if (value.textContent !== text) value.textContent = text;
    }
  }

  /** One shown, named; more, a chip (four keys, how many), the one pointed at named alone; none shown, each named. */
  #fold() {
    const shown = this.items.filter((item) => !item.hidden), chip = shown.length > 1 && !this.whole;
    const named = chip && shown.some((item) => item.id === this.pointed);
    this.element.toggleAttribute("data-chip", chip);
    this.element.toggleAttribute("data-named", named);
    this.count.hidden = !chip;
    this.summary.hidden = !chip || !this.summary.textContent;
    if (this.count.textContent !== String(shown.length)) this.count.textContent = String(shown.length);
    let refocus = false, k = 0;
    for (const [n, item] of this.items.entries()) {
      const off = !this.whole && shown.length > 0 && Boolean(item.hidden), at = off ? -1 : k++;
      const [first, ...aside] = this.laid[n];
      if (off && this.laid[n].some((element) => element.contains(document.activeElement))) refocus = true;
      // (In a chip: four keys, the rest counted; what's beside one, gone.)
      setHidden(first, off || (chip && !named && at >= 4) || (named && item.id !== this.pointed));
      first.toggleAttribute("data-pointed", named && item.id === this.pointed);
      for (const element of aside) setHidden(element, off || chip);
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
