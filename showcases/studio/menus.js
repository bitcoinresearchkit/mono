// @ts-check
// Menus, the toolbar, sharing, importing and exporting. (Functions alone, called from app.js, which keeps Studio's state and runs its start.)
import { capitalized } from "/studio/words.js";
import {
  $, all$, AVERAGES, blankChart, BUBBLE_HIDE, BUBBLE_HIDES, BUBBLE_SIZE, BUBBLE_SIZES, CADENCE, chart, chartImage,
  CHEVRON, clamp, closest, commit, css, DASHES, DEFAULT_MEASURE, DEFAULT_RANGE, defaultOpacity, drawKeys, drawnChart,
  EDGES, engine, escape, FINE, firstForm, firstLook, flowsAt, FORMATS, FORMULA_MAX, GRADIENT_LEAST, GRADIENT_MOST,
  grouped, HEAT_GRADIENT, HEAT_MEASURES, HEAT_ROWS, HEAT_WEIGHTS, heatLike, hideMenu, hideSubmenu, ICONS, indexAt,
  INTERVAL_LABEL, INTERVAL_OF_FIELD, INTERVAL_ORDER, intervalFits, isCatalogPath, isColor, isSeriesName, isView,
  KEY_STEPS, kindColor, kindOf, labelText, letterAt, library, LINES, listedPreviews, loadPresets, lookOfDrawing,
  makeLayer, MARK_PLACES, markable, MAX_DRAWINGS, MAX_LAYERS, MAX_PANES, MAX_STROKE, measure, menuBack, menuOwner,
  menuPicked, menuScrim, MOTION, navigations, normalize, notice, OPACITIES, openCascade, openingView, openView, ORIGIN,
  PAIRED, PALETTE, PEN_COLOR, phone, plural, pointOf, putViews, RANGES, reduced, resolved, revision, ROUTED, saveView,
  SHAPES, SHARES_IMAGES, shownView, showRange, STACKS, stepIn, storage, submenus, takeView, timeAt, toast, TOOLS,
  TYPES, uid, UNREAD, viewsNow, WIDTHS,
} from "/studio/app.js";
import {
  forgetSpans, inSubmenu, known, lastShown, legends, levelOf, menuPicks, periodOf, pointLabel, presetFits, presetShown,
  resolve, rightGap, seriesIn, settled, shownDates, shownRange, subAt, viewName,
} from "/studio/chart.js";
import { folderById, isIn, sideKey, titleOf, treeOf } from "/studio/grid.js";
import { focusKey, labelOf, restoreFocus } from "/studio/inspector.js";
import { isMade, presetEdited, presetLink } from "/studio/presets-edit.js";
/** @import { Chart, Layer, Level } from "/studio/app.js" */
/** @import { Action } from "/studio/command.js" */

/** What a phone's menu sheet hid from a screen reader (the page under it), as it was. */
let menuHidden = /** @type {Map<Element, string | null> | null} */ (null);
/**
 * A phone's menu sheet modal, or not: what's beside it on the page out of reach (in a dialog, it's modal already).
 * A tap there is the scrim's, Tab closes the menu; a screen reader's kept off it by aria-hidden, not inert: inert is
 * style, inherited, the whole page styled again at each open and close (the Charts page: 40-60 ms of the tap).
 */
function menuModal(/** @type {boolean} */ on) {
  const menu = $("menu");
  if (on && !menuHidden && menu.parentElement === document.body) {
    menuHidden = new Map();
    for (const element of document.body.children) {
      if ([menu, menuScrim, ...submenus, $("toast"), $("tip")].includes(/** @type {HTMLElement} */ (element)) || element instanceof HTMLDialogElement || element instanceof HTMLScriptElement || element.matches(".vh")) continue;
      menuHidden.set(element, element.getAttribute("aria-hidden"));
      element.setAttribute("aria-hidden", "true");
    }
  } else if (!on && menuHidden) {
    for (const [element, was] of menuHidden) was === null ? element.removeAttribute("aria-hidden") : element.setAttribute("aria-hidden", was);
    menuHidden = null;
  }
}
/** What opened the menu, as it is now: a redraw (the inspector's) puts a new one in its place, by the same id. */
const ownerNow = () => (menuOwner && !menuOwner.isConnected && menuOwner.id ? document.getElementById(menuOwner.id) : menuOwner);
/** A menu's opener let go of: its open state gone, where the menu set it (a folder's row says its own, open or not). */
const unexpand = (/** @type {Element | null} */ owner) => (owner?.removeAttribute("data-menu-open"), owner?.hasAttribute("aria-haspopup") && owner.removeAttribute("aria-expanded"));
/**
 * A menu beside what opened it (or at a point): below it if it fits, else above, kept within its host; from its left
 * edge (its right, where that fits better), or, asked for a side (the belt's popovers), centered on it.
 */
function placeMenu(menu, host, anchor, point, side = "") {
  // (Opened again as it fades: its fade ended first, its box measured whole and in place.)
  for (const motion of menu.getAnimations()) if (motion instanceof CSSTransition) motion.finish();
  const own = point ? new DOMRect(point.x, point.y, 0, 0) : anchor.getBoundingClientRect();
  // (A row's of the sidebar's list, beside the list: under its ⋯, it hid the rows below, their ⋯ under it.)
  if (!side && !point && anchor.closest("#chart-list")) side = "right";
  // (A belt's popover from the belt's edge, a rail's menu from its icons' column (the search's), not their button's:
  // as far from it as a menu from what opened it, whichever row (of different widths there: a group's head, a
  // folder, a chart in one).)
  const strip = side && !point ? (anchor.closest("#belt") ?? (railed() ? anchor.closest(".sidebar")?.querySelector(".search") : null))?.getBoundingClientRect() : null;
  const rect = !strip ? own : side === "left" || side === "right" ? new DOMRect(strip.left, own.top, strip.width, own.height) : new DOMRect(own.left, strip.top, own.width, strip.height);
  const gap = point ? 2 : 6;
  // Measured from its origin, with all the room it can take; kept within the window, or within its dialog (and no
  // taller than it: a long one scrolls).
  const bounds = host === document.body ? new DOMRect(0, 0, innerWidth, innerHeight) : host.getBoundingClientRect();
  menu.style.maxHeight = host === document.body ? "" : `${bounds.height - 16}px`;
  // Measured at rest: the opening animation starts it scaled down.
  menu.style.animation = "none";
  const origin = fixedOrigin(menu);
  const width = menu.offsetWidth, height = menu.offsetHeight;
  // Beside it (asked for): right or left, as fits, centered on it.
  if (side === "left" || side === "right") {
    const right = side === "right" ? rect.right + gap + width <= bounds.right - 8 : rect.left - gap - width < bounds.left + 8;
    // (Fitting neither side, kept within its host all the same, over what opened it.)
    // (A rail's row's: its first row level with it, as a submenu's with its row (moving straight across, what's in it,
    // not an action under its title); else centered on it.)
    const first = anchor.closest("#chart-list") ? /** @type {HTMLElement | null} */ (menu.querySelector(":scope > button")) : null, lead = first ? first.offsetTop + first.offsetHeight / 2 : height / 2;
    const x = clamp(right ? rect.right + gap : rect.left - gap - width, bounds.left + 8, bounds.right - width - 8), top = clamp(rect.top + rect.height / 2 - lead, bounds.top + 8, bounds.bottom - height - 8);
    placeFixed(menu, origin, x, top);
    menu.style.transformOrigin = `${right ? "0" : "100%"} ${clamp(rect.top + rect.height / 2 - top, 0, height)}px`;
    void menu.offsetWidth;
    menu.style.animation = "";
    return;
  }
  const left = clamp(side ? rect.left + rect.width / 2 - width / 2 : rect.left + width > bounds.right - 8 && !point ? rect.right - width : rect.left, bounds.left + 8, bounds.right - width - 8);
  const fitsBelow = rect.bottom + gap + height <= bounds.bottom - 8, below = side === "above" ? rect.top - gap - height < bounds.top + 8 && fitsBelow : fitsBelow;
  placeFixed(menu, origin, left, below ? rect.bottom + gap : Math.max(bounds.top + 8, rect.top - gap - height));
  menu.style.transformOrigin = `${clamp(rect.left + (side ? rect.width : Math.min(rect.width, 24)) / 2 - left, 0, width)}px ${below ? "0" : "100%"}`;
  void menu.offsetWidth;
  menu.style.animation = "";
}
/**
 * Where a fixed element's 0, 0 lands: the viewport's corner, or, inside a dialog that's the frame for fixed elements
 * (a blur or a transform makes it one), that dialog's. Positions in the viewport then come out right in either.
 */
function fixedOrigin(element) {
  element.style.left = element.style.top = "0px";
  const { left: x, top: y } = element.getBoundingClientRect();
  return { x, y };
}
/**
 * A fixed element put at `left`, `top` in the viewport (from its `origin`, see fixedOrigin), on whole device pixels:
 * its text drawn the same wherever it's drawn (on a layer of its own as it opens or a highlight glides in it, or
 * not), not a pixel's fraction off once it's painted again (Chromium).
 */
function placeFixed(/** @type {HTMLElement} */ element, /** @type {{ x: number, y: number }} */ origin, /** @type {number} */ left, /** @type {number} */ top) {
  const onPixels = (/** @type {number} */ value) => Math.round(value * devicePixelRatio) / devicePixelRatio;
  element.style.left = `${onPixels(left) - origin.x}px`;
  element.style.top = `${onPixels(top) - origin.y}px`;
}
/**
 * The menu put away (its backdrop tapped, Back; `back`, a key: Tab, esc): the focus back on what opened it, if it was
 * in it (always, `back`).
 */
function dismissMenu(back = false) {
  const had = back || $("menu").contains(document.activeElement) || inSubmenu(document.activeElement), owner = ownerNow();
  hideMenu();
  if (had && owner?.isConnected) owner.focus({ preventScroll: true });
}
/** Back, on a phone's menu sheet: a page of it to the menu it came from, else the menu away. */
function menuGoBack() {
  const back = menuBack;
  if (!back) return dismissMenu();
  hideMenu();
  back();
}
/** What a control's called (a menu named by what opened it): its label, else its words, its keys and asides apart. */
const calledBy = (/** @type {HTMLElement} */ element) => (element.getAttribute("aria-label") || element.querySelector(".label, .name")?.textContent || [...element.childNodes].filter((node) => !(node instanceof Element && node.matches("kbd, .aside, .sub, svg"))).map((node) => node.textContent).join(" ")).replace(/\s+/g, " ").trim();
/**
 * A menu's or a submenu's rows written in `host` (a dialog's top layer, opened in one): their keys drawn, its parts
 * named for what they are (its rules, its rows: a choice ticked, a radio; Share's a dialog of buttons, not a menu of
 * them), it named `label`. A row pressed picks (`onPick`), or opens its submenu beside it (a menu's: it stays).
 */
function fillMenu(/** @type {HTMLElement} */ element, /** @type {HTMLElement} */ host, /** @type {string} */ html, /** @type {string} */ label, /** @type {(data: DOMStringMap) => void} */ onPick) {
  // (Moved, its copy fading where it was gone first: its id is found where it is, never in a copy before it.)
  if (element.parentElement !== host) (cutFade(element), host.append(element));
  element.innerHTML = html;
  drawKeys(element);
  if (!element.querySelector(":scope > .share-pop")) {
    for (const rule of all$(".menu-rule", element)) rule.setAttribute("role", "separator");
    for (const button of all$("button", element)) if (!button.hasAttribute("role")) button.setAttribute("role", button.hasAttribute("aria-checked") ? "menuitemradio" : "menuitem");
  }
  element.setAttribute("aria-label", label);
  menuPicks.set(element, onPick);
  element.onclick = (event) => {
    const button = closest(event, "button:not(:disabled)");
    if (!button) return;
    // (A submenu's ‹, over the menu it's from: back to it.)
    if (button.dataset.menuBack && element !== $("menu")) return hideSubmenu(levelOf(element));
    if (cascades(button)) return openCascade(button, event.detail === 0);
    menuPicked(button, onPick);
  };
}
/** ↑↓, Home and End in a menu (or a submenu): the row before or after (round from the ends), the first, the last. Whether it was one. */
function walkMenu(/** @type {KeyboardEvent} */ event, /** @type {HTMLElement} */ root) {
  // (In a grid of them, the intervals': ← → along its row, ↑ ↓ to the next row's, its place kept.)
  const cell = /** @type {HTMLElement | null} */ (document.activeElement?.closest(".menu-row > button") ?? null);
  if (cell && root.contains(cell) && ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) {
    const rows = all$(".menu-row", root).map((row) => all$("button:not(:disabled)", row)).filter((row) => row.length);
    const r = rows.findIndex((row) => row.includes(cell)), c = rows[r].indexOf(cell);
    const across = event.key === "ArrowLeft" ? -1 : event.key === "ArrowRight" ? 1 : 0, down = event.key === "ArrowDown" ? 1 : event.key === "ArrowUp" ? -1 : 0;
    const row = rows[stepIn(r, down, rows.length, true)];
    event.preventDefault();
    event.stopPropagation();
    (across ? rows[r][stepIn(c, across, rows[r].length)] : row[Math.min(c, row.length - 1)])?.focus();
    return true;
  }
  if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return false;
  event.preventDefault();
  event.stopPropagation();
  const items = all$("button:not(:disabled)", root);
  // (From the menu itself, none of its rows yet: ↓ its first, ↑ its last.)
  items[stepIn(items.indexOf(/** @type {HTMLElement} */ (document.activeElement)), Number(KEY_STEPS[event.key]), items.length, true)]?.focus();
  return true;
}
/**
 * A menu's row leading to another menu, with a pointer (a phone's sheet replaces itself instead): its submenu opens
 * beside the menu (no room, over it: see showSubmenu), pointed at a moment (heading for it, inside the triangle from
 * where the pointer was to its near edge, the rows crossed on the way don't switch it), pressed, or → from the row;
 * ← or esc from it closes it. Color's Custom isn't one: it's a page of the same popover.
 */
