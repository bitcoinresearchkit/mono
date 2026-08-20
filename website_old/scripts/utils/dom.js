/**
 * @param {string} id
 * @returns {HTMLElement}
 */
export function getElementById(id) {
  const element = window.document.getElementById(id);
  if (!element) throw `Element with id = "${id}" should exist`;
  return element;
}

/**
 * @param {HTMLElement} element
 */
export function isHidden(element) {
  return element.tagName !== "BODY" && !element.offsetParent;
}

/**
 *
 * @param {HTMLElement} element
 * @param {VoidFunction} callback
 */
export function onFirstIntersection(element, callback) {
  const observer = new IntersectionObserver((entries) => {
    for (let i = 0; i < entries.length; i++) {
      if (entries[i].isIntersecting) {
        callback();
        observer.disconnect();
      }
    }
  });
  observer.observe(element);
}

/**
 * @param {string} text
 */
export function createSpan(text) {
  const span = window.document.createElement("span");
  span.textContent = text;
  return span;
}

/**
 * @param {string} text
 */
export function createSmall(text) {
  const small = window.document.createElement("small");
  small.textContent = text;
  return small;
}

/**
 * @param {string} name
 */
export function createSpanName(name) {
  const spanName = window.document.createElement("span");
  spanName.classList.add("name");
  const [first, second, third] = name.split(" - ");
  spanName.innerHTML = first;

  if (second) {
    const smallRest = window.document.createElement("small");
    smallRest.innerHTML = ` — ${second}`;
    spanName.append(smallRest);

    if (third) {
      throw "Shouldn't have more than one dash";
    }
  }

  return spanName;
}

/**
 * @param {Object} arg
 * @param {string} arg.href
 * @param {string} arg.title
 * @param {string} [arg.text]
 * @param {boolean} [arg.blank]
 * @param {VoidFunction} [arg.onClick]
 * @param {boolean} [arg.preventDefault]
 */
export function createAnchorElement({
  text,
  href,
  blank,
  onClick,
  title,
  preventDefault,
}) {
  const anchor = window.document.createElement("a");
  anchor.href = href;
  anchor.title = title.toUpperCase();

  if (text) {
    anchor.innerText = text;
  }

  if (blank) {
    anchor.target = "_blank";
    anchor.rel = "noopener noreferrer";
  }

  if (onClick || preventDefault) {
    if (onClick) {
      anchor.addEventListener("click", (event) => {
        event.preventDefault();
        onClick();
      });
    }
  }

  return anchor;
}

