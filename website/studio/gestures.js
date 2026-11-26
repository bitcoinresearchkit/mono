// @ts-check
// How a press resolves, one way for every list and control: a click, a hold, a drag that reorders; the click a gesture
// swallows; the click given to a control that slid from under a still pointer.
import { press } from "/modules/bitview-charts/index.js";
import { hideTip, MOTION, reduced } from "/studio/app.js";
import { all$, closest, SLOP, TOUCH, typing } from "/studio/dom.js";
import { clamp } from "/studio/math.js";
import { hideMenu } from "/studio/menus.js";

function restRect(/** @type {HTMLElement} */ element) {
  const moving = [];
  for (let at = /** @type {HTMLElement | null} */ (element); at; at = at.parentElement) for (const motion of at.getAnimations()) if ((motion.playState === "running" || motion.playState === "paused") && Number.isFinite(Number(motion.effect?.getComputedTiming().endTime))) moving.push(motion);
  const was = moving.map((motion) => motion.currentTime);
  try {
    for (const motion of moving) motion.currentTime = Number(motion.effect?.getComputedTiming().endTime);
    return element.getBoundingClientRect();
  } finally {
    moving.forEach((motion, i) => (motion.currentTime = was[i]));
  }
}
const stopCarrying = () => carrying?.();
/** @param {HTMLElement} container @param {string} selector @param {(element: HTMLElement) => (() => boolean) | null} start */
function onHold(container, selector, start) {
  let drop = () => {};
  container.addEventListener("pointerdown", (event) => {
    drop();
    const element = /** @type {HTMLElement | null} */ (event.pointerType === "touch" ? closest(event, selector) : null), held = element && start(element);
    if (!element || !held) return;
    let taken = false;
    const timer = window.setTimeout(() => element.isConnected && held() && ((taken = true), navigator.vibrate?.(8)), 450);
    const stop = press(event, {
      move: (moved) => Math.hypot(moved.clientX - event.clientX, moved.clientY - event.clientY) > SLOP && clearTimeout(timer),
      end: () => (clearTimeout(timer), taken && swallowClick()),
    });
    drop = () => (clearTimeout(timer), stop());
  });
  container.addEventListener("contextmenu", (event) => TOUCH && closest(event, selector) && event.preventDefault());
}
/**
 * By touch the item is picked by holding still first (a quick swipe still scrolls). A hold that began picking and then carried its item off picked nothing: `unpick` ends it.
 * @param {HTMLElement} list
 * @param {{ item: string, idOf: (item: HTMLElement) => string, carried?: (id: string) => string[],
 *   pick?: (id: string) => void, picking?: () => boolean, unpick?: () => void, refuse?: (event: PointerEvent) => boolean, still?: () => boolean, sticky?: boolean,
 *   label: (id: string) => string, landing: (x: number, y: number, ids: string[]) => Landing | null | undefined,
 *   land: (ids: string[], at: Landing, y: number) => void, lift?: () => void, settle?: () => void }} how
 * @typedef {{ item: HTMLElement, after: boolean, down?: boolean, depth?: number } | { zone: HTMLElement }} Landing `down`: before or after it is above or below it (a grid of one column); `depth`: at that level of a tree (its line indented to it)
 */
