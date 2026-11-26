// @ts-check
// The page's means: its elements found and made, text made safe as HTML, keys and pointers read.
import { KEY_GLYPHS, keyGlyph } from "/studio/icons.js";
import { clamp } from "/studio/math.js";
import { grid } from "/studio/later.js";
import { MOTION, reduced } from "/studio/app.js";

/** @type {(id: string) => any} */
const $ = (id) => document.getElementById(id);
/**
 * The media queries more than one place reads, as strings: a list of its own wherever one is polled or listened to
 * (see tokensMedia). The stylesheet and the page's first script keep copies of the layouts'.
 */
const MEDIA = {
  phone: "(max-width: 760px), (max-height: 500px)", light: "(prefers-color-scheme: light)", moreContrast: "(prefers-contrast: more)", noHover: "(hover: none)",
};
/** A key that's part of an IME's composing (a candidate confirmed, put away): the IME's, not the page's. (Safari's
 * confirming Enter comes after the composition ends, as key 229.) @param {KeyboardEvent} event */
const composing = (event) => event.isComposing || event.keyCode === 229;
/** The element an event came from, or its nearest ancestor matching `selector`. @returns {HTMLElement | null} */
const closest = (event, selector) => /** @type {HTMLElement} */ (event.target)?.closest?.(selector) ?? null;
/** An arrow up or down. @param {KeyboardEvent} event */
function upDown(event) {
  return event.key === "ArrowUp" || event.key === "ArrowDown";
}
/** A delete's keys, Backspace or Delete. @param {KeyboardEvent} event */
function erasing(event) {
  return event.key === "Backspace" || event.key === "Delete";
}
/** @returns {HTMLElement[]} */
const all$ = (selector, root = /** @type {ParentNode} */ (document)) => [...root.querySelectorAll(selector)];
const escapeHtml = (text) => String(text).replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
/**
 * The server's words (a series' description: Markdown) as HTML: its paragraphs, lists and fenced blocks; inline, code,
 * strong, emphasis and links to the web. Anything else stays text, escaped: never markup of its own.
 */
