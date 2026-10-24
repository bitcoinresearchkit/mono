[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DiskUsage

# Interface: DiskUsage

Defined in: [modules/bitview-client/index.js:552](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L552)

## Properties

### bitcoin

> **bitcoin**: `string`

Defined in: [modules/bitview-client/index.js:555](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L555)

Human-readable Bitcoin blocks directory size

***

### bitcoinBytes

> **bitcoinBytes**: `number`

Defined in: [modules/bitview-client/index.js:556](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L556)

Bitcoin blocks directory size in bytes

***

### brk

> **brk**: `string`

Defined in: [modules/bitview-client/index.js:553](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L553)

Human-readable brk data size (e.g., "48.8 GiB")

***

### brkBytes

> **brkBytes**: `number`

Defined in: [modules/bitview-client/index.js:554](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L554)

brk data size in bytes

***

### ratio

> **ratio**: `number`

Defined in: [modules/bitview-client/index.js:557](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L557)

Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
