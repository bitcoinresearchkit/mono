[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1343](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1343)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1344](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1344)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1347](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1347)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1345](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1345)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1346](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1346)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1348](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1348)

Value of the output in satoshis.
