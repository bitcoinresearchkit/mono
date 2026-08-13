# <img src="https://qr.davidje13.com/resources/logo.svg" alt="Lean QR" />

Lean QR is a lightweight yet fully-featured library for generating QR Codes. It
runs in Node.js and all recent browsers, and includes wrapper components for
[React, Preact](https://qr.davidje13.com/docs/#extras_react),
[Vue](https://qr.davidje13.com/docs/#extras_vue), and
[native Web Components](https://qr.davidje13.com/docs/#webcomponent). Under 4kB
compressed.

## Features

- ISO 18004 compliant;
- Lightweight (less than 4kB compressed, ~7kB uncompressed, no dependencies);
  - still too big? There's also an
    [even smaller "nano" export](https://qr.davidje13.com/docs/#lean-qr-nano)
    which has reduced features to fit into ~2kB compressed, ~4kB uncompressed!
- Simple yet flexible API, with sensible defaults for all configuration;
- Supports all [standard encodings](https://qr.davidje13.com/docs/#mode)
  out-of-the box:
  - including Unicode 😎;
  - and Shift-JIS 漢字;
  - and supports [custom modes](https://qr.davidje13.com/docs/#custom-modes) for
    special requirements;
- Automatic encoding to minimise output size;
- [Fast enough for lag-free live editing](https://qr.davidje13.com/docs/#comparison);
- Comprehensive automated testing;
- Multiple output formats
  ([canvas](https://qr.davidje13.com/docs/#Bitmap2D_toCanvas) /
  [PNG](https://qr.davidje13.com/docs/#toPngBuffer) /
  [SVG](https://qr.davidje13.com/docs/#toSvgSource) /
  [text](https://qr.davidje13.com/docs/#Bitmap2D_toString)).

You can [see it in action online](https://qr.davidje13.com/), or try it from the
terminal:

```shell
npx lean-qr 'MY MESSAGE HERE'
```

## Basic Usage

See the [Quickstart](https://qr.davidje13.com/docs/#quickstart) documentation
for Node.js / Browser / React / Preact / Vue / Web Components / CLI usage.

```shell
npm install --save lean-qr
```

```html
<canvas id="my-qr-code"></canvas>

<style>
  #my-qr-code {
    image-rendering: pixelated;
    width: 100%;
  }
</style>
```

```javascript
import { generate } from 'lean-qr';

const qrCode = generate('LEAN-QR LIBRARY');
qrCode.toCanvas(document.getElementById('my-qr-code'));
```

## Troubleshooting

### Jest / JSDOM error: "TextEncoder is not defined"

There is a [bug in JSDOM](https://github.com/jsdom/jsdom/issues/2524) which
prevents this class being available in tests. This issue only affects JSDOM, and
can be fixed with:

```js
import { TextEncoder, TextDecoder } from 'node:util';
global.TextEncoder = TextEncoder;
global.TextDecoder = TextDecoder;
```

You can put this in the test file or in Jest's `setupFilesAfterEnv` config. You
should _not_ put this in your application code.

## Full Documentation

- [Quickstart](https://qr.davidje13.com/docs/#quickstart)
- [API Reference](https://qr.davidje13.com/docs/#api)
- [Source Code](https://github.com/davidje13/lean-qr)
- [Comparison With Other Libraries](https://qr.davidje13.com/docs/#comparison)
