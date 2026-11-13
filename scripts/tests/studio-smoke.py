#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["playwright>=1.49"]
# ///
"""Smoke checks for what Studio can't undo: saved charts survive reloads, the page draws, a shared link opens.

Run before a Studio commit: uv run --script scripts/tests/studio-smoke.py [--base http://127.0.0.1:3110]
"""

import argparse
import base64
import json
import mimetypes
import os
import sys
import time
import traceback
import urllib.error
import urllib.request
import zlib
from pathlib import Path
from urllib.parse import quote, unquote, urlparse

from playwright.sync_api import Error as PlaywrightError
from playwright.sync_api import sync_playwright

# The page under test is the one in this checkout, read from disk at each request: Studio's files and the modules it
# loads, by the directories they're served from (see local); everything else is the server's.
ROOT = Path(__file__).resolve().parents[2]
PAGE = (ROOT / "showcases" / "studio" / "index.html").resolve()
OWNED = {"/studio/": PAGE.parent, "/modules/": (ROOT / "modules").resolve()}
mimetypes.add_type("text/javascript", ".mjs")
STORE = "bitview-studio:v2"
PREFS = "bitview-studio:prefs"
VIEWPORT = {"width": 1400, "height": 900}
SYSTEM_CHROMES = [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
]

# The charts the checks save, through the page's own Save: (title, layers, folder it is moved to through the UI).
CHARTS = [
    ("Smoke price", [{"series": "price"}], None),
    ("Smoke ratio", [{"series": "price"}, {"series": "realized_price"}, {"formula": "A / B"}], "Smoke folder"),
    ("Smoke realized", [{"series": "realized_price"}], None),
]
FOLDER = "Smoke folder"


def local(path):
    """
    Whether a path is the checkout's (under a directory in OWNED; one ending in a slash, a directory's index.html; a Studio
    address, its page), and
    its file there, resolved and inside it: (False, None), the server's; (True, None), the checkout has none: missing.
    """
    path = unquote(path)
    for prefix, root in OWNED.items():
        if path == prefix.rstrip("/") or path.startswith(prefix):
            rest = path.removeprefix(prefix.rstrip("/"))
            file = (root / rest.lstrip("/") / ("index.html" if rest.endswith("/") or not rest else "")).resolve()
            # (Studio's own addresses (a preset's, a section's), no file of their own: its page, as the server serves them.)
            if prefix == "/studio/" and not file.exists() and "." not in rest.rsplit("/", 1)[-1]:
                file = root / "index.html"
            return True, (file if file.is_file() and file.is_relative_to(root) else None)
    return False, None


def layers_of(chart):
    return [layer.get("series") or layer.get("formula") or "?" for layer in chart["layers"]]


FIXTURE = sorted(({"title": title, "layers": layers_of({"layers": layers}), "folder": folder or ""} for title, layers, folder in CHARTS), key=lambda chart: chart["title"])


def fixture_of(snap):
    """A snapshot's charts as CHARTS lists them: title, layers, folder's name."""
    return [{"title": c["title"], "layers": c["layers"], "folder": c["folder"]} for c in snap["charts"]]


class Failure(Exception):
    pass


def need(condition, message):
    if not condition:
        raise Failure(message)


# ── Mutations: the checks' own proof that they can fail (--break NAME). The page file is never changed: a copy is
# served with one edit, or a script runs before the page does. Each source edit must match exactly once, else it says so.


def edit(old, new, times=1):
    def apply(html):
        need(html.count(old) == times, f"--break: the page no longer has exactly {times} of {old!r} to edit")
        return html.replace(old, new)

    return apply


