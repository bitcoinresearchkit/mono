[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1374](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1374)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1375](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1375)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1378](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1378)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1376](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1376)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1377](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1377)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1379](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1379)

Value of the output in satoshis.
