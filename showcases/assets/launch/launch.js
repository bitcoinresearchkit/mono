// @ts-check
// Every showcase's launch from a phone's home screen, in one place: each page loads this first in its head
// (`<script src="/assets/launch/launch.js"></script>`, `data-wait` for a page that says itself when it's ready: see
// `bitviewLaunch.ready`).
//
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
  // (iOS uses a launch screen only for a page that says it's an app: said here, for every showcase.)
  if (!document.querySelector('meta[name="apple-mobile-web-app-capable"]')) document.head.append(Object.assign(document.createElement("meta"), { name: "apple-mobile-web-app-capable", content: "yes" }));
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
  // The app's launch only, not each page it goes to after: once a session, as its sessionStorage says.
  try {
    if (sessionStorage.getItem("bitview-launched") !== null) return;
    sessionStorage.setItem("bitview-launched", "");
  } catch {}

  root.dataset.launch = "";
  const top = 2147483600;
  const style = document.createElement("style");
  style.textContent = `
    html[data-launch]::after { content: ""; position: fixed; inset: 0; z-index: ${top}; background: ${ORANGE}; pointer-events: none; animation: bitview-launch-out 450ms ease 6s forwards; }
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
  // (Faded by itself, the page never ready: gone.)
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