// Intercept plain left-clicks for SPA nav; let modified clicks
// (cmd/ctrl/shift/middle) and right-click fall through so the
// anchor's native open-in-new-tab / context-menu behavior works.
/** @param {HTMLElement} el @param {() => void} handler */
export function onPlainClick(el, handler) {
  el.addEventListener("click", (e) => {
    if (e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return;
    e.preventDefault();
    handler();
  });
}

/**
 * @param {Object} arg
 * @param {string | HTMLElement} arg.inside
 * @param {string} arg.title
 * @param {(event: MouseEvent) => void} arg.onClick
 */
export function createButtonElement({ inside: text, onClick, title }) {
  const button = window.document.createElement("button");

  button.append(text);

  button.title = title.toUpperCase();

  button.addEventListener("click", onClick);

  return button;
}

/**
 * @param {Object} args
 * @param {string} args.inputName
 * @param {string} args.inputId
 * @param {string} args.inputValue
 * @param {boolean} [args.inputChecked=false]
 * @param {string} [args.title]
 * @param {'radio' | 'checkbox'} args.type
 * @param {(event: MouseEvent) => void} [args.onClick]
 */
export function createLabeledInput({
  inputId,
  inputName,
  inputValue,
  inputChecked = false,
  title,
  onClick,
  type,
}) {
  const label = window.document.createElement("label");

  inputId = inputId.toLowerCase();

  const input = window.document.createElement("input");
  if (type === "radio") {
    input.type = "radio";
    input.name = inputName;
  } else {
    input.type = "checkbox";
  }
  input.id = inputId;
  input.value = inputValue;
  input.checked = inputChecked;
  label.append(input);

  label.id = `${inputId}-label`;
  if (title) {
    label.title = title;
  }

  if (onClick) {
    input.addEventListener("click", onClick);
  } else {
    label.htmlFor = inputId;
  }

  return {
    label,
    input,
  };
}


/**
 * @template T
 * @typedef {Object} Select
 * @property {HTMLElement} element
 * @property {() => T} get
 * @property {(choice: T) => void} set
 */

/**
 * @template T
 * @param {Object} args
 * @param {T} args.initialValue
 * @param {string} [args.id]
 * @param {readonly T[]} args.choices
 * @param {(value: T) => void} [args.onChange]
 * @param {(choice: T) => string} [args.toKey]
 * @param {(choice: T) => string} [args.toLabel]
 * @param {(choice: T) => string | undefined} [args.toTitle]
 */
export function createRadios({
  id,
  choices,
  initialValue,
  onChange,
  toKey = /** @type {(choice: T) => string} */ ((c) => String(c)),
  toLabel = /** @type {(choice: T) => string} */ ((c) => String(c)),
  toTitle,
}) {
  const fieldset = window.document.createElement("fieldset");

  const initialKey = toKey(initialValue);

  /** @param {string} key */
  const fromKey = (key) =>
    choices.find((c) => toKey(c) === key) ?? initialValue;

  if (choices.length === 1) {
    fieldset.append(createSpan(toLabel(choices[0])));
  } else {
    const groupId = id ?? "";
    choices.forEach((choice) => {
      const key = toKey(choice);
      const { label } = createLabeledInput({
        inputId: `${groupId}-${key.toLowerCase()}`,
        inputName: groupId,
        inputValue: key,
        inputChecked: key === initialKey,
        title: toTitle?.(choice),
        type: "radio",
      });

      const text = window.document.createTextNode(toLabel(choice));
      label.append(text);
      fieldset.append(label);
    });

    fieldset.addEventListener("change", (event) => {
      if (!(event.target instanceof HTMLInputElement)) return;
      onChange?.(fromKey(event.target.value));
    });
  }

  return fieldset;
}

/**
 * @template T
 * @param {Object} args
 * @param {T} args.initialValue
 * @param {string} [args.id]
 * @param {string} [args.label]
 * @param {readonly T[]} args.choices
 * @param {(value: T) => void} [args.onChange]
 * @param {(choice: T) => string} [args.toKey]
 * @param {(choice: T) => string} [args.toLabel]
 * @param {boolean} [args.sorted]
 * @param {{ label: string, items: T[] }[]} [args.groups]
 * @returns {Select<T>}
 */
export function createSelect({
  id,
  label,
  choices: unsortedChoices,
  groups,
  initialValue,
  onChange,
  sorted,
  toKey = /** @type {(choice: T) => string} */ ((c) => String(c)),
  toLabel = /** @type {(choice: T) => string} */ ((c) => String(c)),
}) {
  const choices = sorted
    ? unsortedChoices.toSorted((a, b) => toLabel(a).localeCompare(toLabel(b)))
    : unsortedChoices;

  const initialKey = toKey(initialValue);

  /** @param {string} key */
  const fromKey = (key) =>
    choices.find((c) => toKey(c) === key) ?? initialValue;

  if (choices.length === 1) {
    return {
      element: createSpan(toLabel(choices[0])),
      get: () => initialValue,
      set: () => {},
    };
  }

  const element = window.document.createElement("label");
  if (label) {
    element.append(createSpan(label));
  }

  const select = window.document.createElement("select");
  select.id = id ?? "";
  select.name = id ?? "";
  element.append(select);

  /** @param {T} choice */
  const createOption = (choice) => {
    const key = toKey(choice);
    const option = window.document.createElement("option");
    option.value = key;
    option.textContent = toLabel(choice);
    if (key === initialKey) {
      option.selected = true;
    }
    return option;
  };

  if (groups) {
    groups.forEach(({ label, items }) => {
      const optgroup = window.document.createElement("optgroup");
      optgroup.label = label;
      items.forEach((choice) => optgroup.append(createOption(choice)));
      select.append(optgroup);
    });
  } else {
    choices.forEach((choice) => select.append(createOption(choice)));
  }

  select.addEventListener("change", () => {
    onChange?.(fromKey(select.value));
  });

  const remaining = choices.length - 1;
  if (remaining > 0) {
    element.append(createSmall(`+${remaining}`));
    element.append(createSpan("↓"));
  }

  element.addEventListener("click", (e) => {
    if (e.target !== select && "showPicker" in select) {
      e.preventDefault();
      select.showPicker();
    }
  });

  return {
    element,
    get: () => fromKey(select.value),
    set: (choice) => {
      select.value = toKey(choice);
    },
  };
}

/**
 * @param {string} [title]
 * @param {1 | 2 | 3} [level]
 */
export function createHeader(title = "", level = 1) {
  const headerElement = window.document.createElement("header");

  const headingElement = window.document.createElement(`h${level}`);
  headingElement.innerHTML = title;
  headerElement.append(headingElement);
  headingElement.style.display = "block";

  return {
    headerElement,
    headingElement,
  };
}
