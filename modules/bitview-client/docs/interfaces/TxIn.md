[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [modules/bitview-client/index.js:1324](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1324)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1333](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1333)

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1334](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1334)

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [modules/bitview-client/index.js:1331](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1331)

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [modules/bitview-client/index.js:1327](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1327)

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [modules/bitview-client/index.js:1328](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1328)

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [modules/bitview-client/index.js:1329](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1329)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [modules/bitview-client/index.js:1332](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1332)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1325](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1325)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [modules/bitview-client/index.js:1326](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1326)

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: [modules/bitview-client/index.js:1330](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1330)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
