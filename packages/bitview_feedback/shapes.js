// @ts-check
// What each field of a note may be, in one place: the Worker keeps a value only if it fits, and the job, which writes some
// of them into a file agents read, checks the same shapes on what it pulls.

/** The app's folder alone: `/explorer/address/bc1...` would name an address. */
export const PAGE = /^\/(?:[a-z0-9_-]{1,40}\/)?$/;
/** The exact address, masked by the page: on this site only (not `//elsewhere`), printable ASCII. */
export const LINK = /^(?:\/(?![/\\])[\x21-\x7e]{0,5999})?$/;
export const VERSION = /^[\w-]{0,64}$/;
export const BROWSER = /^(?:chromium|firefox|safari|other)-(?:phone|desk)$/;
/** What's sent so a report can be reproduced. */
export const CONTEXT = {
  build: /^[\w.+-]{0,40}$/,
  os: /^(?:ios|android|macos|windows|linux|other)$/,
  major: /^\d{0,4}$/,
  viewport: /^\d{1,5}x\d{1,5}$/,
  scale: /^\d{1,2}(?:\.\d)?$/,
  theme: /^(?:light|dark)$/,
  mode: /^(?:app|browser)$/,
};
