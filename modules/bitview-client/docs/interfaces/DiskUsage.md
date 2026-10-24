[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DiskUsage

# Interface: DiskUsage

Defined in: [modules/bitview-client/index.js:557](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L557)

## Properties

### bitcoin

> **bitcoin**: `string`

Defined in: [modules/bitview-client/index.js:560](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L560)

Human-readable Bitcoin blocks directory size

***

### bitcoinBytes

> **bitcoinBytes**: `number`

Defined in: [modules/bitview-client/index.js:561](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L561)

Bitcoin blocks directory size in bytes

***

### brk

> **brk**: `string`

Defined in: [modules/bitview-client/index.js:558](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L558)

Human-readable brk data size (e.g., "48.8 GiB")

***

### brkBytes

> **brkBytes**: `number`

Defined in: [modules/bitview-client/index.js:559](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L559)

brk data size in bytes

***

### ratio

> **ratio**: `number`

Defined in: [modules/bitview-client/index.js:562](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L562)

Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