BREAKS = {
    # The library is read from storage after storage was reset: what a load-order loss does, saved charts gone.
    "load-order": ("saved charts", {"source": edit("let library = storage.get(STORE, null);", "let library = (localStorage.removeItem(STORE), null);")}),
    # The same loss from outside: the library cleared at every load, before the page reads it.
    "wipe-library": ("saved charts", {"init": f"try {{ localStorage.removeItem({STORE!r}) }} catch {{}}"}),
    # The page runs and nothing is painted on any canvas.
    "no-draw": ("page draws", {"init": "for (const k of ['stroke','fill','fillRect','fillText','strokeText','drawImage','putImageData']) CanvasRenderingContext2D.prototype[k] = function () {};"}),
    # The page throws after it starts.
    "page-error": ("page draws", {"source": edit("async function start() {", "async function start() { setTimeout(() => { throw new Error('smoke: injected'); }, 200);")}),
    # The server's API answers 503.
    "api-down": ("page draws", {"api": 503}),
    # Compressed links can't be read.
    "bad-link": ("shared link", {"source": edit('new DecompressionStream("deflate-raw")', 'new DecompressionStream("gzip")')}),
    # A migration drops the layers of the older Studio's chart.
    "old-layers-dropped": ("saved charts", {"source": edit("        doc.updated = Number.isFinite(stored.updated) ? stored.updated : 0;\n        return doc;", "        doc.updated = Number.isFinite(stored.updated) ? stored.updated : 0;\n        if (stored.id === 'oldchart1') doc.layers = [];\n        return doc;")}),
    # Move to folder moves another chart than the one it was asked about (a fault in what the fixture is made with).
    "move-wrong-chart": ("saved charts", {"fixture": True, "source": edit("function moveToFolder(/** @type {string[]} */ keys, /** @type {string | null} */ id, quiet = false) {", "function moveToFolder(/** @type {string[]} */ keys, /** @type {string | null} */ id, quiet = false) {\n        keys = [`c:${library.charts.find((doc) => `c:${doc.id}` !== keys[0])?.id ?? ''}`];")}),
    # A first-party GET fails in the network (net::ERR_FAILED), which the page may well catch and go on from.
    "request-failed": ("page draws", {"fail": "/api/oracle/price"}),
    # The chart library's first (empty) frame reaches for the chart before a shared link has given the page one: the page
    # error every link opened used to throw, twice.
    "early-frame": ("shared link", {"source": edit("const span = chart && FINE.has(chart.index) ?", "const span = FINE.has(chart.index) ?", times=2)}),
    # The chart opened from the Charts page is laid out while its card is still growing out of its tile, and never again:
    # drawn at its tile's size in the card's corner, its canvases a tile wide.
    "tile-sized": ("saved charts", {"source": edit("delete document.body.dataset.morph, unscaled(), (opening", "delete document.body.dataset.morph, (opening")}),
    # A link opened replaces the library it opens over.
    "link-wipes-library": ("shared link", {"source": edit("        if (shared) {\n          const like =", "        if (shared) {\n          localStorage.removeItem(STORE); library.charts = []; library.folders = []; library.order = [];\n          const like =")}),
}


def server_up(base):
    try:
        urllib.request.urlopen(f"{base}/studio/", timeout=5).close()
        return True
    except urllib.error.HTTPError:
        return True  # (it answered)
    except Exception:
        return False


def wait_for_server(base, seconds=120):
    """The server may be restarted by whoever owns it meanwhile: wait for it (up to a couple of minutes), say so."""
    end = time.time() + seconds
    told = False
    while not server_up(base):
        if time.time() > end:
            return False
        if not told:
            print(f"      waiting for {base} (up to {seconds}s)", flush=True)
            told = True
        time.sleep(3)
    return True


