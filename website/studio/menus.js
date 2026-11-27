// @ts-check
// Menus, the toolbar, sharing, importing and exporting.
import {
  atWindow, byTouch, chart, chartImage, commit, css, engine, flowsAt, hideTip, library, listedPreviews, loadPresets,
  measure, menuNameOf, MOTION, navigations, notice, onRail, openView, phone, railed, railMenu, reduced, resolved, revision, ROUTED,
  saveView, SHARES_IMAGES, shownView, showRange, storage, submenus, toast, UNREAD,
} from "/studio/app.js";
import { $, all$, closest, drawKeys, escapeHtml, holdPage, KEY_STEPS, keyText, stepIn, keyOf, restoreFocus, cutFade, fadeOut, fixedOrigin, placeFixed } from "/studio/dom.js";
import { clamp } from "/studio/math.js";
import { grouped, plural } from "/studio/format.js";
import { ICONS } from "/studio/icons.js";
import {
  EDGES, heatLike, letterAt,
} from "/studio/vocabulary.js";
import {
  PEN_COLOR,
} from "/studio/tools.js";
import {
  CADENCE, DEFAULT_RANGE, FINE, INTERVAL_ORDER, intervalFits,
} from "/studio/intervals.js";
import {
  inSubmenu, intervalsOf, lastShown, legends, levelOf, menuPicks, pointLabel, resolve, seriesIn, settled, shownDates,
  subAt, viewName, whenSubmenu,
} from "/studio/chart.js";
import { headMenu, pageTitle } from "/studio/command.js";
import { chartPathRows, isIn, sideKey, treeOf } from "/studio/grid.js";
import { tick } from "/studio/presets-edit.js";
import { labelOf, titleOf } from "/studio/looks.js";
import { typedFrom } from "/studio/toolbar.js";
import { flatOf, lookFields, linkDates, dateText, BASE32 } from "/studio/charts-read.js";
import { folderById, presetEdited, presetLink } from "/studio/preset-charts.js";
/** @import { Chart, Layer } from "/studio/vocabulary.js" */
/** @import { Action } from "/studio/command.js" */

