[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DiskUsage

# Interface: DiskUsage

Defined in: [modules/bitview-client/index.js:548](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L548)

## Properties

### bitcoin

> **bitcoin**: `string`

Defined in: [modules/bitview-client/index.js:551](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L551)

Human-readable Bitcoin blocks directory size

***

### bitcoinBytes

> **bitcoinBytes**: `number`

Defined in: [modules/bitview-client/index.js:552](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L552)

Bitcoin blocks directory size in bytes

***

### brk

> **brk**: `string`

Defined in: [modules/bitview-client/index.js:549](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L549)

Human-readable brk data size (e.g., "48.8 GiB")

***

### brkBytes

> **brkBytes**: `number`

Defined in: [modules/bitview-client/index.js:550](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L550)

brk data size in bytes

***

### ratio

> **ratio**: `number`

Defined in: [modules/bitview-client/index.js:553](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L553)

Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
