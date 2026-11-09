// @ts-check
// Every showcase's way to report a bug, in one place (loaded by `/assets/launch/launch.js`, which each page loads first).
//
// - A page has `<button type="button" data-feedback aria-label="Report a bug" title="Report a bug"></button>` in its own
//   bar; launch.js draws the icon and answers its press (and Studio's ⌘K) by calling `bitviewFeedback.open()`.
// - A press opens a text box, and that's all there is: type, send. At the top of a phone's screen, so the keyboard comes
//   up under it; in the bottom corner on a desk. Opened from the press itself, or iOS won't raise the keyboard.
// - Nothing leaves until Send: nothing is asked of the inbox's host as a page loads, on Bitview or on a node of someone's
//   own. What goes with the note is what makes it reproducible, said in one line: the exact address with what can name
//   someone masked (a page whose address holds their own figures sends its folder alone), the build, browser, screen and
//   theme, and the page's recent errors. Nothing that identifies a person.
(() => {
  const INBOX = "https://feedback.bitview.space/";
  const PRIVACY = "https://github.com/bitcoinresearchkit/mono/tree/next/packages/bitview_feedback#readme";
  const INCLUDED = "Includes this page, browser and recent errors.";
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
    [/(?:https?|wss?):\/\/[^\s/"']+/g, ""],
    [new RegExp(`${START}[0-9a-fA-F]{1,4}(?::[0-9a-fA-F]{0,4}){3,7}${END}`, "g"), ":ip"],
    [new RegExp(`${START}\\d{1,3}(?:\\.\\d{1,3}){3}${END}`, "g"), ":ip"],
  ]);
  const mask = (/** @type {string} */ text) => MASKS.reduce((masked, [shape, name]) => masked.replace(shape, name), location.hostname ? text.replaceAll(location.hostname, ":host") : text);

  /** A page whose address holds someone's own figures says so. */
  const addressPrivate = () => Boolean(document.querySelector('meta[name="bitview-feedback"][content="address-private"]'));
  /** Where this is, masked, or nothing when that's only the folder, or a page whose address holds someone's own figures.
   * A page whose state isn't in its address (a Studio chart) says what address would open it. */
  function link() {
    if (addressPrivate()) return "";
    const own = /** @type {any} */ (window).bitviewFeedbackLink?.();
    const place = own ? new URL(own, location.href) : location;
    const exact = mask(place.pathname + place.search + place.hash);
    return exact.replace(/\/$/, "") === page.replace(/\/$/, "") || exact.length > 6000 ? "" : exact;
  }

  const agent = navigator.userAgent;
  const engine = /Firefox\//.test(agent) ? "firefox" : /Chrome\/|Chromium\//.test(agent) ? "chromium" : /Safari\//.test(agent) ? "safari" : "other";
  /** Its major version alone. */
  const major = /(?:Firefox|Chrome|Chromium|Version)\/(\d+)/.exec(agent)?.[1] ?? "";
  const os = /iPhone|iPad|iPod/.test(agent) || (/Macintosh/.test(agent) && navigator.maxTouchPoints > 1) ? "ios" : /Android/.test(agent) ? "android" : /Macintosh/.test(agent) ? "macos" : /Windows/.test(agent) ? "windows" : /Linux|X11/.test(agent) ? "linux" : "other";

  /** How to see what the sender saw, and nothing of who they are: sizes to the nearest 10 px, as a window's exact size is
   * close to a fingerprint and no layout turns on the last digit. @param {HTMLElement} card */
  const context = (card) => ({
    os, major,
    viewport: `${Math.round(innerWidth / 10) * 10}x${Math.round(innerHeight / 10) * 10}`,
    scale: String(Math.round(devicePixelRatio * 2) / 2),
    // (What the page is really showing, whichever way it decided: the card's own background, light or dark as it is.)
    theme: Number(/\d+/.exec(getComputedStyle(card).backgroundColor)?.[0]) > 127 ? "light" : "dark",
    mode: /** @type {any} */ (navigator).standalone === true || matchMedia("(display-mode: standalone)").matches ? "app" : "browser",
    // (Masked before it's cut, or a token cut in half would slip past; none from a page whose address holds someone's own figures.)
    errors: addressPrivate() ? [] : /** @type {string[]} */ (/** @type {any} */ (window).bitviewFeedbackErrors ?? []).map((text) => mask(text).slice(0, 300)),
  });

  /** @type {{ show: () => void } | undefined} */
  let made;

  function make() {
    const host = document.createElement("bitview-feedback");
    const root = host.attachShadow({ mode: "open" });
    root.innerHTML = `<style>
      :host { all: initial; display: contents; color-scheme: inherit; }
      .card {
        --orange: #c24e00;
        --orange-hover: #a94400;
        --surface: light-dark(#fff, #1f1f21);
        --ink: light-dark(#1d1d1f, #f5f5f7);
        --soft: light-dark(rgb(0 0 0 / 0.62), rgb(255 255 255 / 0.64));
        --faint: light-dark(rgb(0 0 0 / 0.55), rgb(255 255 255 / 0.6));
        --hairline: light-dark(rgb(0 0 0 / 0.14), rgb(255 255 255 / 0.16));
        position: fixed; inset: auto 20px calc(20px + env(safe-area-inset-bottom)) auto; width: 380px; max-width: calc(100vw - 24px); max-height: calc(100dvh - 24px); margin: 0; padding: 0; overflow: auto; box-sizing: border-box; border: 0; border-radius: 18px;
        background: var(--surface); color: var(--ink); color-scheme: inherit; font: 450 14px/1.4 Inter, ui-sans-serif, system-ui, -apple-system, sans-serif; -webkit-font-smoothing: antialiased;
        box-shadow: inset 0 0 0 0.5px var(--hairline), 0 1px 2px rgb(0 0 0 / 0.06), 0 22px 48px -16px rgb(0 0 0 / 0.42);
        opacity: 0; transform: translateY(10px) scale(0.97); transition: opacity 0.18s, transform 0.22s cubic-bezier(0.32, 1.2, 0.5, 1), display 0.22s allow-discrete, overlay 0.22s allow-discrete;
      }
      .card:popover-open { opacity: 1; transform: none; }
      @starting-style { .card:popover-open { opacity: 0; transform: translateY(10px) scale(0.97); } }
      @media (prefers-reduced-motion: reduce) { .card { transition: opacity 0.15s, display 0.15s allow-discrete, overlay 0.15s allow-discrete; transform: none; } @starting-style { .card:popover-open { transform: none; } } }
      .composer { position: relative; }
      textarea { display: block; width: 100%; min-height: 120px; box-sizing: border-box; padding: 14px 14px 52px; border: 0; background: none; color: inherit; font: inherit; resize: none; outline: none; }
      ::placeholder { color: var(--faint); }
      button { position: absolute; right: 8px; bottom: 8px; display: grid; place-items: center; width: 36px; height: 36px; border: 0; border-radius: 50%; background: var(--orange); color: #fff; cursor: pointer; -webkit-tap-highlight-color: transparent; transition: background-color 0.15s, scale 0.15s, opacity 0.15s; }
      button::after { content: ""; position: absolute; inset: -4px; }
      button:hover:not(:disabled) { background: var(--orange-hover); }
      button:active:not(:disabled) { scale: 0.94; }
      button:disabled { opacity: 0.35; cursor: default; }
      button:focus-visible { outline: 2px solid var(--orange); outline-offset: 2px; }
      svg { width: 16px; height: 16px; }
      .line { margin: 0; padding: 7px 14px 10px; border-top: 0.5px solid var(--hairline); color: var(--faint); font-size: 12px; line-height: 1.35; }
      .line a { color: var(--soft); }
      .line.failed span { color: light-dark(#a8201a, #ff8a80); }
      .done { position: absolute; inset: 0; display: grid; place-content: center; justify-items: center; gap: 6px; font-size: 15px; font-weight: 600; visibility: hidden; pointer-events: none; }
      .done svg { width: 28px; height: 28px; color: var(--orange); }
      .card.is-done form { visibility: hidden; }
      .card.is-done .done { visibility: visible; }
      @media (max-width: 760px), (max-height: 520px) {
        .card { inset: calc(10px + env(safe-area-inset-top)) 12px auto; width: auto; max-width: 520px; margin-inline: auto; transform: translateY(-10px) scale(0.97); }
        @starting-style { .card:popover-open { transform: translateY(-10px) scale(0.97); } }
        textarea { font-size: 16px; }
      }
    </style><div class="card" popover="auto" role="dialog" aria-label="Report a bug">
      <form novalidate>
        <div class="composer">
          <textarea maxlength="4000" placeholder="What's wrong or missing?" aria-label="What's wrong or missing"></textarea>
          <button type="submit" aria-label="Send" disabled><svg viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M8 13V3.5M3.75 7.5 8 3.25l4.25 4.25" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"/></svg></button>
        </div>
        <p class="line"><span aria-live="polite">${INCLUDED}</span> <a href="${PRIVACY}" target="_blank" rel="noopener noreferrer">Privacy</a></p>
      </form>
      <div class="done" role="status"><svg viewBox="0 0 24 24" fill="none" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg><span id="sent"></span></div>
    </div>`;

    const $ = (/** @type {string} */ selector) => /** @type {any} */ (root.querySelector(selector));
    const card = $(".card"), form = $("form"), message = $("textarea"), send = $("button"), line = $(".line"), said = $(".line span");
    /** The page's version (as it loaded) and the server's build, read as the card opens and sent if they've come by Send;
     * never waited for. */
    let version = "", build = "";
    let sending = false;
    /** @type {ReturnType<typeof setTimeout> | undefined} */
    let closing;

    const ready = () => { send.disabled = sending || !message.value.trim(); };
    const say = (/** @type {string} */ text, failed = false) => { said.textContent = text; line.classList.toggle("failed", failed); };

    message.addEventListener("input", () => { if (line.classList.contains("failed")) say(INCLUDED); ready(); });
    /** Enter sends where there's a real keyboard, as in a chat; a phone's Enter is a new line and its arrow sends. */
    const fine = matchMedia("(hover: hover) and (pointer: fine)");
    message.addEventListener("keydown", (/** @type {KeyboardEvent} */ event) => {
      if (event.key === "Enter" && !event.shiftKey && !event.isComposing && fine.matches) (event.preventDefault(), form.requestSubmit());
    });

    form.addEventListener("submit", async (/** @type {SubmitEvent} */ event) => {
      event.preventDefault();
      if (send.disabled) return;
      sending = message.readOnly = true;
      ready();
      try {
        const body = JSON.stringify({ message: message.value, page, link: link(), version, browser: `${engine}-${compact.matches ? "phone" : "desk"}`, context: { build, ...context(card) } });
        const response = await fetch(INBOX, { method: "POST", body, credentials: "omit", referrerPolicy: "no-referrer", keepalive: true, signal: AbortSignal.timeout(15_000) });
        if (!response.ok) throw response.status;
        message.value = "";
        card.classList.add("is-done");
        // (Written once it's shown: a status already full as it's shown may go unannounced.)
        setTimeout(() => ($("#sent").textContent = "Sent. Thank you."));
        closing = setTimeout(() => card.hidePopover(), 2000);
      } catch (status) {
        say(status === 400 || status === 413 ? "This note can't be sent as it is." : status === 503 ? "The inbox is full for now. Your note is still here." : status === 429 ? "Too many just now. Your note is still here." : "Couldn't send. Your note is still here.", true);
      }
      sending = message.readOnly = false;
      ready();
    });

    /** What had the focus, given back as the card goes, unless a tap elsewhere has already put it somewhere. */
    let opener = /** @type {HTMLElement | null} */ (null);
    card.addEventListener("toggle", (/** @type {ToggleEvent} */ event) => {
      if (event.newState !== "closed") return;
      clearTimeout(closing);
      card.classList.remove("is-done");
      $("#sent").textContent = "";
      if (document.activeElement === document.body || document.activeElement === host) opener?.focus({ preventScroll: true });
    });

    function show() {
      if (card.matches(":popover-open")) return;
      opener = /** @type {HTMLElement | null} */ (document.activeElement);
      say(INCLUDED);
      ready();
      card.showPopover();
      message.focus();
      version = build = "";
      Promise.resolve(/** @type {any} */ (window).bitviewPageVersion?.()).then((said) => (version = said ?? ""), () => {});
      fetch("/health", { cache: "no-store" }).then((response) => response.json()).then((health) => (build = typeof health.version === "string" ? health.version : ""), () => {});
    }

    document.body.append(host);
    return { show };
  }

  /** Opens the card: from a press, for iOS to raise the keyboard. */
  function open() {
    made ??= make();
    made.show();
  }

  /** @type {any} */ (window).bitviewFeedback = { open };
  if (/** @type {any} */ (window).bitviewFeedbackAsked) open();
})();