class Run:
    def __init__(self, playwright, base, chrome, broken):
        self.base = base.rstrip("/")
        self.host = urlparse(self.base).netloc
        self.broken = BREAKS[broken][1] if broken else {}
        self.browser = self.launch(playwright, chrome)
        self.library_dump = None
        self.library_want = None

    def launch(self, playwright, chrome):
        if chrome:
            return playwright.chromium.launch(executable_path=chrome)
        try:
            return playwright.chromium.launch()
        except PlaywrightError as error:
            # Playwright's own browser isn't installed (nothing is installed here): the machine's Chrome or Chromium.
            for path in SYSTEM_CHROMES:
                if os.path.exists(path):
                    return playwright.chromium.launch(executable_path=path)
            raise Failure(f"no browser: Playwright's isn't installed and no Chrome was found (--chrome PATH): {str(error).splitlines()[0]}")

    # A throwaway context: its own storage, no service worker, so every request goes through the routing below.
    def context(self, clipboard=False, clean=False):
        ctx = self.browser.new_context(viewport=VIEWPORT, service_workers="block", timezone_id="UTC", locale="en-US")
        ctx.problems = []
        ctx.aborted = set()  # the requests this script refuses itself: their failure is its doing, not the page's
        # (`clean`: the page as it is on disk, whatever --break says: the fixtures are made on it, unless the fault is in
        # what they're made with.)
        broken = {} if clean else self.broken
        ctx.broken = broken
        if clipboard:
            ctx.grant_permissions(["clipboard-read", "clipboard-write"], origin=self.base)
        if "init" in broken:
            ctx.add_init_script(broken["init"])
        ctx.route("**/*", lambda route: self.route(ctx, route))
        ctx.on("page", lambda page: self.watch(ctx, page))
        return ctx

    def route(self, ctx, route):
        request = route.request
        url = urlparse(request.url)
        if url.scheme not in ("http", "https"):
            return route.continue_()
        if url.netloc != self.host:
            # Nothing leaves for another host.
            ctx.aborted.add(request)
            return route.abort()
        if request.method not in ("GET", "HEAD"):
            # Nothing is ever written to the server.
            ctx.problems.append(f"{request.method} {url.path} blocked: the checks only read")
            ctx.aborted.add(request)
            return route.abort()
        if "fail" in ctx.broken and url.path.startswith(ctx.broken["fail"]):
            return route.abort("failed")  # (a network failure, not ours: not in ctx.aborted)
        if "api" in ctx.broken and url.path.startswith("/api/"):
            return route.fulfill(status=ctx.broken["api"], body="down")
        ours, file = local(url.path)
        if not ours:
            return route.continue_()
        if not file:
            # (Not in the checkout: missing, never the server's copy.)
            return route.fulfill(status=404, body="not in this checkout")
        body = file.read_bytes()
        if "source" in ctx.broken and file == PAGE:
            body = ctx.broken["source"](body.decode("utf-8"))
        kind = mimetypes.guess_type(file.name)[0] or "application/octet-stream"
        kind += "; charset=utf-8" if kind.startswith("text/") else ""
        return route.fulfill(status=200, body=body, headers={"content-type": kind, "cache-control": "no-store"})

    def watch(self, ctx, page):
        problems = ctx.problems
        page.on("pageerror", lambda error: problems.append(f"page error: {str(error).splitlines()[0]}"))
        page.on("console", lambda message: message.type == "error" and problems.append(f"console error: {message.text[:160]}"))
        page.on("requestfailed", lambda request: self.failed(ctx, request))
        page.on("response", lambda response: self.answered(problems, response))

    def failed(self, ctx, request):
        # A first-party request that failed is a problem, whatever the error (ERR_FAILED too: the page may catch it and
        # go on). Not one a reload or a navigation cut short (ERR_ABORTED), nor one this script refused itself.
        reason = request.failure or ""
        if urlparse(request.url).netloc == self.host and "ERR_ABORTED" not in reason and request not in ctx.aborted:
            ctx.problems.append(f"request failed: {request.url} ({reason})")

    def answered(self, problems, response):
        if urlparse(response.url).netloc == self.host and response.status >= 400:
            problems.append(f"request {response.status}: {response.url}")

    def studio(self, ctx, hash=""):
        page = ctx.new_page()
        page.goto(f"{self.base}/studio/{hash}", wait_until="load")
        self.ready(page)
        return page

    def ready(self, page):
        page.wait_for_function("document.documentElement.hasAttribute('data-ready')", timeout=60_000)

    # The page's own words for a chart drawn whole (its data in, its fades done): it fails, saying why, if it never is.
    def drawn(self, page):
        try:
            page.evaluate("studio.ready()")
        except PlaywrightError as error:
            raise Failure(f"the chart never drew: {str(error).splitlines()[0]}")

    def seed(self, ctx, storage):
        # Storage as it is before the page's first load: written from another document of the same origin.
        page = ctx.new_page()
        page.goto(f"{self.base}/studio/palette.js")
        page.evaluate("items => { for (const [key, value] of Object.entries(items)) localStorage.setItem(key, value); }", storage)
        page.close()

    def library(self):
        """A real library, made once through the page's own UI: a folder and three charts, one in the folder."""
        if self.library_dump is None:
            ctx = self.context(clean=not self.broken.get("fixture"))
            page = self.studio(ctx)
            new_folder(page, FOLDER)
            for title, layers, folder in CHARTS:
                page.evaluate("chart => studio.open(chart)", {"title": title, "range": "4y", "layers": layers})
                self.drawn(page)
                page.click("#keep")
                page.wait_for_function("title => (localStorage.getItem('bitview-studio:v2') ?? '').includes(JSON.stringify(title).slice(1, -1))", arg=title, timeout=15_000)
                if folder:
                    move_to_folder(page, title, folder)
            # The fixture is exactly what CHARTS says: each title with its layers and its folder, and the one folder.
            self.library_want = snapshot(page, wait=lambda s: fixture_of(s) == FIXTURE and [f["name"] for f in s["folders"]] == [FOLDER], what="the library made through the UI")
            self.library_dump = dump(page)
            ctx.close()
        return self.library_dump, self.library_want


