// The changelog (docs/CHANGELOG.md, linked beside this page as CHANGELOG.md) read into its releases, for the page to
// show with each release's API changes: a release's heading (`## [v0.12.2](url) - 2026-09-01`, or `## Unreleased`),
// what's said of it, and its notes by kind (`### Breaking Changes`) and topic (`#### \`brk_types\``), each a list item,
// its wrapped lines joined, maybe with items of its own beneath it. Its Markdown is the little the changelog uses:
// code, links, bold, italics. Anything else is text (`<window>` too), so nothing in it is markup.

/**
 * @typedef {{ html: string, text: string }} Note Something said, as shown and as searched.
 * @typedef {{ kind: string, topic?: Note, said: string[], notes: Note[] }} Group A kind's notes, of a topic or not;
 *   what's said before them, as HTML.
 * @typedef {{ version?: string, date?: string, said: string[], groups: Group[] }} Release Its version (`v0.12.2`),
 *   none if not released yet; what's said of it, as HTML (how to upgrade, its changes on GitHub).
 */

const escape = (/** @type {string} */ text) =>
  text.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c] ?? c);

/** A line's Markdown as HTML: its code first, kept as written; then links (to the web or this site), bold, italics. */
export function inline(/** @type {string} */ text) {
  /** @type {string[]} */
  const codes = [];
  const html = escape(text.replace(/`([^`]+)`/g, (_, code) => `\u0000${codes.push(code) - 1}\u0000`))
    .replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (link, label, href) => (/^(https?:|\/|#|\.)/.test(href) ? `<a href="${href}">${label}</a>` : link))
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/(^|[^*\w])\*([^*\s](?:[^*]*[^*\s])?)\*(?![*\w])/g, "$1<em>$2</em>");
  return html.replace(/\u0000(\d+)\u0000/g, (_, i) => `<code>${escape(codes[Number(i)])}</code>`);
}

/** A line's Markdown as the words searched for. */
const plain = (/** @type {string} */ text) => text.replace(/\[([^\]]+)\]\([^)\s]*\)/g, "$1").replace(/[`*]/g, "");

/**
 * The changelog's releases, newest first, as written; what comes before the first (its title) left out.
 * @param {string} markdown
 * @returns {Release[]}
 */
export function releases(markdown) {
  /** @type {Release[]} */
  const all = [];
  /** @type {Release | undefined} */
  let release;
  /** @type {Group | undefined} */
  let group;
  // The item being read, as its lines, then those of each item beneath it; and the paragraph being read.
  /** @type {string[][] | undefined} */
  let item;
  /** @type {string[] | undefined} */
  let paragraph;

  const endItem = () => {
    if (item && group) {
      const [own, ...beneath] = item.map((lines) => lines.join(" "));
      group.notes.push({
        html: inline(own) + (beneath.length ? `<ul class="subs">${beneath.map((text) => `<li>${inline(text)}</li>`).join("")}</ul>` : ""),
        text: plain([own, ...beneath].join(" ")),
      });
    }
    item = undefined;
  };
  // (Said before a kind's notes, it's the kind's; after them, or before any, the release's.)
  const endParagraph = () => {
    if (paragraph && release) (group && !group.notes.length ? group : release).said.push(inline(paragraph.join(" ")));
    paragraph = undefined;
  };
  const end = () => (endItem(), endParagraph());

  for (const line of markdown.replace(/<!--[\s\S]*?-->/g, "").split("\n")) {
    let match;
    if ((match = line.match(/^## (.+)/))) {
      end();
      const version = match[1].match(/\bv?(\d+\.\d+\.\d+(?:-[\w.]+)?)/);
      release = /unreleased/i.test(match[1]) || !version
        ? { said: [], groups: [] }
        : { version: `v${version[1]}`, date: match[1].match(/\d{4}-\d{2}-\d{2}/)?.[0], said: [], groups: [] };
      all.push(release);
      group = undefined;
    } else if (!release) {
      continue;
    } else if ((match = line.match(/^### (.+)/))) {
      end();
      group = { kind: match[1].trim(), said: [], notes: [] };
      release.groups.push(group);
    } else if ((match = line.match(/^#{4,6} (.+)/))) {
      end();
      const topic = { html: inline(match[1].trim()), text: plain(match[1].trim()) };
      // (A kind's first topic is its notes' if it has none of its own yet.)
      if (group && !group.topic && !group.notes.length && !group.said.length) group.topic = topic;
      else release.groups.push((group = { kind: group?.kind ?? "", topic, said: [], notes: [] }));
    } else if ((match = line.match(/^( *)(?:[-*+]|\d+\.) (.*)/))) {
      endParagraph();
      if (match[1] && item) item.push([match[2]]);
      else (endItem(), (item = [[match[2]]]));
      group ??= release.groups[release.groups.push({ kind: "", said: [], notes: [] }) - 1];
    } else if ((match = line.match(/^ +(\S.*)/)) && item) {
      /** @type {string[]} */ (item.at(-1)).push(match[1]);
    } else if (!line.trim()) {
      endParagraph();
    } else if (!line.startsWith(">")) {
      endItem();
      (paragraph ??= []).push(line.trim());
    }
  }
  end();
  return all;
}
