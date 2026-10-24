[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [modules/bitview-client/index.js:1408](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1408)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1417](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1417)

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1418](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1418)

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [modules/bitview-client/index.js:1415](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1415)

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [modules/bitview-client/index.js:1411](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1411)

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [modules/bitview-client/index.js:1412](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1412)

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [modules/bitview-client/index.js:1413](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1413)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [modules/bitview-client/index.js:1416](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1416)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1409](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1409)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [modules/bitview-client/index.js:1410](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1410)

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: [modules/bitview-client/index.js:1414](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1414)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