// A redraw (the inspector's) puts a new opener in its place, by the same id.
const ownerNow = () => (menuOwner && !menuOwner.isConnected && menuOwner.id ? document.getElementById(menuOwner.id) : menuOwner);
const unexpand = (/** @type {Element | null} */ owner) => (owner?.removeAttribute("data-menu-open"), owner?.hasAttribute("aria-haspopup") && owner.removeAttribute("aria-expanded"));
/** @type {HTMLElement | null} */
let menuOwner = null;
const setMenuOwner = (/** @type {HTMLElement} */ owner) => (menuOwner = owner);
/** @type {(() => void) | null} */
let menuBack = null;
const setMenuBack = (/** @type {() => void} */ back) => (menuBack = back);
// Counted, so Share can tell one shown since its press, gone again or not.
let menusShown = 0;
function showMenu(anchor, html, onPick, /** @type {{ x: number, y: number } | null} */ point = null, side = "", asFlyout = false, named = "") {
  const menu = $("menu");
  menusShown++;
  if (cascadeRow && !menu.hidden) return showSubmenu(cascadeRow, html, onPick);
  if (ownerNow() === anchor && !menu.hidden && !point) return hideMenu();
  // Shown for another (a flyout moving on to the next row): one menu moving to this one's, not one fading as another
    // comes. See showSubmenu.
  const was = !menu.hidden && !phone.matches && !point ? menu.getBoundingClientRect() : null;
  if (was) for (const motion of menu.getAnimations()) motion.finish();
  hideSubmenu();
  flyout = asFlyout || openingByHover;
  // Inside a dialog the menu must live in the dialog (the top layer) to be seen.
  const host = anchor.closest("dialog") ?? document.body;
  fillMenu(menu, host, html, named || calledBy(anchor), onPick);
  menu.setAttribute("role", menu.querySelector(":scope > .share-pop") ? "dialog" : "menu");
  menu.classList.toggle("as-sheet", phone.matches);
  if (phone.matches) menu.insertAdjacentHTML("afterbegin", '<div class="menu-handle" aria-hidden="true"></div>');
  if (phone.matches) {
    // A dim kept where it is (a page of the menu, one opened as it fades), back to whole: put in again, it'd show in
        // again from none.
    if (scrimFade) {
      const shows = Number(getComputedStyle(menuScrim).opacity);
      scrimFade.cancel();
      if (shows < 1) menuScrim.animate({ opacity: [shows, 1] }, { duration: MOTION.layout, easing: MOTION.easeLayout });
    }
    if (menuScrim.parentElement !== host || menuScrim.nextElementSibling !== menu) host.insertBefore(menuScrim, menu);
  }
  holdPage("menu", phone.matches && host === document.body);
  for (const button of all$("button", menu)) {
    if (button.getAttribute("aria-haspopup") === "menu" && !button.querySelector(".sub")) button.insertAdjacentHTML("beforeend", `<span class="sub">${ICONS.next}</span>`);
    const key = button.querySelector("kbd")?.textContent;
    if (key) button.dataset.key = key;
  }
  cutFade(menu);
  menu.hidden = false;
  unexpand(menuOwner);
  menuOwner = anchor;
  if (!point && anchor.hasAttribute("aria-haspopup")) anchor.setAttribute("aria-expanded", "true");
  anchor.toggleAttribute("data-menu-open", true);
  if (phone.matches) Object.assign(menu.style, { left: "", top: "", maxHeight: "", transformOrigin: "" });
  else placeMenu(menu, host, anchor, point, side);
  if (was) {
    menu.style.animation = "none";
    slideFrom(menu, was);
  }
  // A flyout leaves the focus be; a finger's menu takes it itself, no row lit as if a key had reached it: ↓ goes on
    // from there.
  const first = /** @type {HTMLElement | null} */ (menu.querySelector("button[aria-checked='true']") ?? menu.querySelector("button:not(:disabled, [data-menu-back])"));
  settleMenu(menu);
  if (!flyout) (byTouch ? menu : first)?.focus({ preventScroll: true });
  if (first && menu.scrollHeight > menu.clientHeight) first.scrollIntoView({ block: "nearest" });
}
/** @param {HTMLElement} anchor @param {(Action | false | null | undefined)[]} rows */
function menuOf(anchor, rows, /** @type {{ x: number, y: number } | null} */ point = null, side = "", { heads = false, title = "", hover: asFlyout = false, named = "", back = /** @type {(() => void) | undefined} */ (undefined) } = {}) {
  const list = /** @type {Action[]} */ (rows.filter(Boolean));
  if (!list.length) return;
  // A menu of choices: the check's column on every row (a way out, a new one too), their icons in line.
  const checks = list.some((row) => row.current !== undefined);
  const html = list.map((row, i) => {
    const before = list[i - 1], apart = i > 0 && (row.apart || (row.group ?? "") !== (before.group ?? "") || (row.danger && !before.danger));
    const head = heads && row.group && row.group !== before?.group ? `<div class="menu-group">${escapeHtml(row.group)}</div>` : "";
    const choice = row.current !== undefined;
    return `${apart ? '<div class="menu-rule"></div>' : ""}${head}<button type="button" data-pick="${escapeHtml(row.key ?? String(i))}"${choice ? ` aria-checked="${Boolean(row.current)}"` : ""}${row.danger ? ' class="danger"' : ""}${row.disabled ? ` aria-disabled="true" data-tip="${escapeHtml(row.disabled)}"` : ""}${row.page ? ' aria-haspopup="menu"' : ""}>${checks ? tick(Boolean(row.current)) : ""}${row.icon ?? ""}<span class="label">${escapeHtml(row.label)}</span>${row.marked ? '<i class="menu-holds" aria-hidden="true"></i>' : ""}${row.kbd ? `<kbd>${escapeHtml(keyText(row.kbd))}</kbd>` : row.disabled || row.aside ? `<span>${escapeHtml(row.disabled || row.aside || "")}</span>` : ""}</button>`;
  }).join("");
  // What it's for, named on top wherever its name is out of sight: a row's on the rail, anything's on a phone. Not a
    // submenu's.
  if (!title && !cascadeRow && (phone.matches || (onRail() && anchor.closest("#chart-list")))) title = menuNameOf(anchor);
  const head = back ? `<div class="menu-group menu-title"><button type="button" class="cap small" data-menu-back="1" aria-label="Back" data-tip="Back">${ICONS.back}</button>${escapeHtml(title)}</div>` : title ? `<div class="menu-group">${escapeHtml(title)}</div>` : "";
  showMenu(anchor, `${head}${html}`, ({ pick: key, menuBack: backed }) => {
    if (backed) return back?.();
    const row = list.find((item, i) => (item.key ?? String(i)) === key);
    if (!row) return;
    if (row.disabled) return notice(row.disabled);
    if (!row.page) return row.run?.();
    const page = row.page(), here = () => menuOf(anchor, rows, point, side, { heads, title, hover: asFlyout, named, back });
    menuOf(anchor, page.options?.() ?? page.rows(), point, side, cascadeRow ? { title: page.heading ? pageTitle(page) : "" } : { title: pageTitle(page), back: here });
  }, point, side, asFlyout, named || title);
  menuBack = back ?? null;
}
// Picked by a key, the focus was in them: back on what opened them (its like, once redrawn), unless something took it
// or a menu the pick opened has it.
function menuPicked(/** @type {HTMLElement} */ button, /** @type {(data: DOMStringMap) => void} */ onPick) {
  const menu = $("menu"), owner = ownerNow(), key = owner && keyOf(owner), scope = owner?.parentElement?.closest("[id]"), target = menuTarget;
  hideMenu();
  onPick(button.dataset);
  if (!menu.hidden) return void (target?.isConnected && outlineMenuTarget(target));
  const active = document.activeElement;
  if (active && active !== document.body && !menu.contains(active) && !inSubmenu(active)) return;
  if (owner?.isConnected) owner.focus({ preventScroll: true });
  else if (scope?.isConnected) restoreFocus(scope, key);
}
const menuScrim = document.createElement("div");
menuScrim.className = "menu-scrim";
let scrimFade = /** @type {Animation | null} */ (null);
/** @type {HTMLElement | null} */
let menuTarget = null;
const outlineMenuTarget = (/** @type {HTMLElement} */ target) => (menuTarget = target).classList.add("menu-target");
let cascadeRow = /** @type {HTMLElement | null} */ (null), cascadeTimer = 0;
let cascadeWait = /** @type {{ row: Element | null, heading: boolean, level: number } | null} */ (null);
function openCascade(/** @type {HTMLElement} */ row, focus = false) {
  const own = /** @type {HTMLElement | null} */ (row.closest("#menu, .submenu"));
  if ($("menu").hidden || !own || own.hidden) return;
  const level = levelOf(own) + 1;
  if (subRows[level] !== row || subAt(level).hidden) {
    hideSubmenu(subAt(level).hidden ? level : level + 1);
    cascadeRow = row;
    try {
      menuPicks.get(own)?.(row.dataset);
    } finally {
      cascadeRow = null;
    }
  } else if (subAt(level).hasAttribute("data-covered")) hideSubmenu(level + 1);
  const sub = subAt(level);
  if (focus && !sub.hidden) /** @type {HTMLElement | undefined} */ (sub.querySelector("button[aria-checked='true']:not(:disabled)") ?? all$("button:not(:disabled)", sub)[0])?.focus();
}
function hideSubmenu(from = 1) {
  clearTimeout(cascadeTimer);
  cascadeWait = null;
  const gone = submenus.slice(from - 1), had = gone.some((sub) => sub.contains(document.activeElement)), back = subRows[from];
  for (const sub of gone.reverse()) {
    if (sub.hidden) continue;
    fadeOut(sub, [{ opacity: 1, transform: "none" }, { opacity: 0, transform: "scale(0.96)" }]);
    sub.hidden = true;
  }
  for (const row of subRows.splice(from)) row?.removeAttribute("aria-expanded");
  for (const menu of [$("menu"), ...submenus].slice(from - 1)) menu.removeAttribute("data-covered");
  if (had && back?.isConnected) back.focus({ preventScroll: true });
}
// A row's submenu opens a moment after the pointer comes onto it (a pass over it opens none); one open, another row
// with one switches it at once (a gap, a rule or a row without one closes it a moment later: crossing a rule doesn't
// close and open again). Heading for the open one, what's under the pointer waits while it's on its way, then shows its
// own; leaving the triangle switches at once.
function startCascade() {
  let last = { x: 0, y: 0 }, toward = false, headedAt = -Infinity;
  const inTriangle = (/** @type {number[]} */ [px, py], /** @type {number[][]} */ [[ax, ay], [bx, by], [cx, cy]]) => {
    const side = (/** @type {number} */ x1, /** @type {number} */ y1, /** @type {number} */ x2, /** @type {number} */ y2) => (px - x2) * (y1 - y2) - (x1 - x2) * (py - y2);
    const d1 = side(ax, ay, bx, by), d2 = side(bx, by, cx, cy), d3 = side(cx, cy, ax, ay);
    return !((d1 < 0 || d2 < 0 || d3 < 0) && (d1 > 0 || d2 > 0 || d3 > 0));
  };
  const waited = () => {
    if (cascadeWait?.heading && performance.now() - headedAt < 100) return void (cascadeTimer = window.setTimeout(waited, 100));
    const row = cascadeWait?.row, level = cascadeWait?.level ?? 1;
    cascadeWait = null;
    row && cascades(row) ? openCascade(/** @type {HTMLElement} */ (row)) : hideSubmenu(level);
  };
  // Scrolled, rows pass under a pointer held still: a wait begun goes, and a browser's own moves after it aren't the
    // pointer moving (as the rail's, see pointedAt).
  let seen = { x: NaN, y: NaN };
  const moves = (/** @type {HTMLElement} */ menu) => menu.addEventListener("pointermove", (event) => {
    if (event.pointerType !== "mouse" || phone.matches || (event.clientX === seen.x && event.clientY === seen.y)) return;
    seen = { x: event.clientX, y: event.clientY };
    const level = levelOf(menu) + 1, next = submenus[level - 1], open = next && !next.hidden ? next : null;
    const row = /** @type {HTMLElement | null} */ (closest(event, "button:not(:disabled)")), at = [event.clientX, event.clientY];
    // Heading for the open submenu: judged over a few pixels (a hand's tremor doesn't turn it), toward it and a row
        // past each end (a curved path still counts).
    const box = open?.getBoundingClientRect(), far = Math.hypot(at[0] - last.x, at[1] - last.y) >= 5;
    if (!box) toward = false;
    else if (far) {
      const edge = box.left >= last.x ? box.left : box.right;
      toward = inTriangle(at, [[last.x, last.y], [edge, box.top - 40], [edge, box.bottom + 40]]);
    }
    if (!box || far) last = { x: at[0], y: at[1] };
    if (far && toward) headedAt = performance.now();
    const opens = row && cascades(row);
    if (row && row === subRows[level] && open?.hasAttribute("data-covered")) return void (clearTimeout(cascadeTimer), (cascadeWait = null), openCascade(row));
    if ((row && row === subRows[level]) || (!opens && !open)) return void (clearTimeout(cascadeTimer), (cascadeWait = null));
    if (toward) {
      if (cascadeWait?.heading) return void (cascadeWait.row = row);
      clearTimeout(cascadeTimer);
      cascadeWait = { row, heading: true, level };
      return void (cascadeTimer = window.setTimeout(waited, 300));
    }
    if (opens && open) return void (clearTimeout(cascadeTimer), (cascadeWait = null), openCascade(/** @type {HTMLElement} */ (row)));
    // A first submenu: a moment from coming onto its row. A gap or a row without one: a moment from the first, the open
        // one closed then.
    if (cascadeWait && !cascadeWait.heading && (cascadeWait.row === row || !opens)) return void (cascadeWait.row = row);
    clearTimeout(cascadeTimer);
    cascadeWait = { row, heading: false, level };
    cascadeTimer = window.setTimeout(waited, 100);
  });
  moves($("menu"));
  whenSubmenu(moves);
  // Left, or scrolled: what it was about to do, it doesn't; the point it was at is forgotten, so back on it, it's a
    // move.
  const calm = () => (clearTimeout(cascadeTimer), (cascadeWait = null));
  const left = () => (calm(), (seen = { x: NaN, y: NaN }));
  for (const type of ["wheel", "scroll"]) $("menu").addEventListener(type, calm, { passive: true });
  whenSubmenu((sub) => ["wheel", "scroll"].forEach((type) => sub.addEventListener(type, calm, { passive: true })));
  $("menu").addEventListener("pointerleave", left);
  whenSubmenu((sub) => (sub.addEventListener("pointerenter", calm), sub.addEventListener("pointerleave", left)));
}
let openingByHover = false;
let flyout = false;
const pinFlyout = () => (flyout = false);
const focusMenu = () => /** @type {HTMLElement | null | undefined} */ ([$("menu"), ...submenus].find((menu) => !menu.hidden && !menu.hasAttribute("data-covered"))?.querySelector("button:not(:disabled, [data-menu-back])"))?.focus({ preventScroll: true });
// Hover to look, press to keep (a mouse): gone as the pointer leaves the menu and what opened it, a moment after (time
// enough to cross to the flyout).
function startFlyouts() {
  let flyTimer = 0, from = /** @type {HTMLElement | null} */ (null);
  const sidebar = $("sidebar");
  const shut = () => {
    clearTimeout(flyTimer);
    // What opened it written anew meanwhile (the chart's place): its flyout goes all the same.
    if (from && flyout && (ownerNow() === from || !from.isConnected)) hideMenu();
    from = null;
    flyout = false;
  };
  // `warm`: one shown this way already, its place taken (as a tip's): the next at once, no fading out, no growing in
    // again.
  const lookSoon = (/** @type {HTMLElement} */ element, /** @type {number} */ wait, /** @type {() => void} */ open, ready = async () => true, warm = false) => {
    clearTimeout(flyTimer);
    flyTimer = window.setTimeout(async () => {
      if (!(await ready()) || !element.isConnected || !element.matches(":hover") || (!$("menu").hidden && !flyout)) return;
      const inPlace = warm && flyout && !$("menu").hidden;
      if (!inPlace) shut();
      from = element;
      open();
    }, wait);
  };
  /** @param {HTMLElement} row */
  const chartKey = (row) => (row.dataset.openChart ? `c:${row.dataset.openChart}` : row.dataset.openPreset !== undefined ? `p:${row.dataset.openPreset}` : "");
  let pressed = "";
  const goSoon = () => {
    clearTimeout(flyTimer);
    flyTimer = window.setTimeout(() => !$("menu").matches(":hover") && !submenus.some((sub) => sub.matches(":hover")) && !from?.matches(":hover") && shut(), HOVER.go);
  };
  // Scrolled, rows pass under a pointer held still: none is pointed at, nor about to be, till it moves. From the
    // wheel's turn: a browser tells of the row come under the pointer before it tells of the scroll; its own moves after
    // a scroll aren't the pointer moving.
  let still = false, at = { x: NaN, y: NaN };
  for (const type of ["wheel", "scroll"]) $("chart-list").addEventListener(type, () => (still = true), { passive: true });
  $("chart-list").addEventListener("pointermove", (event) => {
    if (event.clientX === at.x && event.clientY === at.y) return;
    at = { x: event.clientX, y: event.clientY };
    if (still) (still = false), pointedAt(event);
  });
  $("chart-list").addEventListener("pointerover", (event) => pointedAt(event));
  /** @param {PointerEvent} event */
  function pointedAt(event) {
    // The rail's alone: a row there is its icon, its name out of sight, so its menu says it.
    if (event.pointerType !== "mouse" || !railed() || still) return;
    const row = /** @type {HTMLElement | null} */ (closest(event, "#chart-list :is(.folder-item, .list-head-name, [data-open-chart], [data-open-preset])"));
    if (row && row === from && flyout) return clearTimeout(flyTimer);
    clearTimeout(flyTimer);
    // A chart just pressed: opened, its row written anew under the pointer, not its menu again till another is pointed
        // at or the list is left.
    if (row && pressed && chartKey(row) === pressed) return;
    if (!row) return void (from && goSoon());
    pressed = "";
    if (!$("menu").hidden && !flyout) return void (ownerNow()?.closest("#chart-list") && ownerNow() !== row && railMenu(row));
    // Over a sidebar still widening: once it's done, beside the row where it ends. Scrolled meanwhile, none.
    const widened = async () => (await Promise.all(sidebar.getAnimations().map((animation) => animation.finished)).catch(() => {}), railed() && !still);
    const warm = flyout && !$("menu").hidden && Boolean(from);
    lookSoon(row, warm ? 0 : HOVER.row, () => railMenu(row, true), widened, warm);
  }
  $("chart-list").addEventListener("pointerleave", (event) => event.pointerType === "mouse" && ((pressed = ""), from && goSoon()));
  for (const type of ["pointerdown", "contextmenu"]) $("chart-list").addEventListener(type, () => clearTimeout(flyTimer), true);
  const keeps = (/** @type {HTMLElement} */ menu) => {
    menu.addEventListener("pointerenter", () => flyout && clearTimeout(flyTimer));
    menu.addEventListener("pointerleave", (event) => flyout && event.pointerType === "mouse" && goSoon());
  };
  keeps($("menu"));
  whenSubmenu(keeps);
  /**
   * Kept as it is, as a press opens one (the focus in it, for its keys), not put away and shown again: it would blink.
   * @param {Event} event @param {HTMLElement} anchor
   */
  const keepFlyout = (event, anchor) => {
    if (from !== anchor || !flyout || $("menu").hidden || ownerNow() !== anchor) return false;
    event.stopImmediatePropagation();
    pinFlyout();
    from = null;
    focusMenu();
    return true;
  };
  // A row pressed: its menu as pressed takes the flyout's place (its own already showing, kept; a chart, opened, none).
    // A ⋯'s own menu, shown by its click as the pointer rests on it, is the flyout.
  $("chart-list").addEventListener("click", (event) => {
    const item = /** @type {HTMLElement | null} */ (closest(event, ":is(.chart-row, .folder-row, .list-head) > :is(.chart-item, .folder-item, .list-head-name)"));
    if (!item) return;
    if (railed() && !item.matches("[data-open-chart], [data-open-preset]") && keepFlyout(event, item)) return;
    shut();
    pressed = chartKey(item);
  }, true);
  /** @param {HTMLElement} host @param {(event: Event) => HTMLElement | null} anchorOf @param {(anchor: HTMLElement) => void} open */
  const hoverMenu = (host, anchorOf, open) => {
    host.addEventListener("pointerover", (event) => {
      const anchor = anchorOf(event);
      if (event.pointerType !== "mouse" || phone.matches || !anchor) return;
      if (anchor === from && flyout) return clearTimeout(flyTimer);
      const warm = flyout && !$("menu").hidden && Boolean(from);
      clearTimeout(flyTimer);
      if (!$("menu").hidden && !warm) return;
      lookSoon(anchor, warm ? 0 : HOVER.button, () => (hideTip(), open(anchor)), async () => !anchor.hidden, warm);
    });
    host.addEventListener("pointerout", (event) => {
      const anchor = anchorOf(event);
      if (event.pointerType === "mouse" && anchor && from === anchor && flyout && !(event.relatedTarget instanceof Element && anchor.contains(event.relatedTarget))) goSoon();
    });
    host.addEventListener("click", (event) => {
      const anchor = anchorOf(event);
      if (!anchor) return;
      if (from === anchor || !flyout) clearTimeout(flyTimer);
      keepFlyout(event, anchor);
    }, true);
  };
  const more = $("chart-more");
  hoverMenu(more, () => more, (anchor) => menuOf(anchor, headMenu(), null, "", { hover: true }));
  hoverMenu(document.body, (event) => /** @type {HTMLElement | null} */ (closest(event, "[data-chart-menu], [data-folder-menu], [data-section-menu], [data-preset-menu], [data-group-menu], [data-tile-menu]")), (anchor) => ((openingByHover = true), anchor.click(), (openingByHover = false)));
  hoverMenu($("chart-path"), (event) => /** @type {HTMLElement | null} */ (closest(event, "#chart-place")), (anchor) => menuOf(anchor, chartPathRows(), null, "", { hover: true }));
}
/** @param {{ frames: Keyframe[], duration: number, easing: string } | null} away */
function hideMenu(away = null) {
  flyout = false;
  hideSubmenu();
  menuBack = null;
  menuTarget?.classList.remove("menu-target");
  menuTarget = null;
  if (!$("menu").hidden && $("menu").classList.contains("as-sheet") && menuScrim.isConnected && !reduced()) {
    const fade = (scrimFade = menuScrim.animate({ opacity: 0 }, { duration: MOTION.layout, easing: MOTION.easeLayout }));
    menuScrim.style.pointerEvents = "none";
    const done = (/** @type {boolean} */ gone) => scrimFade === fade && ((scrimFade = null), (menuScrim.style.pointerEvents = ""), gone && menuScrim.remove());
    fade.finished.then(() => done(true), () => done(false));
  } else if (!scrimFade) menuScrim.remove();
  holdPage("menu", false);
  if (away) fadeOut($("menu"), away.frames, away.duration, false, away.easing);
  else fadeOut($("menu"), $("menu").classList.contains("as-sheet") ? [{ transform: getComputedStyle($("menu")).transform }, { transform: "translateY(100%)" }] : [{ opacity: 1, transform: "none" }, { opacity: 0, transform: "scale(0.96)" }], $("menu").classList.contains("as-sheet") ? MOTION.layout : MOTION.micro);
  $("menu").hidden = true;
  unexpand(ownerNow());
  // Only a menu that was open: closing again keeps who opened the last.
  if (menuOwner) (closedOwner = ownerNow()), (closedAt = performance.now());
  menuOwner = null;
}
let closedOwner = /** @type {HTMLElement | null} */ (null), closedAt = 0;
function placeMenu(menu, host, anchor, point, side = "") {
  // Opened again as it fades: its fade ended first, its box measured whole and in place.
  for (const motion of menu.getAnimations()) if (motion instanceof CSSTransition) motion.finish();
  const own = point ? new DOMRect(point.x, point.y, 0, 0) : anchor.getBoundingClientRect();
  // A row's menu opens beside the list: under its ⋯ it hid the rows below.
  if (!side && !point && anchor.closest("#chart-list")) side = "right";
  // A belt's popover opens from the belt's edge, a rail's menu from its icons' column, not their button's: as far from
    // it as a menu from what opened it, whichever row.
  const strip = side && !point ? (anchor.closest("#belt") ?? (railed() ? anchor.closest(".sidebar")?.querySelector(".search") : null))?.getBoundingClientRect() : null;
  const rect = !strip ? own : side === "left" || side === "right" ? new DOMRect(strip.left, own.top, strip.width, own.height) : new DOMRect(own.left, strip.top, own.width, strip.height);
  const gap = point ? 2 : 6;
  const bounds = host === document.body ? new DOMRect(0, 0, innerWidth, innerHeight) : host.getBoundingClientRect();
  menu.style.maxHeight = host === document.body ? "" : `${bounds.height - 16}px`;
  // Measured at rest: the opening animation starts it scaled down.
  menu.style.animation = "none";
  const origin = fixedOrigin(menu);
  const width = menu.offsetWidth, height = menu.offsetHeight;
  if (side === "left" || side === "right") {
    const right = side === "right" ? rect.right + gap + width <= bounds.right - 8 : rect.left - gap - width < bounds.left + 8;
    // A rail's row's: its first row level with it, as a submenu's with its row; else centered on it.
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
function dismissMenu(back = false) {
  const had = back || $("menu").contains(document.activeElement) || inSubmenu(document.activeElement), owner = ownerNow();
  hideMenu();
  if (had && owner?.isConnected) owner.focus({ preventScroll: true });
}
function menuGoBack() {
  const back = menuBack;
  if (!back) return dismissMenu();
  hideMenu();
  back();
}
const calledBy = (/** @type {HTMLElement} */ element) => (element.getAttribute("aria-label") || element.querySelector(".label, .name")?.textContent || [...element.childNodes].filter((node) => !(node instanceof Element && node.matches("kbd, .aside, .sub, svg"))).map((node) => node.textContent).join(" ")).replace(/\s+/g, " ").trim();
function fillMenu(/** @type {HTMLElement} */ element, /** @type {HTMLElement} */ host, /** @type {string} */ html, /** @type {string} */ label, /** @type {(data: DOMStringMap) => void} */ onPick) {
  // Moved, its copy fading where it was gone first: its id is found where it is, never in a copy before it.
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
    if (button.dataset.menuBack && element !== $("menu")) return hideSubmenu(levelOf(element));
    if (cascades(button)) return openCascade(button, event.detail === 0);
    menuPicked(button, onPick);
  };
}
function walkMenu(/** @type {KeyboardEvent} */ event, /** @type {HTMLElement} */ root) {
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
  items[stepIn(items.indexOf(/** @type {HTMLElement} */ (document.activeElement)), Number(KEY_STEPS[event.key]), items.length, true)]?.focus();
  return true;
}
// Pointed at a moment (heading for it, inside the triangle from where the pointer was to its near edge, the rows
// crossed on the way don't switch it), pressed, or → from the row; ← or esc closes it. Color's Custom isn't one: it's a
// page of the same popover.
const cascades = (/** @type {Element} */ button) => !phone.matches && button.getAttribute("aria-haspopup") === "menu" && !button.hasAttribute("data-pick-custom");
/** @type {(HTMLElement | undefined)[]} */
const subRows = [];
function showSubmenu(/** @type {HTMLElement} */ row, /** @type {string} */ html, /** @type {(data: DOMStringMap) => void} */ onPick) {
  const own = /** @type {HTMLElement} */ (row.closest("#menu, .submenu")), level = levelOf(own) + 1, sub = subAt(level);
  // Shown for another row of its menu: one submenu moving to this one's, not one fading as another comes (a sweep down
    // the rows would flash a row).
  const was = sub.hidden ? null : sub.getBoundingClientRect(), had = sub.contains(document.activeElement);
  for (const motion of sub.getAnimations()) motion.finish();
  // Its menu still moving (gliding to another row, growing in): there first, this placed beside where it stays.
  for (const motion of own.getAnimations()) motion.finish();
  const host = /** @type {HTMLElement} */ ($("menu").parentElement);
  fillMenu(sub, host, html, calledBy(row), onPick);
  for (const button of all$("button[aria-haspopup='menu']", sub)) if (!button.querySelector(".sub")) button.insertAdjacentHTML("beforeend", `<span class="sub">${ICONS.next}</span>`);
  cutFade(sub);
  sub.hidden = false;
  subRows[level]?.removeAttribute("aria-expanded");
  (subRows[level] = row).setAttribute("aria-expanded", "true");
  // Beside the menu it's from, its first row level with the row; no room that way, over that menu (hidden till this
    // goes), titled, ‹ back to it: never over the menus before it, the way back to each kept.
  const parent = own.getBoundingClientRect(), at = row.getBoundingClientRect(), bounds = host === document.body ? new DOMRect(0, 0, innerWidth, innerHeight) : host.getBoundingClientRect();
  sub.style.maxHeight = host === document.body ? "" : `${bounds.height - 16}px`;
  sub.style.maxWidth = "";
  sub.style.animation = "none";
  const origin = fixedOrigin(sub), fits = { right: parent.right + 4 + sub.offsetWidth <= bounds.right - 8, left: parent.left - 4 - sub.offsetWidth >= bounds.left + 8 };
  const way = level > 1 ? subAt(level - 1).dataset.way ?? "right" : fits.right || !fits.left ? "right" : "left", over = !fits[way];
  sub.dataset.way = way;
  // Over its menu, no wider than the room from that menu's edge on: a long name cut, not the menus before it covered.
  if (over) sub.style.maxWidth = `${way === "right" ? bounds.right - 8 - parent.left : parent.right - bounds.left - 8}px`;
  const width = sub.offsetWidth;
  const covering = over && (had || own.contains(document.activeElement));
  own.toggleAttribute("data-covered", over);
  if (over) {
    const head = sub.querySelector(":scope > .menu-group:first-child"), back = `<button type="button" class="cap small" role="menuitem" data-menu-back="1" aria-label="Back" data-tip="Back">${ICONS.back}</button>`;
    if (head) (head.classList.add("menu-title"), head.insertAdjacentHTML("afterbegin", back));
    else sub.insertAdjacentHTML("afterbegin", `<div class="menu-group menu-title">${back}${escapeHtml(calledBy(row))}</div>`);
  }
  const height = sub.offsetHeight, top = clamp(over ? parent.top : at.top - (/** @type {HTMLElement | null} */ (sub.querySelector(":scope > button"))?.offsetTop ?? 4), bounds.top + 8, bounds.bottom - height - 8);
  const left = over ? (way === "right" ? parent.left : parent.right - width) : way === "right" ? parent.right + 4 : parent.left - 4 - width;
  placeFixed(sub, origin, clamp(left, bounds.left + 8, bounds.right - width - 8), top);
  sub.style.transformOrigin = `${way === "right" ? "0" : "100%"} ${clamp(at.top + at.height / 2 - top, 0, height)}px`;
  settleMenu(sub);
  if (covering) /** @type {HTMLElement | null} */ (sub.querySelector("button:not(:disabled, [data-menu-back])"))?.focus({ preventScroll: true });
  if (was) {
    slideFrom(sub, was);
    if (had && !over) row.focus({ preventScroll: true });
    return;
  }
  void sub.offsetWidth;
  sub.style.animation = "";
}
/** @param {HTMLElement} menu @param {DOMRect} was */
const slideFrom = (menu, was) => ((now) => reduced() || menu.animate({ translate: [`${was.left - now.left}px ${was.top - now.top}px`, "0 0"] }, { duration: MOTION.micro, easing: MOTION.easeLayout }))(menu.getBoundingClientRect());
function settleMenu(/** @type {HTMLElement} */ menu) {
  for (const words of all$(":scope > button:not([data-tip]) > .label, :scope > .menu-group", menu)) {
    if (words.scrollWidth > words.clientWidth) (words.closest("button") ?? words).dataset.tip = words.textContent ?? "";
  }
  const top = menu.querySelector(":scope > :is(.menu-group:first-child, .menu-handle + .menu-group)");
  menu.style.scrollPaddingTop = top instanceof HTMLElement ? `${top.offsetTop + top.offsetHeight}px` : "";
}
// A rest long enough that a pass down the list opens none, no longer (as macOS's submenus wait); after leaving, time to
// cross to the menu.
const HOVER = { row: 200, button: 200, go: 260 };

const fieldOpener = /** @type {WeakMap<Element, HTMLElement>} */ (new WeakMap());
/**
 * Never the page: a screen reader's place is kept and the app's single keys are back.
 * @param {HTMLElement} field
 */
function letGo(field) {
  const opener = fieldOpener.get(field), first = field.id === "home-filter" ? all$(".tile-open", $("home-grid"))[0] : null;
  const to = /** @type {HTMLElement | undefined} */ ([opener, first, field.closest(".card-head, #inspector, .home-tools, #panel")].find((element) => element instanceof HTMLElement && element.isConnected && element !== field && element.checkVisibility()));
  if (!to) return field.blur();
  if (!to.matches("button, a[href], input, textarea, select, [tabindex]")) to.tabIndex = -1;
  to.focus({ preventScroll: true });
}

const NO_INTERVAL = "Not every series or formula in this chart supports it";
function supportedIntervals(/** @type {Chart} */ doc = chart) {
  const lists = seriesIn(doc).map(intervalsOf).filter((ids) => ids !== undefined), formulas = doc.layers.flatMap((layer) => layer.formula ? [layer.formula] : []);
  return (id) => doc.layers.every((layer) => intervalFits(layer.maxInterval, id)) && lists.every((indexes) => indexes.includes(id)) && formulas.every((formula) => flowsAt(formula, id));
}
function stepInterval(delta) {
  const at = INTERVAL_ORDER.indexOf(chart.index), ids = delta < 0 ? INTERVAL_ORDER.slice(0, at).reverse() : INTERVAL_ORDER.slice(at + 1);
  const next = ids.find(supportedIntervals());
  if (!next) return;
  switchInterval(next);
  toast(`Interval: ${CADENCE[next]}`);
}

function switchInterval(/** @type {string} */ index) {
  saveView();
  commit((doc) => (doc.index = index), { look: true });
}
/** @param {string} key */
function switchWindow(key) {
  saveView();
  commit((doc) => void atWindow(doc, key), { look: true });
}

function setRange(range) {
  commit((doc) => (doc.range = range), { coalesce: "range", look: true, quiet: true, view: true }) || showRange();
}

function leaveTitle() {
  const field = /** @type {HTMLInputElement} */ ($("title")), title = field.value.trim();
  // Only what was typed: one focused and left as it was isn't an edit (another tab's rename, taken in meanwhile,
    // stays).
  if (title !== chart.title && field.value !== typedFrom.get(field)) commit((doc) => (doc.title = title), { coalesce: "title" });
  // Left, it's untouched again: a rename taken in while it's not focused shows.
  typedFrom.set(field, field.value);
}

function renameInPlace() {
  const field = /** @type {HTMLInputElement} */ ($("title"));
  field.readOnly = false;
  field.focus();
  field.select();
}
const FIELDS = "#title, #label, .number-input, .level-name";

/** @returns {Action[]} */
const exportRows = () => /** @type {[string, string, string, string, string][]} */ ([...(SHARES_IMAGES ? [["share", "Share image", "", ICONS.share, "Share"]] : []), ["copy", "Copy image", "", ICONS.copy, "Copy"], ["open", "Open image", "In a new tab", ICONS.link, "Open"], ["png", "Download image", "PNG", ICONS.image, "Download"], ["csv", "Download data", "CSV", ICONS.table, "Download"], ["json", "Download chart file", "JSON", ICONS.file, "Download"]]).map(([kind, label, note, icon, verb]) => ({ key: kind, icon, label, aside: note, verb, apart: kind === "csv", run: () => exportAs(kind) }));
/**
 * Asked for in the click itself, its text to come: a browser lets that write.
 * @param {Promise<string>} link
 */
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
let qrModule = /** @type {Promise<{ generate: (text: string) => { size: number, get(x: number, y: number): boolean } }> | null} */ (null);
// The most a QR code holds (lowest correction, alphanumeric mode: a QR link's base32): a longer link isn't tried, a
// shorter one that still doesn't fit the library refuses.
const QR_MOST = 4296;
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
    // A module whole device pixels, two at least: a finer dense code doesn't scan, so the popover grows; as tall as a
        // sheet shows, a phone on its side's too.
    const room = Math.min(phone.matches ? Math.min(256, innerWidth * 0.72) : 208, innerHeight * 0.7 - 24);
    const per = Math.max(2, Math.floor((room * devicePixelRatio) / side)), width = (side * per) / devicePixelRatio;
    return `<svg class="share-qr" viewBox="0 0 ${side} ${side}" width="${width}" height="${width}" shape-rendering="crispEdges" role="img" aria-label="A QR code of this chart's link"><rect width="${side}" height="${side}" fill="#fff"/><path fill="#000" d="${d}"/></svg>`;
  } catch {
    return null;
  }
}