function proseHtml(/** @type {string} */ markdown) {
  // (Code first: what's in it is as written.)
  const inline = (/** @type {string} */ text) => text.split(/(`[^`]+`)/).map((part, i) => (i % 2 ? `<code>${escapeHtml(part.slice(1, -1))}</code>` : escapeHtml(part)
    .replace(/\*\*(?=\S)(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/\*(?=\S)([^*]+?)\*/g, "<em>$1</em>")
    .replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>'))).join("");
  // (Line by line: a fenced block whole, blank lines and all; a list's items together, a line under one going on
  // with it; the rest paragraphs, a blank line between them.)
  const html = /** @type {string[]} */ ([]), lines = markdown.trim().split("\n");
  let paragraph = /** @type {string[]} */ ([]), list = /** @type {{ tag: string, items: string[] } | null} */ (null);
  const close = () => {
    // (Its last two words held together by a no-break space: no line ends on one word alone.)
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
/** How far an arrow, Home or End moves a place in a row: one on or back, or to an end. */
const KEY_STEPS = /** @type {Record<string, number | undefined>} */ ({ ArrowUp: -1, ArrowLeft: -1, ArrowDown: 1, ArrowRight: 1, Home: -Infinity, End: Infinity });
/**
 * A place `step` on from `at` in a row of `n` (±Infinity: its ends): from none (-1), as from just before its first
 * going on, just past its last going back; past an end, round (`wrap`) or held there.
 */
const stepIn = (/** @type {number} */ at, /** @type {number} */ step, /** @type {number} */ n, wrap = false) => {
  if (!Number.isFinite(step)) return step < 0 ? 0 : n - 1;
  const to = (at < 0 ? (step > 0 ? -1 : n) : at) + step;
  return wrap ? ((to % n) + n) % n : clamp(to, 0, n - 1);
};
/** What of the page shows (a keyboard over it, iOS's, shortens it; every browser Studio runs in has one). */
const viewport = /** @type {VisualViewport} */ (window.visualViewport);
/** A device of fingers (no hover): its words "tap", not "click". */
const TOUCH = matchMedia(MEDIA.noHover).matches, CLICK = TOUCH ? "tap" : "click";
/** How far a finger moves and still taps, or holds (iOS's own, some 10pt): past it, a scroll (or a sheet's pull). */
const SLOP = 10;
/** Whether an element scrolls up and down: its content taller than it, and let scroll. @param {Element} element */
const scrollsY = (element) => element.scrollHeight > element.clientHeight + 1 && /auto|scroll/.test(getComputedStyle(element).overflowY);
/** A Mac's keys (⌘, ⌥), else a PC's (Ctrl, Alt): as shortcuts are written. */
const MAC = /Mac|iPhone|iPad/.test(navigator.platform);
/**
 * A shortcut as this platform writes it: ⌘Z on a Mac; Ctrl+Z, Ctrl+Shift+Z, Alt+↑ elsewhere. The erase key by its
 * name, as esc and space are (its symbol reads as a box in a keycap's box): delete on a Mac, Backspace elsewhere.
 */
const keyText = (/** @type {string} */ text) => {
  const named = text.replace(/⌫/g, MAC ? "delete" : "Backspace");
  return MAC ? named : named.replace(/[⌘⇧⌥⌃]+/g, (mods, at) => {
    const names = [["⌘⌃", "Ctrl"], ["⌥", "Alt"], ["⇧", "Shift"]].filter(([glyphs]) => [...glyphs].some((glyph) => mods.includes(glyph))).map(([, name]) => name);
    const next = named[at + mods.length];
    return `${names.join("+")}${next && !" -".includes(next) ? "+" : ""}`;
  });
};
/** "⇧ ⌘ Z" reads ⇧⌘Z; alternatives like "[ ]" keep their space. */
const keysOf = (kbd) => {
  const keys = kbd.split(" ");
  return keyText(keys.slice(0, -1).every((key) => "⌘⇧⌥⌃".includes(key)) ? keys.join("") : keys.join(" "));
};
/** The same, as keycaps: one for a modified key, one per alternative. */
const keycaps = (kbd) => ((keys) => (keys.length === 2 && keys.every((key) => /^[A-Z]$/.test(key)) ? `<span class="sequence"><kbd>${keys[0]}</kbd><span class="then" aria-hidden="true">then</span><kbd>${keys[1]}</kbd></span>` : keys.map((key) => `<kbd>${escapeHtml(key)}</kbd>`).join("")))(keysOf(kbd).split(" "));
/** Keys by their names in aria-keyshortcuts, from how they're written here (a Mac's glyphs, or "Ctrl+"). */
const ARIA_KEYS = /** @type {Record<string, string>} */ ({ "⌘": "Meta", "⇧": "Shift", "⌥": "Alt", "⌃": "Control", ctrl: "Control", shift: "Shift", alt: "Alt", "↵": "Enter", enter: "Enter", "⌫": "Backspace", backspace: "Backspace", esc: "Escape", space: "Space", "↑": "ArrowUp", "↓": "ArrowDown", "←": "ArrowLeft", "→": "ArrowRight", "−": "-", "+": "Plus", "⇥": "Tab", tab: "Tab" });
/** A key as aria-keyshortcuts names it ("⌘⇧Z" is Meta+Shift+Z, "Ctrl+K" Control+K); a sequence ("D L"), alternatives ("[ ]"), none. */
function ariaKey(/** @type {string} */ text) {
  const written = text.trim();
  if (!written || /\s/.test(written)) return "";
  let parts = written.length > 1 && written.includes("+") ? written.split("+").filter(Boolean) : [];
  if (!parts.length) {
    let rest = written;
    while (rest.length > 1 && "⌘⇧⌥⌃".includes(rest[0])) (parts.push(rest[0]), (rest = rest.slice(1)));
    parts.push(rest);
  }
  // (A Mac's ⌫ is written "delete": Backspace.)
  return parts.map((part) => (part.toLowerCase() === "delete" ? (MAC ? "Backspace" : "Delete") : ARIA_KEYS[part] ?? ARIA_KEYS[part.toLowerCase()] ?? (part.length === 1 ? part.toUpperCase() : part))).join("+");
}
/** A key that isn't a single one (a letter, a digit, a sign): with ⌘, ⌥ or ⌃, or named (esc, ⌫, ↵, an arrow). */
const NAMED_KEY = /[⌘⌥⌃]|esc|⌫|↵|[←→↑↓]|delete|backspace|ctrl|alt/i;
/**
 * Each keycap under `root`: one on a control said as its shortcut, not its name; one with ⇧ or ↵ in it drawn
 * (once), each beside its glyph for a screen reader; a single key marked (gone while those are off).
 */
function drawKeys(/** @type {ParentNode} */ root) {
  for (const key of root.querySelectorAll("kbd")) {
    const text = key.textContent ?? "";
    key.toggleAttribute("data-single", !NAMED_KEY.test(text));
    // (A key on a control isn't part of its name, said as its shortcut instead: "Collapse", not "Collapse backslash".)
    // (Not where a key isn't the control's own: a drawing tool's comes after D, ⌘K's rows are found by typing.)
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
  // (A key shown only in a control's tip, its data-kbd, said as its shortcut too: Undo's ⌘Z, Full screen's ⇧F.)
  for (const control of /** @type {HTMLElement[]} */ ([...root.querySelectorAll("[data-kbd]:not([aria-keyshortcuts])")])) {
    const name = control.matches('[role="option"]') ? "" : ariaKey(keysOf(control.dataset.kbd ?? ""));
    if (name) control.setAttribute("aria-keyshortcuts", name);
  }
}
/** Typing into a field (a tick isn't one: keys still do what they do). */
const typing = (target) => (target instanceof HTMLInputElement && !["checkbox", "radio", "button"].includes(target.type)) || target instanceof HTMLTextAreaElement || target?.isContentEditable;
/** What holds the page out of reach (an open dialog, a phone's menu sheet), and the page's parts it made inert. */
const holders = new Set(), heldParts = /** @type {HTMLElement[]} */ ([]);
/**
 * The page behind what dims it (a dialog, a phone's menu sheet) inert while `holder` is open, as a modal's is: out of
 * reach of a click, Tab, Find and a screen reader. What floats over the page stays live: the dialogs, menus, their
 * scrims, the toast and the tip. (Inert is inherited style: the page styled again as it flips, once a hold begins and
 * once it ends, not at each holder.)
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

/** Rebuilding a panel's HTML would drop focus: these find the same control again afterwards. */
function focusKey(container) {
  const active = document.activeElement;
  return active instanceof HTMLElement && container.contains(active) ? keyOf(active) : null;
}
/**
 * What finds an element's like once it's drawn again: its id, or its tag and data (not what marks it for a moment:
 * a tip, a key; a legend item named (pointed), a profile's group lit). @param {HTMLElement} element
 */
const keyOf = (element) => ({ tag: element.tagName, id: element.id, data: Object.entries(element.dataset).filter(([name]) => !["tip", "kbd", "pointed", "lit"].includes(name)) });
function restoreFocus(container, key) {
  if (!key || container.contains(document.activeElement)) return;
  const find = () => (key.id ? container.querySelector(`#${CSS.escape(key.id)}`) : [...container.querySelectorAll(key.tag)].find((element) => element instanceof HTMLElement && key.data.length > 0 && key.data.every(([name, value]) => element.dataset[name] === value)));
  let match = find();
  // (Its like on a preset's tile written as a slot yet: the tile written for it, see fillSlot.)
  const preset = match ? undefined : key.data.find(([name]) => name === "openPreset" || name === "presetMenu")?.[1], slot = preset === undefined ? null : container.querySelector(`.tile-slot[data-slot="${CSS.escape(preset)}"]`);
  if (slot) (grid.fillSlot(/** @type {HTMLElement} */ (slot)), (match = find()));
  /** @type {HTMLElement | null | undefined} */ (match)?.focus();
}
// Back and Forward, between the grid and the charts opened from it.
/**
 * Whether a dialog (⌘K, the guide, the shortcuts) is open. Each is a popover in the top layer, not a modal dialog
 * (Safari's AutoFill measures a modal's whole page as ⌘K's field takes the focus); what a modal does, done here: the
 * page behind held inert (holdPage), the keys kept in it, esc, a click beside it, Back on a phone.
 * @param {Element} element
 */
function isOpen(element) {
  return element.matches(":popover-open");
}
/** `then` once a dialog has closed (by then, maybe open again: it says so); `once`, the first time alone. @param {HTMLElement} element @param {() => void} then */
function onClosed(element, then, once = false) {
  const heard = (/** @type {Event} */ event) => {
    if (/** @type {ToggleEvent} */ (event).oldState !== "open") return;
    if (once) element.removeEventListener("toggle", heard);
    then();
  };
  element.addEventListener("toggle", heard);
}
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
  if (reduced() || !element.checkVisibility({ visibilityProperty: true })) return;
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
export {
  $, all$, CLICK, closest, composing, drawKeys, erasing, escapeHtml, holdPage, KEY_STEPS, keycaps, keysOf, keyText,
  MAC, MEDIA, NAMED_KEY, proseHtml, scrollsY, SLOP, stepIn, TOUCH, typing, upDown, viewport, focusKey, keyOf, restoreFocus, isOpen, onClosed, cutFade, fadeOut, fading, scrolled, outOfSight, twinIn,
};