# ── The page's own UI, as a person uses it


def new_folder(page, name):
    page.click('button[aria-label="More for Library"]')
    page.get_by_role("menuitem", name="New folder").first.click()
    page.keyboard.type(name)
    page.keyboard.press("Enter")
    page.wait_for_function("name => (localStorage.getItem('bitview-studio:v2') ?? '').includes(JSON.stringify(name).slice(1, -1))", arg=name, timeout=15_000)


def move_to_folder(page, title, folder):
    """Move the open chart to a folder, through its menu; stored, that chart is in the folder and no other chart is."""
    page.click("#chart-more")
    page.get_by_role("menuitem", name="Move to folder").click()
    page.locator("[role=menu] span.label", has_text=folder).click()
    snapshot(page, wait=lambda s: any(c["title"] == title and c["folder"] == folder for c in s["charts"]) and all(c["folder"] != folder for c in s["charts"] if c["title"] != title), what=f"'{title}' moved to '{folder}'")


# What storage holds, as the checks compare it: the charts (title, what each layer plots, its folder's name) and the folders.
SNAPSHOT = """() => {
  const text = localStorage.getItem('bitview-studio:v2');
  if (text === null) return null;
  const stored = JSON.parse(text);
  const folders = stored.folders ?? [];
  const nameOf = id => folders.find(folder => folder.id === id)?.name ?? '';
  return {
    schema: stored.schema,
    charts: (stored.charts ?? []).map(c => ({ id: c.id, title: c.title, layers: c.layers.map(l => l.series ?? l.formula ?? '?'), folder: nameOf(c.folder) })).sort((a, b) => a.title.localeCompare(b.title)),
    folders: folders.map(f => ({ id: f.id, name: f.name })).sort((a, b) => a.name.localeCompare(b.name)),
  };
}"""


def snapshot(page, wait=None, timeout=15.0, what="storage"):
    """What storage holds; with `wait`, once that holds of it: else, at the deadline, a failure saying what it holds."""
    end = time.time() + timeout
    while True:
        value = page.evaluate(SNAPSHOT)
        if wait is None or (value and wait(value)):
            return value
        if time.time() > end:
            held = None if value is None else {"charts": fixture_of(value), "folders": [f["name"] for f in value["folders"]]}
            raise Failure(f"{what}: not as expected after {timeout:.0f}s; stored: {json.dumps(held)}")
        time.sleep(0.2)


def dump(page):
    return page.evaluate("() => Object.fromEntries(Object.keys(localStorage).filter(k => k.startsWith('bitview-studio:')).map(k => [k, localStorage.getItem(k)]))")


def settle(page):
    # A load saves once it's idle (a second at most): what's stored after that is what the page keeps.
    time.sleep(1.6)


def say(want, got, what):
    need(got is not None, f"{what}: nothing is stored: the library is gone")
    need(got["folders"] == want["folders"], f"{what}: folders {[f['name'] for f in got['folders']]}, expected {[f['name'] for f in want['folders']]}")
    need(got["charts"] == want["charts"], f"{what}: charts {[(c['title'], c['folder']) for c in got['charts']]}, expected {[(c['title'], c['folder']) for c in want['charts']]}")


def library_count(page):
    return page.evaluate("() => document.querySelector('#chart-list .list-head-name[aria-label^=\"Library\"]')?.getAttribute('aria-label')")


