import { createHeader } from "./header/index.js";
import { createRoutePage, normalizePath, resolvePath } from "./routes.js";
import "./utils/press.js";
import { getEventAnchor, isPlainLeftClick } from "./utils/event.js";
import { revealPage, transitionPage } from "./utils/transition.js";

/** @typedef {import("./routes.js").RoutePath} RoutePath */

/** @type {HTMLElement | undefined} */
let currentPage;

/** @type {Map<RoutePath, Promise<HTMLElement>>} */
const pageByPath = new Map();

let renderId = 0;

const header = createHeader();
document.body.append(header);

/** @param {RoutePath} pathname */
async function getPage(pathname) {
  let page = pageByPath.get(pathname);

  if (!page) {
    page = createRoutePage(pathname).then((element) => {
      element.hidden = true;
      element.inert = true;
      document.body.append(element);

      return element;
    });
    pageByPath.set(pathname, page);
  }

  return page;
}

/** @param {HTMLElement} page */
function activatePage(page) {
  if (currentPage && currentPage !== page) {
    currentPage.hidden = true;
    currentPage.inert = true;
    currentPage.dispatchEvent(new Event("pageinactive"));
  }

  page.hidden = false;
  page.inert = false;
  currentPage = page;
  page.dispatchEvent(new Event("pageactive"));
}

async function renderPage() {
  const id = ++renderId;
  const pathname = normalizePath(window.location.pathname);
  const page = await getPage(pathname);

  if (id === renderId) activatePage(page);
}

/** @param {string} path */
function navigate(path) {
  if (path === `${window.location.pathname}${window.location.hash}`) return;
  history.pushState(null, "", path);
  void getPage(normalizePath(window.location.pathname));
  void transitionPage(renderPage);
}

document.addEventListener("click", (event) => {
  if (!isPlainLeftClick(event)) return;

  const anchor = getEventAnchor(event);
  if (!anchor) return;

  const url = new URL(anchor.href);
  if (url.origin !== window.location.origin) return;
  if (url.pathname === window.location.pathname && url.hash) return;

  const pathname = resolvePath(url.pathname);
  if (!pathname) return;

  event.preventDefault();
  navigate(`${pathname}${url.hash}`);
});

window.addEventListener("popstate", () => void renderPage());

void renderPage().then(() => {
  requestAnimationFrame(() => void revealPage());
});
