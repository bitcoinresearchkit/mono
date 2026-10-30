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
// - A page's own choices, installed: laid out below the status bar (`data-below-status`, its viewport tag before this
//   script, its foot's room `--safe-bottom`), and its scrolling list softened under the page's top (`data-edge`, the
//   list's selector; `--edge-ground`, the page's ground). Studio's both, so far: each app's to try on a phone.
(() => {
  const ORANGE = "#e65b00";
  const root = document.documentElement, script = /** @type {HTMLScriptElement | null} */ (document.currentScript);
  const standalone = /** @type {any} */ (navigator).standalone;
  // (A newer version of the page, said: its own file, every showcase's from here.)
  document.head.append(Object.assign(document.createElement("script"), { src: "/assets/version/version.js" }));
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
  // A page that asks (`data-below-status`), installed upright on an iPhone, laid out below the status bar, not under it:
  // iOS 26 on lays a fade over a page's top under it, some 16px past it, whatever's drawn there. Decided before the page
  // is drawn: its viewport tag (put before this script) replaced without viewport-fit=cover (one edited isn't read
  // again). Its foot's inset read first, while under it, and kept: below it, iOS says none though the page still reaches
  // under the home indicator. `--safe-bottom` is the foot's room to keep, wherever the page is: its own CSS uses it, not
  // env(safe-area-inset-bottom). Started on its side (no inset said there either way): under it till turned upright.
  const below = script?.hasAttribute("data-below-status"), edge = script?.dataset.edge;
  if (below) {
    const sheet = document.createElement("style");
    sheet.textContent = `
      :root { --safe-bottom: env(safe-area-inset-bottom); }
      @media (orientation: portrait) { :root[data-kept-bottom] { --safe-bottom: var(--kept-bottom); } }`;
    document.head.append(sheet);
    if (standalone === true) {
      const upright = matchMedia("(orientation: portrait)");
      const lay = () => {
        const probe = document.createElement("div");
        probe.style.cssText = "position: fixed; height: env(safe-area-inset-bottom); visibility: hidden";
        root.append(probe);
        root.style.setProperty("--kept-bottom", `${probe.getBoundingClientRect().height}px`);
        probe.remove();
        root.toggleAttribute("data-kept-bottom", true);
        const tag = document.querySelector('meta[name="viewport"]');
        tag?.replaceWith(Object.assign(document.createElement("meta"), { name: "viewport", content: (tag.getAttribute("content") ?? "").replace(/,\s*viewport-fit=cover/, "") }));
      };
      if (upright.matches) lay();
      else upright.addEventListener("change", function turned() {
        if (upright.matches) upright.removeEventListener("change", turned), lay();
      });
    }
  }
  // Installed, a page's scrolling list (`data-edge`, its selector) softened where it scrolls up to the page's top, as an
  // app's list goes under its bar: a progressive blur, as iOS's own, six layers each blurring twice the one under it,
  // each masked to a band, the strongest at the top (no variable blur in CSS: bands of it, overlapping), under the
  // page's ground (`--edge-ground`) fading out downward. None at the list's top; all there once it's scrolled 32px, each
  // layer by the scroll's own timeline (a layer under a container's opacity would blur nothing, its backdrop cut there),
  // under reduced motion too (it follows the scroll: no motion of its own). Over the page's own (z-index 5).
  if (edge && matchMedia("(display-mode: standalone)").matches) {
    const sheet = document.createElement("style");
    sheet.textContent = `
      ${edge} { scroll-timeline: --bitview-edge block; }
      .bitview-edge { position: fixed; inset: 0 0 auto; z-index: 5; height: 44px; pointer-events: none; }
      .bitview-edge > i { position: absolute; inset: 0; -webkit-backdrop-filter: blur(var(--blur)); backdrop-filter: blur(var(--blur)); -webkit-mask-image: linear-gradient(to top, var(--band)); mask-image: linear-gradient(to top, var(--band)); opacity: 0; }
      .bitview-edge > i:nth-child(1) { --blur: 0.5px; --band: transparent 0%, black 14%, black 29%, transparent 43%; }
      .bitview-edge > i:nth-child(2) { --blur: 1px; --band: transparent 14%, black 29%, black 43%, transparent 57%; }
      .bitview-edge > i:nth-child(3) { --blur: 2px; --band: transparent 29%, black 43%, black 57%, transparent 71%; }
      .bitview-edge > i:nth-child(4) { --blur: 4px; --band: transparent 43%, black 57%, black 71%, transparent 86%; }
      .bitview-edge > i:nth-child(5) { --blur: 8px; --band: transparent 57%, black 71%, black 86%, transparent 100%; }
      .bitview-edge > i:nth-child(6) { --blur: 16px; --band: transparent 71%, black 86%; }
      .bitview-edge > i:nth-child(7) { background: linear-gradient(var(--edge-ground), color-mix(in srgb, var(--edge-ground) 55%, transparent) 35%, color-mix(in srgb, var(--edge-ground) 0%, transparent)); -webkit-backdrop-filter: none; backdrop-filter: none; -webkit-mask-image: none; mask-image: none; }
      @supports (animation-timeline: scroll()) {
        .bitview-edge > i { animation: bitview-edge linear both !important; animation-timeline: --bitview-edge !important; animation-range: 0 32px !important; }
      }
      @keyframes bitview-edge { to { opacity: 1; } }`;
    document.head.append(sheet);
    document.addEventListener("DOMContentLoaded", () => {
      for (const list of document.querySelectorAll(edge)) {
        const layers = document.createElement("div");
        layers.className = "bitview-edge";
        layers.setAttribute("aria-hidden", "true");
        layers.innerHTML = "<i></i>".repeat(7);
        list.prepend(layers);
      }
    });
  }
  /** Said by a page that waits (`data-wait`): it's ready. Else as it loads. */
  const api = { ready: () => {} };
  /** @type {any} */ (window).bitviewLaunch = api;
  if (!standalone) return;
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
    .bitview-launch-bar { position: fixed; inset: 0 0 auto; z-index: ${top + 1}; height: calc(env(safe-area-inset-top) + 12px); background: ${ORANGE}; pointer-events: none; }
    @keyframes bitview-launch-out { to { opacity: 0; } }`;
  document.head.append(style);
  // The status bar's: iOS colors it from the first fixed box at the top edge's middle, a pseudo-element found as the
  // page itself (passed over): an element of its own, over the cover.
  const bar = document.createElement("div");
  bar.className = "bitview-launch-bar";
  root.append(bar);
  const gone = () => (delete root.dataset.launch, bar.remove());
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
      // The bar the page's own (its ground, all but clear over it, as iOS reads a strip) once the page shows at the top
      // edge's middle, where iOS reads it: the cube's top face come down past it (its unit's 0.08 above the point it's
      // drawn back to, 0.15 under its middle), read as it goes.
      const perUnit = 400 / 1.26, page = `color-mix(in srgb, ${getComputedStyle(document.body).backgroundColor} 1%, transparent)`;
      const follow = () => {
        if (!("launch" in root.dataset)) return;
        const scale = new DOMMatrix(getComputedStyle(root, "::after").transform).a;
        if (0.15 - (innerHeight / 2 - 4) / (perUnit * scale) < -0.0804) bar.style.background = page;
        else requestAnimationFrame(follow);
      };
      requestAnimationFrame(follow);
    })), gone);
  };
  if (!script?.hasAttribute("data-wait")) addEventListener("load", () => api.ready(), { once: true });
})();