def open_each(run, page, want, what):
    """Every saved chart opens from the Library's page (its folders' charts too), with its title and layers."""
    for chart in want["charts"]:
        open_saved(page, chart["title"])
        page.wait_for_function("title => document.getElementById('title')?.value === title && !document.body.hasAttribute('data-home')", arg=chart["title"], timeout=15_000)
        got = page.evaluate("() => studio.chart().layers.map(l => l.series ?? l.formula ?? '?')")
        need(got == chart["layers"], f"{what}: '{chart['title']}' opened with layers {got}, expected {chart['layers']}")
        run.drawn(page)
        fills_card(page, f"{what}: '{chart['title']}'")


# What a chart opened from the Charts page is drawn at: its canvases' far edges against its card's size (laid out, not as
# a motion scales it), and the biggest canvas's drawn pixels against its box.
FILLS = """() => {
  const card = document.getElementById('chart');
  const box = card.getBoundingClientRect(), scale = box.width / card.clientWidth || 1;
  let right = 0, bottom = 0, biggest = null;
  for (const canvas of card.querySelectorAll('canvas')) {
    const at = canvas.getBoundingClientRect();
    right = Math.max(right, (at.right - box.left) / scale);
    bottom = Math.max(bottom, (at.bottom - box.top) / scale);
    if (!biggest || canvas.width * canvas.height > biggest.width * biggest.height) biggest = canvas;
  }
  const at = biggest?.getBoundingClientRect();
  return { card: [card.clientWidth, card.clientHeight], reach: [right, bottom], drawn: biggest ? [biggest.width / devicePixelRatio, biggest.height / devicePixelRatio] : [0, 0], shown: at ? [at.width / scale, at.height / scale] : [0, 0] };
}"""


def fills_card(page, what):
    """The chart fills its card: it is drawn across the card's size (an axis and a time scale beside the plot), not at its
    tile's, whatever the chart (a tile's 184 by 198 in the card's corner passes `studio.ready()` and the picture's pixels)."""
    page.wait_for_function("() => !document.body.hasAttribute('data-morph')", timeout=15_000)
    got = page.evaluate(FILLS)
    (width, height), (reach_x, reach_y), (drawn_x, drawn_y), (shown_x, shown_y) = got["card"], got["reach"], got["drawn"], got["shown"]
    need(width > 400 and height > 300, f"{what}: the card is {width}x{height}: too small to tell a tile from it")
    need(abs(reach_x - width) <= 2 and abs(reach_y - height) <= 2, f"{what}: the chart is drawn {reach_x:.0f}x{reach_y:.0f} in a card of {width}x{height} (a tile's size, left in the card's corner)")
    # (Its pixels as it shows them: a canvas drawn at one size and shown at another is a blur.)
    need(abs(drawn_x - shown_x) <= 1.5 and abs(drawn_y - shown_y) <= 1.5, f"{what}: the biggest canvas is drawn {drawn_x:.0f}x{drawn_y:.0f} and shown {shown_x:.0f}x{shown_y:.0f}")
    need(drawn_x >= width * 0.7 and drawn_y >= height * 0.9, f"{what}: the biggest canvas is drawn {drawn_x:.0f}x{drawn_y:.0f}, for a card of {width}x{height}")


def open_saved(page, title):
    page.click('button[aria-label="View Library"]')
    page.locator("#home").get_by_role("button", name=title, exact=True).click()


# A chart's picture, read off its canvases: pixels that differ from the plot's background colour, and how many of the
# plot's columns have some (a line across the whole view); a chart with no data has a background and none of either.
PIXELS = """() => {
  let best = { off: 0, cols: 0, width: 0 };
  for (const canvas of document.querySelectorAll('#chart canvas')) {
    if (canvas.width * canvas.height < 50000) continue;
    const g = canvas.getContext('2d');
    if (!g) continue;
    const { data } = g.getImageData(0, 0, canvas.width, canvas.height), counts = new Map();
    for (let i = 0; i < data.length; i += 4) if (data[i + 3]) { const k = (data[i] << 16) | (data[i + 1] << 8) | data[i + 2]; counts.set(k, (counts.get(k) ?? 0) + 1); }
    const bg = [...counts.entries()].sort((a, b) => b[1] - a[1])[0]?.[0];
    if (bg === undefined) continue;
    let off = 0;
    const cols = new Set();
    for (let i = 0; i < data.length; i += 4) {
      if (!data[i + 3]) continue;
      if (Math.abs(data[i] - (bg >> 16)) + Math.abs(data[i + 1] - ((bg >> 8) & 255)) + Math.abs(data[i + 2] - (bg & 255)) > 90) { off++; cols.add((i / 4) % canvas.width); }
    }
    if (off > best.off) best = { off, cols: cols.size, width: canvas.width };
  }
  return best;
}"""