/** @param {Chart} doc */
function portable(doc) {
  if (doc === chart) saveView();
  const kept = doc === chart ? shownView() : openView(doc);
  return { v: 3, title: doc.title, index: doc.index, range: doc.range, ...(doc.halvings ? { halvings: true } : {}), ...(doc.legend ? { legend: doc.legend } : {}), panes: doc.panes, layers: doc.layers.map(({ from, ...layer }) => ((one) => (one.gradient && !one.byGradient ? { ...one, byGradient: false } : one))(from ? { ...layer, [layer.type]: from } : layer)), ...(doc.drawings?.length ? { drawings: doc.drawings } : {}), ...(kept ? { view: kept } : {}) };
}

/** @param {Chart} shown */
function toSketch(shown) {
  const doc = flatOf(shown);
  const place = (/** @type {string} */ id) => doc.panes.findIndex((pane) => pane.id === id) + 1;
  const plain = doc.panes.every((pane, i) => pane.mode === (i ? 0 : 1) && Object.keys(pane).every((key) => key === "id" || key === "mode"));
  const ordered = doc.layers.every((layer, i) => layer.letter === letterAt(i));
  if (shown === chart) saveView();
  const dates = linkDates(shown);
  return {
    ...(doc.title ? { title: doc.title } : {}),
    ...(doc.index !== "day1" ? { index: doc.index } : {}),
    ...(doc.range !== DEFAULT_RANGE ? { range: doc.range } : {}),
    ...(doc.halvings ? { halvings: true } : {}),
    ...(doc.legend ? { legend: doc.legend } : {}),
    ...(plain ? {} : { panes: doc.panes.map(({ id, ...pane }) => pane) }),
    layers: doc.layers.map(({ id, letter, pane, from, ref, colorFrom, type, width, dash, side, path, paths, series, formula, byGradient, ...all }) => ((rest) => ({
      ...(series !== undefined ? { series } : formula !== undefined ? { formula } : {}),
      ...(from ? { [type]: from.map((member) => doc.layers.find((item) => item.id === member)?.letter ?? member) } : {}),
      // A signal's none ("") is said too: read back, it isn't marked on another.
      ...(ref !== undefined ? { ref: ref && (doc.layers.find((item) => item.id === ref)?.letter ?? ref) } : {}),
      ...(colorFrom ? { colorFrom: doc.layers.find((item) => item.id === colorFrom)?.letter ?? colorFrom } : {}),
      ...(ordered ? {} : { letter }),
      ...rest,
      // A series' type always: left out, it's read as the series is mostly drawn, which one tab knows and another
            // doesn't yet (a line opened as a baseline, or candles).
      ...((type !== "line" || series !== undefined) && !from && !heatLike({ type }) ? { type } : {}),
      ...(all.gradient && !byGradient ? { byGradient: false } : {}),
      ...(width !== 1.5 && type !== "cloud" ? { width } : {}),
      ...(dash && type !== "cloud" ? { dash } : {}),
      ...(side !== "right" ? { side } : {}),
      ...(place(pane) > 1 ? { pane: place(pane) } : {}),
    }))(all.byLayers && from?.some((member) => !EDGES.includes(member)) ? (({ color, colorBelow, byLayers, ...others }) => others)(all) : all)),
    ...(doc.drawings?.length ? { drawings: doc.drawings.map(({ id, layer, points, color, width, dash, ...rest }) => ({
      ...rest,
      layer: doc.layers.find((item) => item.id === layer)?.letter ?? layer,
      points: points.map(([time, value]) => [dateText(time), value]),
      ...(color !== PEN_COLOR ? { color } : {}),
      ...(width !== 1.5 ? { width } : {}),
      ...(dash ? { dash } : {}),
    })) } : {}),
    ...(dates ? { view: dates } : {}),
  };
}

