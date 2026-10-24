[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1436](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1436)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1437](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1437)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1440](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1440)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1438](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1438)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1439](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1439)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1441](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1441)

Value of the output in satoshis.
