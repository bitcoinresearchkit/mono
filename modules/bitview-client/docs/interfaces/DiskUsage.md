[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DiskUsage

# Interface: DiskUsage

Defined in: [Developer/mono/modules/bitview-client/index.js:546](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L546)

## Properties

### bitcoin

> **bitcoin**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:549](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L549)

Human-readable Bitcoin blocks directory size

***

### bitcoinBytes

> **bitcoinBytes**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:550](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L550)

Bitcoin blocks directory size in bytes

***

### brk

> **brk**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:547](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L547)

Human-readable brk data size (e.g., "48.8 GiB")

***

### brkBytes

> **brkBytes**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:548](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L548)

brk data size in bytes

***

### ratio

> **ratio**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:551](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L551)

Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
