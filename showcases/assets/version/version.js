// @ts-check
// Every showcase's word that a newer version of it is served, in one place (loaded by `/assets/launch/launch.js`,
// which each page loads first).
//
// - The page's version is the `Bitview-Version` header the server sends with it (a hash of the page; a release
//   server's: a debug one sends none, and nothing's said). Not its ETag: a CDN that rewrites pages drops that.
// - Read as the page loads, then again as it's come back to and every half hour while it's seen, online: a HEAD
//   request, its headers alone (a few hundred bytes).
// - Newer, it's said (at the top on a phone, in the bottom corner on a desk), Reload a press away; never reloaded on
//   its own (the page's state is the reader's). Said once a version: dismissed, or reloaded to a page still the old
//   one, it isn't said again; a newer one still is.
(() => {
  const TOLD = `bitview-version-told:${location.pathname}`;
  const url = location.href.split("#")[0];
  const served = () => fetch(url, { method: "HEAD", cache: "no-store" }).then((response) => response.headers.get("bitview-version"));
  /** The page's version as loaded: read as it loads (a deploy in the moment between the page and this read is
   * missed), else on the next look (the read failed, or a CDN's copy from before the header). */
  let was = served().catch(() => null);
  /** Shown, not yet dismissed: nothing more's looked at meanwhile. */
  let showing = false;

  async function look() {
    if (showing || document.hidden || !navigator.onLine) return;
    if (!(await was)) {
      was = served().catch(() => null);
      return;
    }
    try {
      const now = await served();
      if (!now || now === (await was) || now === told() || showing) return;
      showing = true;
      // (Before the page's body, it waits for it: told only once it's shown.)
      if (!document.body) await new Promise((resolve) => document.addEventListener("DOMContentLoaded", resolve, { once: true }));
      say();
      tell(now);
    } catch {}
  }
  const told = () => {
    try {
      return localStorage.getItem(TOLD);
    } catch {
      return null;
    }
  };
  const tell = (/** @type {string} */ version) => {
    try {
      localStorage.setItem(TOLD, version);
    } catch {}
  };

  /** A phone's layout: the notice at its top, as a phone's own come, clear of whatever a page keeps at its bottom
   * (Studio's tab bar, its toasts' Undo); on a desk, in the bottom corner, off the middle a page's toasts take and its
   * sidebar's foot. */
  const phone = matchMedia("(max-width: 760px)");

  /** The notice, its look in a shadow root: the page's styles don't reach into it, nor its the page's. Its colours
   * are custom properties of names no page has (custom properties, as all inherited ones, do reach in); light or
   * dark as the page is (its colour scheme, inherited: Studio dark on a phone set light stays dark). */
  function say() {
    const host = document.createElement("bitview-version");
    const root = host.attachShadow({ mode: "open" });
    root.innerHTML = `<style>
      :host {
        --bitview-version-orange: #c24e00;
        --bitview-version-orange-hover: #a94400;
        --bitview-version-surface: light-dark(rgb(255 255 255 / 0.92), rgb(32 32 34 / 0.9));
        --bitview-version-ink: light-dark(#1d1d1f, #f5f5f7);
        --bitview-version-soft: light-dark(rgb(0 0 0 / 0.62), rgb(255 255 255 / 0.64));
        --bitview-version-faint: light-dark(rgb(0 0 0 / 0.55), rgb(255 255 255 / 0.6));
        --bitview-version-hairline: light-dark(rgb(0 0 0 / 0.1), rgb(255 255 255 / 0.12));
        --bitview-version-press: light-dark(rgb(0 0 0 / 0.06), rgb(255 255 255 / 0.1));
        --bitview-version-tint: light-dark(rgb(230 91 0 / 0.1), rgb(230 91 0 / 0.16));
        all: initial; position: fixed; right: 20px; bottom: calc(20px + env(safe-area-inset-bottom)); z-index: 2147483000; max-width: calc(100vw - 24px); font-family: Inter, ui-sans-serif, system-ui, -apple-system, sans-serif; color-scheme: inherit; -webkit-font-smoothing: antialiased;
      }
      .box { display: flex; align-items: center; gap: 10px; padding: 8px; border-radius: 14px; background: var(--bitview-version-surface); color: var(--bitview-version-ink); box-shadow: inset 0 0 0 0.5px var(--bitview-version-hairline), 0 1px 2px rgb(0 0 0 / 0.06), 0 20px 44px -16px rgb(0 0 0 / 0.38); backdrop-filter: blur(24px) saturate(1.8); -webkit-backdrop-filter: blur(24px) saturate(1.8); }
      .mark { display: grid; place-items: center; flex: none; width: 32px; height: 32px; border-radius: 9px; background: var(--bitview-version-tint); }
      .mark img { width: 16px; height: 19px; }
      .text { flex: 1; min-width: 0; margin-right: 2px; }
      .text b, .text span { display: block; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
      .text b { font-size: 14px; font-weight: 600; line-height: 1.25; letter-spacing: -0.01em; }
      .text span { margin-top: 1px; color: var(--bitview-version-soft); font-size: 12.5px; font-weight: 450; line-height: 1.3; }
      button { display: grid; place-items: center; flex: none; border: 0; color: inherit; font: inherit; cursor: pointer; transition: background-color 0.15s, scale 0.15s; -webkit-tap-highlight-color: transparent; }
      button:active { scale: 0.96; }
      .reload { height: 30px; padding: 0 12px; border-radius: 8px; background: var(--bitview-version-orange); color: #fff; font-size: 13px; font-weight: 600; letter-spacing: -0.005em; box-shadow: inset 0 0.5px 0 rgb(255 255 255 / 0.2), 0 1px 2px rgb(194 78 0 / 0.35); }
      .reload:hover { background: var(--bitview-version-orange-hover); }
      .close { width: 26px; height: 26px; border-radius: 7px; background: none; color: var(--bitview-version-faint); }
      .close:hover { background: var(--bitview-version-press); color: var(--bitview-version-ink); }
      svg { width: 10px; height: 10px; }
      button:focus-visible { outline: 2px solid var(--bitview-version-orange); outline-offset: 2px; }
      @media (max-width: 760px) { :host { top: calc(10px + env(safe-area-inset-top)); right: 12px; bottom: auto; left: 12px; max-width: none; } }
    </style><div class="box" role="status"><span class="mark" aria-hidden="true"><img src="/assets/logo/logo.svg" alt="" /></span><span class="text"><b></b><span></span></span><button class="reload" type="button">Reload</button><button class="close" type="button" aria-label="Dismiss"><svg viewBox="0 0 10 10" aria-hidden="true"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg></button></div>`;
    const box = /** @type {HTMLElement} */ (root.querySelector(".box"));
    const still = matchMedia("(prefers-reduced-motion: reduce)").matches, edge = () => (phone.matches ? -1 : 1);
    // In from its edge with a little spring, back to the edge it's at as it goes: a fade alone for reduced motion.
    box.animate(still ? [{ opacity: 0 }, { opacity: 1 }] : [{ opacity: 0, transform: `translateY(${14 * edge()}px) scale(0.94)` }, { opacity: 1, transform: "none" }], { duration: still ? 200 : 420, easing: "cubic-bezier(0.32, 1.25, 0.5, 1)" });
    root.querySelector(".reload")?.addEventListener("click", () => location.reload());
    root.querySelector(".close")?.addEventListener("click", () => {
      const out = box.animate(still ? [{ opacity: 1 }, { opacity: 0 }] : [{ opacity: 1, transform: "none" }, { opacity: 0, transform: `translateY(${10 * edge()}px) scale(0.97)` }], { duration: 180, easing: "cubic-bezier(0.4, 0, 1, 1)", fill: "forwards" });
      const gone = () => (host.remove(), (showing = false));
      out.finished.then(gone, gone);
    });
    document.body.append(host);
    // (Written once it's in: a status already full as it's added may go unannounced.)
    setTimeout(() => {
      const [title, line] = root.querySelectorAll(".text > *");
      title.textContent = "New version available";
      line.textContent = "Reload to update";
    });
  }

  document.addEventListener("visibilitychange", look);
  setInterval(look, 30 * 60_000);
})();
