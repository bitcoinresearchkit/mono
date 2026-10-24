[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1380](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1380)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1381](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1381)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1384](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1384)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1382](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1382)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1383](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1383)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1385](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1385)

Value of the output in satoshis.
