[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1285](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1285)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1286](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1286)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1289](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1289)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1287](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1287)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1288](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1288)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1290](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1290)

Value of the output in satoshis.
