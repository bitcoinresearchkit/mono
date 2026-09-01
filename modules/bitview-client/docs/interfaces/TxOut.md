[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [Developer/mono/modules/bitview-client/index.js:1421](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1421)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1422](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1422)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1425](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1425)

Bitcoin address, omitted for scripts without an address.

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1423](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1423)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:1424](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1424)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1426](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1426)

Value of the output in satoshis.
