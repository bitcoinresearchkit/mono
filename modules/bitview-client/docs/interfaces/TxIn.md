[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [modules/bitview-client/index.js:1347](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1347)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1356](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1356)

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1357](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1357)

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [modules/bitview-client/index.js:1354](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1354)

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [modules/bitview-client/index.js:1350](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1350)

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [modules/bitview-client/index.js:1351](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1351)

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [modules/bitview-client/index.js:1352](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1352)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [modules/bitview-client/index.js:1355](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1355)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1348](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1348)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [modules/bitview-client/index.js:1349](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1349)

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: [modules/bitview-client/index.js:1353](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1353)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