const cascades = (/** @type {Element} */ button) => !phone.matches && button.getAttribute("aria-haspopup") === "menu" && !button.hasAttribute("data-pick-custom");
/** The row each submenu shows beside, by its level (from 1). @type {(HTMLElement | undefined)[]} */
const subRows = [];
function showSubmenu(/** @type {HTMLElement} */ row, /** @type {string} */ html, /** @type {(data: DOMStringMap) => void} */ onPick) {
  const own = /** @type {HTMLElement} */ (row.closest("#menu, .submenu")), level = levelOf(own) + 1, sub = subAt(level);
  // (Shown for another row of its menu: one submenu, moving to this one's, its rows this one's (not one fading as
  // another comes: a sweep down the rows, a flash a row); from where it shows now, a move under way too.)
  const was = sub.hidden ? null : sub.getBoundingClientRect(), had = sub.contains(document.activeElement);
  for (const motion of sub.getAnimations()) motion.finish();
  // (Its menu still moving (gliding to another row, growing in): there first, this placed beside where it stays.)
  for (const motion of own.getAnimations()) motion.finish();
  const host = /** @type {HTMLElement} */ ($("menu").parentElement);
  fillMenu(sub, host, html, calledBy(row), onPick);
  // (One leading on: its › at its end, as the menu's.)
  for (const button of all$("button[aria-haspopup='menu']", sub)) if (!button.querySelector(".sub")) button.insertAdjacentHTML("beforeend", `<span class="sub">${ICONS.next}</span>`);
  cutFade(sub);
  sub.hidden = false;
  subRows[level]?.removeAttribute("aria-expanded");
  (subRows[level] = row).setAttribute("aria-expanded", "true");
  // Beside the menu it's from, its first row level with the row, the way the submenus go (the first's: right, or left
  // where there's no room); no room that way, over that menu, in its place (it hidden till this goes), titled, ‹
  // back to it: never over the menus before it, the way back to each kept.
  const parent = own.getBoundingClientRect(), at = row.getBoundingClientRect(), bounds = host === document.body ? new DOMRect(0, 0, innerWidth, innerHeight) : host.getBoundingClientRect();
  sub.style.maxHeight = host === document.body ? "" : `${bounds.height - 16}px`;
  sub.style.maxWidth = "";
  sub.style.animation = "none";
  const origin = fixedOrigin(sub), fits = { right: parent.right + 4 + sub.offsetWidth <= bounds.right - 8, left: parent.left - 4 - sub.offsetWidth >= bounds.left + 8 };
  const way = level > 1 ? subAt(level - 1).dataset.way ?? "right" : fits.right || !fits.left ? "right" : "left", over = !fits[way];
  sub.dataset.way = way;
  // (Over its menu, no wider than the room from that menu's edge on: a long name cut, not the menus before it covered.)
  if (over) sub.style.maxWidth = `${way === "right" ? bounds.right - 8 - parent.left : parent.right - bounds.left - 8}px`;
  const width = sub.offsetWidth;
  // (The focus on the row it covers (pressed, or by keys): into it, on its first row.)
  const covering = over && (had || own.contains(document.activeElement));
  own.toggleAttribute("data-covered", over);
  if (over) {
    const head = sub.querySelector(":scope > .menu-group:first-child"), back = `<button type="button" class="cap small" role="menuitem" data-menu-back="1" aria-label="Back" data-tip="Back">${ICONS.back}</button>`;
    if (head) (head.classList.add("menu-title"), head.insertAdjacentHTML("afterbegin", back));
    else sub.insertAdjacentHTML("afterbegin", `<div class="menu-group menu-title">${back}${escape(calledBy(row))}</div>`);
  }
  // (Its first row level with the row, under its title if it has one; over its menu, level with it.)
  const height = sub.offsetHeight, top = clamp(over ? parent.top : at.top - (/** @type {HTMLElement | null} */ (sub.querySelector(":scope > button"))?.offsetTop ?? 4), bounds.top + 8, bounds.bottom - height - 8);
  const left = over ? (way === "right" ? parent.left : parent.right - width) : way === "right" ? parent.right + 4 : parent.left - 4 - width;
  placeFixed(sub, origin, clamp(left, bounds.left + 8, bounds.right - width - 8), top);
  sub.style.transformOrigin = `${way === "right" ? "0" : "100%"} ${clamp(at.top + at.height / 2 - top, 0, height)}px`;
  settleMenu(sub);
  if (covering) /** @type {HTMLElement | null} */ (sub.querySelector("button:not(:disabled, [data-menu-back])"))?.focus({ preventScroll: true });
  if (was) {
    slideFrom(sub, was);
    // (The focus in its rows as they're swapped: on the row it's now beside, for the keys to go on from (over it, in
    // it: see covering).)
    if (had && !over) row.focus({ preventScroll: true });
    return;
  }
  void sub.offsetWidth;
  sub.style.animation = "";
}
/** A menu shown again elsewhere (its rows changed), sliding there from where it was (`was`, its box then). @param {HTMLElement} menu @param {DOMRect} was */
const slideFrom = (menu, was) => ((now) => reduced.matches || menu.animate({ translate: [`${was.left - now.left}px ${was.top - now.top}px`, "0 0"] }, { duration: MOTION.micro, easing: MOTION.easeLayout }))(menu.getBoundingClientRect());
/**
 * A menu or a submenu as placed: a row's label or its title cut short, whole on hover (a row saying why it can't act
 * says that); its title held on top (see .menu-group), a row reached by keys scrolled in under it, not behind it.
 */
function settleMenu(/** @type {HTMLElement} */ menu) {
  for (const words of all$(":scope > button:not([data-tip]) > .label, :scope > .menu-group", menu)) {
    if (words.scrollWidth > words.clientWidth) (words.closest("button") ?? words).dataset.tip = words.textContent ?? "";
  }
  const top = menu.querySelector(":scope > :is(.menu-group:first-child, .menu-handle + .menu-group)");
  menu.style.scrollPaddingTop = top instanceof HTMLElement ? `${top.offsetTop + top.offsetHeight}px` : "";
}
/** Whether the sidebar is a rail of icons. */
const railed = () => document.body.dataset.sidebar === "collapsed";
/** Whether the sidebar's list shows as the rail (a phone's is a sheet, whole). */
const onRail = () => railed() && !phone.matches;
/**
 * A pointer's rest before a menu shows for it (ms), on a sidebar's row or a button: long enough that a pass down the
 * list opens none, no longer (as macOS's submenus wait). And after it's left, before the menu goes: time to cross to it.
 */
const HOVER = { row: 200, button: 200, go: 260 };
/**
 * The elements scrolled, as their scroll events say (a scroll by hand or by script, its event at the next frame: one
 * scrolled by script in the task it's faded out in isn't one yet): only they can be anywhere but at their start.
 * Those gone let go of as another comes (a row written anew, its scroll put back, each time).
 * @type {Set<Element>}
 */
const scrolled = new Set();
/**
 * Of a box that clips what's in it, its parts wholly out of its sight, the outermost (not looked into: a drawing's
 * own parts; nor what moves (`moving`), what's in it might come into sight), each with its twin in the box's copy
 * (walked alongside) and what holds its place as it's laid out (its display, its size: not as it shows, transformed);
 * a part laid out in a line (inline, its size its content's) or a table's, none.
 * @param {Element} element @param {Element} copy @param {Set<Element | null>} moving
 * @returns {[Element, HTMLElement, Record<string, string>][]}
 */
function outOfSight(element, copy, moving) {
  const view = element.getBoundingClientRect(), out = /** @type {[Element, HTMLElement, Record<string, string>][]} */ ([]);
  const walk = (/** @type {Element} */ node, /** @type {Element} */ twin) => {
    for (let i = 0; i < node.children.length; i++) {
      const part = node.children[i], box = part.getBoundingClientRect();
      if (moving.has(part)) continue;
      if (box.bottom <= view.top || box.top >= view.bottom || box.right <= view.left || box.left >= view.right) {
        const { display, width, height, boxSizing } = getComputedStyle(part);
        if (box.width && box.height && /^(block|flex|grid|flow-root|inline-block|inline-flex|inline-grid|list-item)$/.test(display)) out.push([part, /** @type {HTMLElement} */ (twin.children[i]), { display, width, height, "box-sizing": boxSizing, "min-width": width, "max-width": width, "min-height": height, "max-height": height, flex: "none" }]);
      } else if (part.childElementCount && !(part instanceof SVGElement)) walk(part, twin.children[i]);
    }
  };
  if (!moving.has(element)) walk(element, copy);
  return out;
}
/** A copy's twin of an element in what it's a copy of: the same way down. @param {Element} copy @param {Element} of @param {Element} element */
function twinIn(copy, of, element) {
  const way = [];
  for (let at = element; at !== of; at = /** @type {Element} */ (at.parentElement)) way.push(Array.prototype.indexOf.call(/** @type {Element} */ (at.parentElement).children, at));
  return way.reduceRight((into, i) => into.children[i], copy);
}
/**
 * What's put away (a menu, a tip) seen going as it came: a copy of it as it was, where it was, playing its way out
 * (out of reach, unread), the thing itself gone at once, as everything here knows it (a browser that doesn't play
 * `display` out, Firefox, would drop it at once). Done once its copy's gone (at once, none made).
 * (`prune`: its parts out of its sight copied empty, see below.)
 * @param {HTMLElement} element @param {Keyframe[]} frames @returns {Promise<void> | undefined}
 */
