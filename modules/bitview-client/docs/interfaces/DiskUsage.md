[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DiskUsage

# Interface: DiskUsage

Defined in: [modules/bitview-client/index.js:542](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L542)

## Properties

### bitcoin

> **bitcoin**: `string`

Defined in: [modules/bitview-client/index.js:545](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L545)

Human-readable Bitcoin blocks directory size

***

### bitcoinBytes

> **bitcoinBytes**: `number`

Defined in: [modules/bitview-client/index.js:546](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L546)

Bitcoin blocks directory size in bytes

***

### brk

> **brk**: `string`

Defined in: [modules/bitview-client/index.js:543](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L543)

Human-readable brk data size (e.g., "48.8 GiB")

***

### brkBytes

> **brkBytes**: `number`

Defined in: [modules/bitview-client/index.js:544](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L544)

brk data size in bytes

***

### ratio

> **ratio**: `number`

Defined in: [modules/bitview-client/index.js:547](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L547)

Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
