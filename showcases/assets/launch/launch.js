// @ts-check
// Every showcase's launch from a phone's home screen, in one place: each page loads this first in its head
// (`<script src="/assets/launch/launch.js"></script>`, `data-wait` for a page that says itself when it's ready: see
// `bitviewLaunch.ready`).
//
// - Each showcase is an app of its own, the one added: its folder its start and its scope, named by the page
//   (`data-app="Studio"`: "Bitview Studio", "Studio" under its icon; `data-color`, its design's own colour, Bitview's
//   stamped cube in it); the front page, at "/", is Bitview, in its orange. Its manifest made and its icon linked here,
//   for all of them; iOS takes them and its launch screen from the page it's added from, at that moment.
// - In Safari, where a page is added to a home screen: its launch screen, Bitview's orange at the phone's size (iOS's
//   white else while the app starts), made as soon as the page starts, not once it has loaded: added to a home screen
//   before then, an app would launch white for good.
// - Launched from there (not a page it goes to after, nor a reload): the same orange over the page from its first frame,
//   that frame the orange alone (for iOS to hand over to it without a flash of white). Once the page is ready and on
//   the screen, the orange is Bitview's cube seen up close, which draws back as a camera would (evenly to the eye,
//   eased in and out) and fades over its second half, never coming to rest, the page sharpening behind it (a layer
//   blurring what's behind it: iOS's own blur, on its GPU; the page itself never blurred or drawn again). The status
//   bar orange with it, the page's own after. Then nothing of it is left. Reduced motion: gone at once.
// - Never ready (its script held back), the orange fades after a while all the same, the page's word of it shown.
(() => {
  const ORANGE = "#e65b00";
  const root = document.documentElement, script = /** @type {HTMLScriptElement | null} */ (document.currentScript);
  const standalone = /** @type {any} */ (navigator).standalone;
  // (A newer version of the page, said: its own file, every showcase's from here.)
  document.head.append(Object.assign(document.createElement("script"), { src: "/assets/version/version.js" }));
  // (A way to report a bug: its own file. A page's button is a bare icon, drawn from before the page is: nothing moves.)
  document.head.append(Object.assign(document.createElement("script"), { src: "/assets/feedback/feedback.js", fetchPriority: "low" }));
  document.head.append(Object.assign(document.createElement("style"), { textContent: `:where(button[data-feedback]) { position: relative; display: inline-grid; place-items: center; appearance: none; margin: 0; padding: 0; border: 0; background: none; color: inherit; opacity: 0.6; cursor: pointer; &:hover { opacity: 1; } &::before { content: ""; width: 16px; height: 16px; background: currentColor; mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none' stroke='%23000' stroke-width='1.4' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M5.6 5.9a2.4 2.4 0 0 1 4.8 0'/%3E%3Crect x='4.6' y='5.9' width='6.8' height='8.1' rx='3.4'/%3E%3Cpath d='M8 8.6V14M4.6 9.4H2.2M11.4 9.4h2.4M5 6.7 3.2 5.1M11 6.7l1.8-1.6M5 12.6l-1.8 1.5M11 12.6l1.8 1.5M6.3 2.2l.7 1.4M9.7 2.2 9 3.6'/%3E%3C/svg%3E") center / contain no-repeat; } &::after { content: ""; position: absolute; inset: min(0px, calc((100% - 44px) / 2)); } }` }));
  // (The page's recent errors, for a note to carry if one is ever sent: in memory, this page's, the last five, from its
  // very start, which is why here. Failed files too: a script that didn't load explains a lot.)
  const errors = /** @type {string[]} */ (/** @type {any} */ (window).bitviewFeedbackErrors = []);
  const noted = (/** @type {string} */ text) => { if (text && errors.at(-1) !== text && (errors.push(text.slice(0, 1000)), errors.length > 5)) errors.shift(); };
  const place = (/** @type {string} */ address) => { try { const url = new URL(address, location.href); return url.origin === location.origin ? url.pathname : "another site"; } catch { return "somewhere"; } };
  addEventListener("error", (event) => noted(event.message ? `${event.message} (${place(event.filename)}:${event.lineno})` : `Couldn't load ${place(/** @type {any} */ (event.target)?.src ?? /** @type {any} */ (event.target)?.href ?? "")}`), true);
  addEventListener("unhandledrejection", (event) => noted(String(event.reason?.message ?? event.reason)));
  // (Answered here, so a press before that file has come isn't lost: it opens as soon as it has.)
  document.addEventListener("click", (event) => {
    if (!/** @type {Element} */ (event.target).closest?.("[data-feedback]")) return;
    const feedback = /** @type {any} */ (window).bitviewFeedback;
    if (feedback) feedback.open();
    else /** @type {any} */ (window).bitviewFeedbackAsked = true;
  });
  // (Its app: the showcase's folder, wherever in it the page is (a Studio link's chart, a preset's address).)
  const app = script?.dataset.app, name = app ? `Bitview ${app}` : "Bitview", short = app ?? name;
  const folder = app ? location.pathname.replace(/^(\/[^/]+).*$/, "$1/") : "/", start = `${location.origin}${folder}`;
  for (const [tag, attributes] of /** @type {const} */ ([
    ["meta", { name: "application-name", content: name }],
    ["meta", { name: "apple-mobile-web-app-title", content: short }],
    // (iOS uses a launch screen only for a page that says it's an app.)
    ["meta", { name: "apple-mobile-web-app-capable", content: "yes" }],
  ])) document.head.append(Object.assign(document.createElement(tag), attributes));
  // Its icon: Bitview's cube in the app's colour, a file (assets/favicon/apps/, each made from app-icon.svg in its page's
  // `data-color` by showcases/home.mjs --icons): one drawn here never came on a phone in Lockdown Mode, its home screen
  // left with the manifest's orange alone. The manifest made once the page is read, its description the page's own.
  const icon = `${location.origin}/assets/favicon/apps/${app ? folder.slice(1, -1) : "bitview"}.png`;
  const link = (/** @type {string} */ rel, /** @type {string} */ href) => document.head.append(Object.assign(document.createElement("link"), { rel, href }));
  link("apple-touch-icon", icon);
  document.addEventListener("DOMContentLoaded", () => {
    const description = /** @type {HTMLMetaElement | null} */ (document.querySelector('meta[name="description"]'))?.content;
    const manifest = {
      id: start, name, short_name: short, description, start_url: start, scope: start, display: "standalone",
      theme_color: "#08090a", background_color: ORANGE, icons: [{ src: icon, sizes: "512x512", type: "image/png" }],
    };
    link("manifest", `data:application/manifest+json,${encodeURIComponent(JSON.stringify(manifest))}`);
  });
  if (standalone === false) {
    setTimeout(() => {
      const canvas = document.createElement("canvas"), context = canvas.getContext("2d"), link = document.createElement("link");
      canvas.width = Math.round(screen.width * devicePixelRatio);
      canvas.height = Math.round(screen.height * devicePixelRatio);
      if (!context) return;
      context.fillStyle = ORANGE;
      context.fillRect(0, 0, canvas.width, canvas.height);
      link.rel = "apple-touch-startup-image";
      link.href = canvas.toDataURL("image/png");
      document.head.append(link);
    });
  }
  /** Said by a page that waits (`data-wait`): it's ready. Else as it loads. */
  const api = { ready: () => {} };
  /** @type {any} */ (window).bitviewLaunch = api;
  if (!standalone) return;
  // Installed, its status bar solid in the page's own color, never blurred: iOS (26 on) colors it from the first fixed
  // box at the screen's top edge, and finding the launch's full-screen blur there (its closing step) kept a blur of the
  // page under the status bar up to the first reload. That box here, a few pixels tall, over everything (the launch's
  // layers too): the launch's orange while it runs, then the page's theme-color for its color scheme (else its
  // ground), followed as the page changes it (Studio's theme).
  // (iOS reads its color as the box comes: a new one for each color, the old one's kept else (tried on an iPhone: a
  // color changed in place left the launch's orange there).)
  let status = document.createElement("div"), striped = "";
  const strip = (/** @type {string} */ color) => {
    if (color === striped) return;
    striped = color;
    const next = document.createElement("div");
    next.style.cssText = "position: fixed; inset: 0 0 auto; z-index: 2147483647; height: 8px; pointer-events: none";
    next.style.background = color;
    status.isConnected ? status.replaceWith(next) : root.append(next);
    status = next;
  };
  let launching = false;
  const paint = () => {
    if (launching) return;
    // (The page's own ground, the body's, else the root's: what's under the bar, exactly (a theme-color can drift from
    // it: Calendar's grey over its beige); none (a page all canvas), its theme-color, else black: a strip see-through
    // would leave iOS its blur.)
    const meta = /** @type {HTMLMetaElement[]} */ ([...document.querySelectorAll('meta[name="theme-color"]')]).find((tag) => !tag.media || matchMedia(tag.media).matches);
    const ground = [document.body, root].map((element) => (element ? getComputedStyle(element).backgroundColor : "")).find((color) => color && color !== "transparent" && !/rgba\(.*,\s*0\)$/.test(color));
    const color = ground || meta?.content || "#000";
    if (color) strip(color);
    // (Its edge in the same color: after this script's run, once the edge is made.)
    queueMicrotask(() => edge.isConnected && edged());
  };
  paint();
  // Under it, what scrolls up to it dissolves, not cut: a soft edge 28px tall, the page blurred more and more toward the
  // top (four layers) and its color over it, as iOS's own bars end. In by how far the page has scrolled (none at its
  // top, where nothing's cut: over a header's words none), whole by 28px; none on a page that doesn't scroll (an app's
  // own frame: Studio, Trail), nor while the page's own bar (fixed or sticky) is up there, its edge its own.
  const edge = document.createElement("div");
  edge.setAttribute("aria-hidden", "true");
  edge.style.cssText = "position: fixed; inset: 0 0 auto; z-index: 2147483000; height: 28px; pointer-events: none";
  edge.innerHTML = [1, 2, 4, 8].map((px, k) => `<i style="position: absolute; inset: 0; backdrop-filter: blur(${px}px); -webkit-backdrop-filter: blur(${px}px); mask-image: linear-gradient(#000 ${k * 18}%, transparent ${40 + k * 20}%); -webkit-mask-image: linear-gradient(#000 ${k * 18}%, transparent ${40 + k * 20}%); opacity: 0"></i>`).join("") + '<i class="tint" style="position: absolute; inset: 0; opacity: 0"></i>';
  const tint = /** @type {HTMLElement} */ (edge.lastElementChild);
  const edged = () => {
    tint.style.background = `linear-gradient(${striped}, color-mix(in srgb, ${striped} 70%, transparent) 30%, color-mix(in srgb, ${striped} 25%, transparent) 65%, transparent)`;
    // (The page's own bar at the top: its edge is its own.)
    let own = false;
    for (let element = document.elementFromPoint(innerWidth / 2, 12); element && element !== document.body && element !== root; element = element.parentElement) {
      const position = getComputedStyle(element).position;
      if (position === "fixed" || position === "sticky") (own = element !== status && element !== edge), (element = null);
      if (!element) break;
    }
    const shown = own || document.documentElement.scrollHeight <= innerHeight + 1 ? 0 : Math.min(1, Math.max(0, scrollY / 28));
    // (On each layer: an opacity on the whole would cut them off from the page behind, nothing to blur till it's 1.)
    const opacity = String(Math.round(shown * 100) / 100);
    for (const layer of /** @type {HTMLElement[]} */ ([...edge.children])) if (layer.style.opacity !== opacity) layer.style.opacity = opacity;
  };
  let edging = 0;
  addEventListener("scroll", () => (edging ||= requestAnimationFrame(() => ((edging = 0), edged()))), { passive: true });
  addEventListener("load", () => (root.append(edge), edged()), { once: true });
  addEventListener("DOMContentLoaded", paint, { once: true });
  addEventListener("load", paint, { once: true });
  new MutationObserver(paint).observe(document.head ?? root, { subtree: true, childList: true, attributes: true, attributeFilter: ["content", "media"] });
  matchMedia("(prefers-color-scheme: dark)").addEventListener("change", paint);
  // The app's launch only, not each page it goes to after: once a session, as its sessionStorage says.
  try {
    if (sessionStorage.getItem("bitview-launched") !== null) return;
    sessionStorage.setItem("bitview-launched", "");
  } catch {}

  root.dataset.launch = "";
  const top = 2147483600;
  // (The root orange too, under the cover until the page shows: on a phone (Lockdown's, slow), iOS starts an app drawn
  // under its status bar with the page a status bar's height short of the screen, and grows it a moment later; the
  // cover fills the page alone, and the strip left at the foot showed the page's own ground, dark.)
  const style = document.createElement("style");
  style.textContent = `
    html[data-launch]::after { content: ""; position: fixed; inset: 0; z-index: ${top}; background: ${ORANGE}; pointer-events: none; animation: bitview-launch-out 450ms ease 6s forwards; }
    html[data-launch=""], html[data-launch="page"] { background: ${ORANGE}; }
    @layer bitview-launch {
      html[data-launch=""] body, html[data-launch=""] body * { visibility: hidden !important; }
    }
    html[data-launch="out"]::after { inset: auto; top: 50%; left: 50%; width: 400px; aspect-ratio: 1.26 / 1.4622; translate: -50% -50%; transform-origin: 50% 60.26%; background: var(--bitview-launch-cube) center / 100% 100% no-repeat; animation: none; }
    html[data-launch="out"]::before { content: ""; position: fixed; inset: 0; z-index: ${top - 1}; pointer-events: none; }
    @keyframes bitview-launch-out { to { opacity: 0; } }`;
  document.head.append(style);
  // (The status bar the launch's orange meanwhile: the strip's, see status.)
  launching = true;
  strip(ORANGE);
  const pageBar = () => ((launching = false), paint());
  const gone = () => (delete root.dataset.launch, pageBar());
  // (Faded by itself, the page never ready: the root its own as the fade starts (the page's size long settled), gone at
  // its end.)
  root.addEventListener("animationstart", (event) => event.target === root && event.pseudoElement === "::after" && (root.dataset.launch = "fade"));
  root.addEventListener("animationend", (event) => event.target === root && event.pseudoElement === "::after" && gone());
  // The page drawn only once a frame of the orange alone is on the screen. iOS fades its launch screen out at the page's
  // first frame, and a page drawn into that frame makes it reach the screen late, the blank web view showing through
  // white meanwhile (the home page, in the iOS 27 Simulator: 6 frames, to half white; with it hidden, none, and no hold
  // before that frame helps further). Hidden, it's laid out all the same, for its scripts to measure; hidden from a
  // layer, whose !important outranks any page's own visibility, however specific (Trail's scene, Studio's touch
  // counts). Left on a phone: 2-4 frames to ~10%, the blank web view fading in before even that first frame arrives,
  // with an orange page background, theme-color or manifest background alike.
  const shown = new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => {
    if (root.dataset.launch === "") root.dataset.launch = "page";
    resolve(undefined);
  })));
  // The cube, the logo's, in the launch's orange, decoded before it's shown (shown undecoded, a frame of nothing).
  const cube = fetch("/assets/logo/logo.svg").then((response) => response.text()).then(async (text) => {
    const url = `url("data:image/svg+xml,${encodeURIComponent(text.replace(/fill="[^"]*"/, `fill="${ORANGE}"`))}")`, image = new Image();
    image.src = url.slice(5, -2);
    await image.decode();
    return url;
  });
  let started = false;
  api.ready = () => {
    if (started || !("launch" in root.dataset)) return;
    started = true;
    if (matchMedia("(prefers-reduced-motion: reduce)").matches) return gone();
    // (Two frames on: iOS shows a page's first frame late, a motion begun at once would be most of the way through
    // unseen.)
    Promise.all([cube, shown]).then(([url]) => requestAnimationFrame(() => requestAnimationFrame(() => {
      if (!("launch" in root.dataset)) return;
      root.style.setProperty("--bitview-launch-cube", url);
      root.dataset.launch = "out";
      // Scales of its 400px box (the logo's 92px at 1): at first, close enough for the screen to be inside its solid
      // front (half the screen's height within a fifth of its unit, half its width within half), whatever its shape;
      // at last, a little over the logo's size.
      const unit = 92 / 1.26, first = (1.05 * Math.max(innerHeight / 2 / 0.2, innerWidth / 2 / 0.5)) / unit, last = 2.4, box = 400 / 92, steps = 24;
      const frames = Array.from({ length: steps + 1 }, (_, k) => ({ transform: `scale(${(first ** (1 - k / steps) * last ** (k / steps)) / box})` }));
      const timing = { pseudoElement: "::after", duration: 900, fill: /** @type {const} */ ("forwards") };
      const drawn = root.animate(frames, { ...timing, easing: "cubic-bezier(.45, 0, .2, 1)" });
      root.animate([{ opacity: 1, offset: 0.5 }, { opacity: 0 }], { ...timing, easing: "ease-in-out" });
      const blur = (/** @type {number} */ px, /** @type {number} */ offset) => ({ backdropFilter: `blur(${px}px)`, webkitBackdropFilter: `blur(${px}px)`, offset });
      root.animate([blur(24, 0), blur(24, 0.2), blur(0, 1)], { ...timing, pseudoElement: "::before", easing: "cubic-bezier(.3, 0, .2, 1)" });
      drawn.finished.then(gone, gone);
      // The status bar the page's own once the page shows at the top edge's middle, where iOS reads it: the cube's top
      // face come down past it (its unit's 0.08 above the point it's drawn back to, 0.15 under its middle), read as it
      // goes.
      const perUnit = 400 / 1.26;
      const follow = () => {
        if (!("launch" in root.dataset)) return;
        const scale = new DOMMatrix(getComputedStyle(root, "::after").transform).a;
        if (0.15 - (innerHeight / 2 - 4) / (perUnit * scale) < -0.0804) pageBar();
        else requestAnimationFrame(follow);
      };
      requestAnimationFrame(follow);
    })), gone);
  };
  if (!script?.hasAttribute("data-wait")) addEventListener("load", () => api.ready(), { once: true });
})();
