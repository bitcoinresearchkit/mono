[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DiskUsage

# Interface: DiskUsage

Defined in: [modules/bitview-client/index.js:537](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L537)

## Properties

### bitcoin

> **bitcoin**: `string`

Defined in: [modules/bitview-client/index.js:540](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L540)

Human-readable Bitcoin blocks directory size

***

### bitcoinBytes

> **bitcoinBytes**: `number`

Defined in: [modules/bitview-client/index.js:541](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L541)

Bitcoin blocks directory size in bytes

***

### brk

> **brk**: `string`

Defined in: [modules/bitview-client/index.js:538](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L538)

Human-readable brk data size (e.g., "48.8 GiB")

***

### brkBytes

> **brkBytes**: `number`

Defined in: [modules/bitview-client/index.js:539](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L539)

brk data size in bytes

***

### ratio

> **ratio**: `number`

Defined in: [modules/bitview-client/index.js:542](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L542)

Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
