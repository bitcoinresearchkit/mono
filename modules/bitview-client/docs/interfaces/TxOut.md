[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1426](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1426)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1427](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1427)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1430](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1430)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1428](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1428)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1429](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1429)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1431](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1431)

Value of the output in satoshis.