function fadeOut(element, frames, duration = MOTION.micro, placed = false, easing = MOTION.easeLayout, prune = false) {
  // (Hidden already, its way out is under way: left be. Shown, a copy going before is gone first.)
  if (element.hidden) return;
  cutFade(element);
  // (Not seen (no box, or hidden by what it's in): nothing to see go.)
  if (reduced.matches || !element.checkVisibility({ visibilityProperty: true })) return;
  const copy = /** @type {HTMLElement} */ (element.cloneNode(true)), style = getComputedStyle(element), after = getComputedStyle(element, "::after");
  // (What moves in it, where it's scrolled (those of it scrolled: none else can be, not read one by one, a page's every
  // element) and how it shows, asked before its copy's put in: after, the copy would be styled and laid out at once
  // for it.)
  const moving = element.getAnimations({ subtree: true }), scrolls = /** @type {[Element, number, number][]} */ ([]);
  for (const source of scrolled) if (element.contains(source) && (source.scrollTop || source.scrollLeft)) scrolls.push([source, source.scrollTop, source.scrollLeft]);
  // (From where it shows: caught on its way in (a menu shut as it opens), its opacity and its place as they are
  // then, not as it'd be once in.)
  const shows = /** @type {Keyframe} */ ({ ...frames[0] });
  for (const property of ["opacity", "transform", "translate", "scale"]) if (property in shows) shows[property] = style.getPropertyValue(property);
  // (`prune`, where it's asked (the Charts page's step: a page of tiles copied whole had every one styled and laid out
  // again for its way out): of what clips what's in it, its parts wholly out of its sight copied as boxes of the size
  // they're laid out at, empty (a space in them: not :empty), held at it (their display and size theirs, firmly).)
  const unseen = prune && (style.overflowX !== "visible" || style.overflowY !== "visible") ? outOfSight(element, copy, new Set(moving.map((animation) => /** @type {KeyframeEffect | null} */ (animation.effect)?.target ?? null))) : [];
  const seen = (/** @type {Element} */ source) => !unseen.some(([part]) => part.contains(source));
  for (const [, twin, held] of unseen) {
    twin.replaceChildren(" ");
    for (const [property, value] of Object.entries(held)) twin.style.setProperty(property, value, "important");
  }
  // (Its ids kept, as its styles and its drawings' references (an SVG's url(#…)) need them: after it, it's never the
  // one an id finds while it's there.)
  copy.inert = true;
  copy.setAttribute("aria-hidden", "true");
  // (Its way in not played again, nor any transition of its own (a sheet's from its start): it's going.)
  copy.style.animation = "none";
  copy.style.transition = "none";
  // (`placed`: where it was in its own frame (a transformed one's, a sheet's, a dialog's), over what takes its place
  // (a panel's view: shown, and its room at its end, by its own id's style).)
  if (placed) Object.assign(copy.style, { position: "absolute", left: `${element.offsetLeft}px`, top: `${element.offsetTop}px`, width: `${element.offsetWidth}px`, height: `${element.offsetHeight}px`, margin: "0", display: style.display, visibility: "visible", zIndex: "3", pointerEvents: "none", ...(after.content !== "none" ? { paddingBottom: `calc(${style.paddingBottom} + ${after.height})` } : {}) });
  element.after(copy);
  // (Scrolled as it was, its own and what's in it; its drawings (a canvas) as they were.)
  for (const [source, top, left] of scrolls) if (seen(source)) Object.assign(twinIn(copy, element, source), { scrollTop: top, scrollLeft: left });
  for (const source of /** @type {HTMLCanvasElement[]} */ (all$("canvas", element))) if (source.width && source.height && seen(source)) /** @type {HTMLCanvasElement} */ (twinIn(copy, element, source)).getContext("2d")?.drawImage(source, 0, 0);
  // (Moving as it was, its own and what's in it (a glide to a new place), on from where they'd got: not at their ends.)
  for (const animation of moving) {
    const effect = animation.effect, into = effect instanceof KeyframeEffect && !effect.pseudoElement && effect.target && seen(effect.target) ? twinIn(copy, element, effect.target) : undefined;
    if (!into || animation instanceof CSSAnimation || animation instanceof CSSTransition || animation.playState === "idle") continue;
    const own = /** @type {KeyframeEffect} */ (effect), twin = into.animate(own.getKeyframes(), own.getTiming());
    twin.currentTime = animation.currentTime;
    if (animation.playState === "paused") twin.pause();
  }
  const gone = () => {
    copy.remove();
    if (fading.get(element) === copy) fading.delete(element);
  };
  fading.set(element, copy);
  return copy.animate([shows, ...frames.slice(1)], { duration, easing, fill: "forwards" }).finished.then(gone, gone);
}
/** What's on its way out, by what it's a copy of (see fadeOut). @type {Map<HTMLElement, HTMLElement>} */
const fading = new Map();
/** Shown again (or on to a neighbor's, as a tip), its copy going is gone at once: one, never two. @param {HTMLElement} element */
const cutFade = (element) => (fading.get(element)?.remove(), fading.delete(element));
/** What opened a field (a menu's opener), where the focus goes as it's left. */
const fieldOpener = /** @type {WeakMap<Element, HTMLElement>} */ (new WeakMap());
/**
 * A field left by ↵ or esc: the focus to what opened it (Rename's ⋯), else the grid's first chart (from its filter,
 * as ↓), else the place that holds it (the chart's head, the panel, the grid's tools): never the page, a screen
 * reader's place kept, the app's single keys back. @param {HTMLElement} field
 */
function letGo(field) {
  const opener = fieldOpener.get(field), first = field.id === "home-filter" ? all$(".tile-open", $("home-grid"))[0] : null;
  const to = /** @type {HTMLElement | undefined} */ ([opener, first, field.closest(".card-head, #inspector, .home-tools, #panel")].find((element) => element instanceof HTMLElement && element.isConnected && element !== field && element.checkVisibility()));
  if (!to) return field.blur();
  if (!to.matches("button, a[href], input, textarea, select, [tabindex]")) to.tabIndex = -1;
  to.focus({ preventScroll: true });
}

/** Why an interval isn't offered for this chart (shown, disabled, wherever intervals are). */
const NO_INTERVAL = "Not every series or formula in this chart supports it";
/** The intervals every series the chart reads has (those known so far). */
function supportedIntervals(/** @type {Chart} */ doc = chart) {
  const lists = seriesIn(doc).map((name) => known.get(name)?.indexes).filter(Boolean), formulas = doc.layers.flatMap((layer) => layer.formula ? [layer.formula] : []);
  return (id) => doc.layers.every((layer) => intervalFits(layer.maxInterval, id)) && lists.every((indexes) => indexes.includes(id)) && formulas.every((formula) => flowsAt(formula, id));
}
function stepInterval(delta) {
  const at = INTERVAL_ORDER.indexOf(chart.index), ids = delta < 0 ? INTERVAL_ORDER.slice(0, at).reverse() : INTERVAL_ORDER.slice(at + 1);
  const next = ids.find(supportedIntervals());
  if (!next) return;
  switchInterval(next);
  toast(`Interval: ${CADENCE[next]}`);
}
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
/** The presets as last written. */
let rangesShown = "";
/** Another interval for the open chart: its view kept first, so the new one opens on the same time (on its preset, if it showed one). */
function switchInterval(/** @type {string} */ index) {
  saveView();
  commit((doc) => (doc.index = index), { look: true });
}
/** The interval's button. */
function renderInterval() {
  $("interval").innerHTML = `${ICONS.recent}${INTERVAL_LABEL[chart.index] ?? chart.index}${CHEVRON}`;
  $("interval").setAttribute("aria-label", `${INTERVAL_LABEL[chart.index] ?? chart.index}, ${(CADENCE[chart.index] ?? chart.index).toLowerCase()} interval`);
}
/** A preset is a view: no rebuild, the chart glides there. A step, as the interval is (see travel: undone, it glides back). */
function setRange(range) {
  commit((doc) => (doc.range = range), { coalesce: "range", redraw: "none", look: true, quiet: true, view: true });
  showRange();
}
/**
 * The title as wide as it reads, its text or what it'd be named (up to its room), where the field can't size itself
 * (Safari, Firefox: no field-sizing).
 */
function fitTitle() {
  if (CSS.supports("field-sizing", "content")) return;
  const field = /** @type {HTMLInputElement} */ ($("title")), style = getComputedStyle(field), text = field.value || field.placeholder;
  measure.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
  const spacing = parseFloat(style.letterSpacing) || 0;
  field.style.width = `${Math.ceil(measure.measureText(text).width + spacing * text.length + parseFloat(style.paddingLeft) + parseFloat(style.paddingRight)) + 2}px`;
}
// A title is trimmed once it's done: stray spaces don't make a name.
function leaveTitle() {
  const field = /** @type {HTMLInputElement} */ ($("title")), title = field.value.trim();
  // (Only what was typed: one focused and left as it was isn't an edit (another tab's rename, taken in meanwhile, stays).)
  if (title !== chart.title && field.value !== typedFrom.get(field)) commit((doc) => (doc.title = title), { coalesce: "title", redraw: "none" });
  // (Left, it's untouched again: a rename taken in while it's not focused (a tab in the background) shows.)
  typedFrom.set(field, field.value);
}
/**
 * On a phone, the title is its name, read (a tap to read it brings no keyboard); Rename (⋯) edits it where it is.
 * Elsewhere, a field as ever.
 */
const titleReads = () => (/** @type {HTMLInputElement} */ ($("title")).readOnly = phone.matches);
/** The title renamed where it is (see titleReads). */
function renameInPlace() {
  const field = /** @type {HTMLInputElement} */ ($("title"));
  field.readOnly = false;
  field.focus();
  field.select();
}
// Single-line fields: Enter is done, Escape takes back what was typed; both leave the field.
const FIELDS = "#title, #label, .number-input, .level-name";
const typedFrom = new WeakMap();
/**
 * Out of the open chart: its image (shared, where there's a share sheet for a picture: a phone's, its Photos and
 * messages; copied, opened, downloaded), its data, its file. @returns {Action[]}
 */
const exportRows = () => /** @type {[string, string, string, string, string][]} */ ([...(SHARES_IMAGES ? [["share", "Share image", "", ICONS.share, "Share"]] : []), ["copy", "Copy image", "", ICONS.copy, "Copy"], ["open", "Open image", "In a new tab", ICONS.link, "Open"], ["png", "Download image", "PNG", ICONS.image, "Download"], ["csv", "Download data", "CSV", ICONS.table, "Download"], ["json", "Download chart file", "JSON", ICONS.file, "Download"]]).map(([kind, label, note, icon, verb]) => ({ key: kind, icon, label, aside: note, verb, apart: kind === "csv", run: () => exportAs(kind) }));
/** A link copied (asked for in the click itself, its text to come: a browser lets that write). @param {Promise<string>} link */
async function copyLink(link) {
  try {
    await navigator.clipboard.write([new ClipboardItem({ "text/plain": link.then((text) => new Blob([text], { type: "text/plain" })) })]);
    return true;
  } catch {
    try {
      await navigator.clipboard.writeText(await link);
      return true;
    } catch {
      return false;
    }
  }
}
/** lean-qr, loaded the first time a QR code is asked for (its types ambient: what's used, typed here); failing, asked again next time. */
let qrModule = /** @type {Promise<{ generate: (text: string) => { size: number, get(x: number, y: number): boolean } }> | null} */ (null);
/**
 * The most a QR code holds, in characters (at its lowest correction, in its alphanumeric mode: a QR link's base32):
 * a link longer can't be one, and isn't tried; one shorter that still doesn't fit, the library refuses.
 */
const QR_MOST = 4296;
/**
 * A link as a QR code, an SVG (its modules square, whole device pixels each, each row's runs one shape), dark on white
 * as scanners read it, its quiet zone (four modules) in. Null where it's too long for one, or it can't be made.
 */
async function qrCode(/** @type {string} */ text) {
  if (new TextEncoder().encode(text).length > QR_MOST) return null;
  try {
    const { generate } = await (qrModule ??= /** @type {any} */ (/** @type {unknown} */ (import("/modules/lean-qr/2.7.3/index.mjs").catch((error) => {
      qrModule = null;
      throw error;
    }))));
    const code = generate(text), n = code.size, pad = 4, side = n + 2 * pad;
    let d = "";
    for (let y = 0; y < n; y++) {
      for (let x = 0; x < n; x++) {
        if (!code.get(x, y)) continue;
        const from = x;
        while (x + 1 < n && code.get(x + 1, y)) x++;
        d += `M${from + pad} ${y + pad}h${x - from + 1}v1h${from - x - 1}z`;
      }
    }
    // (A module whole device pixels, two at least: a dense code finer than that doesn't scan; its popover grows. As
    // tall as a sheet shows, a phone on its side's too.)
    const room = Math.min(phone.matches ? Math.min(256, innerWidth * 0.72) : 208, innerHeight * 0.7 - 24);
    const per = Math.max(2, Math.floor((room * devicePixelRatio) / side)), width = (side * per) / devicePixelRatio;
    return `<svg class="share-qr" viewBox="0 0 ${side} ${side}" width="${width}" height="${width}" shape-rendering="crispEdges" role="img" aria-label="A QR code of this chart's link"><rect width="${side}" height="${side}" fill="#fff"/><path fill="#000" d="${d}"/></svg>`;
  } catch {
    return null;
  }
}

