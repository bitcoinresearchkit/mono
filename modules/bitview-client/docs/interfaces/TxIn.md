[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [modules/bitview-client/index.js:1376](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1376)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1385](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1385)

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1386](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1386)

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [modules/bitview-client/index.js:1383](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1383)

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [modules/bitview-client/index.js:1379](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1379)

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [modules/bitview-client/index.js:1380](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1380)

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [modules/bitview-client/index.js:1381](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1381)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [modules/bitview-client/index.js:1384](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1384)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1377](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1377)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [modules/bitview-client/index.js:1378](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1378)

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: [modules/bitview-client/index.js:1382](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1382)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