def drew(page, what):
    ink = page.evaluate(PIXELS)
    need(ink["off"] >= 500 and ink["cols"] >= ink["width"] * 0.5, f"{what}: nothing is drawn on the chart's canvas ({ink['off']} marked pixels over {ink['cols']} of {ink['width']} columns)")
    return ink


def no_problems(ctx, what):
    need(not ctx.problems, f"{what}: " + "; ".join(dict.fromkeys(ctx.problems)))


# A library as an older Studio wrote it (schema 6: no order, no presets, no deletions), to be read and kept.
OLD = {
    "charts": [
        {
            "id": "oldchart1",
            "title": "Smoke old",
            "index": "day1",
            "range": "4y",
            "panes": [{"id": "oldpane1", "mode": 1}],
            "layers": [{"id": "oldlayer1", "letter": "A", "color": "ink", "type": "line", "width": 1.5, "dash": 0, "pane": "oldpane1", "side": "right", "series": "price", "path": "price.spot"}],
            "updated": 1700000000000,
        }
    ],
    "active": "oldchart1",
    "folders": [],
    "schema": 6,
}


# ── The checks


def check_saved_charts(run):
    """Saved charts and libraries survive reloads, a first load over storage already written, and an older Studio's library."""
    stored, want = run.library()
    need(len(want["charts"]) == len(CHARTS) and len(want["folders"]) == 1, f"the library made through the UI holds {len(want['charts'])} charts, {len(want['folders'])} folders")
    # A tab that loads the library the page itself wrote (before its first load), then is reloaded three times.
    ctx = run.context()
    run.seed(ctx, stored)
    page = run.studio(ctx)
    for n in (1, 2, 3):
        settle(page)
        say(want, snapshot(page), f"load {n}")
        need(library_count(page) == f"Library, {len(CHARTS)}", f"load {n}: the sidebar says {library_count(page)!r}")
        page.reload(wait_until="load")
        run.ready(page)
    settle(page)
    say(want, snapshot(page), "the last reload")
    open_each(run, page, want, "after reloads")
    settle(page)
    say(want, snapshot(page), "after opening each")
    ctx.close()
    # The same library in a browser that never saw it: storage populated before the page's first load, then reloaded. (Its
    # preferences left out: nothing says to open on the chart, so the page opens on the Charts page, and each chart is opened from it.)
    ctx = run.context()
    run.seed(ctx, {key: value for key, value in stored.items() if key != PREFS})
    page = run.studio(ctx)
    for n in (1, 2):
        settle(page)
        say(want, snapshot(page), f"fresh browser, load {n}")
        page.reload(wait_until="load")
        run.ready(page)
    need(page.evaluate("() => document.body.hasAttribute('data-home')"), "fresh browser, no preferences: the page didn't open on the Charts page")
    open_each(run, page, want, "fresh browser")
    ctx.close()
    # A library saved by an older Studio: read, kept (not set aside as unreadable), saved again whole, and it opens.
    # What it must come out as, worked out from OLD alone (not from what the page made of it): the same title, layers, no folder.
    older = {"charts": [{"title": c["title"], "layers": layers_of(c), "folder": ""} for c in OLD["charts"]]}
    ctx = run.context()
    # (No preferences saved, as in a browser that has only ever held this library: the page opens on the Charts page.)
    run.seed(ctx, {STORE: json.dumps(OLD)})
    page = run.studio(ctx)
    settle(page)
    got = snapshot(page)
    need(got is not None and fixture_of(got) == older["charts"], f"older library: stored {got and fixture_of(got)}, expected {older['charts']}")
    need(page.evaluate("() => localStorage.getItem('bitview-studio:unread')") is None, "older library: its chart was set aside as unreadable")
    open_each(run, page, older, "older library")
    run.drawn(page)
    drew(page, "the older library's chart")
    ctx.close()
    return f"{len(want['charts'])} charts in {len(want['folders'])} folder survive 3 reloads, a fresh browser's first load, and an older library; each opens"


