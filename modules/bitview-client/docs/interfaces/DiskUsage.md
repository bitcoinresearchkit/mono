[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DiskUsage

# Interface: DiskUsage

Defined in: [modules/bitview-client/index.js:496](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L496)

## Properties

### bitcoin

> **bitcoin**: `string`

Defined in: [modules/bitview-client/index.js:499](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L499)

Human-readable Bitcoin blocks directory size

***

### bitcoinBytes

> **bitcoinBytes**: `number`

Defined in: [modules/bitview-client/index.js:500](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L500)

Bitcoin blocks directory size in bytes

***

### brk

> **brk**: `string`

Defined in: [modules/bitview-client/index.js:497](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L497)

Human-readable brk data size (e.g., "48.8 GiB")

***

### brkBytes

> **brkBytes**: `number`

Defined in: [modules/bitview-client/index.js:498](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L498)

brk data size in bytes

***

### ratio

> **ratio**: `number`

Defined in: [modules/bitview-client/index.js:501](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L501)

Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
