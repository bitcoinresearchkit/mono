import {
  searchInput,
  searchLabelElement,
  searchResultsElement,
} from "../utils/elements.js";
import { bitview } from "../utils/client.js";
import { lazy } from "../options/lazy.js";
import { createChartSearch } from "./search-client.js";

/**
 * @param {Options} options
 */
export function init(options) {
  console.log("search: init");

  const chartSearch = lazy(createChartSearch);

  /** @type {HTMLLIElement | undefined} */
  let highlighted;

  /** @param {HTMLLIElement} [li] */
  function setHighlight(li) {
    if (highlighted) delete highlighted.dataset.highlight;
    highlighted = li;
    if (li) li.dataset.highlight = "";
  }

  const HEX64_RE = /^[0-9a-f]{64}$/i;
  const ADDR_RE = /^([13][a-km-zA-HJ-NP-Z1-9]{25,34}|bc1[a-z0-9]{8,87})$/;

  /** @param {string} label @param {string} href @param {Element | null} [before] @param {boolean} [blank] */
  function createResultLink(label, href, before, blank = false) {
    const li = window.document.createElement("li");
    const a = window.document.createElement("a");
    a.href = href;
    a.textContent = label;
    a.title = label;
    if (blank) {
      a.target = "_blank";
      a.rel = "noreferrer";
    }
    if (href === window.location.pathname) setHighlight(li);
    if (!blank) {
      a.addEventListener("click", (e) => {
        e.preventDefault();
        setHighlight(li);
        if (href === "/") {
          options.selectOption(options.list[0]);
          return;
        }
        history.pushState(null, "", href);
        options.resolveUrl();
      });
    }
    li.append(a);
    searchResultsElement.insertBefore(li, before ?? null);
  }

  /** @type {AbortController | undefined} */
  let lookupController;
  let inputVersion = 0;

  /** @param {string} needle @param {AbortSignal} signal */
  async function lookup(needle, signal) {
    /** @type {Array<[string, string]>} */
    const results = [];

    if (HEX64_RE.test(needle)) {
      const [blockRes, txRes] = await Promise.allSettled([
        bitview.getBlock(needle, { signal }),
        bitview.getTx(needle, { signal }),
      ]);
      if (signal.aborted) return;
      if (blockRes.status === "fulfilled")
        results.push(["Block", `/block/${needle}`]);
      if (txRes.status === "fulfilled")
        results.push(["Transaction", `/tx/${needle}`]);
    } else if (ADDR_RE.test(needle)) {
      try {
        const { isvalid } = await bitview.validateAddress(needle, { signal });
        if (signal.aborted || !isvalid) return;
        results.push(["Address", `/address/${needle}`]);
      } catch {
        return;
      }
    } else {
      return;
    }

    const before = searchResultsElement.firstElementChild;
    for (const [label, href] of results) {
      createResultLink(`${label} ${needle}`, href, before);
    }
    // Remove "No results" placeholder if present
    const last = searchResultsElement.lastElementChild;
    if (last && !last.querySelector("a")) last.remove();
  }

  async function inputEvent() {
    const version = ++inputVersion;
    const needle = /** @type {string} */ (searchInput.value).trim();

    if (lookupController) lookupController.abort();

    searchResultsElement.scrollTo({ top: 0 });
    searchResultsElement.innerHTML = "";
    setHighlight();

    if (!needle.length) return;

    const indexMatch = needle.match(
      /^(?:(block|b)|(transaction|tx))?\s*#?\s*(\d+)$/i,
    );
    const isExplorerLookup = Boolean(
      indexMatch || HEX64_RE.test(needle) || ADDR_RE.test(needle),
    );
    /** @type {Array<[title: string, href: string, blank: boolean]>} */
    let matches = [];
    if (!isExplorerLookup) {
      const loading = window.document.createElement("li");
      loading.textContent = "Loading charts…";
      loading.style.color = "var(--off-color)";
      searchResultsElement.appendChild(loading);
      try {
        matches = await chartSearch()(needle);
      } catch (error) {
        console.error(error);
        if (version === inputVersion)
          loading.textContent = "Search unavailable";
        return;
      }
    }
    if (version !== inputVersion) return;
    searchResultsElement.innerHTML = "";

    if (indexMatch) {
      const num = indexMatch[3];
      const entries = indexMatch[1]
        ? [["Block", `/block/${num}`]]
        : indexMatch[2]
          ? [["Transaction", `/tx/${num}`]]
          : [
              ["Block", `/block/${num}`],
              ["Transaction", `/tx/${num}`],
            ];
      for (const [label, href] of entries) {
        createResultLink(`${label} #${num}`, href);
      }
    }

    lookupController = new AbortController();
    lookup(needle, lookupController.signal);

    if (matches.length) {
      matches.forEach(([title, href, blank]) => {
        createResultLink(title, href, undefined, blank);
      });
    }

    if (!searchResultsElement.children.length) {
      const li = window.document.createElement("li");
      li.textContent = "No results";
      li.style.color = "var(--off-color)";
      searchResultsElement.appendChild(li);
    }
  }

  options.selected.onChange(() => {
    const selected = options.selected.value;
    const href =
      selected?.kind === "explorer"
        ? window.location.pathname
        : selected?.path.length
          ? `/${selected.path.join("/")}`
          : null;
    if (!href) return setHighlight();
    for (const li of searchResultsElement.children) {
      const a = li.querySelector("a");
      if (a && a.getAttribute("href") === href) {
        return setHighlight(/** @type {HTMLLIElement} */ (li));
      }
    }
    setHighlight();
  });

  void inputEvent();

  searchInput.addEventListener("input", () => void inputEvent());
  const len = searchInput.value.length;
  searchInput.setSelectionRange(len, len);
}

document.addEventListener("keydown", (e) => {
  const el = document.activeElement;

  const isTextInput =
    el?.tagName === "INPUT" &&
    /** @type {HTMLInputElement} */ (el).type === "text";

  if (e.key === "/" && !isTextInput) {
    e.preventDefault();
    searchLabelElement.click();
    searchInput.focus();
  }
});
