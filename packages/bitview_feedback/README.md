# bitview_feedback

The inbox behind every Bitview page's Feedback button: a Cloudflare Worker on its own hostname, so the public servers stay stateless and take no new POST route, and every node's pages send to one place.

**What it keeps.** One row per note: the UTC date, the app's folder, the page's exact address (masked, and only from a page that declares it shareable), the page's version, the server's build, a browser class (`safari-phone`) with its major version and operating system, the viewport to the nearest 10 px, pixel ratio, theme, whether it's an installed app, the page's recent errors (under the same declaration), and the note. It keeps no IP, full user agent, cookie, contact or time of day. The card says in one line what's included; the whole list is here.

**What's masked.** A page sends its folder alone unless it declares that its address and errors hold nothing of the sender's own (`<meta name="bitview-feedback" content="address-shareable" />`), so a new page can't leak by default and Stack's amounts never leave. From a page that has declared it, the address and errors have anything shaped like a Bitcoin address, extended key, long hex string, IP, site address or the node's hostname replaced before they leave the page.

**What passes through without being kept.** The sender's IP goes to Cloudflare, as for any site behind it, and the rate limiter counts it in the edge's memory for a minute. The browser's `Origin` header arrives and is ignored. Rows are deleted once read, though Cloudflare's point-in-time recovery for D1 keeps them restorable for 7 days on the free plan (30 on paid), and a restore can show roughly when a row appeared. What the project keeps after reading is deleted after 30 days, except a note a person chose to act on: its short title, folder, date and device class stay in the task list until that task is done.

**Who reads it.** The maintainer's job (`triage.mjs`, here) reads the inbox through Cloudflare's API. An AI model with no tools classifies each note (it goes to the model provider), and a person decides what becomes work. Anyone wanting a reply can leave a way to reach them in the note itself. The public hostname has no way to read the inbox.

**Limits.** 32 KB a body, 4,000 characters a note, 5 notes a minute per address (/64 on IPv6, approximate, per Cloudflare location), 500 unread at once. On Cloudflare's free plan a flood stops the inbox at the daily quota instead of billing. Notes wait here unread until the maintainer's job reads them, at most 50 a day.

## Deploying

```
bunx wrangler@4 d1 create bitview-feedback       # put its id in wrangler.toml
bunx wrangler@4 d1 execute bitview-feedback --remote --file schema.sql
bunx wrangler@4 deploy
```

Before deploying, confirm the account's Workers and D1 plan is Free and that no other Worker shares it: the free daily limits are account-wide and are what stops a flood from billing.

## In a flood

The sheet says when a note couldn't be sent. To empty a flooded inbox after reading what's useful: `bunx wrangler@4 d1 execute bitview-feedback --remote --command "DELETE FROM feedback"`. To close it, `bunx wrangler@4 delete`; to open it again, deploy. Turnstile, loaded inside the opened sheet, is the next step if floods recur.
