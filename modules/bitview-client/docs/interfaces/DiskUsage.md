[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DiskUsage

# Interface: DiskUsage

Defined in: [modules/bitview-client/index.js:488](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L488)

## Properties

### bitcoin

> **bitcoin**: `string`

Defined in: [modules/bitview-client/index.js:491](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L491)

Human-readable Bitcoin blocks directory size

***

### bitcoinBytes

> **bitcoinBytes**: `number`

Defined in: [modules/bitview-client/index.js:492](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L492)

Bitcoin blocks directory size in bytes

***

### brk

> **brk**: `string`

Defined in: [modules/bitview-client/index.js:489](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L489)

Human-readable brk data size (e.g., "48.8 GiB")

***

### brkBytes

> **brkBytes**: `number`

Defined in: [modules/bitview-client/index.js:490](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L490)

brk data size in bytes

***

### ratio

> **ratio**: `number`

Defined in: [modules/bitview-client/index.js:493](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L493)

Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