def check_page_draws(run):
    """No page errors or failed first-party requests, and the chart's canvas has its line."""
    # A first visit.
    ctx = run.context()
    page = run.studio(ctx)
    page.wait_for_function("() => document.querySelector('#live')?.classList.contains('waiting') === false", timeout=30_000)
    time.sleep(1.0)
    no_problems(ctx, "first visit")
    ctx.close()
    # A chart of the library, and one with a formula and two lines.
    stored, want = run.library()
    ctx = run.context()
    run.seed(ctx, stored)
    page = run.studio(ctx)
    run.drawn(page)
    need(page.evaluate("studio.errors()") == [], f"the open chart has layers that show nothing: {page.evaluate('studio.errors()')}")
    first = drew(page, "the chart open at load")
    open_saved(page, "Smoke ratio")
    page.wait_for_function("() => document.getElementById('title')?.value === 'Smoke ratio' && !document.body.hasAttribute('data-home')", timeout=15_000)
    run.drawn(page)
    need(page.evaluate("studio.errors()") == [], f"Smoke ratio has layers that show nothing: {page.evaluate('studio.errors()')}")
    second = drew(page, "Smoke ratio")
    fills_card(page, "Smoke ratio, opened from the Charts page")
    time.sleep(1.0)
    no_problems(ctx, "with a chart open")
    ctx.close()
    return f"no errors or failed requests; {first['off']} and {second['off']} marked pixels over {first['cols']} and {second['cols']} columns"


def read_link(link):
    """What a link carries, read with no help from the page: its hash, inflated (raw deflate, base64url) and parsed, with the fields after it."""
    hash, *rest = link.split("#chart=", 1)[1].split("&")
    chart = json.loads(zlib.decompress(base64.urlsafe_b64decode(hash + "=" * (-len(hash) % 4)), -15))
    fields = dict(field.split("=", 1) for field in rest)
    if "range" in fields:
        fields["range"] = fields["range"].lower()
    if "from" in fields:
        # (A view up to the latest has no end: it runs up to now.)
        chart["view"] = {"from": fields.pop("from"), **({"to": fields.pop("to")} if "to" in fields else {})}
    return {**chart, **fields}


def shape(chart):
    return {"title": chart["title"], "layers": [layer.get("series") or layer.get("formula") for layer in chart["layers"]], "range": chart.get("range")}


