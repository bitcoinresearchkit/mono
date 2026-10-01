[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxOut

# Interface: TxOut

Defined in: [Developer/mono/modules/bitview-client/index.js:1286](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1286)

## Properties

### scriptpubkey

> **scriptpubkey**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1287](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1287)

Script pubkey (locking script), encoded as hexadecimal.

***

### scriptpubkeyAddress?

> `optional` **scriptpubkeyAddress?**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1290](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1290)

Bitcoin address, omitted for scripts without an address.

***

### scriptpubkeyAsm

> **scriptpubkeyAsm**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1288](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1288)

Script pubkey in assembly format.

***

### scriptpubkeyType

> **scriptpubkeyType**: [`OutputTypeNormalized`](../type-aliases/OutputTypeNormalized.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:1289](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1289)

Esplora/mempool.space script type.

***

### value

> **value**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1291](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1291)

Value of the output in satoshis.