function dragToReorder(list, how) {
  /** @type {{ id: string, x: number, y: number, at: [number, number], touch: boolean, held: boolean, began: boolean, timer: number, touched: EventTarget | null, stop: () => void } | null} */
  let hand = null;
  const pin = (/** @type {Event} */ event) => (hand?.held || drag) && event.preventDefault();
  /** @type {{ ids: string[], ghost: HTMLElement, x: number, y: number, at: Landing | null, scroller: HTMLElement, box: DOMRect, frame: number, touch: boolean, width: number, height: number, top: number } | null} */
  let drag = null;
  /** @type {HTMLElement | null} */
  let kept = null;
  const settleGhost = (/** @type {string[]} */ ids) => {
    const ghost = kept;
    kept = null;
    if (!ghost) return;
    if (reduced()) return ghost.remove();
    const hidden = /** @type {HTMLElement[]} */ (ids.map(itemNamed).filter(Boolean));
    for (const item of hidden) item.style.opacity = "0";
    requestAnimationFrame(() => {
      for (const item of hidden) item.style.opacity = "";
      const items = /** @type {HTMLElement[]} */ (ids.map(itemNamed).filter(Boolean)), to = items[0] && restRect(items[0]);
      const away = to ? `translate(${to.left + 8}px, ${to.top + (to.height - ghost.offsetHeight) / 2}px)` : `${ghost.style.transform} scale(0.96)`;
      const timing = { duration: MOTION.panel, easing: MOTION.ease };
      for (const item of items) item.animate({ opacity: [0, 1] }, timing);
      ghost.animate({ transform: [ghost.style.transform, away], opacity: [1, 0] }, { ...timing, fill: "forwards" }).finished.then(() => ghost.remove(), () => ghost.remove());
    });
  };
  // By id, not element: the list may have been drawn anew since the press.
  const itemNamed = (/** @type {string} */ id) => all$(how.item, list).find((element) => how.idOf(element) === id);
  const unmark = () => [list, ...all$(".drop-before, .drop-after, .drop-target", list)].forEach((element) => (element.classList.remove("drop-before", "drop-after", "drop-down", "drop-target"), element.style.removeProperty("--drop-depth")));
  const endDrag = (landed = false, back = !landed) => {
    const unpick = Boolean(hand?.began && drag && !landed);
    if (hand) clearTimeout(hand.timer), hand.stop();
    hand?.touched?.removeEventListener("touchmove", pin);
    hand = null;
    if (carrying === endDrag) carrying = null;
    if (!drag) return;
    cancelAnimationFrame(drag.frame);
    const ghost = drag.ghost, from = back && itemNamed(drag.ids[0])?.getBoundingClientRect();
    if (landed && !back) kept = ghost;
    else if (reduced()) ghost.remove();
    else {
      const away = from ? `translate(${from.left + 8}px, ${from.top + (from.height - drag.height) / 2}px)` : `${ghost.style.transform} scale(0.96)`;
      ghost.animate({ transform: [ghost.style.transform, away], opacity: [1, 0] }, { duration: from ? MOTION.small : MOTION.micro, easing: MOTION.ease, fill: "forwards" }).finished.then(() => ghost.remove(), () => ghost.remove());
    }
    unmark();
    all$(".dragging", list).forEach((element) => element.classList.remove("dragging"));
    list.classList.remove("carrying");
    document.body.classList.remove("carrying");
    drag = null;
    how.settle?.();
    if (unpick) how.unpick?.();
  };
  const landingNow = () => {
    if (!drag) return null;
    const box = drag.box, own = list.getBoundingClientRect();
    if (drag.x < Math.max(box.left, own.left) || drag.x > Math.min(box.right, own.right) || drag.y < Math.max(box.top, own.top) || drag.y > Math.min(box.bottom, own.bottom)) return null;
    const found = how.landing(drag.x, drag.y, drag.ids);
    const kept = drag.at && ("item" in drag.at ? drag.at.item : "zone" in drag.at ? drag.at.zone : null);
    return found === undefined ? (how.sticky && kept?.isConnected ? drag.at : null) : found;
  };
  const sameLanding = (/** @type {Landing | null} */ a, /** @type {Landing | null} */ b) => (!a || !b ? a === b : "item" in a && "item" in b ? a.item === b.item && a.after === b.after && a.depth === b.depth : "zone" in a && "zone" in b && a.zone === b.zone);
  // Read first, written after: nothing is laid out twice a move.
  const follow = () => {
    if (!drag) return;
    const at = landingNow(), marked = drag.ids.every((id) => itemNamed(id)?.classList.contains("dragging"));
    // A finger's above it, in sight (below where there's no room above): beside it the thumb was over it.
    const above = drag.y - drag.height - 36;
    drag.ghost.style.transform = drag.touch ? `translate(${clamp(drag.x - drag.width / 2, 8, innerWidth - drag.width - 8)}px, ${above >= drag.top + 8 ? above : drag.y + 36}px)` : `translate(${drag.x + 14}px, ${drag.y + 12}px)`;
    if (!marked) for (const element of all$(how.item, list)) element.classList.toggle("dragging", drag.ids.includes(how.idOf(element)));
    if (marked && sameLanding(at, drag.at) && (!at || ("item" in at ? at.item : at.zone).isConnected)) return;
    unmark();
    drag.at = at;
    if (at && "item" in at) {
      at.item.classList.add(at.after ? "drop-after" : "drop-before", ...(at.down ? ["drop-down"] : []));
      if (at.depth !== undefined) at.item.style.setProperty("--drop-depth", String(at.depth));
    }
    else if (at && "zone" in at) at.zone.classList.add("drop-target");
  };
  const edge = () => {
    if (!drag) return;
    const box = drag.box, reach = 48;
    const by = drag.y < box.top + reach ? drag.y - box.top - reach : drag.y > box.bottom - reach ? drag.y - box.bottom + reach : 0;
    if (by && drag.x >= box.left && drag.x <= box.right) {
      drag.scroller.scrollTop += by / 3;
      follow();
    }
    drag.frame = requestAnimationFrame(edge);
  };
  function lift(/** @type {PointerEvent} */ event) {
    if (!hand) return;
    pressStart = null;
    const ids = how.carried?.(hand.id) ?? [hand.id], ghost = document.createElement("div");
    ghost.className = "drag-ghost";
    ghost.setAttribute("aria-hidden", "true");
    ghost.innerHTML = `${how.label(ids[0])}${ids.length > 1 ? `<span class="drag-count">${ids.length}</span>` : ""}`;
    document.body.append(ghost);
    let scroller = /** @type {HTMLElement} */ (document.scrollingElement);
    for (let node = /** @type {HTMLElement | null} */ (list); node; node = node.parentElement) {
      if (/(auto|scroll)/.test(getComputedStyle(node).overflowY)) {
        scroller = node;
        break;
      }
    }
    drag = { ids, ghost, x: event.clientX, y: event.clientY, at: null, scroller, box: scroller.getBoundingClientRect(), frame: 0, touch: hand.touch, width: ghost.offsetWidth, height: ghost.offsetHeight, top: parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--safe-top")) || 0 };
    how.lift?.();
    list.classList.add("carrying");
    document.body.classList.add("carrying");
    getSelection()?.removeAllRanges();
    hideTip();
    hideMenu();
    list.setPointerCapture(event.pointerId);
    follow();
    drag.frame = requestAnimationFrame(edge);
  }
  // Not passive, asked before it scrolls: a long list would start scrolling before the hold can stop it.
  list.addEventListener("touchmove", pin, { passive: false });
  list.addEventListener("pointerdown", (event) => {
    if (event.button !== 0 || carrying) return;
    const item = /** @type {HTMLElement | null} */ (closest(event, how.item));
    if (!item || typing(event.target) || how.refuse?.(event)) return;
    hand = { id: how.idOf(item), x: event.clientX, y: event.clientY, at: [event.clientX, event.clientY], touch: event.pointerType === "touch", held: false, began: false, timer: 0, touched: null, stop: press(event, { captor: list, move, end }) };
    carrying = endDrag;
    if (hand.touch) {
      hand.touched = event.target;
      hand.touched?.addEventListener("touchmove", pin, { passive: false });
    }
    if (hand.touch) hand.timer = window.setTimeout(() => {
      if (!hand) return;
      if (!itemNamed(hand.id)) return endDrag();
      hand.held = true;
      [hand.x, hand.y] = hand.at;
      hand.began = !how.picking?.();
      navigator.vibrate?.(8);
      how.pick?.(hand.id);
    }, 450);
  });
  function move(/** @type {PointerEvent} */ event) {
    if (!hand) return;
    if (drag) {
      drag.x = event.clientX;
      drag.y = event.clientY;
      return follow();
    }
    const moved = Math.hypot(event.clientX - hand.x, event.clientY - hand.y);
    hand.at = [event.clientX, event.clientY];
    if (hand.touch && !hand.held) return void (moved > SLOP && endDrag());
    if ((hand.touch ? moved > SLOP : moved >= 4) && !how.still?.() && itemNamed(hand.id)) lift(event);
  }
  /** @param {PointerEvent | null} event */
  function end(event) {
    if (!hand) return;
    if (!event) return endDrag();
    if (drag) {
      drag.x = event.clientX;
      drag.y = event.clientY;
    }
    // Where they land, read now (not as last marked: the list may have been drawn anew), unless now there's no place.
    const carried = drag && { ids: drag.ids, at: how.still?.() ? null : landingNow(), y: drag.y };
    // A drag, or a hold, makes no click: it'd open, or pick again, what was pressed.
    if (carried || hand.held) swallowClick();
    const began = hand.began;
    endDrag(true, !carried?.at);
    if (carried?.at) how.land(carried.ids, carried.at, carried.y);
    if (carried?.at) settleGhost(carried.ids);
    // A hold that became a drag moved it: it picked nothing (picking was on before, it stays).
    if (carried && began) how.unpick?.();
  }
  // No native drag of an image or link inside it.
  list.addEventListener("dragstart", (event) => event.preventDefault());
  document.addEventListener("keydown", (event) => {
    if (!drag || event.key !== "Escape") return;
    event.preventDefault();
    event.stopImmediatePropagation();
    endDrag();
  }, true);
}
/** @type {(() => void) | null} */
let carrying = null;
let swallowUntil = 0;
const swallowClick = () => (swallowUntil = performance.now() + 600);
// A press clicks what it pressed: a control that slid from under a still pointer (a fold above it, a glide) is still the one clicked. The browser's click, its release elsewhere, goes to what holds both: taken back, this one given.
let pressStart = /** @type {{ target: Element, x: number, y: number, box: DOMRect } | null} */ (null);
let slidFrom = /** @type {Element | null} */ (null);
const clickSlid = (/** @type {MouseEvent} */ { clientX, clientY, screenX, screenY, ctrlKey, shiftKey, altKey, metaKey, detail }) => ((target) => ((slidFrom = null), target?.isConnected && target.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true, composed: true, view: window, clientX, clientY, screenX, screenY, ctrlKey, shiftKey, altKey, metaKey, detail: detail || 1 }))))(slidFrom);
// Added where app.js's start comes to them: their order among the window's other capture listeners is the start's.
function startGestures() {
  window.addEventListener("pointerdown", () => (swallowUntil = 0), true);
  window.addEventListener("click", (event) => {
    if (performance.now() > swallowUntil || event.detail === 0) return;
    swallowUntil = 0;
    event.preventDefault();
    event.stopPropagation();
  }, true);
  window.addEventListener("pointerdown", (event) => {
    slidFrom = null;
    pressStart = event.isPrimary && event.button === 0 && event.target instanceof Element ? { target: event.target, x: event.clientX, y: event.clientY, box: event.target.getBoundingClientRect() } : null;
    const was = pressStart;
    if (!was) return;
    press(event, {
      move: (moved) => Math.hypot(moved.clientX - was.x, moved.clientY - was.y) > SLOP && pressStart === was && (pressStart = null),
      end: (release) => {
        if (pressStart !== was) return;
        pressStart = null;
        const at = release?.target;
        if (!release || !(at instanceof Node) || was.target.contains(at) || !was.target.isConnected || Math.hypot(release.clientX - was.x, release.clientY - was.y) > SLOP) return;
        const box = was.target.getBoundingClientRect();
        if (Math.hypot(box.x - was.box.x, box.y - was.box.y) < 1) return;
        slidFrom = was.target;
        setTimeout(() => slidFrom === was.target && clickSlid(release));
      },
    });
  }, true);
  window.addEventListener("click", (event) => {
    if (!slidFrom || !event.isTrusted) return;
    if (event.target === slidFrom || event.defaultPrevented) return void (slidFrom = null);
    event.preventDefault();
    event.stopImmediatePropagation();
    clickSlid(event);
  }, true);
}

export { dragToReorder, onHold, startGestures, stopCarrying, swallowClick };
