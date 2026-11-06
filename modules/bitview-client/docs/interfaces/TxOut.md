[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [modules/bitview-client/index.js:1488](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1488)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [modules/bitview-client/index.js:1489](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1489)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [modules/bitview-client/index.js:1492](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1492)

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [modules/bitview-client/index.js:1490](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1490)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [modules/bitview-client/index.js:1491](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1491)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [modules/bitview-client/index.js:1493](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1493)

Value of the output in satoshis.
