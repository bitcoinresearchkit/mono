[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1404](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1404)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1405](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1405)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1408](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1408)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1406](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1406)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1407](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1407)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1409](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1409)

Value of the output in satoshis.
