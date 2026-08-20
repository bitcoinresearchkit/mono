import { createPartialOptions } from "./partial.js";
import {
  createAnchorElement,
  createButtonElement,
  createSmall,
} from "../utils/dom.js";
import { pushHistory, resetParams } from "../utils/url.js";
import { readStored, writeToStorage } from "../utils/storage.js";
import { stringToId } from "../utils/format.js";
import { logUnused } from "./unused.js";
import { setQr } from "../panes/share.js";
import { getConstant } from "./constants.js";
import { colors } from "../utils/colors.js";
import { Unit } from "../utils/units.js";
import { brk } from "../utils/client.js";

export function initOptions() {
  const LS_SELECTED_KEY = `selected_path`;

  const savedPath = /** @type {string[]} */ (
    JSON.parse(readStored(LS_SELECTED_KEY) || "[]") || []
  ).filter((v) => v);

  const partialOptions = createPartialOptions();

  /** @type {Option[]} */
  const list = [];

  /** @type {Map<string, HTMLLIElement>} */
  const liByPath = new Map();

  /** @type {Set<(option: Option) => void>} */
  const selectedListeners = new Set();

  /** @type {HTMLLIElement[]} */
  let highlightedLis = [];

  /**
   * @param {Option | undefined} sel
   */
  function updateHighlight(sel) {
    if (!sel) return;
    for (const li of highlightedLis) {
      delete li.dataset.highlight;
    }
    highlightedLis = [];
    let pathKey = "";
    for (const segment of sel.path) {
      pathKey = pathKey ? `${pathKey}/${segment}` : segment;
      const li = liByPath.get(pathKey);
      if (li) {
        li.dataset.highlight = "";
        highlightedLis.push(li);
      }
    }
    if (!highlightedLis.length) {
      const li = liByPath.get(stringToId(sel.name));
      if (li) {
        li.dataset.highlight = "";
        highlightedLis.push(li);
      }
    }
  }

  const selected = {
    /** @type {Option | undefined} */
    value: undefined,
    /** @param {Option} v */
    set(v) {
      this.value = v;
      updateHighlight(v);
      selectedListeners.forEach((cb) => cb(v));
    },
    /** @param {(option: Option) => void} cb */
    onChange(cb) {
      selectedListeners.add(cb);
      if (this.value) cb(this.value);
      return () => selectedListeners.delete(cb);
    },
  };

  /**
   * @param {string[]} nodePath
   */
  function isOnSelectedPath(nodePath) {
    const selectedPath = selected.value?.path;
    return (
      selectedPath &&
      nodePath.length <= selectedPath.length &&
      nodePath.every((v, i) => v === selectedPath[i])
    );
  }

  /**
   * @template T
   * @param {() => T} fn
   * @returns {() => T}
   */
  function lazy(fn) {
    /** @type {T | undefined} */
    let cached;
    let computed = false;
    return () => {
      if (!computed) {
        computed = true;
        cached = fn();
      }
      return /** @type {T} */ (cached);
    };
  }

  /**
   * @param {(AnyFetchedSeriesBlueprint | FetchedPriceSeriesBlueprint)[]} [arr]
   */
  function arrayToMap(arr) {
    /** @type {Map<Unit, AnyFetchedSeriesBlueprint[]>} */
    const map = new Map();
    /** @type {Map<Unit, Set<number>>} */
    const priceLines = new Map();

    if (!arr) return map;

    // Cache arrays for common units outside loop
    /** @type {AnyFetchedSeriesBlueprint[] | undefined} */
    let usdArr;
    /** @type {AnyFetchedSeriesBlueprint[] | undefined} */
    let satsArr;

    for (let i = 0; i < arr.length; i++) {
      const blueprint = arr[i];

      // Check for undefined series
      if (!blueprint.series) {
        throw new Error(`Blueprint has undefined series: ${blueprint.title}`);
      }

      // Check for price pattern blueprint (has usd/sats sub-series)
      // Use unknown cast for safe property access check
      const maybePriceSeries =
        /** @type {{ usd?: AnySeriesPattern, sats?: AnySeriesPattern }} */ (
          /** @type {unknown} */ (blueprint.series)
        );
      if (maybePriceSeries.usd?.by && maybePriceSeries.sats?.by) {
        const { usd, sats } = maybePriceSeries;
        if (!usdArr) map.set(Unit.usd, (usdArr = []));
        usdArr.push({ ...blueprint, series: usd, unit: Unit.usd });

        if (!satsArr) map.set(Unit.sats, (satsArr = []));
        satsArr.push({ ...blueprint, series: sats, unit: Unit.sats });
        continue;
      }

      // After continue, we know this is a regular series blueprint
      const regularBlueprint = /** @type {AnyFetchedSeriesBlueprint} */ (
        blueprint
      );
      const s = regularBlueprint.series;
      const unit = regularBlueprint.unit;
      if (!unit) continue;

      let unitArr = map.get(unit);
      if (!unitArr) map.set(unit, (unitArr = []));
      unitArr.push(regularBlueprint);

      // Track baseline base values for auto price lines
      const type = regularBlueprint.type;
      if (type === "Baseline") {
        let priceSet = priceLines.get(unit);
        if (!priceSet) priceLines.set(unit, (priceSet = new Set()));
        priceSet.add(regularBlueprint.options?.baseValue?.price ?? 0);
      } else if (!type || type === "Line") {
        // Check if manual price line - avoid Object.values() array allocation
        const by = s.by;
        for (const k in by) {
          if (by[/** @type {Index} */ (k)]?.path?.includes("constant_")) {
            priceLines.get(unit)?.delete(parseFloat(regularBlueprint.title));
          }
          break;
        }
      }
    }

    // Add price lines at end for remaining values
    for (const [unit, values] of priceLines) {
      const arr = map.get(unit);
      if (!arr) continue;
      for (const baseValue of values) {
        const s = getConstant(brk.series.constants, baseValue);
        arr.push({
          series: s,
          title: `${baseValue}`,
          color: colors.gray,
          unit,
          options: {
            lineStyle: 4,
            lastValueVisible: false,
            crosshairMarkerVisible: false,
          },
        });
      }
    }

    return map;
  }

  /**
   * @param {Option} option
   */
  function selectOption(option) {
    if (selected.value === option) return;
    pushHistory(option.path);
    resetParams(option);
    writeToStorage(LS_SELECTED_KEY, JSON.stringify(option.path));
    selected.set(option);
  }

  /**
   * @param {Object} args
   * @param {Option} args.option
   * @param {string} [args.name]
   */
  function createOptionElement({ option, name }) {
    const title = option.title;
    if (option.kind === "link") {
      const href = option.url();

      if (option.qrcode) {
        return createButtonElement({
          inside: option.name,
          title,
          onClick: () => {
            setQr(option.url());
          },
        });
      } else {
        return createAnchorElement({
          href,
          blank: true,
          text: option.name,
          title,
        });
      }
    } else {
      return createAnchorElement({
        href: `/${option.path.join("/")}`,
        title,
        text: name || option.name,
        onClick: () => {
          selectOption(option);
        },
      });
    }
  }

  /** @type {Option | undefined} */
  let savedOption;

  /**
   * @typedef {{ type: "group"; name: string; serName: string; path: string[]; pathKey: string; count: number; children: ProcessedNode[] }} ProcessedGroup
   * @typedef {{ type: "option"; option: Option; path: string[]; pathKey: string }} ProcessedOption
   * @typedef {ProcessedGroup | ProcessedOption} ProcessedNode
   */

  const savedPathStr = savedPath?.join("/");

  /**
   * @param {PartialOptionsTree} partialTree
   * @param {string[]} parentPath
   * @param {string} parentPathStr
   * @returns {{ nodes: ProcessedNode[], count: number }}
   */
  function processPartialTree(
    partialTree,
    parentPath = [],
    parentPathStr = "",
  ) {
    /** @type {ProcessedNode[]} */
    const nodes = [];
    let totalCount = 0;

    for (let i = 0; i < partialTree.length; i++) {
      const anyPartial = partialTree[i];
      if ("tree" in anyPartial) {
        const serName = stringToId(anyPartial.name);
        const pathStr = parentPathStr ? `${parentPathStr}/${serName}` : serName;
        const path = parentPath.concat(serName);
        const { nodes: children, count } = processPartialTree(
          anyPartial.tree,
          path,
          pathStr,
        );

        // Skip groups with no children
        if (count === 0) continue;

        totalCount += count;
        nodes.push({
          type: "group",
          name: anyPartial.name,
          serName,
          path,
          pathKey: pathStr,
          count,
          children,
        });
      } else {
        const option = /** @type {Option} */ (anyPartial);
        const name = option.name;
        const serName = stringToId(name);
        const pathStr = parentPathStr ? `${parentPathStr}/${serName}` : serName;
        const path = parentPath.concat(serName);

        // Transform partial to full option
        if ("kind" in anyPartial && anyPartial.kind === "explorer") {
          option.kind = anyPartial.kind;
          option.path = [];
          option.name = name;
        } else if ("kind" in anyPartial && anyPartial.kind === "heatmap") {
          Object.assign(
            option,
            /** @satisfies {HeatmapOption} */ ({
              ...anyPartial,
              path,
            }),
          );
        } else if ("url" in anyPartial) {
          Object.assign(
            option,
            /** @satisfies {UrlOption} */ ({
              kind: "link",
              path,
              name,
              title: name,
              qrcode: !!anyPartial.qrcode,
              url: anyPartial.url,
            }),
          );
        } else {
          const title = option.title || name;
          const topArr = anyPartial.top;
          const bottomArr = anyPartial.bottom;
          const topFn = lazy(() => arrayToMap(topArr));
          const bottomFn = lazy(() => arrayToMap(bottomArr));
          Object.assign(
            option,
            /** @satisfies {ChartOption} */ ({
              kind: "chart",
              name,
              title,
              path,
              top: topFn,
              bottom: bottomFn,
            }),
          );
        }

        list.push(option);
        totalCount++;

        if (savedPathStr && pathStr === savedPathStr) {
          savedOption = option;
        }

        nodes.push({
          type: "option",
          option,
          path,
          pathKey: pathStr,
        });
      }
    }

    return { nodes, count: totalCount };
  }

  logUnused(brk.series, partialOptions);
  const { nodes: processedTree } = processPartialTree(partialOptions);

  /**
   * @param {ProcessedNode[]} nodes
   * @param {HTMLElement} parentEl
   * @param {boolean} autoOpen
   */
  function buildTreeDOM(nodes, parentEl, autoOpen) {
    const ul = window.document.createElement("ul");

    for (const node of nodes) {
      const li = window.document.createElement("li");
      ul.append(li);

      liByPath.set(node.pathKey, li);

      if (node.type === "group") {
        const details = window.document.createElement("details");
        details.dataset.name = node.serName;
        li.appendChild(details);

        const summary = window.document.createElement("summary");
        details.append(summary);
        summary.append(node.name);

        summary.append(createSmall(`[${node.count.toLocaleString("en-us")}]`));

        let built = false;
        if (autoOpen && isOnSelectedPath(node.path)) {
          built = true;
          details.open = true;
          buildTreeDOM(node.children, details, true);
        }
        details.addEventListener("toggle", () => {
          if (details.open && !built) {
            built = true;
            buildTreeDOM(node.children, details, false);
          }
          updateHighlight(selected.value);
        });
      } else {
        const element = createOptionElement({
          option: node.option,
        });
        li.append(element);
      }
    }

    parentEl.append(ul);
  }

  /** @type {HTMLElement | null} */
  let parentEl = null;

  /**
   * @param {HTMLElement} el
   */
  function setParent(el) {
    if (parentEl) return;
    parentEl = el;
    buildTreeDOM(processedTree, el, true);
    updateHighlight(selected.value);
  }

  const tree = /** @type {OptionsTree} */ (partialOptions);

  function resolveUrl() {
    const segments = window.location.pathname.split("/").filter((v) => v);
    let folder = tree;
    for (let i = 0; i < segments.length; i++) {
      const match = folder.find((v) => segments[i] === stringToId(v.name));
      if (!match) break;
      if (i < segments.length - 1) {
        if (!("tree" in match)) break;
        folder = match.tree;
      } else if (!("tree" in match)) {
        selected.set(match);
        return;
      } else {
        break;
      }
    }
    selected.set(!segments.length && savedOption ? savedOption : list[0]);
  }

  resolveUrl();

  if (!selected.value) {
    const option = savedOption || list[0];
    if (option) {
      selected.set(option);
    }
  }

  return {
    selected,
    list,
    tree,
    setParent,
    createOptionElement,
    selectOption,
    resolveUrl,
  };
}
/** @typedef {ReturnType<typeof initOptions>} Options */
