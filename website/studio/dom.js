// @ts-check
// The page's means: its elements found and made, text made safe as HTML, keys and pointers read.
import { KEY_GLYPHS, keyGlyph } from "/studio/icons.js";
import { clamp } from "/studio/math.js";

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

export {
  $, all$, CLICK, closest, composing, drawKeys, erasing, escapeHtml, holdPage, KEY_STEPS, keycaps, keysOf, keyText,
  MAC, MEDIA, NAMED_KEY, proseHtml, scrollsY, SLOP, stepIn, TOUCH, typing, upDown, viewport,
};
