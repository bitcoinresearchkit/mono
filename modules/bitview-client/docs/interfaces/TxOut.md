[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1386](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1386)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1387](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1387)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1390](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1390)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1388](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1388)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1389](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1389)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1391](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1391)

Value of the output in satoshis.