// Only what a link or a chat would break on is encoded: brackets and braces too (a chat cuts a trailing "]}" off), a
// "+" (read as a space by what decodes forms), an "&" (a title's "&chart=" never reads as a field).
const linkText = (/** @type {Chart} */ doc) => JSON.stringify(toSketch(doc)).replace(/[\s"<>`%#\\{}[\]+&]/g, (char) => encodeURIComponent(char));
/**
 * Links Studio writes are compressed; a link written out as JSON is read too (an assistant's, which can't compress it).
 * Compressing failed: written out.
 * @param {Chart} doc @param {Promise<Uint8Array>} [bytes]
 */
const shareLink = (doc, bytes = packedBytes(doc), look = chartLook(doc)) => bytes.then((packed) => `${pageBase()}#chart=${toBase64url(packed)}${look}`, () => `${pageBase()}#chart=${linkText(doc)}`);
// Share links open this copy, whose storage belongs to its origin.
const ORIGIN = `${location.origin}/studio`;
const pageBase = () => (ROUTED ? `${ORIGIN}/` : location.href.replace(/[?#].*$/, ""));
/** @param {Chart} doc */
const chartLook = (doc) => ((fields) => (fields ? `&${fields}` : ""))(lookFields(doc, { index: "day1", range: DEFAULT_RANGE }));
/** @param {Chart} doc */
async function packedBytes(doc) {
  const { index: _i, range: _r, view: _v, ...sketch } = /** @type {any} */ (toSketch(doc));
  const stream = new Blob([JSON.stringify(sketch)]).stream().pipeThrough(new CompressionStream("deflate-raw"));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

/**
 * A QR code's alphanumeric mode takes 5.5 bits a character, where base64 takes 8 in byte mode.
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

function linksOf(doc = chart) {
  // A preset as made but for how it's looked at: its address and that, once the presets are read. Edited, gone from
    // them, or none: the chart.
  const address = doc.preset !== undefined ? loadPresets().then(() => presetLink(doc), () => null) : Promise.resolve(null);
  const links = address.then((at) => {
    if (at) return { copy: `${ORIGIN}${at}`, short: `${ORIGIN}${at}` };
    // Its content and how it's looked at, both now: the link of one moment.
    const bytes = packedBytes(doc), look = chartLook(doc), copy = shareLink(doc, bytes, look);
    return { copy, short: bytes.then((packed) => `${pageBase()}#c=${toBase32(packed)}${look}`, () => copy) };
  });
  return { copy: links.then((all) => all.copy), short: links.then((all) => all.short) };
}

function toBase64url(/** @type {Uint8Array} */ bytes) {
  const native = /** @type {any} */ (bytes).toBase64?.({ alphabet: "base64url", omitPadding: true });
  if (native !== undefined) return native;
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function download(name, blob) {
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = name;
  link.click();
  // Kept while a browser may still be asking where it goes (Safari's iOS sheet, after a while).
  setTimeout(() => URL.revokeObjectURL(link.href), 40000);
}
const slugOf = (/** @type {string} */ text) => text.trim().replace(/[^\w-]+/g, "-").replace(/^-|-$/g, "").toLowerCase();
const fileName = (extension, doc = chart) => `${slugOf(titleOf(doc) || "chart") || "chart"}.${extension}`;
function chartsFile(/** @type {Chart[]} */ docs, folders = library.folders.filter((folder) => docs.some((doc) => isIn(doc.folder, folder.id))), /** @type {any[]} */ also = [], /** @type {Chart[]} */ edited = []) {
  const ids = new Set(docs.map((doc) => doc.id)), kept = new Set(folders.map((folder) => `f:${folder.id}`));
  const charts = [...docs.map((doc) => ({ id: doc.id, ...(doc.folder ? { folder: doc.folder } : {}), ...portable(doc) })), ...also];
  const presetsToo = edited.map((doc) => ({ preset: doc.preset, ...(doc.base ? { base: doc.base } : {}), ...portable(doc) }));
  return new Blob([JSON.stringify({ v: 2, charts, folders, order: treeOf().map(({ item }) => sideKey(item)).filter((key) => kept.has(key) || ids.has(key.slice(2))), ...(presetsToo.length ? { presets: presetsToo } : {}) }, null, 2)], { type: "application/json" });
}
/** @returns {any[]} */
const keptAside = () => ((aside) => (Array.isArray(aside) ? aside : []))(storage.get(UNREAD, []));
const asideCount = () => keptAside().length;
const anythingKept = () => library.charts.length > 0 || library.folders.length > 0 || library.presets.some(presetEdited) || asideCount() > 0;
function downloadAll(rescue = false) {
  const aside = keptAside(), edited = library.presets.filter(presetEdited), charts = rescue ? [...library.charts, ...listedPreviews()] : library.charts;
  if (!charts.length && !library.folders.length && !aside.length && !edited.length) return notice("No charts yet");
  download(`bitview-charts-${new Date().toLocaleDateString("en-CA")}.json`, chartsFile(charts, library.folders, aside, edited));
  toast(`Downloaded ${plural(charts.length, "chart")}${library.folders.length ? ` and ${plural(library.folders.length, "folder")}` : ""}${edited.length ? `, ${plural(edited.length, "edited preset")}` : ""}${aside.length ? `, and ${aside.length} kept aside` : ""}, in one file`);
}
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

/** @param {Layer[]} which */
async function exportData(which) {
  // Still loading, what's there may be what was before: nothing then.
  if (!settled()) return notice("The chart is still loading");
  const doc = structuredClone(chart), requested = new Set(which.map((layer) => layer.id));
  const data = await resolve({ ...doc, id: `${doc.id}:export`, layers: doc.layers.map(layer => ({ ...layer, hidden: false })) });
  const layers = doc.layers.filter((layer) => requested.has(layer.id) && data.results.get(layer.id)?.values);
  if (!layers.length) return notice("Its data isn't here yet");
  const quote = (text) => `"${String(text).replace(/"/g, '""')}"`;
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
/** @param {HTMLCanvasElement} canvas @returns {Promise<Blob>} */
const pngOf = (canvas) => new Promise((resolve, reject) => canvas.toBlob((blob) => (blob ? resolve(blob) : reject(new Error("No image"))), "image/png"));
/** @param {unknown} error */
const imageTrouble = (error) => (error instanceof Error && (error.message === UNREADABLE || error.message === TOO_SMALL) ? error.message : "Couldn't make the image");
// Its address is kept as long as this page is: dropped sooner, saving it fails. The tab opens at once, while the click
// counts: one opened once it's drawn is blocked.
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
    Object.assign(image, { src: url, alt: titleOf(chart) });
    image.style.cssText = `display: block; width: ${canvas.width / devicePixelRatio}px; max-width: 100%; height: auto`;
    page.body.style.cssText = `margin: 0; min-height: 100vh; display: grid; place-items: center; background: ${css("--card")}`;
    page.body.replaceChildren(image);
  }, (error) => (tab.close(), notice(imageTrouble(error))));
}
/** @type {string[]} */
const openedImages = [];
// Asked for in the tap itself, drawn meanwhile: it still counts.
function shareImage() {
  if (!canvasReadable()) return void notice(UNREADABLE);
  const name = fileName("png");
  chartImage(pictureSize())
    .then(pngOf)
    .then((blob) => navigator.share({ files: [new File([blob], name, { type: "image/png" })], title: titleOf(chart) }))
    .catch((/** @type {unknown} */ error) => error instanceof Error && error.name !== "AbortError" && notice(error.message === UNREADABLE ? UNREADABLE : "Couldn't share the image"));
}
// Given the picture while it's still being drawn, so the click that asked counts.
function copyImage() {
  if (!canvasReadable()) return void notice(UNREADABLE);
  const png = chartImage(pictureSize()).then(pngOf);
  navigator.clipboard.write([new ClipboardItem({ "image/png": png })]).then(() => toast("Copied the image: paste it anywhere"), () => notice("Couldn't copy the image"));
}
// Some browsers give noise instead of the page's own drawing unless the site's allowed (a privacy setting, as Mullvad's
// and Tor's): asked each time, allowed since, it's given.
function canvasReadable() {
  try {
    // Many pixels of many colors, as a chart's are: a randomizer spares a single pixel and a plain color.
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
const TOO_SMALL = "Its legends don't fit a picture this size: make it larger";
// As the chart shows in a window wide enough; narrower, or upright (a phone's), a picture's own, 16:9, as one's shared:
// not a strip as tall as the screen.
const pictureSize = () => ((box) => (box.width >= 640 && box.width > box.height ? {} : { width: 1200, height: 675 }))($("chart").getBoundingClientRect());
const shownAs = () => `${chart.id} ${revision} ${navigations}`;
// Each written whole, as "Values on" writes its own: the navigator's tight form (’18) beside a whole 2026 reads as two.
function viewDates() {
  const times = resolved?.times ?? [], span = shownDates();
  if (!times.length || !span) return { span: "", values: "" };
  const last = times[lastShown()], [from, to] = [pointLabel(span.from), pointLabel(span.to)];
  return { span: from === to ? to : `${from} – ${to}`, values: chart.legend === "off" ? "" : `Values on ${pointLabel(last)}` };
}

/** @returns {{ x: number, y: number, width: number, height: number, svg?: string, image?: HTMLImageElement, text?: string, font?: string, color?: string, align?: CanvasTextAlign, faded?: boolean, clip?: number, cut?: boolean }[]} */
function measureLegends(scale) {
  const shown = legends(), origin = engine.element.getBoundingClientRect();
  const box = (element) => {
    const rect = element.getBoundingClientRect();
    return { x: (rect.left - origin.left) * scale, y: (rect.top - origin.top) * scale, width: rect.width * scale, height: rect.height * scale };
  };
  const pieces = [];
  // Words the page cuts (wider than their box) are cut in the picture too; the others whole, however a canvas measures
    // them (a hair wider than the page lays them).
  const cutOff = (/** @type {HTMLElement} */ element) => element.scrollWidth > element.clientWidth;
  /** @param {HTMLElement} key @param {boolean} faded */
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
    for (const key of /** @type {HTMLElement[]} */ (all$(":scope > .legend-key", legend))) if (key.offsetParent) keyPieces(key, false);
    if (legend.hasAttribute("data-apart")) continue;
    for (const button of legend.querySelectorAll("button")) {
      if (button.style.display === "none" || !button.offsetParent || button.getBoundingClientRect().right > edge + 1) continue;
      const mark = button.querySelector(".series-mark"), faded = button.classList.contains("hidden");
      if (mark) pieces.push({ ...box(mark), svg: mark.outerHTML, faded });
      for (const element of /** @type {HTMLElement[]} */ ([button.querySelector(".name"), button.querySelector(".legend-value, .legend-error")])) {
        if (!element?.textContent?.trim() || !element.getClientRects().length) continue;
        const style = getComputedStyle(element), inset = parseFloat(style.paddingLeft) * scale, at = box(element);
        const text = element.textContent.trim(), after = getComputedStyle(element, "::after");
        // A value older than the view's end: its date after it as the page shows it (its ::after, not in its text),
                // quieter.
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
/** @returns {() => void} */
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
const APART = { row: 20, top: 6, bottom: 10, gap: 8, columnGap: 14, textGap: 8, mark: 16, markHeight: 12, markGap: 6, size: 12, nameWeight: 510, valueWeight: 590 };
const apartFonts = (scale = 1, sans = css("--sans")) => ({ name: `${APART.nameWeight} ${APART.size * scale}px ${sans}`, value: `${APART.valueWeight} ${APART.size * scale}px ${sans}` });
function apartLayout(/** @type {number} */ width) {
  const groups = apartItems();
  const g = measure, fonts = apartFonts();
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
const apartItems = () => all$("#chart .legend[data-apart]").map((legend) => all$("button", legend).filter((button) => button.style.display !== "none").map((button) => {
  const value = /** @type {HTMLElement | null} */ (button.querySelector(".legend-value, .legend-error")), after = value ? getComputedStyle(value, "::after") : null;
  const asOf = value?.dataset.asOf && after && after.content !== "none" && after.display !== "none" ? ` · ${value.dataset.asOf}` : "";
  return { mark: button.querySelector(".series-mark")?.outerHTML ?? "", name: button.querySelector(".name")?.textContent?.trim() ?? "", value: `${value?.textContent?.trim() ?? ""}${asOf}`, faded: button.classList.contains("hidden") };
})).filter((items) => items.length);
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
  anythingKept, APART, apartFonts, apartLayout, canvasReadable, cascades, chartsFile, closedAt, closedOwner, 
  copyLink, dismissMenu, download, downloadAll, downloadChart, downloadFolder, drawMarks, exportAs,
  exportData, exportRows, fieldOpener, FIELDS, flyout, focusMenu,
  hideMenu, hideSubmenu, leaveTitle, letGo, linksOf, 
  measureLegends, menuGoBack, menuOf, menuScrim, menusShown, NO_INTERVAL, openCascade, openedImages,
  outlineMenuTarget, ownerNow, pinFlyout, portable, qrCode, renameInPlace,
  switchWindow, setApart, setMenuBack, setMenuOwner,
  setRange, shareLink, showMenu, shownAs, startCascade, startFlyouts, stepInterval, supportedIntervals, switchInterval,
  TOO_SMALL, toSketch, UNREADABLE, viewDates, walkMenu,
};
