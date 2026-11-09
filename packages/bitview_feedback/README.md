# bitview_feedback

The inbox behind every Bitview page's Feedback button: a Cloudflare Worker on its own hostname, so the public servers stay stateless and take no new POST route, and every node's pages send to one place.

**What it keeps.** One row per note: the UTC date, the app's folder, the page's exact address unless the sender unticks it, the page's version, the server's build, a browser class (`safari-phone`) with its major version and operating system, the viewport to the nearest 10 px, pixel ratio, theme, whether it's an installed app, the page's recent errors, the note, and a contact if the sender gave one. It keeps no IP, full user agent, cookie or time of day. The sheet shows every value before it goes.

**What's masked.** The address and errors have anything shaped like a Bitcoin address, extended key, 64-hex hash, IP or the node's hostname replaced before they leave the page. A page whose address holds someone's own figures (Stack's amounts) starts with the box unticked.

**What passes through without being kept.** The sender's IP goes to Cloudflare, as for any site behind it, and the rate limiter counts it in the edge's memory for a minute. The browser's `Origin` header arrives and is ignored. Rows are deleted once read, though Cloudflare's point-in-time recovery for D1 keeps them restorable for 7 days on the free plan (30 on paid), and a restore can show roughly when a row appeared. What the project keeps after reading is deleted after 30 days.

**Who reads it.** The maintainer's job reads the inbox through Cloudflare's API. An AI model with no tools classifies each note, and a person decides what becomes work. The public hostname has no way to read the inbox.

**Limits.** 32 KB a body, 4,000 characters a note, 5 notes a minute per address (/64 on IPv6, approximate, per Cloudflare location), 2,000 unread at once. On Cloudflare's free plan a flood stops the inbox at the daily quota instead of billing.

## Deploying

```
bunx wrangler@4 d1 create bitview-feedback       # put its id in wrangler.toml
bunx wrangler@4 d1 execute bitview-feedback --remote --file schema.sql
bunx wrangler@4 deploy
```

## In a flood

The sheet says when a note couldn't be sent. To close the inbox, `bunx wrangler@4 delete`. To open it again, deploy. Turnstile, loaded inside the opened sheet, is the next step if floods recur.
