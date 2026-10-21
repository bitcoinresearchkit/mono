[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1275](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1275)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1276](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1276)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1279](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1279)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1277](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1277)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1278](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1278)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1280](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1280)

Value of the output in satoshis.
