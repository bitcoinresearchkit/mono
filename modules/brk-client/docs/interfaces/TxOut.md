[**brk-client**](../README.md)

***

[brk-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [Developer/brk/modules/brk-client/index.js:1370](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1370)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [Developer/brk/modules/brk-client/index.js:1371](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1371)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [Developer/brk/modules/brk-client/index.js:1374](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1374)

Bitcoin address, omitted for scripts without an address.

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [Developer/brk/modules/brk-client/index.js:1372](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1372)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [Developer/brk/modules/brk-client/index.js:1373](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1373)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [Developer/brk/modules/brk-client/index.js:1375](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L1375)

Value of the output in satoshis.
