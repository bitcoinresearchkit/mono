[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1352](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1352)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1353](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1353)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1356](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1356)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1354](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1354)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1355](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1355)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1357](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1357)

Value of the output in satoshis.
