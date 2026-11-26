// @ts-check
// A showcase's home-screen launch, loaded first in each head (`data-wait`: `bitviewLaunch.ready`).
// iOS takes the manifest, icon and launch screen from the page when it is added: made as the page starts, else an app added earlier launches white for good.
(() => {
  const ORANGE = "#e65b00";
  const root = document.documentElement, script = /** @type {HTMLScriptElement | null} */ (document.currentScript);
  const standalone = /** @type {any} */ (navigator).standalone;
  const asset = (/** @type {string} */ path) => new URL(`../${path}`, /** @type {HTMLScriptElement} */ (script).src).href;
  document.head.append(Object.assign(document.createElement("script"), { src: asset("version/version.js") }));
  const app = script?.dataset.app, name = app ? `Bitview ${app}` : "Bitview", short = app ?? name;
  const folder = app ? location.pathname.replace(/^(\/[^/]+).*$/, "$1/") : "/", start = `${location.origin}${folder}`;
  for (const [tag, attributes] of /** @type {const} */ ([
    ["meta", { name: "application-name", content: name }],
    ["meta", { name: "apple-mobile-web-app-title", content: short }],
    // iOS uses a launch screen only for a page that says it's an app.
    ["meta", { name: "apple-mobile-web-app-capable", content: "yes" }],
  ])) document.head.append(Object.assign(document.createElement(tag), attributes));
  // Its icon is a file (made by scripts/home.mjs --icons): one drawn here never came on a phone in Lockdown Mode.
  const icon = asset(`favicon/apps/${app ? folder.slice(1, -1) : "bitview"}.png`);
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
  const api = { ready: () => {} };
  /** @type {any} */ (window).bitviewLaunch = api;
  if (!standalone) return;
  // iOS (26 on) colors its status bar from the first fixed box at the top edge: a full-screen blur there kept a blur under the bar until a reload. So this box is the launch's orange, then the page's color.
  // iOS reads its color as the box comes: a new box for each color (a color changed in place left the launch's orange).
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
    // The page's own ground (body's, else root's), not theme-color, which can drift from it; a see-through strip would leave iOS its blur.
    const meta = /** @type {HTMLMetaElement[]} */ ([...document.querySelectorAll('meta[name="theme-color"]')]).find((tag) => !tag.media || matchMedia(tag.media).matches);
    const ground = [document.body, root].map((element) => (element ? getComputedStyle(element).backgroundColor : "")).find((color) => color && color !== "transparent" && !/rgba\(.*,\s*0\)$/.test(color));
    const color = ground || meta?.content || "#000";
    if (color) strip(color);
    queueMicrotask(() => edge.isConnected && edged());
  };
  paint();
  const edge = document.createElement("div");
  edge.setAttribute("aria-hidden", "true");
  edge.style.cssText = "position: fixed; inset: 0 0 auto; z-index: 2147483000; height: 28px; pointer-events: none";
  edge.innerHTML = [1, 2, 4, 8].map((px, k) => `<i style="position: absolute; inset: 0; backdrop-filter: blur(${px}px); -webkit-backdrop-filter: blur(${px}px); mask-image: linear-gradient(#000 ${k * 18}%, transparent ${40 + k * 20}%); -webkit-mask-image: linear-gradient(#000 ${k * 18}%, transparent ${40 + k * 20}%); opacity: 0"></i>`).join("") + '<i class="tint" style="position: absolute; inset: 0; opacity: 0"></i>';
  const tint = /** @type {HTMLElement} */ (edge.lastElementChild);
  const edged = () => {
    tint.style.background = `linear-gradient(${striped}, color-mix(in srgb, ${striped} 70%, transparent) 30%, color-mix(in srgb, ${striped} 25%, transparent) 65%, transparent)`;
    let own = false;
    for (let element = document.elementFromPoint(innerWidth / 2, 12); element && element !== document.body && element !== root; element = element.parentElement) {
      const position = getComputedStyle(element).position;
      if (position === "fixed" || position === "sticky") (own = element !== status && element !== edge), (element = null);
      if (!element) break;
    }
    const shown = own || document.documentElement.scrollHeight <= innerHeight + 1 ? 0 : Math.min(1, Math.max(0, scrollY / 28));
    // On each layer: an opacity on the whole would cut them off from the page behind, leaving nothing to blur.
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
  try {
    if (sessionStorage.getItem("bitview-launched") !== null) return;
    sessionStorage.setItem("bitview-launched", "");
  } catch {}

  root.dataset.launch = "";
  const top = 2147483600;
  // The root orange too: iOS starts an app with the page a status bar's height short and grows it later; the foot would show the page's ground.
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
  launching = true;
  strip(ORANGE);
  const pageBar = () => ((launching = false), paint());
  const gone = () => (delete root.dataset.launch, pageBar());
  root.addEventListener("animationstart", (event) => event.target === root && event.pseudoElement === "::after" && (root.dataset.launch = "fade"));
  root.addEventListener("animationend", (event) => event.target === root && event.pseudoElement === "::after" && gone());
  // Drawn only once a frame of the orange alone is on screen: a page in the frame iOS fades its launch screen at arrives late, white.
  // Hidden from a layer, whose !important outranks any page's own visibility.
  const shown = new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(() => {
    if (root.dataset.launch === "") root.dataset.launch = "page";
    resolve(undefined);
  })));
  // The cube, decoded before it's shown (shown undecoded, a frame of nothing).
  const cube = fetch(asset("logo/logo.svg")).then((response) => response.text()).then(async (text) => {
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
    // Two frames on: iOS shows a page's first frame late, a motion begun at once would be mostly unseen.
    Promise.all([cube, shown]).then(([url]) => requestAnimationFrame(() => requestAnimationFrame(() => {
      if (!("launch" in root.dataset)) return;
      root.style.setProperty("--bitview-launch-cube", url);
      root.dataset.launch = "out";
      const unit = 92 / 1.26, first = (1.05 * Math.max(innerHeight / 2 / 0.2, innerWidth / 2 / 0.5)) / unit, last = 2.4, box = 400 / 92, steps = 24;
      const frames = Array.from({ length: steps + 1 }, (_, k) => ({ transform: `scale(${(first ** (1 - k / steps) * last ** (k / steps)) / box})` }));
      const timing = { pseudoElement: "::after", duration: 900, fill: /** @type {const} */ ("forwards") };
      const drawn = root.animate(frames, { ...timing, easing: "cubic-bezier(.45, 0, .2, 1)" });
      root.animate([{ opacity: 1, offset: 0.5 }, { opacity: 0 }], { ...timing, easing: "ease-in-out" });
      const blur = (/** @type {number} */ px, /** @type {number} */ offset) => ({ backdropFilter: `blur(${px}px)`, webkitBackdropFilter: `blur(${px}px)`, offset });
      root.animate([blur(24, 0), blur(24, 0.2), blur(0, 1)], { ...timing, pseudoElement: "::before", easing: "cubic-bezier(.3, 0, .2, 1)" });
      drawn.finished.then(gone, gone);
      // iOS reads the top edge's middle: the status bar becomes the page's own once the page shows there.
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