/** A chart as a file or a link carries it: its view too, where it isn't on its preset. @param {Chart} doc */
function portable(doc) {
  if (doc === chart) saveView();
  const kept = doc === chart ? shownView() : openView(doc);
  return { v: 3, title: doc.title, index: doc.index, range: doc.range, ...(doc.halvings ? { halvings: true } : {}), ...(doc.legend ? { legend: doc.legend } : {}), panes: doc.panes, layers: doc.layers.map(({ from, ...layer }) => ((one) => (one.gradient && !one.byGradient ? { ...one, byGradient: false } : one))(from ? { ...layer, [layer.type]: from } : layer)), ...(doc.drawings?.length ? { drawings: doc.drawings } : {}), ...(kept ? { view: kept } : {}) };
}
/** The view a file or a link carried with a chart, kept for it (as `saveView` keeps one). @param {Chart} doc */
function keepCarriedView(doc, input) {
  const carried = input?.view;
  if (!isView(carried)) return;
  const all = viewsNow();
  all[doc.id] = { index: carried.index, from: carried.from, to: carried.to, ...(carried.latest ? { latest: carried.latest } : {}), at: Date.now() };
  putViews(all);
}
/** A chart's content, ids aside (a file or a link gets new ones): panes in order, layers by their pane's place, keys sorted. */
const contentOf = (/** @type {Chart} */ doc) => JSON.stringify([doc.title, doc.index, doc.range, Boolean(doc.halvings), doc.legend ?? "", doc.panes.map(({ id, ...pane }) => pane), doc.layers.map(({ id, pane, path, paths, ...layer }) => ({ ...layer, pane: doc.panes.findIndex((item) => item.id === pane), ...(layer.from ? { from: layer.from.map((member) => (EDGES.includes(member) ? member : doc.layers.findIndex((item) => item.id === member))) } : {}), ...(layer.ref ? { ref: doc.layers.findIndex((item) => item.id === layer.ref) } : {}), ...(layer.colorFrom ? { colorFrom: doc.layers.findIndex((item) => item.id === layer.colorFrom) } : {}) })), (doc.drawings ?? []).map(({ id, layer, ...drawing }) => ({ ...drawing, layer: doc.layers.findIndex((item) => item.id === layer) }))], (_key, value) => (value && typeof value === "object" && !Array.isArray(value) ? Object.fromEntries(Object.entries(value).sort(([a], [b]) => (a < b ? -1 : 1))) : value));
/** Two charts alike (a link opened again is the chart already here), whatever range each shows (a view, as a pan is). */
const sameChart = (/** @type {Chart} */ a, /** @type {Chart} */ b) => contentOf({ ...a, range: "", index: "" }) === contentOf({ ...b, range: "", index: "" });
/**
 * A chart come in (a link's, a file's) that's here already: that one, at its own range (yours: a link's isn't kept over
 * it) and the view it came with (none, as made), its
 * chunks at hand (for its views before) fetched anew; else null. @param {Chart} doc
 */
function twinOf(doc) {
  const twin = library.charts.find((other) => sameChart(other, doc));
  if (!twin) return null;
  takeView(doc.id, twin.id);
  forgetSpans(twin.id);
  return twin;
}
/** A time as a link writes it: its date ("2021-11-10"), with its time where it has one ("2021-11-10T14:30:00Z"). */
const dateText = (/** @type {number} */ time) => ((iso) => (iso.endsWith("T00:00:00Z") ? iso.slice(0, 10) : iso))(new Date(time * 1000).toISOString().replace(/\.\d+Z$/, "Z"));
/**
 * How a link says to look at its chart, each a field of its own after its address or its chart, in the words of
 * Studio's own controls: `range` or the dates `from` and `to`, and `interval`, in any case. What it doesn't know,
 * as if it weren't there.
 * @param {URLSearchParams} params
 */
function linkLookOf(params) {
  const range = params.get("range")?.toLowerCase(), interval = params.get("interval")?.toLowerCase() ?? "", from = params.get("from"), to = params.get("to"), view = { from, ...(to === null ? {} : { to }) };
  return { ...(RANGES.some(([id]) => id === range) ? { range: /** @type {string} */ (range) } : {}), ...(Object.hasOwn(INTERVAL_OF_FIELD, interval) ? { index: INTERVAL_OF_FIELD[interval] } : {}), ...(datedView("day1", view) ? { view } : {}) };
}
/** A view by its dates, as Studio keeps one, at an interval (`from` alone, up to now; one date both, that point alone); dates it can't read or in the wrong order, null. @param {string} index @param {any} view */
function datedView(index, view) {
  const from = timeOfText(view?.from), to = view?.to === undefined && from !== null ? Date.now() / 1000 : timeOfText(view?.to);
  return from !== null && to !== null && from <= to ? { index, from: [from, 0], to: [to, 0] } : null;
}
/** A time as a link has it: a date (as dateText writes it, or any ISO date), or seconds since 1970; else null. */
const timeOfText = (/** @type {any} */ value) => {
  // (Without a zone, UTC, as every date here: "2021-01-01T10:00" or "2021-01-01 10:00" is 10:00 UTC.)
  const text = typeof value === "string" ? value.trim().replace(/^(\d{4}-\d\d-\d\d)[T ](\d\d:\d\d(?::\d\d(?:\.\d+)?)?)$/, "$1T$2Z") : null;
  const time = Number.isFinite(value) ? value : text !== null && Number.isFinite(Date.parse(text)) ? Date.parse(text) / 1000 : NaN;
  return Math.abs(time) < 1e11 ? Math.floor(time) : null;
};
/**
 * A chart as one writes it into a link (see the guide, the "About" section): its defaults left out, panes by number,
 * a cloud and a drawing by their layers' letters (all its letters, where they aren't in order), times as dates; what
 * fromSketch reads back as the same chart.
 * @param {Chart} doc
 */
function toSketch(doc) {
  const place = (/** @type {string} */ id) => doc.panes.findIndex((pane) => pane.id === id) + 1;
  const plain = doc.panes.every((pane, i) => pane.mode === (i ? 0 : 1) && !pane.stack && !pane.reverse && !pane.average && pane.size === undefined);
  const ordered = doc.layers.every((layer, i) => layer.letter === letterAt(i));
  if (doc === chart) saveView();
  const dates = linkDates(doc);
  return {
    ...(doc.title ? { title: doc.title } : {}),
    // (Its interval, but 1D, a link's default.)
    ...(doc.index !== "day1" ? { index: doc.index } : {}),
    ...(doc.range !== DEFAULT_RANGE ? { range: doc.range } : {}),
    ...(doc.halvings ? { halvings: true } : {}),
    ...(doc.legend ? { legend: doc.legend } : {}),
    ...(plain ? {} : { panes: doc.panes.map(({ id, ...pane }) => pane) }),
    // (Not a series' path in the catalog: learned again where it opens.)
    layers: doc.layers.map(({ id, letter, pane, from, ref, colorFrom, type, width, dash, side, path, paths, series, formula, byGradient, ...all }) => ((rest) => ({
      // (What it is first, then how it looks.)
      ...(series !== undefined ? { series } : formula !== undefined ? { formula } : {}),
      ...(from ? { [type]: from.map((member) => doc.layers.find((item) => item.id === member)?.letter ?? member) } : {}),
      // (A signal's none (""), said too: read back, it isn't marked on another.)
      ...(ref !== undefined ? { ref: ref && (doc.layers.find((item) => item.id === ref)?.letter ?? ref) } : {}),
      ...(colorFrom ? { colorFrom: doc.layers.find((item) => item.id === colorFrom)?.letter ?? colorFrom } : {}),
      ...(ordered ? {} : { letter }),
      ...rest,
      // (A series' type always: left out, it's read as the series is mostly drawn, which one tab knows and another
      // doesn't yet: a line opened as a baseline, or candles.)
      ...((type !== "line" || series !== undefined) && !from && !heatLike({ type }) ? { type } : {}),
      // (A gradient on, said by itself; kept off, so.)
      ...(all.gradient && !byGradient ? { byGradient: false } : {}),
      ...(width !== 1.5 && type !== "cloud" ? { width } : {}),
      ...(dash && type !== "cloud" ? { dash } : {}),
      ...(side !== "right" ? { side } : {}),
      ...(place(pane) > 1 ? { pane: place(pane) } : {}),
      // (A cloud colored by its layers: its colors theirs, read back so, left out.)
    }))(all.byLayers && from?.some((member) => !EDGES.includes(member)) ? (({ color, colorBelow, byLayers, ...others }) => others)(all) : all)),
    ...(doc.drawings?.length ? { drawings: doc.drawings.map(({ id, layer, points, color, width, dash, ...rest }) => ({
      ...rest,
      layer: doc.layers.find((item) => item.id === layer)?.letter ?? layer,
      points: points.map(([time, value]) => [dateText(time), value]),
      ...(color !== PEN_COLOR ? { color } : {}),
      ...(width !== 1.5 ? { width } : {}),
      ...(dash ? { dash } : {}),
    })) } : {}),
    // (Its view at its interval: the dates of its first bar and its last, past the data too.)
    ...(dates ? { view: dates } : {}),
  };
}
/** The view a link carries for a chart: the one it shows, or was left on; its range showing, none. @param {Chart} doc */
const carriedView = (doc) => (doc === chart ? (drawnChart === chart.id ? shownView() : openingView(doc)) : openView(doc));
/** The dates a chart shows, where they aren't its range's: the first bar's and the last's, past the data too; else null. @param {Chart} doc */
function linkDates(doc) {
  const kept = carriedView(doc);
  return kept ? Object.fromEntries((/** @type {const} */ (["from", "to"])).map((edge) => [edge, dateText(timeAt(kept.index, indexAt(kept.index, kept[edge][0]) + Math.round(kept[edge][1])))])) : null;
}
/**
 * Whether a chart's view ends on the latest data, with the room a chart leaves after it (dragged past that, it ends
 * where it ends): the open one's, by its width; another's, not known, so its end is said. @param {Chart} doc
 */
// (Within half again the room after the latest (a drag's slack: one let go of past it is a view of its own).)
const endsAtLatest = (doc) => ((view) => doc === chart && Boolean(view?.latest) && (view.latest[1] * engine.time.width) / (view.latest[1] - view.latest[0]) <= rightGap() * 1.5)(carriedView(doc));
/** How a link says to look at a chart, each a field of its own (see linkLookOf): what `made` opens with anyway, left out. @param {Chart} doc @param {{ index: string, range: string }} made */
function lookFields(doc, made) {
  const dates = linkDates(doc);
  // (A time in a date's field to the minute: "2026-11-08T14:30". A view at the latest, up to it, as it comes: no end.
  // Values as the buttons show them: "1W", "Max".)
  const field = (/** @type {string} */ text) => text.replace(/:00Z$/, "");
  const look = [...(doc.range !== made.range ? [`range=${RANGES.find(([id]) => id === doc.range)?.[1] ?? doc.range}`] : []), ...(dates ? [`from=${field(dates.from)}`, ...(endsAtLatest(doc) ? [] : [`to=${field(dates.to)}`])] : [])];
  return [...look, ...(doc.index !== made.index ? [`interval=${INTERVAL_LABEL[doc.index]}`] : [])].join("&");
}
/**
 * A chart as its link's #chart= writes it, readable. (Only what a link or a chat would break on encoded: brackets and
 * braces too, a chat cuts a trailing "]}" off; a "+", read as a space by what decodes forms.)
 */
