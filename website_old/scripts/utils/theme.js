import { readStored, removeStored, writeToStorage } from "./storage.js";

const preferredColorSchemeMatchMedia = window.matchMedia(
  "(prefers-color-scheme: dark)",
);
const stored = readStored("theme");
const initial = stored
  ? stored === "dark"
  : preferredColorSchemeMatchMedia.matches;

export let dark = initial;

/** @type {Set<() => void>} */
const callbacks = new Set();

/** @param {() => void} callback */
export function onChange(callback) {
  callbacks.add(callback);
  return () => callbacks.delete(callback);
}

const themeButton = /** @type {HTMLButtonElement | null} */ (
  document.getElementById("theme-button")
);
let running = false;

/** @param {boolean} value */
function setDark(value) {
  if (running || dark === value) return;
  dark = value;
  running = true;
  if (themeButton) themeButton.disabled = true;
  const swap = () => {
    apply(value);
    callbacks.forEach((cb) => cb());
  };
  document.documentElement.classList.add("no-transitions");
  const restore = () => {
    document.documentElement.classList.remove("no-transitions");
    running = false;
    if (themeButton) themeButton.disabled = false;
  };
  if (document.startViewTransition) {
    document.startViewTransition(swap).finished.finally(restore);
  } else {
    swap();
    requestAnimationFrame(restore);
  }
}

/** @param {boolean} isDark */
function apply(isDark) {
  document.documentElement.style.colorScheme = isDark ? "dark" : "light";
}
apply(initial);

preferredColorSchemeMatchMedia.addEventListener("change", ({ matches }) => {
  if (!readStored("theme")) {
    setDark(matches);
  }
});

function invert() {
  const newValue = !dark;
  setDark(newValue);
  if (newValue === preferredColorSchemeMatchMedia.matches) {
    removeStored("theme");
  } else {
    writeToStorage("theme", newValue ? "dark" : "light");
  }
}

themeButton?.addEventListener("click", invert);
