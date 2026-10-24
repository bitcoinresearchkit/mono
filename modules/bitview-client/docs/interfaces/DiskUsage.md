[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DiskUsage

# Interface: DiskUsage

Defined in: [modules/bitview-client/index.js:532](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L532)

## Properties

### bitcoin

> **bitcoin**: `string`

Defined in: [modules/bitview-client/index.js:535](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L535)

Human-readable Bitcoin blocks directory size

***

### bitcoinBytes

> **bitcoinBytes**: `number`

Defined in: [modules/bitview-client/index.js:536](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L536)

Bitcoin blocks directory size in bytes

***

### brk

> **brk**: `string`

Defined in: [modules/bitview-client/index.js:533](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L533)

Human-readable brk data size (e.g., "48.8 GiB")

***

### brkBytes

> **brkBytes**: `number`

Defined in: [modules/bitview-client/index.js:534](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L534)

brk data size in bytes

***

### ratio

> **ratio**: `number`

Defined in: [modules/bitview-client/index.js:537](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L537)

Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