// (An "&" too: a title's "&chart=" never reads as a field of the link.)
const linkText = (/** @type {Chart} */ doc) => JSON.stringify(toSketch(doc)).replace(/[\s"<>`%#\\{}[\]+&]/g, (char) => encodeURIComponent(char));
/**
 * A chart's link, compressed (every link Studio writes; one written out, JSON, is read too: an assistant's, which
 * can't compress it); compressing failed, written out. @param {Chart} doc @param {Promise<Uint8Array>} [bytes] its
 * compressed bytes, made already
 */
const shareLink = (doc, bytes = packedBytes(doc), look = chartLook(doc)) => bytes.then((packed) => `${pageBase()}#chart=${toBase64url(packed)}${look}`, () => `${pageBase()}#chart=${linkText(doc)}`);
/** Where a chart's link opens: the Studio's own page (from a file or another server, the page's address). */
const pageBase = () => (ROUTED ? `${ORIGIN}/` : location.href.replace(/[?#].*$/, ""));
/** How to look at a chart, as what comes after its link: what a chart opens with anyway, left out. @param {Chart} doc */
const chartLook = (doc) => ((fields) => (fields ? `&${fields}` : ""))(lookFields(doc, { index: "day1", range: DEFAULT_RANGE }));
/** A chart as a link holds it, compressed: its sketch (what a written link writes), but how it's looked at, in fields of their own. @param {Chart} doc */
async function packedBytes(doc) {
  const { index: _i, range: _r, view: _v, ...sketch } = /** @type {any} */ (toSketch(doc));
  const stream = new Blob([JSON.stringify(sketch)]).stream().pipeThrough(new CompressionStream("deflate-raw"));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}
const BASE32 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
/**
 * Bytes in capitals and digits (base32), as a QR code's link holds them ("#c="): its denser alphanumeric mode takes
 * 5.5 bits a character, where base64 takes 8 in its byte mode.
 */
function toBase32(/** @type {Uint8Array} */ bytes) {
  let text = "", bits = 0, value = 0;
  for (const byte of bytes) {
    value = (value << 8) | byte;
    bits += 8;
    while (bits >= 5) text += BASE32[(value >>> (bits -= 5)) & 31];
  }
  return bits ? text + BASE32[(value << (5 - bits)) & 31] : text;
}
/** A QR link's bytes back (base32, any case). @param {string} text */
function fromBase32(text) {
  const bytes = [];
  let bits = 0, value = 0;
  for (const char of text.toUpperCase()) {
    const digit = BASE32.indexOf(char);
    if (digit < 0) throw new Error("That link doesn't contain a chart");
    value = ((value << 5) | digit) & 0xffff;
    if ((bits += 5) >= 8) bytes.push((value >>> (bits -= 8)) & 255);
  }
  return Uint8Array.from(bytes);
}
/**
 * The open chart's links, from one compressing of it (the same chart in both): the one to copy, and the one for a QR
 * code (base32: the fewest modules; compressing failed, the copy's, written out).
 */
function linksOf(doc = chart) {
  // (A preset as it's made but for how it's looked at, once the presets are read (one reloaded on shows before them):
  // its address, and that. Edited, gone from them, or none: the chart.)
  const address = doc.preset !== undefined || isMade(doc) ? loadPresets().then(() => presetLink(doc), () => null) : Promise.resolve(null);
  const links = address.then((at) => {
    if (at) return { copy: `${ORIGIN}${at}`, short: `${ORIGIN}${at}` };
    // (Its content and how it's looked at, both now: the link of one moment.)
    const bytes = packedBytes(doc), look = chartLook(doc), copy = shareLink(doc, bytes, look);
    return { copy, short: bytes.then((packed) => `${pageBase()}#c=${toBase32(packed)}${look}`, () => copy) };
  });
  return { copy: links.then((all) => all.copy), short: links.then((all) => all.short) };
}
/**
 * A link's base64 as bytes, in either alphabet (its URL-safe one, or one with "+" and "/", as links always opened:
 * its "+" read from the address as a space, a form's way, a "+" again), by the platform's own decoder where there's
 * one (a byte at a time otherwise).
 */
function fromBase64(/** @type {string} */ text) {
  const plain = text.replace(/[- ]/g, "+").replace(/_/g, "/");
  return /** @type {Uint8Array<ArrayBuffer>} */ (/** @type {any} */ (Uint8Array).fromBase64?.(plain) ?? Uint8Array.from(atob(plain), (char) => char.charCodeAt(0)));
}
/** Bytes as a link writes them: base64's URL-safe alphabet, unpadded (by the platform's own encoder where there's one). */
function toBase64url(/** @type {Uint8Array} */ bytes) {
  const native = /** @type {any} */ (bytes).toBase64?.({ alphabet: "base64url", omitPadding: true });
  if (native !== undefined) return native;
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}
/** A compressed link's chart: base64url (`#chart=`), or base32 (`#c=`, a QR code's). @param {string} encoded */
async function fromLink(encoded, base32 = false) {
  const bytes = base32 ? fromBase32(encoded) : fromBase64(encoded);
  // A chart is a few kilobytes (its drawings at most, about a megabyte): inflating stops at two, whatever the link claims.
  const reader = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("deflate-raw")).getReader();
  const chunks = [];
  for (let size = 0; ;) {
    const { done, value } = await reader.read();
    if (done) break;
    if ((size += value.length) > 2_000_000) {
      reader.cancel();
      throw new Error("Too large to be a chart");
    }
    chunks.push(value);
  }
  // (A sketch, as now; a chart's whole record, as older links held it, read as it was written: no sketch's defaults.)
  const input = JSON.parse(await new Blob(chunks).text());
  if (input?.v === undefined) return readChart(input);
  const doc = validate(input);
  keepCarriedView(doc, input);
  return doc;
}
/**
 * A chart as written by hand: a chart file's JSON with only its layers needed. Its panes by number (a layer's
 * "pane", from 1, else the first), its layers' ids and letters in order (their own kept), a cloud by its layers'
 * letters: as `validate` reads a chart file.
 */
function fromSketch(/** @type {any} */ input) {
  if (!input || typeof input !== "object" || !Array.isArray(input.layers)) throw new Error("it lists no layers");
  const layers = input.layers.filter((/** @type {any} */ layer) => layer && typeof layer === "object").slice(0, MAX_LAYERS);
  const number = (/** @type {any} */ value) => (Number.isInteger(value) ? value : typeof value === "string" && /^\d{1,3}$/.test(value) ? Number(value) : null);
  // (As many panes as given, or as the layers' numbers ask, whichever's more.)
  const asked = Math.max(1, ...layers.map((/** @type {any} */ layer) => number(layer.pane) ?? 1)), listed = Array.isArray(input.panes) ? input.panes : [];
  const given = Array.from({ length: clamp(Math.max(asked, listed.length), 1, MAX_PANES) }, (_, i) => listed[i] ?? {});
  // Ids, letters and colors given are kept; the others, the first free ones in order (none taken twice).
  const free = (/** @type {Set<string>} */ taken, /** @type {(n: number) => string} */ make) => {
    let n = 0;
    return () => {
      while (taken.has(make(n))) n++;
      const next = make(n);
      taken.add(next);
      return next;
    };
  };
  const ownString = (/** @type {any} */ value, /** @type {RegExp} */ shape) => (typeof value === "string" && shape.test(value) ? value : null);
  const paneIds = new Set(given.map((/** @type {any} */ pane) => ownString(pane?.id, /^[a-z0-9]{1,16}$/)).filter((id) => id !== null)), nextPane = free(paneIds, (n) => `p${n}`);
  const panes = given.slice(0, MAX_PANES).map((/** @type {any} */ pane, /** @type {number} */ i) => ({ ...(i ? {} : { mode: 1 }), ...pane, id: ownString(pane?.id, /^[a-z0-9]{1,16}$/) ?? nextPane() }));
  // (A layer's own id, but an edge's name: a cloud's "top" is its pane's.)
  const ownId = (/** @type {any} */ value) => ((id) => (id && !EDGES.includes(id) ? id : null))(ownString(value, /^[a-z0-9]{1,16}$/));
  const nextId = free(new Set(layers.map((/** @type {any} */ layer) => ownId(layer.id)).filter(Boolean)), (n) => `l${n}`);
  const ids = layers.map((/** @type {any} */ layer) => ownId(layer.id) ?? nextId());
  const nextLetter = free(new Set(layers.map((/** @type {any} */ layer) => ownString(layer.letter, /^[A-Z]{1,2}$/)).filter(Boolean)), letterAt);
  const letters = layers.map((/** @type {any} */ layer) => ownString(layer.letter, /^[A-Z]{1,2}$/) ?? nextLetter());
  // (A cloud without one, its first layer's, colored by its layers as one made here is; others, the palette's first
  // unused.)
  const usedColors = new Set(layers.map((/** @type {any} */ layer) => (isColor(layer.color) ? layer.color : null)).filter(Boolean));
  let cycled = 0;
  const colors = layers.map((/** @type {any} */ layer) => {
    if (isColor(layer.color)) return layer.color;
    if (Array.isArray(layer.cloud)) return null;
    // (A series' first look, its unit as known here: one asked for meanwhile, the price's ink, else the next free.)
    const look = typeof layer.series === "string" ? firstLook(seriesName(layer.series), usedColors, typeof layer.type === "string" ? layer.type : undefined) : {};
    // (Its kind's own, as one made here (see kindColor); else the next free.)
    const own = kindColor({ type: typeof layer.heatmap === "string" ? "heatmap" : typeof layer.type === "string" ? layer.type : look.type, measure: layer.measure });
    const color = look.color ?? own ?? PALETTE.find((name) => !usedColors.has(name)) ?? PALETTE[cycled++ % PALETTE.length];
    usedColors.add(color);
    return color;
  });
  const letterId = (/** @type {any} */ ref) => (typeof ref === "string" && ids.includes(ref) ? ref : ((key) => (letters.includes(key) ? ids[letters.indexOf(key)] : ref))(typeof ref === "string" ? ref.toUpperCase() : ref));
  const index = Object.hasOwn(INTERVAL_LABEL, input.index) ? input.index : "day1", dated = datedView(index, input.view);
  return { ...input,
    ...(Array.isArray(input.drawings) ? { drawings: input.drawings.slice(0, MAX_DRAWINGS).filter((/** @type {any} */ drawing) => drawing && typeof drawing === "object").map((/** @type {any} */ drawing) => ({
      ...drawing,
      layer: letterId(drawing.layer),
      points: Array.isArray(drawing.points) ? drawing.points.slice(0, MAX_STROKE).map((/** @type {any} */ point) => (Array.isArray(point) ? [timeOfText(point[0]), point[1]] : point)) : drawing.points,
    })) } : {}),
    // (A view by dates, at the chart's interval; one as Studio keeps it, as it is.)
    ...(dated ? { view: dated } : {}),
    panes, layers: layers.map((/** @type {any} */ layer, /** @type {number} */ i) => {
    const kind = Array.isArray(layer.trend) ? "trend" : Array.isArray(layer.cloud) ? "cloud" : null, members = kind ? layer[kind].map(letterId) : null;
    return {
      ...layer,
      id: ids[i],
      letter: letters[i],
      color: colors[i] ?? colors[ids.indexOf(members?.find((member) => ids.includes(member)))] ?? PALETTE[0],
      // (A cloud of no colors given: colored by its layers.)
      ...(kind === "cloud" && !colors[i] && layer.colorBelow === undefined ? { byLayers: true } : {}),
      // (A gradient given alone: what it's colored by (kept off, `byGradient: false`, said so). None, nor a color: a
      // heatmap of an amount on its steps, as one made here (see kindLook).)
      ...(Array.isArray(layer.gradient) && layer.byGradient === undefined ? { byGradient: true } : {}),
      ...(!isColor(layer.color) && layer.gradient === undefined && layer.byGradient === undefined && kindOf({ type: typeof layer.heatmap === "string" ? "heatmap" : layer.type, measure: layer.measure }) === "heat" ? { byGradient: true, gradient: structuredClone(HEAT_GRADIENT) } : {}),
      ...(number(layer.pane) !== null ? { pane: panes[clamp(/** @type {number} */ (number(layer.pane)), 1, panes.length) - 1].id } : layer.pane === undefined ? { pane: panes[0].id } : {}),
      ...(typeof layer.series === "string" ? { series: seriesName(layer.series), ...(({ type, base }) => ({ ...(type && layer.type === undefined ? { type } : {}), ...(base !== undefined && layer.base === undefined ? { base } : {}) }))(firstForm(seriesName(layer.series), typeof layer.type === "string" ? layer.type : undefined)) } : {}),
      ...(kind ? { [kind]: members } : {}),
      ...(typeof layer.ref === "string" ? { ref: letterId(layer.ref) } : {}),
      ...(typeof layer.colorFrom === "string" ? { colorFrom: letterId(layer.colorFrom) } : {}),
    };
  }) };
}
/** A series' name as a sketch may write it ("Realized price", "realized-price"): as the catalog does. */
const seriesName = (/** @type {string} */ text) => text.trim().toLowerCase().replace(/[\s-]+/g, "_");
/**
 * A chart as it comes in, however it was written: a link's sketch, a file, an item of a file of several (a whole
 * chart passes through the sketch's reading as it is), validated, its view kept.
 */
function readChart(/** @type {any} */ input) {
  const sketch = fromSketch(input), doc = validate(sketch);
  keepCarriedView(doc, sketch);
  return doc;
}
/** A line's style, read back: its color and line style where valid. */
function lineStyleOf(line) {
  return { ...(isColor(line?.color) ? { color: line.color } : {}), ...(DASHES.some(([style]) => style === line?.dash) ? { dash: line.dash } : {}), ...(line?.line === false ? { line: /** @type {false} */ (false) } : {}), ...(line?.tag === true ? { tag: true } : {}) };
}
/** A layer's levels, read back: each a value, with its color and line style if valid (a plain number is a value); none, nothing. @returns {Level[] | undefined} */
function levelsOf(input) {
  if (!Array.isArray(input)) return undefined;
  const levels = input.map((level) => (typeof level === "number" ? { value: level } : level)).filter((level) => Number.isFinite(level?.value)).slice(0, 12);
  return levels.length ? levels.map((level) => ({ value: level.value, ...lineStyleOf(level), ...(typeof level.name === "string" && level.name.trim() ? { name: level.name.trim().slice(0, 40) } : {}) })) : undefined;
}
/** A gradient read back: 2 to 4 stops, each a place (0 to 100, whole) and a color, in order; else none. @returns {[number, string][] | undefined} */
function gradientOf(/** @type {any} */ input) {
  if (!Array.isArray(input) || input.length < GRADIENT_LEAST || input.length > GRADIENT_MOST) return undefined;
  const stops = input.filter((stop) => Array.isArray(stop) && Number.isFinite(stop[0]) && isColor(stop[1])).map(([at, color]) => /** @type {[number, string]} */ ([Math.round(clamp(at, 0, 100)), color]));
  if (stops.length !== input.length) return undefined;
  // (Each its own place, a point apart at least (one on another would hide it, see moveStop): up from the first,
  // then back from 100.)
  stops.sort((a, b) => a[0] - b[0]);
  for (let k = 1; k < stops.length; k++) stops[k][0] = Math.max(stops[k][0], stops[k - 1][0] + 1);
  for (let k = stops.length - 1; k >= 0; k--) stops[k][0] = Math.min(stops[k][0], k < stops.length - 1 ? stops[k + 1][0] - 1 : 100);
  return stops;
}
/**
 * A formula as Studio writes it now, from one saved before: lag( (shift's name before it, the same meaning) as shift(,
 * where that still fits (else as it was: one cut would be broken). Declared, not a const: charts are read at the
 * start, before the script below runs.
 */
function formulaNow(/** @type {string} */ formula) {
  const now = formula.replace(/(?<![A-Za-z0-9_])lag(?=\s*\()/g, "shift");
  return now.length <= FORMULA_MAX ? now : formula;
}
/** Validates chart fields (only `keepIds` preserves valid, unique ids). @returns {Chart} */
function validate(input, keepIds = false) {
  const taken = new Set();
  const idOf = (id) => (keepIds && typeof id === "string" && /^[a-z0-9]{1,16}$/.test(id) && !taken.has(id) ? (taken.add(id), id) : uid());
  if (!input || !Array.isArray(input.layers) || !Array.isArray(input.panes) || !input.panes.length) throw new Error("Not a studio chart");
  const doc = blankChart(String(input.title ?? "").slice(0, 120));
  doc.index = Object.hasOwn(INTERVAL_LABEL, input.index) ? input.index : "day1";
  doc.range = RANGES.some(([id]) => id === input.range) ? input.range : DEFAULT_RANGE;
  if (input.halvings === true) doc.halvings = true;
  if (input.legend === "off") doc.legend = "off";
  const panes = input.panes.slice(0, MAX_PANES);
  doc.panes = panes.map((pane) => ({ id: idOf(pane?.id), mode: [0, 1, 2, 3].includes(pane?.mode) ? pane.mode : 0, ...(Number.isFinite(pane?.size) && pane.size > 0 && pane.size < 100 ? { size: pane.size } : {}), ...(STACKS.some(([id]) => id && id === pane?.stack) ? { stack: pane.stack } : {}), ...(pane?.reverse === true ? { reverse: true } : {}), ...(Object.hasOwn(AVERAGES, pane?.average) ? { average: pane.average } : {}) }));
  const paneFor = (id) => doc.panes[Math.max(0, panes.findIndex((pane) => pane?.id === id))].id;
  /** Each layer's id as read, to its own: its drawings follow it. */
  const layerIds = new Map();
  for (const layer of input.layers.slice(0, MAX_LAYERS)) {
    if (!layer || typeof layer !== "object") continue;
    // Made from layers: as a link writes it ("cloud", "trend"), or as a file keeps it ("from", and its type).
    const made = Array.isArray(layer.trend) ? "trend" : Array.isArray(layer.cloud) ? "cloud" : Array.isArray(layer.from) && (layer.type === "cloud" || layer.type === "trend") ? layer.type : null;
    const madeOf = made ? layer.trend ?? layer.cloud ?? layer.from : null;
    // (A heatmap: its cohort, as the URPD names it, and its weight, measure and densities where valid.)
    const heat = !made && typeof layer.heatmap === "string" && /^[a-z0-9_]{1,40}$/.test(layer.heatmap) ? layer.heatmap : null;
    // (A profile: its holder groups, one or a list, and each one's own color where given.)
    const cohort = (/** @type {any} */ id) => typeof id === "string" && /^[a-z0-9_]{1,40}$/.test(id);
    const groups = !made && !heat ? [...new Set((Array.isArray(layer.profile) ? layer.profile : [layer.profile]).filter(cohort))].slice(0, 40) : [];
    const profiled = !made && !heat && (Array.isArray(layer.profile) || cohort(layer.profile));
    /** @type {Record<string, any>} */
    const spec = {
      ...(heat || profiled ? { ...(heat ? { heatmap: heat } : { profile: groups, ...(layer.cumulative === true ? { cumulative: true } : {}), ...(layer.cohortColors && typeof layer.cohortColors === "object" ? ((colors) => (Object.keys(colors).length ? { cohortColors: colors } : {}))(Object.fromEntries(Object.entries(layer.cohortColors).filter(([id, color]) => groups.includes(id) && isColor(color)))) : {}) }), weight: HEAT_WEIGHTS.some(([id]) => id && id === layer.weight) ? layer.weight : undefined, measure: HEAT_MEASURES.some(([id]) => id !== DEFAULT_MEASURE && id === layer.measure) ? layer.measure : undefined, rows: HEAT_ROWS.includes(layer.rows) ? layer.rows : undefined } : madeOf ? { from: madeOf.filter((id) => typeof id === "string").map((id) => (EDGES.includes(id.toLowerCase()) ? id.toLowerCase() : id)).slice(0, MAX_LAYERS + 2) } : typeof layer.series === "string"
        ? { series: isSeriesName(layer.series) ? layer.series : "invalid_series_name", ...(isCatalogPath(layer.path) ? { path: layer.path } : {}) }
        : { formula: formulaNow(String(layer.formula ?? "").slice(0, FORMULA_MAX)), ...(layer.paths && typeof layer.paths === "object" ? { paths: Object.fromEntries(Object.entries(layer.paths).filter(([name, path]) => isSeriesName(name) && isCatalogPath(path)).slice(0, 40)) } : {}) }),
      ...(typeof layer.label === "string" ? { label: layer.label.slice(0, 120) } : {}),
      color: isColor(layer.color) ? layer.color : undefined,
      type: heat ? "heatmap" : profiled ? "profile" : made ?? (TYPES.some(([type]) => type === layer.type) || layer.type === "bubbles" || (layer.type === "signal" && typeof layer.series !== "string") ? layer.type : "line"),
      width: WIDTHS.includes(layer.width) ? layer.width : 1.5,
      dash: DASHES.some(([style]) => style === layer.dash) ? layer.dash : 0,
      pane: paneFor(layer.pane),
      side: layer.side === "left" ? "left" : "right",
      hidden: layer.hidden === true || undefined,
      priceLine: layer.priceLine === true || undefined,
      outline: layer.outline === true || undefined,
      fit: layer.fit === false || layer.fit === "zero" ? layer.fit : undefined,
      maxInterval: INTERVAL_ORDER.includes(layer.maxInterval) ? layer.maxInterval : undefined,
      colorBelow: isColor(layer.colorBelow) ? layer.colorBelow : undefined,
      colors: layer.colors === 1 || layer.colors === 4 ? layer.colors : undefined,
      colorFading: isColor(layer.colorFading) ? layer.colorFading : undefined,
      colorBelowFading: isColor(layer.colorBelowFading) ? layer.colorBelowFading : undefined,
      opacity: OPACITIES.includes(layer.opacity) && layer.opacity !== defaultOpacity({ type: made ?? "line" }) ? layer.opacity : undefined,
      letter: typeof layer.letter === "string" && /^[A-Z]{1,2}$/.test(layer.letter) ? layer.letter : undefined,
      ...(layer.format !== "auto" && FORMATS.some(([id]) => id === layer.format) ? { format: layer.format } : {}),
      ...(Number.isFinite(layer.base) ? { base: layer.base } : {}),
      ...(SHAPES.some(([id]) => id !== "straight" && id === layer.shape) ? { shape: layer.shape } : {}),
      ...(layer.fill === "above" ? { fill: "above" } : {}),
      ...(OPACITIES.includes(layer.fillOpacity) ? { fillOpacity: layer.fillOpacity } : {}),
      ...(layer.solid === true ? { solid: true } : {}),
      ...(layer.hollow === "none" ? { hollow: "none" } : {}),
      ...((made === "trend" || (!made && (layer.type === "signal" || layer.type === "bubbles"))) && typeof layer.ref === "string" ? { ref: layer.ref } : {}),
      ...(!made && layer.type === "bubbles" ? { size: BUBBLE_SIZES.includes(layer.size) && layer.size !== BUBBLE_SIZE ? layer.size : undefined, hide: BUBBLE_HIDES.includes(layer.hide) && layer.hide !== BUBBLE_HIDE ? layer.hide : undefined, scaleTo: layer.scaleTo === "all" ? "all" : undefined } : {}),
      ...(MARK_PLACES.some(([place]) => place !== "above" && place === layer.place) ? { place: layer.place } : {}),
      ...(!made && layer.type === "signal" && layer.results === true ? { results: true } : {}),
      ...(typeof layer.colorFrom === "string" ? { colorFrom: layer.colorFrom } : {}),
      ...(layer.byLevels === true ? { byLevels: true } : {}),
      ...(layer.byLayers === true ? { byLayers: true } : {}),
      // (A gradient, on where it's said to be (a link's alone, see fromSketch); off, kept.)
      ...((stops) => (stops ? { gradient: stops, ...(layer.byGradient === true ? { byGradient: true } : {}), ...(layer.upToDate === true ? { upToDate: true } : {}) } : {}))(gradientOf(layer.gradient)),
      levels: levelsOf(layer.levels),
      ...(typeof layer.underName === "string" && layer.underName.trim() && levelsOf(layer.levels) ? { underName: layer.underName.trim().slice(0, 40) } : {}),
      baseLine: Object.keys(lineStyleOf(layer.baseLine)).length ? lineStyleOf(layer.baseLine) : undefined,
    };
    const id = idOf(layer.id);
    if (typeof layer.id === "string") layerIds.set(layer.id, id);
    doc.layers.push(makeLayer({ ...Object.fromEntries(Object.entries(spec).filter(([, value]) => value !== undefined)), id }, doc));
  }
  // What layers are made from, by their ids here, and a cloud's edges (normalize leaves out what's not a layer).
  // (A layer named that isn't one: none, to pick (not another put in its place).)
  for (const layer of doc.layers) if (layer.ref) layer.ref = layerIds.get(layer.ref) ?? "";
  // (A signal said without its layer: marked on the first that can be.)
  for (const layer of doc.layers) if (layer.type === "signal" && layer.ref === undefined) layer.ref = markable(doc)[0]?.id;
  for (const layer of doc.layers) if (layer.colorFrom) layer.colorFrom = layerIds.get(layer.colorFrom);
  for (const layer of doc.layers) if (layer.from) layer.from = layer.from.flatMap((id) => (EDGES.includes(id) ? [id] : layerIds.has(id) ? [/** @type {string} */ (layerIds.get(id))] : []));
  // Drawings: each on a layer read, its points numbers (as many as its kind takes), its look where valid.
  doc.drawings = (Array.isArray(input.drawings) ? input.drawings : []).slice(0, MAX_DRAWINGS).flatMap((drawing) => {
    const kind = drawing?.kind, layer = layerIds.get(drawing?.layer);
    if (!Object.hasOwn(TOOLS, kind) || !layer || !Array.isArray(drawing.points)) return [];
    const points = drawing.points.slice(0, MAX_STROKE).filter((point) => Array.isArray(point) && point.length === 2 && point.every(Number.isFinite) && Math.abs(point[0]) < 1e11).map(([time, value]) => pointOf(time, value));
    // (A stroke's points or a path's, as many as it has (a path two at least); else as many as its kind takes.)
    const takes = kind === "brush" || kind === "path" ? MAX_STROKE : PAIRED.has(kind) ? 2 : 1, least = kind === "path" ? 2 : kind === "brush" ? 1 : takes, text = String(drawing.text ?? "").slice(0, 500);
    if (points.length < least || (kind === "text" && !text.trim())) return [];
    return [{
      id: idOf(drawing.id), kind, layer, points: points.slice(0, takes),
      ...lookOfDrawing(kind, drawing),
      ...(kind === "text" ? { text } : {}),
      ...(LINES.has(kind) && typeof drawing.label === "string" && labelText(drawing.label) ? { label: labelText(drawing.label), ...(Number.isFinite(drawing.at) ? { at: clamp(drawing.at, 0, 1) } : {}) } : {}),
    }];
  });
  normalize(doc);
  return doc;
}

function download(name, blob) {
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = name;
  link.click();
  // (Kept while a browser may still be asking where it goes: Safari's iOS sheet, after a while.)
  setTimeout(() => URL.revokeObjectURL(link.href), 40000);
}
/** A name as a file's: words joined by dashes, lowercase. */
const slugOf = (/** @type {string} */ text) => text.trim().replace(/[^\w-]+/g, "-").replace(/^-|-$/g, "").toLowerCase();
const fileName = (extension, doc = chart) => `${slugOf(titleOf(doc) || "chart") || "chart"}.${extension}`;
/**
 * Several charts in one file (their ids with them), their folders (`folders`: those they're in, at any depth) and
 * their order in the sidebar too: imported anywhere else, they're arranged as here.
 */
function chartsFile(/** @type {Chart[]} */ docs, folders = library.folders.filter((folder) => docs.some((doc) => isIn(doc.folder, folder.id))), /** @type {any[]} */ also = [], /** @type {Chart[]} */ edited = []) {
  const ids = new Set(docs.map((doc) => doc.id)), kept = new Set(folders.map((folder) => `f:${folder.id}`));
  const charts = [...docs.map((doc) => ({ id: doc.id, ...(doc.folder ? { folder: doc.folder } : {}), ...portable(doc) })), ...also];
  // (The presets as edited, by which they are: imported, edited there too.)
  const presetsToo = edited.map((doc) => ({ preset: doc.preset, ...(doc.base ? { base: doc.base } : {}), ...portable(doc) }));
  return new Blob([JSON.stringify({ v: 2, charts, folders, order: treeOf().map(({ item }) => sideKey(item)).filter((key) => kept.has(key) || ids.has(key.slice(2))), ...(presetsToo.length ? { presets: presetsToo } : {}) }, null, 2)], { type: "application/json" });
}
/** Charts kept aside (saved, but not readable here), as they were. @returns {any[]} */
const keptAside = () => ((aside) => (Array.isArray(aside) ? aside : []))(storage.get(UNREAD, []));
const asideCount = () => keptAside().length;
/** Whether there's anything Download all would take: your charts, the presets as edited, those kept aside. */
const anythingKept = () => library.charts.length > 0 || library.folders.length > 0 || library.presets.some(presetEdited) || asideCount() > 0;
/** All the charts in one file, every folder with them (an empty one too), the presets as edited, and those kept aside, as they were. */
/** Every chart of yours, its folders, the edited presets (`rescue`: what isn't saved, its previews too) in a file. */
function downloadAll(rescue = false) {
  const aside = keptAside(), edited = library.presets.filter(presetEdited), charts = rescue ? [...library.charts, ...listedPreviews()] : library.charts;
  if (!charts.length && !library.folders.length && !aside.length && !edited.length) return notice("No charts yet");
  download(`bitview-charts-${new Date().toLocaleDateString("en-CA")}.json`, chartsFile(charts, library.folders, aside, edited));
  toast(`Downloaded ${plural(charts.length, "chart")}${library.folders.length ? ` and ${plural(library.folders.length, "folder")}` : ""}${edited.length ? `, ${plural(edited.length, "edited preset")}` : ""}${aside.length ? `, and ${aside.length} kept aside` : ""}, in one file`);
}
/** A folder's charts, at any depth, in a file with it and the folders in it: imported, made anew as here. */
function downloadFolder(/** @type {string} */ id) {
  const folder = folderById(id);
  if (!folder) return;
  const docs = library.charts.filter((doc) => isIn(doc.folder, id)), folders = library.folders.filter((other) => isIn(other.id, id));
  if (!docs.length && folders.length === 1) return notice("Nothing in it yet");
  download(`bitview-${slugOf(folder.name) || "folder"}-${new Date().toLocaleDateString("en-CA")}.json`, chartsFile(docs, folders));
  toast(`Downloaded “${folder.name}”: ${plural(docs.length, "chart")}${folders.length > 1 ? ` and ${plural(folders.length - 1, "folder")}` : ""}, in one file`);
}
function downloadChart(/** @type {Chart} */ doc) {
  download(fileName("json", doc), new Blob([JSON.stringify(portable(doc), null, 2)], { type: "application/json" }));
  toast(`Downloaded “${titleOf(doc)}”`);
}

/**
 * Layers' data as a CSV (the chart's whole, Export's; one layer's, its own actions'), as the chart has it: one row
 * per date at its interval, a column per layer (a cloud's, its low and its high but for an edge), the hidden ones too.
 * @param {Layer[]} which
 */
async function exportData(which) {
  // (Still loading, what's there may be what was before: nothing then.)
  if (!settled()) return notice("The chart is still loading");
  const doc = structuredClone(chart), requested = new Set(which.map((layer) => layer.id));
  const data = await resolve({ ...doc, id: `${doc.id}:export`, layers: doc.layers.map(layer => ({ ...layer, hidden: false })) });
  const layers = doc.layers.filter((layer) => requested.has(layer.id) && data.results.get(layer.id)?.values);
  if (!layers.length) return notice("Its data isn't here yet");
  const quote = (text) => `"${String(text).replace(/"/g, '""')}"`;
  // (A cloud's columns: its low, its high, but for an edge.)
  const ends = (/** @type {Layer} */ layer) => (layer.type === "cloud" ? ["low", "high"].filter((end) => !layer.from?.includes(end === "low" ? "bottom" : "top")) : [""]);
  const header = ["date", ...layers.flatMap((layer) => ends(layer).map((end) => quote(`${layer.letter} ${viewName(doc, layer)}${end ? ` (${end})` : ""}`)))].join(",");
  const columns = layers.flatMap((layer) => ((result) => ends(layer).map((end) => (end === "low" ? result?.low : result?.values)))(data.results.get(layer.id)));
  const rows = data.times.map((time, i) => {
    const cells = columns.map((values) => ((value) => (value !== undefined && Number.isFinite(value) ? String(value) : ""))(values?.[i]));
    return cells.some(Boolean) ? [new Date(time * 1000).toISOString().slice(0, FINE.has(doc.index) ? 16 : 10), ...cells].join(",") : null;
  }).filter(Boolean);
  download(requested.size === 1 && doc.layers.length > 1 ? fileName("csv", doc).replace(/\.csv$/, `-${labelOf(layers[0]).trim().replace(/[^\w-]+/g, "-").replace(/^-|-$/g, "").toLowerCase() || "layer"}.csv`) : fileName("csv", doc), new Blob([[header, ...rows].join("\n")], { type: "text/csv" }));
  return toast(`Downloaded ${grouped.format(rows.length)} rows`);
}
function exportAs(kind) {
  if (kind === "json") return downloadChart(chart);
  if (!chart.layers.length) return notice("Add a layer first");
  if (!settled() || !resolved) return notice("The chart is still loading");
  if (!resolved.times.length) return notice("This chart has no data yet");
  if (kind === "csv") return exportData(chart.layers);
  if (kind === "copy") return copyImage();
  if (kind === "share") return shareImage();
  if (kind === "open") return openImage();
  const name = fileName("png");
  chartImage(pictureSize()).then(pngOf).then((blob) => (download(name, blob), toast("Downloaded the image")), (error) => notice(imageTrouble(error)));
}
/** A canvas as a PNG file's contents. @param {HTMLCanvasElement} canvas @returns {Promise<Blob>} */
const pngOf = (canvas) => new Promise((resolve, reject) => canvas.toBlob((blob) => (blob ? resolve(blob) : reject(new Error("No image"))), "image/png"));
/** Why the image couldn't be made, as said: unreadable here (see canvasReadable), else plainly. @param {unknown} error */
const imageTrouble = (error) => (error instanceof Error && (error.message === UNREADABLE || error.message === TOO_SMALL) ? error.message : "Couldn't make the image");
/**
 * The chart's picture alone, in a tab of its own (titled with the chart's): to look at, save, or show (an assistant
 * in a browser shares this, not a screenshot of the page). Round it, the picture's own ground (nothing framing it);
 * its address kept as long as this page is (dropped any sooner, saving it fails). The tab opened at once, while the
 * click counts: one opened once it's drawn is blocked.
 */
function openImage() {
  const tab = window.open("", "_blank");
  if (!tab) return void notice("Couldn't open a tab: allow pop-ups here, or download the image");
  tab.document.title = `${titleOf(chart)} · Bitview Studio`;
  chartImage(pictureSize()).then((canvas) => pngOf(canvas).then((blob) => /** @type {const} */ ([canvas, blob]))).then(([canvas, blob]) => {
    if (tab.closed) return;
    const url = URL.createObjectURL(new File([blob], fileName("png"), { type: "image/png" })), page = tab.document, image = page.createElement("img");
    openedImages.push(url);
    tab.addEventListener("pagehide", () => {
      URL.revokeObjectURL(url);
      openedImages.splice(openedImages.indexOf(url) >>> 0, 1);
    }, { once: true });
    // (At its size on screen, as on a screen of this density; smaller where the window is.)
    Object.assign(image, { src: url, alt: titleOf(chart) });
    image.style.cssText = `display: block; width: ${canvas.width / devicePixelRatio}px; max-width: 100%; height: auto`;
    page.body.style.cssText = `margin: 0; min-height: 100vh; display: grid; place-items: center; background: ${css("--card")}`;
    page.body.replaceChildren(image);
  }, (error) => (tab.close(), notice(imageTrouble(error))));
}
/** The images opened in tabs of their own, their addresses let go of as this page goes. @type {string[]} */
const openedImages = [];
/** The image through the share sheet, asked for in the tap itself (drawn meanwhile: it still counts); let go of, nothing said. */
function shareImage() {
  if (!canvasReadable()) return void notice(UNREADABLE);
  const name = fileName("png");
  chartImage(pictureSize())
    .then(pngOf)
    .then((blob) => navigator.share({ files: [new File([blob], name, { type: "image/png" })], title: titleOf(chart) }))
    .catch((/** @type {unknown} */ error) => error instanceof Error && error.name !== "AbortError" && notice(error.message === UNREADABLE ? UNREADABLE : "Couldn't share the image"));
}
/** The image on the clipboard, given the picture while it's still being drawn (so the click that asked counts). */
function copyImage() {
  if (!canvasReadable()) return void notice(UNREADABLE);
  const png = chartImage(pictureSize()).then(pngOf);
  navigator.clipboard.write([new ClipboardItem({ "image/png": png })]).then(() => toast("Copied the image: paste it anywhere"), () => notice("Couldn't copy the image"));
}
/**
 * Whether this browser gives a page its own drawing back: some keep it (a privacy setting, as Mullvad's and Tor's),
 * giving noise unless the site's allowed. Asked each time (allowed since, it's given).
 */
function canvasReadable() {
  try {
    // (Many pixels of many colors, as a chart's are: a randomizer spares a single pixel, and a plain color.)
    const size = 16, context = /** @type {CanvasRenderingContext2D} */ (Object.assign(document.createElement("canvas"), { width: size, height: size }).getContext("2d", { willReadFrequently: true }));
    for (let x = 0; x < size; x++) for (let y = 0; y < size; y++) {
      context.fillStyle = `rgb(${x * 16} ${y * 16} ${(x * 7 + y * 11) % 256})`;
      context.fillRect(x, y, 1, 1);
    }
    const data = context.getImageData(0, 0, size, size).data;
    for (let x = 0; x < size; x++) for (let y = 0; y < size; y++) {
      const at = (y * size + x) * 4;
      if (data[at] !== x * 16 || data[at + 1] !== y * 16 || data[at + 2] !== (x * 7 + y * 11) % 256) return false;
    }
    return true;
  } catch {
    return false;
  }
}
const UNREADABLE = "A privacy setting in this browser blocks reading the chart's image. Allow it for this site to export images";
/** Why a picture couldn't be made at a size: its legends don't fit under a plot there, even laid out larger. */
const TOO_SMALL = "Its legends don't fit a picture this size: make it larger";
/**
 * The size a picture of yours is taken at: as the chart shows in a window wide enough; narrower, or upright (a
 * phone's), a picture's own, 16:9, as one's shared, not a strip as tall as the screen.
 */
const pictureSize = () => ((box) => (box.width >= 640 && box.width > box.height ? {} : { width: 1200, height: 675 }))($("chart").getBoundingClientRect());
/** What a picture's of: the chart open, as edited (its interval too, and back), the charts opened so far. */
const shownAs = () => `${chart.id} ${revision} ${navigations}`;
/**
 * The view's dates, as the foot says them, and the date of the values the legends show (the view's last point): each
 * written whole, as "Values on" writes its own (the navigator's tight form, ’18, beside a whole 2026 read as two).
 */
function viewDates() {
  const times = resolved?.times ?? [], span = shownDates();
  if (!times.length || !span) return { span: "", values: "" };
  const last = times[lastShown()], [from, to] = [pointLabel(span.from), pointLabel(span.to)];
  return { span: from === to ? to : `${from} – ${to}`, values: chart.legend === "off" ? "" : `Values on ${pointLabel(last)}` };
}
/** A text cut with an ellipsis to fit a width, as the page cuts a long name. */
function fitted(context, text, width) {
  if (context.measureText(text).width <= width + 1) return text;
  let cut = text;
  while (cut.length > 1 && context.measureText(`${cut}…`).width > width) cut = cut.slice(0, -1);
  return `${cut.trimEnd()}…`;
}
/**
 * Each pane's legend as the picture writes it (see presentLegends), at rest: its layers' marks, names and values, in
 * the chart's own coordinates, scaled.
 * @returns {{ x: number, y: number, width: number, height: number, svg?: string, image?: HTMLImageElement, text?: string, font?: string, color?: string, align?: CanvasTextAlign, faded?: boolean, clip?: number, cut?: boolean }[]}
 */
function measureLegends(scale) {
  const shown = legends(), origin = engine.element.getBoundingClientRect();
  const box = (element) => {
    const rect = element.getBoundingClientRect();
    return { x: (rect.left - origin.left) * scale, y: (rect.top - origin.top) * scale, width: rect.width * scale, height: rect.height * scale };
  };
  const pieces = [];
  // (Words the page cuts (wider than their box): cut in the picture too; the others whole, however a canvas
  // measures them (a hair wider than the page lays them, at another size).)
  const cutOff = (/** @type {HTMLElement} */ element) => element.scrollWidth > element.clientWidth;
  /** A key as it shows: its strip's colors in their order, a profile's groups' squares; its names where they show. @param {HTMLElement} key @param {boolean} faded */
  const keyPieces = (key, faded) => {
    const strip = /** @type {HTMLElement | null} */ (key.querySelector(".key-strip")), cells = strip ? all$("i", strip) : [];
    if (strip) pieces.push({ ...box(strip), faded, svg: `<svg viewBox="0 0 ${cells.length} 1" width="${strip.offsetWidth}" height="${strip.offsetHeight}" preserveAspectRatio="none">${cells.map((cell, i) => `<rect x="${i}" width="1.02" height="1" fill="${getComputedStyle(cell).backgroundColor}"/>`).join("")}</svg>` });
    for (const square of /** @type {HTMLElement[]} */ (all$(":scope > span:not(.key-strip) > i, :scope > i", key))) pieces.push({ ...box(square), faded, svg: `<svg viewBox="0 0 8 8" width="${square.offsetWidth}" height="${square.offsetHeight}"><rect width="8" height="8" rx="2" fill="${getComputedStyle(square).backgroundColor}"/></svg>` });
    for (const words of /** @type {HTMLElement[]} */ (all$(":scope > span:not(.key-strip, :has(> i)), :scope > span > span", key))) {
      if (!words.getClientRects().length) continue;
      const style = getComputedStyle(words);
      pieces.push({ ...box(words), faded, text: words.textContent?.trim() ?? "", font: `${style.fontWeight} ${parseFloat(style.fontSize) * scale}px ${style.fontFamily}`, color: style.color, align: /** @type {CanvasTextAlign} */ ("left"), cut: cutOff(words) });
    }
  };
  for (const legend of shown) {
    const edge = legend.getBoundingClientRect().right;
    // (A profile's key, as it shows.)
    for (const key of /** @type {HTMLElement[]} */ (all$(":scope > .legend-key", legend))) if (key.offsetParent) keyPieces(key, false);
    // (Its items set apart: under the plot, see apartLayout.)
    if (legend.hasAttribute("data-apart")) continue;
    for (const button of legend.querySelectorAll("button")) {
      // (Hidden, or past its edge.)
      if (button.style.display === "none" || !button.offsetParent || button.getBoundingClientRect().right > edge + 1) continue;
      const mark = button.querySelector(".series-mark"), faded = button.classList.contains("hidden");
      if (mark) pieces.push({ ...box(mark), svg: mark.outerHTML, faded });
      for (const element of /** @type {HTMLElement[]} */ ([button.querySelector(".name"), button.querySelector(".legend-value, .legend-error")])) {
        if (!element?.textContent?.trim() || !element.getClientRects().length) continue;
        // (Its words where they're drawn: past its padding; a value beside its name.)
        const style = getComputedStyle(element), inset = parseFloat(style.paddingLeft) * scale, at = box(element);
        const text = element.textContent.trim(), after = getComputedStyle(element, "::after");
        // (A value older than the view's end: its date after it as the page shows it (its ::after, not in its text,
        // where a rule shows it), quieter; the value's words up to where they end.)
        const asOf = element.dataset.asOf && after.content !== "none" && after.display !== "none" ? element.dataset.asOf : "";
        const range = asOf ? document.createRange() : null;
        range?.selectNodeContents(element);
        const end = range ? ((words) => words.x + words.width)(box(range)) : at.x + at.width;
        pieces.push({ ...at, x: at.x + inset, width: end - at.x - inset, faded, text, font: `${style.fontWeight} ${parseFloat(style.fontSize) * scale}px ${style.fontFamily}`, color: style.color, align: /** @type {CanvasTextAlign} */ (["right", "end"].includes(style.textAlign) ? "right" : "left"), cut: cutOff(element) });
        if (asOf) {
          pieces.push({ ...at, x: end, width: at.x + at.width - end, faded, text: ` · ${asOf}`, font: `${after.fontWeight} ${parseFloat(after.fontSize) * scale}px ${after.fontFamily}`, color: after.color, align: /** @type {CanvasTextAlign} */ ("left") });
        }
      }
    }
  }
  return pieces;
}
/**
 * A picture's long or truncated legends (as presentLegends wrote them, the panes at the picture's width): set apart,
 * under the plot (see apartItems), the plot whole, none of it over its lines; a key (profile groups or cloud colors)
 * stays over the plot. Undone by what it returns.
 * @returns {() => void}
 */
function setApart() {
  const apart = legends().filter((legend) => {
    const first = /** @type {HTMLElement | undefined} */ (all$("button", legend).find((button) => button.style.display !== "none")), row = first?.offsetHeight ?? 0;
    return row > 0 && (legend.offsetHeight > row * 2 + 6 || all$(".name", legend).some((name) => name.scrollWidth > name.clientWidth));
  }).map((legend) => /** @type {[HTMLElement, string]} */ ([legend, legend.style.display]));
  for (const [legend] of apart) {
    legend.dataset.apart = "";
    if (!legend.querySelector(".legend-key")) legend.style.display = "none";
  }
  return () => {
    for (const [legend, display] of apart) delete legend.dataset.apart, (legend.style.display = display);
  };
}
/** A picture's legends set apart (see setApart): their rows, the space over, under and between them. */
const APART = { row: 20, top: 6, bottom: 10, gap: 8, columnGap: 14, textGap: 8, mark: 16, markHeight: 12, markGap: 6, size: 12, nameWeight: 510, valueWeight: 590 };
const apartFonts = (scale = 1, sans = css("--sans")) => ({ name: `${APART.nameWeight} ${APART.size * scale}px ${sans}`, value: `${APART.valueWeight} ${APART.size * scale}px ${sans}` });
/** A picture's legend layout, measured in CSS pixels once for both its height and its words. */
function apartLayout(/** @type {number} */ width) {
  const groups = apartItems();
  const g = measure, fonts = apartFonts();
  // (A column as wide as an item's longest word and its value: its name wraps, one long name no reason for one column.)
  const itemWidth = (item) => {
    g.font = fonts.name;
    const name = Math.max(0, ...item.name.split(/\s+/).map((word) => g.measureText(word).width));
    g.font = fonts.value;
    return name + g.measureText(item.value).width + (item.mark ? APART.mark + APART.markGap : 0) + APART.columnGap + APART.textGap;
  };
  let widest = width < 500 ? 150 : 175;
  for (const group of groups) for (const item of group) widest = Math.max(widest, itemWidth(item));
  const cols = Math.max(1, Math.floor(width / widest)), column = width / cols;
  const items = [];
  let y = APART.top;
  for (const group of groups) {
    for (let start = 0; start < group.length; start += cols) {
      let rows = 1;
      group.slice(start, start + cols).forEach((item, i) => {
        g.font = fonts.value;
        const mark = item.mark ? APART.mark + APART.markGap : 0, valueWidth = Math.min(g.measureText(item.value).width, Math.max(0, column - APART.columnGap - APART.textGap - mark - 48)), nameWidth = Math.max(1, column - APART.columnGap - APART.textGap - valueWidth - mark);
        g.font = fonts.name;
        const lines = [""];
        // Break words first, and split a single word only when it cannot fit even on its own line.
        for (const word of item.name.split(/\s+/)) {
          let at = lines.length - 1;
          if (lines[at] && g.measureText(`${lines[at]} ${word}`).width > nameWidth) lines.push("");
          if (g.measureText(word).width <= nameWidth) {
            lines[lines.length - 1] += `${lines.at(-1) ? " " : ""}${word}`;
            continue;
          }
          for (const char of word) {
            at = lines.length - 1;
            if (lines[at] && g.measureText(lines[at] + char).width > nameWidth) lines.push("");
            lines[lines.length - 1] += char;
          }
        }
        rows = Math.max(rows, lines.length);
        items.push({ ...item, lines, nameWidth, valueWidth, x: i * column, y });
      });
      y += rows * APART.row;
    }
    y += APART.gap;
  }
  return { items, column, height: groups.length ? y - APART.gap + APART.bottom : 0 };
}
/** Each legend set apart, its items as they show (a hidden layer's left out, as over the plot), their values as read. */
const apartItems = () => all$("#chart .legend[data-apart]").map((legend) => all$("button", legend).filter((button) => button.style.display !== "none").map((button) => {
  const value = /** @type {HTMLElement | null} */ (button.querySelector(".legend-value, .legend-error")), after = value ? getComputedStyle(value, "::after") : null;
  // (A value older than the view's end: its date after it, as over the plot (see measureLegends).)
  const asOf = value?.dataset.asOf && after && after.content !== "none" && after.display !== "none" ? ` · ${value.dataset.asOf}` : "";
  return { mark: button.querySelector(".series-mark")?.outerHTML ?? "", name: button.querySelector(".name")?.textContent?.trim() ?? "", value: `${value?.textContent?.trim() ?? ""}${asOf}`, faded: button.classList.contains("hidden") };
})).filter((items) => items.length);
/** The legends' marks, drawn from their own drawings (one that fails is left out). */
async function drawMarks(pieces) {
  await Promise.all(pieces.filter((piece) => piece.svg).map((piece) => new Promise((resolve) => {
    const image = new Image();
    image.onload = () => resolve((piece.image = image));
    image.onerror = () => resolve(null);
    image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(String(piece.svg).replace("<svg", '<svg xmlns="http://www.w3.org/2000/svg"'))}`;
  })));
  pieces.splice(0, pieces.length, ...pieces.filter((piece) => !piece.svg || piece.image));
}
export {
  anythingKept, APART, apartFonts, apartLayout, calledBy, canvasReadable, cascades, chartsFile, contentOf, copyLink,
  cutFade, datedView, dismissMenu, download, downloadAll, downloadChart, downloadFolder, drawMarks, exportAs,
  exportData, exportRows, fadeOut, fading, fieldOpener, FIELDS, fillMenu, fitted, fitTitle, fixedOrigin, fromLink,
  fromSketch, HOVER, keepCarriedView, leaveTitle, letGo, linkLookOf, linksOf, lookFields, measureLegends, menuGoBack,
  menuHidden, menuModal, NO_INTERVAL, onRail, openedImages, ownerNow, placeFixed, placeMenu, portable, qrCode, railed,
  readChart, renameInPlace, renderInterval, renderRanges, sameChart, scrolled, setApart, setRange, settleMenu,
  shareLink, shownAs, showSubmenu, slideFrom, stepInterval, subRows, supportedIntervals, switchInterval, titleReads,
  TOO_SMALL, toSketch, twinOf, typedFrom, unexpand, UNREADABLE, validate, viewDates, walkMenu,
};
