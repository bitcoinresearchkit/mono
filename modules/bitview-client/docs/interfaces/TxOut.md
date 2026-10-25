[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1476](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1476)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1477](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1477)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1480](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1480)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1478](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1478)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1479](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1479)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1481](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1481)

Value of the output in satoshis.
