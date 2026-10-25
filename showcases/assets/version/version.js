// @ts-check
// Every showcase's word that a newer version of it is served, in one place (loaded by `/assets/launch/launch.js`,
// which each page loads first).
//
// - The page's version is its validator, the ETag the server sends with it (a hash of the page: a release server's;
//   a debug one sends none, and nothing's said). The loaded page's is read from the copy the browser keeps of it
//   (a download only if that copy's gone).
// - Looked at as the page is come back to and every half hour while it's seen, online: the browser asked to check
//   its copy (the server says "unchanged", or sends the newer page, the reload's then).
// - Newer, it's said (at the top on a phone, in the bottom corner on a desk), Reload a press away; never reloaded on its own (the page's state is the reader's).
//   Said once a version: dismissed, or reloaded to a page still the old one, it isn't said again; a newer one still is.
(() => {
  const TOLD = `bitview-version-told:${location.pathname}`;
  const url = location.href.split("#")[0];
  const loaded = fetch(url, { cache: "force-cache" }).then((response) => response.headers.get("etag")).catch(() => null);
  /** Shown, not yet dismissed: nothing more's looked at meanwhile. */
  let showing = false;

  async function look() {
    if (showing || document.hidden || !navigator.onLine) return;
    const was = await loaded;
    if (!was) return;
    try {
      const served = (await fetch(url, { cache: "no-cache" })).headers.get("etag");
      if (!served || served === was || served === told() || showing) return;
      showing = true;
      // (Before the page's body, it waits for it: told only once it's shown.)
      if (!document.body) await new Promise((resolve) => document.addEventListener("DOMContentLoaded", resolve, { once: true }));
      say();
      tell(served);
    } catch {}
  }
  const told = () => {
    try {
      return localStorage.getItem(TOLD);
    } catch {
      return null;
    }
  };
  const tell = (/** @type {string} */ etag) => {
    try {
      localStorage.setItem(TOLD, etag);
    } catch {}
  };

  /** A phone's layout: the notice at its top, as a phone's own come, clear of whatever a page keeps at its bottom
   * (Studio's tab bar, its toasts' Undo); on a desk, in the bottom corner, off the middle a page's toasts take and its
   * sidebar's foot. */
  const phone = matchMedia("(max-width: 760px)");

  /** The notice, its look in a shadow root: the page's styles don't reach into it, nor its the page's (its own
   * custom properties none: a page's, of any name, would reach in). */
  function say() {
    const host = document.createElement("bitview-version");
    const root = host.attachShadow({ mode: "open" });
    root.innerHTML = `<style>
      :host { all: initial; position: fixed; right: 20px; bottom: calc(20px + env(safe-area-inset-bottom)); z-index: 2147483000; max-width: calc(100vw - 24px); font-family: Inter, ui-sans-serif, system-ui, -apple-system, sans-serif; font-size: 14px; font-weight: 500; line-height: 1.3; letter-spacing: -0.005em; color-scheme: light dark; -webkit-font-smoothing: antialiased; }
      .box { display: flex; align-items: center; gap: 6px; height: 48px; padding: 0 6px 0 18px; border-radius: 24px; background: light-dark(rgb(255 255 255 / 0.82), rgb(36 36 38 / 0.8)); color: light-dark(#1d1d1f, #f5f5f7); box-shadow: inset 0 0 0 0.5px light-dark(rgb(0 0 0 / 0.12), rgb(255 255 255 / 0.14)), 0 1px 2px rgb(0 0 0 / 0.08), 0 14px 36px -10px rgb(0 0 0 / 0.32); backdrop-filter: blur(20px) saturate(1.8); -webkit-backdrop-filter: blur(20px) saturate(1.8); }
      .dot { flex: none; width: 7px; height: 7px; border-radius: 50%; background: #e65b00; box-shadow: 0 0 0 3px rgb(230 91 0 / 0.18); }
      p { margin: 0 6px 0 4px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
      button { display: grid; place-items: center; flex: none; height: 36px; border: 0; border-radius: 18px; color: inherit; font: inherit; cursor: pointer; transition: background-color 0.15s, scale 0.15s; -webkit-tap-highlight-color: transparent; }
      button:active { scale: 0.96; }
      .reload { padding: 0 16px; background: #e65b00; color: #fff; font-weight: 600; }
      .reload:hover { background: #f06808; }
      .close { width: 36px; padding: 0; background: none; color: light-dark(rgb(0 0 0 / 0.45), rgb(255 255 255 / 0.5)); }
      .close:hover { background: light-dark(rgb(0 0 0 / 0.06), rgb(255 255 255 / 0.1)); }
      svg { width: 12px; height: 12px; }
      button:focus-visible { outline: 2px solid #e65b00; outline-offset: 2px; }
      @media (max-width: 760px) { :host { top: calc(12px + env(safe-area-inset-top)); right: auto; bottom: auto; left: 50%; translate: -50% 0; } }
    </style><div class="box" role="status"><i class="dot" aria-hidden="true"></i><p>A new version is ready</p><button class="reload" type="button">Reload</button><button class="close" type="button" aria-label="Dismiss"><svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg></button></div>`;
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
  }

  document.addEventListener("visibilitychange", look);
  setInterval(look, 30 * 60_000);
})();
