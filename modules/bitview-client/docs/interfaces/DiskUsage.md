[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DiskUsage

# Interface: DiskUsage

Defined in: [modules/bitview-client/index.js:572](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L572)

## Properties

### bitcoin

> **bitcoin**: `string`

Defined in: [modules/bitview-client/index.js:575](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L575)

Human-readable Bitcoin blocks directory size

***

### bitcoinBytes

> **bitcoinBytes**: `number`

Defined in: [modules/bitview-client/index.js:576](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L576)

Bitcoin blocks directory size in bytes

***

### brk

> **brk**: `string`

Defined in: [modules/bitview-client/index.js:573](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L573)

Human-readable brk data size (e.g., "48.8 GiB")

***

### brkBytes

> **brkBytes**: `number`

Defined in: [modules/bitview-client/index.js:574](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L574)

brk data size in bytes

***

### ratio

> **ratio**: `number`

Defined in: [modules/bitview-client/index.js:577](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L577)

Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
