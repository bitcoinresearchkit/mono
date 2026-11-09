// @ts-check
// Every showcase's way to send a note, in one place (loaded by `/assets/launch/launch.js`, which each page loads first).
//
// - A page has `<button type="button" data-feedback>Feedback</button>` where its own quiet links are, in its own look;
//   launch.js answers its press (and Studio's ⌘K) by calling `bitviewFeedback.open()`.
// - One card over the page, its look in a shadow root, light or dark as the page is. At the top of a phone's screen,
//   Send in its top row, so the keyboard comes up under it and never covers Send; in the bottom corner on a desk.
//   Opened from the press itself, or iOS won't raise the keyboard.
// - Nothing leaves until Send: nothing is asked of the inbox's host as a page loads, on Bitview or on a node of someone's
//   own. What goes with the note is written out in the card, as it'll be sent.
(() => {
  const INBOX = "https://feedback.bitview.space/";
  const compact = matchMedia("(max-width: 760px), (max-height: 520px)");

  /** The app's folder alone: a whole path can name an address someone is looking at. */
  const page = `/${location.pathname.split("/")[1] ? `${location.pathname.split("/")[1]}/` : ""}`;
  /** All of where this is, offered to attach (a chart's state is in Studio's), when there's more to it than the folder. */
  const here = () => location.pathname + location.search + location.hash;
  const attachable = () => here().replace(/\/$/, "") !== page.replace(/\/$/, "") && here().length <= 6000;

  const agent = navigator.userAgent;
  const engine = /Firefox\//.test(agent) ? "firefox" : /Chrome\/|Chromium\//.test(agent) ? "chromium" : /Safari\//.test(agent) ? "safari" : "other";
  const browser = () => `${engine}-${compact.matches ? "phone" : "desk"}`;
  const NAMES = { firefox: "Firefox", chromium: "Chromium", safari: "Safari", other: "another browser" };

  /** @type {{ host: HTMLElement, show: () => void } | undefined} */
  let made;

  function make() {
    const host = document.createElement("bitview-feedback");
    const root = host.attachShadow({ mode: "open" });
    root.innerHTML = `<style>
      :host { all: initial; }
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
      @media (max-width: 760px), (max-height: 520px) {
        .card { inset: calc(10px + env(safe-area-inset-top)) 12px auto; width: auto; max-width: 520px; margin-inline: auto; transform: translateY(-10px) scale(0.97); }
        @starting-style { .card:popover-open { transform: translateY(-10px) scale(0.97); } }
        textarea, input[type="text"] { font-size: 16px; }
      }
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
      .card.is-done form { visibility: hidden; }
      .card.is-done .done { visibility: visible; }
    </style><div class="card" popover="auto" role="dialog" aria-labelledby="title">
      <header><button class="close" type="button" aria-label="Close"><svg viewBox="0 0 10 10" aria-hidden="true"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg></button><h2 id="title">Feedback</h2><button class="send" type="submit" form="note" disabled>Send</button></header>
      <form id="note" novalidate>
        <p class="said" id="said" role="alert"></p>
        <textarea id="message" maxlength="4000" placeholder="What's wrong, confusing, or missing?" aria-label="Your feedback"></textarea>
        <input id="contact" type="text" maxlength="200" autocomplete="off" placeholder="Contact, if you'd like a reply (optional)" aria-label="Contact, optional" />
        <label class="attach" hidden><input id="attach" type="checkbox" /><span>Include this page's exact address</span></label>
        <p class="sent-with" id="with"></p>
        <p class="sent-with">Goes to the Bitview project, through Cloudflare, and an AI helps read it. IP addresses aren't stored.</p>
      </form>
      <div class="done" role="status"><svg viewBox="0 0 24 24" fill="none" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/></svg><span id="sent"></span></div>
    </div>`;

    const $ = (/** @type {string} */ selector) => /** @type {any} */ (root.querySelector(selector));
    const card = $(".card"), form = $("form"), message = $("#message"), contact = $("#contact"), attach = $("#attach"), attachRow = $(".attach"), said = $("#said"), send = $(".send");
    /** The page's version as served, read when the card opens and sent if it's come by Send; never waited for. */
    let version = "";
    let sending = false;

    const describe = () => {
      const where = attach.checked ? here() : page;
      const also = $("#with");
      also.textContent = "";
      also.append("Also sent: ", Object.assign(document.createElement("b"), { textContent: where }), `, its version, ${NAMES[/** @type {keyof typeof NAMES} */ (engine)]}, ${compact.matches ? "small screen" : "large screen"}.`);
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
      sending = true;
      ready();
      said.textContent = "";
      try {
        const body = JSON.stringify({ message: message.value, contact: contact.value, page, link: attach.checked ? here() : "", version, browser: browser() });
        const response = await fetch(INBOX, { method: "POST", body, credentials: "omit", referrerPolicy: "no-referrer", keepalive: true, signal: AbortSignal.timeout(15_000) });
        if (!response.ok) throw response.status;
        message.value = contact.value = "";
        attach.checked = false;
        $("#sent").textContent = "Sent. Thank you.";
        card.classList.add("is-done");
      } catch (status) {
        said.textContent = `${status === 503 ? "The inbox is full for now." : status === 429 ? "Too many sent just now." : "Couldn't send it."} Your note is still here; try again later.`;
      }
      sending = false;
      ready();
    });

    /** What had the focus, given it back as the card goes. */
    let opener = /** @type {HTMLElement | null} */ (null);
    card.addEventListener("toggle", (/** @type {ToggleEvent} */ event) => {
      if (event.newState !== "closed") return;
      card.classList.remove("is-done");
      opener?.focus({ preventScroll: true });
    });

    function show() {
      if (card.matches(":popover-open")) return;
      opener = /** @type {HTMLElement | null} */ (document.activeElement);
      attachRow.hidden = !attachable();
      if (attachRow.hidden) attach.checked = false;
      describe();
      said.textContent = "";
      ready();
      card.showPopover();
      message.focus();
      version = "";
      fetch(location.href.split("#")[0], { method: "HEAD", cache: "no-store" }).then((response) => (version = response.headers.get("bitview-version") ?? ""), () => {});
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
