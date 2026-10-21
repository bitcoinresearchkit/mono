[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [modules/bitview-client/index.js:1257](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1257)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1266](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1266)

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1267](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1267)

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [modules/bitview-client/index.js:1264](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1264)

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [modules/bitview-client/index.js:1260](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1260)

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [modules/bitview-client/index.js:1261](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1261)

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [modules/bitview-client/index.js:1262](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1262)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [modules/bitview-client/index.js:1265](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1265)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1258](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1258)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [modules/bitview-client/index.js:1259](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1259)

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: [modules/bitview-client/index.js:1263](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1263)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
