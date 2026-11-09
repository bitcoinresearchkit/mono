// @ts-check
// Every showcase's way to send a note, in one place (loaded by `/assets/launch/launch.js`, which each page loads first).
//
// - A page has `<button type="button" data-feedback>Feedback</button>` where its own quiet links are, in its own look;
//   launch.js answers its press (and Studio's ⌘K) by calling `bitviewFeedback.open()`.
// - One card over the page, its look in a shadow root, light or dark as the page is. At the top of a phone's screen,
//   Send in its top row, so the keyboard comes up under it and never covers Send; in the bottom corner on a desk.
//   Opened from the press itself, or iOS won't raise the keyboard.
// - Nothing leaves until Send: nothing is asked of the inbox's host as a page loads, on Bitview or on a node of someone's
//   own. What goes with the note is what makes it reproducible, each value written out in the card as it'll be sent:
//   the exact address with what can name someone masked (a page whose address holds their own figures starts unticked),
//   the build, browser, screen and theme, and the page's recent errors. Nothing that identifies a person.
(() => {
  const INBOX = "https://feedback.bitview.space/";
  const compact = matchMedia("(max-width: 760px), (max-height: 520px)");

  /** The app's folder alone, always sent: "/" for anything that isn't one (`/index.html`). */
  const folder = location.pathname.split("/")[1], page = /^[a-z0-9_-]{1,40}$/.test(folder) ? `/${folder}/` : "/";

  /** Where a token can start: an address written inside encoded JSON comes right after an escape like `%22`. */
  const START = "(?<=^|[^0-9A-Za-z]|%[0-9A-Fa-f]{2})", END = "(?![0-9A-Za-z])";
  /** By shape, wherever it appears: what can name someone's coins, or the node they're on. */
  const MASKS = /** @type {const} */ ([
    [new RegExp(`${START}(?:bc1|tb1|bcrt1)[02-9ac-hj-np-z]{6,}${END}`, "gi"), ":address"],
    [new RegExp(`${START}[13][1-9A-HJ-NP-Za-km-z]{25,34}${END}`, "g"), ":address"],
    [new RegExp(`${START}[xyztuvYZUV](?:pub|prv)[1-9A-HJ-NP-Za-km-z]{70,}${END}`, "g"), ":key"],
    [new RegExp(`${START}[0-9a-fA-F]{64}${END}`, "g"), ":hash"],
    [/https?:\/\/[^\s/"']+/g, ""],
    [new RegExp(`${START}\\d{1,3}(?:\\.\\d{1,3}){3}${END}`, "g"), ":ip"],
  ]);
  const mask = (/** @type {string} */ text) => MASKS.reduce((masked, [shape, name]) => masked.replace(shape, name), location.hostname ? text.replaceAll(location.hostname, ":host") : text);

  /** All of where this is, masked, when there's more to it than the folder. A page whose state isn't in its address (a
   * Studio chart) says what address would open it. */
  const here = () => {
    const own = /** @type {any} */ (window).bitviewFeedbackLink?.();
    const place = own ? new URL(own, location.href) : location;
    return mask(place.pathname + place.search + place.hash);
  };
  const attachable = () => here().replace(/\/$/, "") !== page.replace(/\/$/, "") && here().length <= 6000;
  /** A page whose address holds someone's own figures says so: its box starts unticked. */
  const startsUnticked = () => Boolean(document.querySelector('meta[name="bitview-feedback"][content="address-unticked"]'));
  /** As it'll read to the sender, its escapes undone. @param {string} link */
  const readable = (link) => {
    try {
      return decodeURIComponent(link);
    } catch {
      return link;
    }
  };

  const agent = navigator.userAgent;
  const engine = /Firefox\//.test(agent) ? "firefox" : /Chrome\/|Chromium\//.test(agent) ? "chromium" : /Safari\//.test(agent) ? "safari" : "other";
  /** Its major version alone. */
  const major = /(?:Firefox|Chrome|Chromium|Version)\/(\d+)/.exec(agent)?.[1] ?? "";
  const NAMES = { firefox: "Firefox", chromium: "Chromium", safari: "Safari", other: "Another browser" };
  const os = /iPhone|iPad|iPod/.test(agent) || (/Macintosh/.test(agent) && navigator.maxTouchPoints > 1) ? "ios" : /Android/.test(agent) ? "android" : /Macintosh/.test(agent) ? "macos" : /Windows/.test(agent) ? "windows" : /Linux|X11/.test(agent) ? "linux" : "other";
  const SYSTEMS = { ios: "iOS", android: "Android", macos: "macOS", windows: "Windows", linux: "Linux", other: "another system" };

  /** The page's own recent errors, kept from its start by launch.js, masked. */
  const errors = () => /** @type {string[]} */ (/** @type {any} */ (window).bitviewFeedbackErrors ?? []).map((text) => mask(text).slice(0, 300));
  /** Everything that says how to see what the sender saw, and nothing of who they are: sizes to the nearest 10 px, as a
   * window's exact size is close to a fingerprint and no layout turns on the last digit. @param {HTMLElement} card */
  const context = (card) => ({
    os, major,
    viewport: `${Math.round(innerWidth / 10) * 10}x${Math.round(innerHeight / 10) * 10}`,
    scale: String(Math.round(devicePixelRatio * 2) / 2),
    // (What the page is really showing, whichever way it decided: the card's own background, light or dark as it is.)
    theme: Number(/\d+/.exec(getComputedStyle(card).backgroundColor)?.[0]) > 127 ? "light" : "dark",
    mode: /** @type {any} */ (navigator).standalone === true || matchMedia("(display-mode: standalone)").matches ? "app" : "browser",
    errors: errors(),
  });

  /** @type {{ host: HTMLElement, show: () => void } | undefined} */
  let made;

  function make() {
    const host = document.createElement("bitview-feedback");
    const root = host.attachShadow({ mode: "open" });
    root.innerHTML = `<style>
      :host { all: initial; color-scheme: inherit; }
      .card {
        --orange: #c24e00;
        --orange-hover: #a94400;
        --surface: light-dark(#fff, #1f1f21);
        --ink: light-dark(#1d1d1f, #f5f5f7);
        --soft: light-dark(rgb(0 0 0 / 0.62), rgb(255 255 255 / 0.64));
        --faint: light-dark(rgb(0 0 0 / 0.55), rgb(255 255 255 / 0.6));
        --hairline: light-dark(rgb(0 0 0 / 0.14), rgb(255 255 255 / 0.16));
        --press: light-dark(rgb(0 0 0 / 0.06), rgb(255 255 255 / 0.1));
        --field: light-dark(rgb(0 0 0 / 0.035), rgb(255 255 255 / 0.06));
        position: fixed; inset: auto 20px calc(20px + env(safe-area-inset-bottom)) auto; width: 384px; max-width: calc(100vw - 24px); max-height: calc(100dvh - 24px); margin: 0; padding: 14px; overflow: auto; box-sizing: border-box; border: 0; border-radius: 16px;
        background: var(--surface); color: var(--ink); color-scheme: inherit; font: 450 14px/1.4 Inter, ui-sans-serif, system-ui, -apple-system, sans-serif; -webkit-font-smoothing: antialiased;
        box-shadow: inset 0 0 0 0.5px var(--hairline), 0 1px 2px rgb(0 0 0 / 0.06), 0 22px 48px -16px rgb(0 0 0 / 0.42);
        opacity: 0; transform: translateY(10px) scale(0.97); transition: opacity 0.18s, transform 0.22s cubic-bezier(0.32, 1.2, 0.5, 1), display 0.22s allow-discrete, overlay 0.22s allow-discrete;
      }
      .card:popover-open { opacity: 1; transform: none; }
      @starting-style { .card:popover-open { opacity: 0; transform: translateY(10px) scale(0.97); } }
      @media (prefers-reduced-motion: reduce) { .card { transition: opacity 0.15s, display 0.15s allow-discrete, overlay 0.15s allow-discrete; transform: none; } @starting-style { .card:popover-open { transform: none; } } }
      header { display: flex; align-items: center; gap: 8px; margin: -4px -6px 10px -4px; }
      h2 { flex: 1; margin: 0; font-size: 14px; font-weight: 600; letter-spacing: -0.01em; }
      button { border: 0; color: inherit; font: inherit; cursor: pointer; -webkit-tap-highlight-color: transparent; transition: background-color 0.15s, scale 0.15s; }
      button:active { scale: 0.96; }
      .close { display: grid; place-items: center; width: 34px; height: 34px; border-radius: 9px; background: none; color: var(--faint); }
      .close:hover { background: var(--press); color: var(--ink); }
      svg { width: 11px; height: 11px; }
      textarea, input[type="text"] { display: block; width: 100%; box-sizing: border-box; border: 0; border-radius: 9px; background: var(--field); color: inherit; font: inherit; box-shadow: inset 0 0 0 0.5px var(--hairline); }
      textarea { min-height: 104px; padding: 10px 11px; resize: vertical; }
      input[type="text"] { height: 40px; margin-top: 8px; padding: 0 11px; }
      ::placeholder { color: var(--faint); }
      textarea:focus-visible, input:focus-visible, button:focus-visible { outline: 2px solid var(--orange); outline-offset: 1px; }
      .attach { display: flex; align-items: center; gap: 8px; min-height: 36px; margin-top: 4px; color: var(--soft); font-size: 13px; cursor: pointer; }
      .exact { margin: 0 2px 2px 24px; color: var(--soft); font: 12px/1.35 ui-monospace, SFMono-Regular, Menlo, monospace; overflow: hidden; display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 2; overflow-wrap: anywhere; }
      .exact:empty { display: none; }
      details { margin-top: 6px; color: var(--soft); font-size: 12.5px; }
      summary { cursor: pointer; padding: 4px 2px; }
      summary:focus-visible { outline: 2px solid var(--orange); outline-offset: 1px; border-radius: 4px; }
      dl { display: grid; grid-template-columns: auto 1fr; gap: 3px 10px; margin: 4px 2px; }
      dt { color: var(--faint); }
      dd { margin: 0; color: var(--ink); overflow-wrap: anywhere; }
      dd.mono { font: 12px/1.35 ui-monospace, SFMono-Regular, Menlo, monospace; }
      .attach[hidden] { display: none; }
      .attach input { width: 16px; height: 16px; margin: 0; accent-color: var(--orange); }
      .sent-with { margin: 8px 2px 0; color: var(--soft); font-size: 12.5px; }
      .sent-with + .sent-with { margin-top: 2px; }
      .sent-with b { color: var(--ink); font-weight: 500; }
      .said { margin: 0 2px 8px; color: light-dark(#a8201a, #ff8a80); font-size: 13px; }
      .said:empty { display: none; }
      .send { height: 34px; padding: 0 14px; border-radius: 9px; background: var(--orange); color: #fff; font-size: 14px; font-weight: 600; }
      .send:hover:not(:disabled) { background: var(--orange-hover); }
      .send:disabled { opacity: 0.45; cursor: default; }
      .done { position: absolute; inset: 0; display: grid; place-content: center; justify-items: center; gap: 8px; font-size: 15px; font-weight: 600; visibility: hidden; pointer-events: none; }
      .done svg { width: 30px; height: 30px; color: var(--orange); }
      @media (max-width: 760px), (max-height: 520px) {
        .card { inset: calc(10px + env(safe-area-inset-top)) 12px auto; width: auto; max-width: 520px; margin-inline: auto; transform: translateY(-10px) scale(0.97); }
        @starting-style { .card:popover-open { transform: translateY(-10px) scale(0.97); } }
        textarea, input[type="text"] { font-size: 16px; }
        .close { width: 44px; height: 44px; }
        .send { height: 44px; padding: 0 18px; }
      }
      .card.is-done form { visibility: hidden; }
      .card.is-done .done { visibility: visible; }
    </style><div class="card" popover="auto" role="dialog" aria-labelledby="title">
      <header><button class="close" type="button" aria-label="Close"><svg viewBox="0 0 10 10" aria-hidden="true"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg></button><h2 id="title">Feedback</h2><button class="send" type="submit" form="note" disabled>Send</button></header>
      <form id="note" novalidate>
        <p class="said" id="said" role="alert"></p>
        <textarea id="message" maxlength="4000" placeholder="What's wrong, confusing, or missing?" aria-label="Your feedback"></textarea>
        <input id="contact" type="text" maxlength="200" autocomplete="off" placeholder="Contact, if you'd like a reply (optional)" aria-label="Contact, optional" />
        <label class="attach" hidden><input id="attach" type="checkbox" /><span>Include this page's exact address</span></label>
        <p class="exact" id="exact"></p>
        <details><summary>Also sent: page version, build, browser, screen, theme, recent errors</summary><dl id="included"></dl></details>
        <p class="sent-with">Goes to the Bitview project, through Cloudflare, and an AI helps read it. Bitview doesn't store IP addresses and deletes notes after 30 days.</p>
      </form>
      <div class="done" role="status"><svg viewBox="0 0 24 24" fill="none" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg><span id="sent"></span></div>
    </div>`;

    const $ = (/** @type {string} */ selector) => /** @type {any} */ (root.querySelector(selector));
    const card = $(".card"), form = $("form"), message = $("#message"), contact = $("#contact"), attach = $("#attach"), attachRow = $(".attach"), said = $("#said"), send = $(".send");
    /** The page's version (as it loaded) and the server's build, read as the card opens and sent if they've come by
     * Send; never waited for. */
    let version = "", build = "";
    let sending = false;

    /** What will go, written out as it will go. */
    const describe = () => {
      $("#exact").textContent = attach.checked ? readable(here()) : "";
      const { viewport, scale, theme, mode, errors: recent } = context(card);
      const rows = [
        ["Page", attach.checked ? "The exact address, above" : page],
        ["Version", version || "Not known"],
        ["Build", build || "Not known"],
        ["Browser", `${NAMES[/** @type {keyof typeof NAMES} */ (engine)]}${major ? ` ${major}` : ""}, ${SYSTEMS[os]}, ${mode === "app" ? "installed app" : "in the browser"}`],
        ["Screen", `About ${viewport.replace("x", " × ")}, ${scale}×, ${theme}`],
        ["Errors", recent.length ? recent.join("\n") : "None"],
      ];
      $("#included").replaceChildren(...rows.flatMap(([name, value]) => [
        Object.assign(document.createElement("dt"), { textContent: name }),
        Object.assign(document.createElement("dd"), { textContent: value, className: name === "Errors" && recent.length ? "mono" : "" }),
      ]));
    };
    const ready = () => { send.disabled = sending || !message.value.trim(); };

    message.addEventListener("input", () => { said.textContent = ""; ready(); });
    attach.addEventListener("change", describe);
    $(".close").addEventListener("click", () => card.hidePopover());
    message.addEventListener("keydown", (/** @type {KeyboardEvent} */ event) => {
      if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) form.requestSubmit();
    });

    form.addEventListener("submit", async (/** @type {SubmitEvent} */ event) => {
      event.preventDefault();
      if (send.disabled) return;
      sending = message.readOnly = contact.readOnly = true;
      ready();
      said.textContent = "";
      try {
        const body = JSON.stringify({ message: message.value, contact: contact.value, page, link: attach.checked ? here() : "", version, browser: `${engine}-${compact.matches ? "phone" : "desk"}`, context: { build, ...context(card) } });
        const response = await fetch(INBOX, { method: "POST", body, credentials: "omit", referrerPolicy: "no-referrer", keepalive: true, signal: AbortSignal.timeout(15_000) });
        if (!response.ok) throw response.status;
        message.value = contact.value = "";
        card.classList.add("is-done");
        // (Written once it's shown: a status already full as it's shown may go unannounced.)
        setTimeout(() => ($("#sent").textContent = "Sent. Thank you."));
      } catch (status) {
        said.textContent = status === 400 || status === 413 ? "This note can't be sent as it is; try shortening it." : `${status === 503 ? "The inbox is full for now." : status === 429 ? "Too many sent just now." : "Couldn't send it."} Your note is still here; try again later.`;
      }
      sending = message.readOnly = contact.readOnly = false;
      ready();
    });

    /** What had the focus, given it back as the card goes. */
    let opener = /** @type {HTMLElement | null} */ (null);
    card.addEventListener("toggle", (/** @type {ToggleEvent} */ event) => {
      if (event.newState !== "closed") return;
      card.classList.remove("is-done");
      $("#sent").textContent = "";
      opener?.focus({ preventScroll: true });
    });

    function show() {
      if (card.matches(":popover-open")) return;
      opener = /** @type {HTMLElement | null} */ (document.activeElement);
      attachRow.hidden = !attachable();
      attach.checked = !attachRow.hidden && !startsUnticked();
      describe();
      said.textContent = "";
      ready();
      card.showPopover();
      message.focus();
      version = build = "";
      Promise.resolve(/** @type {any} */ (window).bitviewPageVersion?.()).then((said) => ((version = said ?? ""), describe()), () => {});
      fetch("/health", { cache: "no-store" }).then((response) => response.json()).then((health) => ((build = typeof health.version === "string" ? health.version : ""), describe()), () => {});
    }

    document.body.append(host);
    return { host, show };
  }

  /** Opens the card: from a press, for iOS to raise the keyboard. */
  function open() {
    made ??= make();
    made.show();
  }

  /** @type {any} */ (window).bitviewFeedback = { open };
  if (/** @type {any} */ (window).bitviewFeedbackAsked) open();
})();
