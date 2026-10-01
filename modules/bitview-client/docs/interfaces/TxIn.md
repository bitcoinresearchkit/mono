[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [Developer/mono/modules/bitview-client/index.js:1260](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1260)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1269](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1269)

Inner redeemscript in assembly (for P2SH-wrapped SegWit: scriptsig + witness both present)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1270](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1270)

Inner witnessscript in assembly (for P2WSH: last witness item decoded as script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [Developer/mono/modules/bitview-client/index.js:1267](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1267)

Whether this input is a coinbase (block reward) input

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [Developer/mono/modules/bitview-client/index.js:1263](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1263)

Information about the previous output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1264](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1264)

Signature script (hex, for non-SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1265](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1265)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1268](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1268)

Input sequence number

***

### txid

> **txid**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1261](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1261)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1262](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1262)

Output index being spent (u16: coinbase is 65535, mempool.space uses u32: 4294967295)

***

### witness?

> `optional` **witness?**: [`Witness`](../type-aliases/Witness.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:1266](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1266)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
