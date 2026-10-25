[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1454](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1454)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1455](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1455)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1458](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1458)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1456](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1456)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1457](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1457)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1459](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1459)

Value of the output in satoshis.
