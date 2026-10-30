[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: modules/bitview-client/index.js:1480

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: modules/bitview-client/index.js:1481

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: modules/bitview-client/index.js:1484

Bitcoin address, omitted for scripts without an address (P2PK included, as Esplora).

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: modules/bitview-client/index.js:1482

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: modules/bitview-client/index.js:1483

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: modules/bitview-client/index.js:1485

Value of the output in satoshis.