def check_shared_link(run):
    """A link built the way the page builds it opens as that chart in an empty browser, with no page error or failed request, and leaves a library alone."""
    shared = {"title": "Smoke shared", "range": "4y", "layers": [{"series": "price"}, {"series": "realized_price"}, {"formula": "A / B"}]}
    # The sharer's side: Share, in the page, copies the link.
    ctx = run.context(clipboard=True, clean=True)
    page = run.studio(ctx)
    page.evaluate("chart => studio.open(chart)", shared)
    run.drawn(page)
    sketch = page.evaluate("studio.chart()")
    page.click("#share")
    page.wait_for_function("() => document.querySelector('.share-title')", timeout=15_000)
    copied = page.evaluate("navigator.clipboard.readText()")
    ctx.close()
    need(copied.startswith(f"{run.base}/studio/#chart="), f"Share copied {copied[:80]!r}")
    need(read_link(copied) == sketch, "the copied link doesn't carry the chart it was made from")
    packed = zlib.compress(json.dumps(sketch, separators=(",", ":")).encode())[2:-4]
    links = {
        "Share's link": copied,
        "QR link": f"{run.base}/studio/#c=" + base64.b32encode(packed).decode().rstrip("="),
        "written link": f"{run.base}/studio/#chart=" + quote(json.dumps(sketch), safe=""),
    }
    want = shape(sketch)
    for name, link in links.items():
        # An empty browser: the shared chart is what shows, as a preview, and nothing is saved by opening it.
        ctx = run.context()
        page = ctx.new_page()
        page.goto(link, wait_until="load")
        run.ready(page)
        run.drawn(page)
        got = shape(page.evaluate("studio.chart()"))
        need(got == want, f"{name}: opened {got}, expected {want}")
        shown = page.evaluate("() => document.getElementById('title').value")
        need(shown == want["title"], f"{name}: the title shows {shown!r}")
        need(page.is_visible("#preview-note"), f"{name}: it didn't open as a preview")
        need(page.evaluate("() => location.hash") == "", f"{name}: the link stays in the address bar once read")
        drew(page, name)
        settle(page)
        saved = page.evaluate(SNAPSHOT)
        need(not (saved and saved["charts"]), f"{name}: opening it saved a chart: {saved['charts'] if saved else None}")
        no_problems(ctx, name)
        ctx.close()
    # A browser with a library: the link opens over it, and the library is as it was, after the open and a reload.
    stored, library = run.library()
    ctx = run.context()
    run.seed(ctx, stored)
    page = ctx.new_page()
    page.goto(links["Share's link"], wait_until="load")
    run.ready(page)
    run.drawn(page)
    need(shape(page.evaluate("studio.chart()")) == want, "with a library: the shared chart isn't the one that shows")
    settle(page)
    say(library, snapshot(page), "a link opened over a library")
    page.reload(wait_until="load")
    run.ready(page)
    settle(page)
    say(library, snapshot(page), "a library, reloaded after a link")
    need(library_count(page) == f"Library, {len(CHARTS)}", f"after the link the sidebar says {library_count(page)!r}")
    open_each(run, page, library, "after a link")
    no_problems(ctx, "a link over a library")
    ctx.close()
    return f"3 links (Share's, QR's, written) open as '{want['title']}' in an empty browser, nothing saved; a library of {len(library['charts'])} charts is untouched; no page errors"


CHECKS = [
    ("saved charts", check_saved_charts),
    ("page draws", check_page_draws),
    ("shared link", check_shared_link),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__.strip().splitlines()[0])
    parser.add_argument("--base", default="http://127.0.0.1:3110", help="the server the page's API requests go to (read only)")
    parser.add_argument("--only", choices=[name for name, _ in CHECKS], help="run one check")
    parser.add_argument("--chrome", help="a Chrome or Chromium executable, if Playwright's own isn't installed and none is found")
    parser.add_argument("--break", dest="broken", choices=sorted(BREAKS), help="proof the checks can fail: serve the page with one fault (the file is never changed); " + "; ".join(f"{name}: {BREAKS[name][0]}" for name in sorted(BREAKS)))
    args = parser.parse_args()
    failed = 0
    with sync_playwright() as playwright:
        if not wait_for_server(args.base.rstrip("/")):
            print(f"FAIL  setup: {args.base} isn't answering (waited 2 minutes)")
            return 1
        try:
            run = Run(playwright, args.base, args.chrome, args.broken)
        except Failure as error:
            print(f"FAIL  setup: {error}")
            return 1
        try:
            for name, check in CHECKS:
                if args.only and args.only != name:
                    continue
                started = time.time()
                for attempt in (1, 2):
                    try:
                        detail, *notes = check(run).split("\n")
                        print(f"ok    {name}: {detail} ({time.time() - started:.1f}s)", flush=True)
                        for note in notes:
                            print(note, flush=True)
                        break
                    except Exception as error:  # a Failure, or a timeout or a missing control: not what a person would see
                        if not server_up(run.base):
                            # The server went away mid-check (restarted by its owner): wait for it, then once more.
                            if attempt == 1 and wait_for_server(run.base):
                                continue
                            failed += 1
                            print(f"FAIL  {name}: {run.base} isn't answering (waited 2 minutes)", flush=True)
                            break
                        failed += 1
                        reason = str(error) if isinstance(error, Failure) else f"{type(error).__name__}: {str(error).splitlines()[0]}"
                        print(f"FAIL  {name}: {reason} ({time.time() - started:.1f}s)", flush=True)
                        if os.environ.get("STUDIO_SMOKE_TRACE"):
                            traceback.print_exc()
                        break
        finally:
            run.browser.close()
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
