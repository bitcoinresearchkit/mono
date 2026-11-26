// @ts-check
// The page's means: its elements found and made, text made safe as HTML, keys and pointers read.
import { KEY_GLYPHS, keyGlyph } from "/studio/icons.js";
import { clamp } from "/studio/math.js";
import { grid } from "/studio/later.js";
import { MOTION, reduced } from "/studio/app.js";

/** @type {(id: string) => any} */
const $ = (id) => document.getElementById(id);
// Copies of the layouts' are in the stylesheet and the page's first script: keep them alike.
const MEDIA = {
  phone: "(max-width: 760px), (max-height: 500px)", light: "(prefers-color-scheme: light)", moreContrast: "(prefers-contrast: more)", noHover: "(hover: none)",
};
/**
 * Safari's confirming Enter comes after the composition ends, as key 229.
 * @param {KeyboardEvent} event
 */
const composing = (event) => event.isComposing || event.keyCode === 229;
/** @returns {HTMLElement | null} */
const closest = (event, selector) => /** @type {HTMLElement} */ (event.target)?.closest?.(selector) ?? null;
/** @param {KeyboardEvent} event */
function upDown(event) {
  return event.key === "ArrowUp" || event.key === "ArrowDown";
}
/** @param {KeyboardEvent} event */
function erasing(event) {
  return event.key === "Backspace" || event.key === "Delete";
}
/** @returns {HTMLElement[]} */
const all$ = (selector, root = /** @type {ParentNode} */ (document)) => [...root.querySelectorAll(selector)];
const escapeHtml = (text) => String(text).replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
// Anything but what's listed stays escaped text, never markup of its own.
function proseHtml(/** @type {string} */ markdown) {
  // Code first: what's in it is as written.
  const inline = (/** @type {string} */ text) => text.split(/(`[^`]+`)/).map((part, i) => (i % 2 ? `<code>${escapeHtml(part.slice(1, -1))}</code>` : escapeHtml(part)
    .replace(/\*\*(?=\S)(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(?=\S)([^*]+?)\*/g, "<em>$1</em>")
    .replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>'))).join("");
  const html = /** @type {string[]} */ ([]), lines = markdown.trim().split("\n");
  let paragraph = /** @type {string[]} */ ([]), list = /** @type {{ tag: string, items: string[] } | null} */ (null);
  const close = () => {
    // A no-break space holds the last two words: no line ends on one word alone.
    if (paragraph.length) html.push(`<p>${inline(paragraph.join(" ").replace(/ (\S+)$/, "\u00a0$1"))}</p>`);
    if (list) html.push(`<${list.tag}>${list.items.map((item) => `<li>${inline(item)}</li>`).join("")}</${list.tag}>`);
    paragraph = [];
    list = null;
  };
  for (let at = 0; at < lines.length; at++) {
    const line = lines[at], fence = /^\s*(`{3,})/.exec(line), item = /^\s*([-*+]|\d+\.)\s+/.exec(line);
    if (fence) {
      close();
      const code = [];
      for (at++; at < lines.length && !(/^`+$/.test(lines[at].trim()) && lines[at].trim().length >= fence[1].length); at++) code.push(lines[at]);
      html.push(`<pre><code>${escapeHtml(code.join("\n"))}</code></pre>`);
    } else if (!line.trim()) close();
    else if (item) {
      const tag = /\d/.test(item[1]) ? "ol" : "ul";
      if (paragraph.length || list?.tag !== tag) close();
      (list ??= { tag, items: [] }).items.push(line.slice(item[0].length));
    } else if (list) list.items[list.items.length - 1] += ` ${line.trim()}`;
    else paragraph.push(line.trim());
  }
  close();
  return html.join("");
}
const KEY_STEPS = /** @type {Record<string, number | undefined>} */ ({ ArrowUp: -1, ArrowLeft: -1, ArrowDown: 1, ArrowRight: 1, Home: -Infinity, End: Infinity });
const stepIn = (/** @type {number} */ at, /** @type {number} */ step, /** @type {number} */ n, wrap = false) => {
  if (!Number.isFinite(step)) return step < 0 ? 0 : n - 1;
  const to = (at < 0 ? (step > 0 ? -1 : n) : at) + step;
  return wrap ? ((to % n) + n) % n : clamp(to, 0, n - 1);
};
const viewport = /** @type {VisualViewport} */ (window.visualViewport);
const TOUCH = matchMedia(MEDIA.noHover).matches, CLICK = TOUCH ? "tap" : "click";
// How far a finger moves and still taps (iOS's own): past it, a scroll or a sheet's pull.
const SLOP = 10;
/** @param {Element} element */
const scrollsY = (element) => element.scrollHeight > element.clientHeight + 1 && /auto|scroll/.test(getComputedStyle(element).overflowY);
const MAC = /Mac|iPhone|iPad/.test(navigator.platform);
// The erase key goes by its name (delete on a Mac, Backspace elsewhere): its symbol reads as a box in a keycap.
const keyText = (/** @type {string} */ text) => {
  const named = text.replace(/⌫/g, MAC ? "delete" : "Backspace");
  return MAC ? named : named.replace(/[⌘⇧⌥⌃]+/g, (mods, at) => {
    const names = [["⌘⌃", "Ctrl"], ["⌥", "Alt"], ["⇧", "Shift"]].filter(([glyphs]) => [...glyphs].some((glyph) => mods.includes(glyph))).map(([, name]) => name);
    const next = named[at + mods.length];
    return `${names.join("+")}${next && !" -".includes(next) ? "+" : ""}`;
  });
};
const keysOf = (kbd) => {
  const keys = kbd.split(" ");
  return keyText(keys.slice(0, -1).every((key) => "⌘⇧⌥⌃".includes(key)) ? keys.join("") : keys.join(" "));
};
const keycaps = (kbd) => ((keys) => (keys.length === 2 && keys.every((key) => /^[A-Z]$/.test(key)) ? `<span class="sequence"><kbd>${keys[0]}</kbd><span class="then" aria-hidden="true">then</span><kbd>${keys[1]}</kbd></span>` : keys.map((key) => `<kbd>${escapeHtml(key)}</kbd>`).join("")))(keysOf(kbd).split(" "));
const ARIA_KEYS = /** @type {Record<string, string>} */ ({ "⌘": "Meta", "⇧": "Shift", "⌥": "Alt", "⌃": "Control", ctrl: "Control", shift: "Shift", alt: "Alt", "↵": "Enter", enter: "Enter", "⌫": "Backspace", backspace: "Backspace", esc: "Escape", space: "Space", "↑": "ArrowUp", "↓": "ArrowDown", "←": "ArrowLeft", "→": "ArrowRight", "−": "-", "+": "Plus", "⇥": "Tab", tab: "Tab" });
function ariaKey(/** @type {string} */ text) {
  const written = text.trim();
  if (!written || /\s/.test(written)) return "";
  let parts = written.length > 1 && written.includes("+") ? written.split("+").filter(Boolean) : [];
  if (!parts.length) {
    let rest = written;
    while (rest.length > 1 && "⌘⇧⌥⌃".includes(rest[0])) (parts.push(rest[0]), (rest = rest.slice(1)));
    parts.push(rest);
  }
  return parts.map((part) => (part.toLowerCase() === "delete" ? (MAC ? "Backspace" : "Delete") : ARIA_KEYS[part] ?? ARIA_KEYS[part.toLowerCase()] ?? (part.length === 1 ? part.toUpperCase() : part))).join("+");
}
const NAMED_KEY = /[⌘⌥⌃]|esc|⌫|↵|[←→↑↓]|delete|backspace|ctrl|alt/i;
function drawKeys(/** @type {ParentNode} */ root) {
  for (const key of root.querySelectorAll("kbd")) {
    const text = key.textContent ?? "";
    key.toggleAttribute("data-single", !NAMED_KEY.test(text));
    // A key on a control isn't part of its name: said as its shortcut ("Collapse", not "Collapse backslash"), except where it isn't the control's own (a drawing tool's comes after D).
    const control = !key.hasAttribute("aria-hidden") ? key.closest('button, a, [role="option"], [role="menuitem"]') : null;
    if (control) {
      key.setAttribute("aria-hidden", "true");
      const name = control.matches('[role="option"], .belt-tools button') ? "" : ariaKey(text);
      if (name && !control.hasAttribute("aria-keyshortcuts")) control.setAttribute("aria-keyshortcuts", name);
    }
    if (![...text].some((glyph) => KEY_GLYPHS[glyph]) || key.querySelector(".key")) continue;
    key.textContent = "";
    for (const glyph of text) {
      if (KEY_GLYPHS[glyph]) key.insertAdjacentHTML("beforeend", `${keyGlyph(glyph)}<span class="vh">${glyph}</span>`);
      else key.append(glyph);
    }
  }
  for (const control of /** @type {HTMLElement[]} */ ([...root.querySelectorAll("[data-kbd]:not([aria-keyshortcuts])")])) {
    const name = control.matches('[role="option"]') ? "" : ariaKey(keysOf(control.dataset.kbd ?? ""));
    if (name) control.setAttribute("aria-keyshortcuts", name);
  }
}
const typing = (target) => (target instanceof HTMLInputElement && !["checkbox", "radio", "button"].includes(target.type)) || target instanceof HTMLTextAreaElement || target?.isContentEditable;
const holders = new Set(), heldParts = /** @type {HTMLElement[]} */ ([]);
/**
 * Inert is inherited style: the page restyles once a hold begins and once it ends, not at each holder.
 * @param {unknown} holder @param {boolean} on
 */
function holdPage(holder, on) {
  const was = holders.size > 0;
  on ? holders.add(holder) : holders.delete(holder);
  if (holders.size > 0 === was) return;
  if (was) for (const part of heldParts.splice(0)) part.inert = false;
  else for (const part of /** @type {HTMLCollectionOf<HTMLElement>} */ (document.body.children)) {
    if (part.inert || part.matches("dialog, .dialog-scrim, .menu, .menu-scrim, #toast, #tip, script, .vh")) continue;
    part.inert = true;
    heldParts.push(part);
  }
}

function focusKey(container) {
  const active = document.activeElement;
  return active instanceof HTMLElement && container.contains(active) ? keyOf(active) : null;
}
/** @param {HTMLElement} element */
const keyOf = (element) => ({ tag: element.tagName, id: element.id, data: Object.entries(element.dataset).filter(([name]) => !["tip", "kbd", "pointed", "lit"].includes(name)) });
function restoreFocus(container, key) {
  if (!key || container.contains(document.activeElement)) return;
  const find = () => (key.id ? container.querySelector(`#${CSS.escape(key.id)}`) : [...container.querySelectorAll(key.tag)].find((element) => element instanceof HTMLElement && key.data.length > 0 && key.data.every(([name, value]) => element.dataset[name] === value)));
  let match = find();
  const preset = match ? undefined : key.data.find(([name]) => name === "openPreset" || name === "presetMenu")?.[1], slot = preset === undefined ? null : container.querySelector(`.tile-slot[data-slot="${CSS.escape(preset)}"]`);
  if (slot) (grid.fillSlot(/** @type {HTMLElement} */ (slot)), (match = find()));
  /** @type {HTMLElement | null | undefined} */ (match)?.focus();
}
/**
 * Popovers in the top layer, not modal dialogs (Safari's AutoFill measures a modal's whole page as ⌘K's field takes focus); what a modal does is done here: the page behind inert (holdPage), keys kept in, esc, a click beside it, Back on a phone.
 * @param {Element} element
 */
function isOpen(element) {
  return element.matches(":popover-open");
}
/** @param {HTMLElement} element @param {() => void} then */
function onClosed(element, then, once = false) {
  const heard = (/** @type {Event} */ event) => {
    if (/** @type {ToggleEvent} */ (event).oldState !== "open") return;
    if (once) element.removeEventListener("toggle", heard);
    then();
  };
  element.addEventListener("toggle", heard);
}
/**
 * Scrolled elements, as their scroll events say (the event comes at the next frame): only they can be anywhere but at their start.
 * @type {Set<Element>}
 */
const scrolled = new Set();
/**
 * @param {Element} element
 * @param {Element} copy
 * @param {Set<Element | null>} moving
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
/** @param {Element} copy @param {Element} of @param {Element} element */
function twinIn(copy, of, element) {
  const way = [];
  for (let at = element; at !== of; at = /** @type {Element} */ (at.parentElement)) way.push(Array.prototype.indexOf.call(/** @type {Element} */ (at.parentElement).children, at));
  return way.reduceRight((into, i) => into.children[i], copy);
}
/**
 * Firefox doesn't play `display` out and would drop the thing at once: a copy plays the way out, the thing itself is gone at once.
 * @param {HTMLElement} element @param {Keyframe[]} frames @returns {Promise<void> | undefined}
 */
function fadeOut(element, frames, duration = MOTION.micro, placed = false, easing = MOTION.easeLayout, prune = false) {
  if (element.hidden) return;
  cutFade(element);
  if (reduced() || !element.checkVisibility({ visibilityProperty: true })) return;
  const copy = /** @type {HTMLElement} */ (element.cloneNode(true)), style = getComputedStyle(element), after = getComputedStyle(element, "::after");
  // Asked before the copy is put in: after, the copy would be styled and laid out at once for it.
  const moving = element.getAnimations({ subtree: true }), scrolls = /** @type {[Element, number, number][]} */ ([]);
  for (const source of scrolled) if (element.contains(source) && (source.scrollTop || source.scrollLeft)) scrolls.push([source, source.scrollTop, source.scrollLeft]);
  const shows = /** @type {Keyframe} */ ({ ...frames[0] });
  for (const property of ["opacity", "transform", "translate", "scale"]) if (property in shows) shows[property] = style.getPropertyValue(property);
  // `prune` (the Charts page's step): a page of tiles copied whole had every one styled and laid out again for its way out, so parts wholly out of sight are copied as empty boxes of their size.
  const unseen = prune && (style.overflowX !== "visible" || style.overflowY !== "visible") ? outOfSight(element, copy, new Set(moving.map((animation) => /** @type {KeyframeEffect | null} */ (animation.effect)?.target ?? null))) : [];
  const seen = (/** @type {Element} */ source) => !unseen.some(([part]) => part.contains(source));
  for (const [, twin, held] of unseen) {
    twin.replaceChildren(" ");
    for (const [property, value] of Object.entries(held)) twin.style.setProperty(property, value, "important");
  }
  // Ids kept: its styles and an SVG's url(#…) references need them; inert, so no id lookup finds it.
  copy.inert = true;
  copy.setAttribute("aria-hidden", "true");
  copy.style.animation = "none";
  copy.style.transition = "none";
  if (placed) Object.assign(copy.style, { position: "absolute", left: `${element.offsetLeft}px`, top: `${element.offsetTop}px`, width: `${element.offsetWidth}px`, height: `${element.offsetHeight}px`, margin: "0", display: style.display, visibility: "visible", zIndex: "3", pointerEvents: "none", ...(after.content !== "none" ? { paddingBottom: `calc(${style.paddingBottom} + ${after.height})` } : {}) });
  element.after(copy);
  for (const [source, top, left] of scrolls) if (seen(source)) Object.assign(twinIn(copy, element, source), { scrollTop: top, scrollLeft: left });
  for (const source of /** @type {HTMLCanvasElement[]} */ (all$("canvas", element))) if (source.width && source.height && seen(source)) /** @type {HTMLCanvasElement} */ (twinIn(copy, element, source)).getContext("2d")?.drawImage(source, 0, 0);
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
function fixedOrigin(element) {
  element.style.left = element.style.top = "0px";
  const { left: x, top: y } = element.getBoundingClientRect();
  return { x, y };
}
// Whole device pixels: text drawn the same wherever it's drawn, not a pixel's fraction off once painted again (Chromium).
function placeFixed(/** @type {HTMLElement} */ element, /** @type {{ x: number, y: number }} */ origin, /** @type {number} */ left, /** @type {number} */ top) {
  const onPixels = (/** @type {number} */ value) => Math.round(value * devicePixelRatio) / devicePixelRatio;
  element.style.left = `${onPixels(left) - origin.x}px`;
  element.style.top = `${onPixels(top) - origin.y}px`;
}
/** @type {Map<HTMLElement, HTMLElement>} */
const fading = new Map();
/** @param {HTMLElement} element */
const cutFade = (element) => (fading.get(element)?.remove(), fading.delete(element));
export {
  $, all$, CLICK, closest, composing, drawKeys, erasing, escapeHtml, holdPage, KEY_STEPS, keycaps, keysOf, keyText,
  MAC, MEDIA, NAMED_KEY, proseHtml, scrollsY, SLOP, stepIn, TOUCH, typing, upDown, viewport, focusKey, keyOf, restoreFocus, isOpen, onClosed, cutFade, fadeOut, fading, scrolled, outOfSight, twinIn, fixedOrigin, placeFixed,
};
