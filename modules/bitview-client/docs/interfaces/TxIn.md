[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [modules/bitview-client/index.js:1358](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1358)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1367](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1367)

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1368](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1368)

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [modules/bitview-client/index.js:1365](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1365)

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [modules/bitview-client/index.js:1361](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1361)

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [modules/bitview-client/index.js:1362](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1362)

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [modules/bitview-client/index.js:1363](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1363)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [modules/bitview-client/index.js:1366](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1366)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1359](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1359)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [modules/bitview-client/index.js:1360](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1360)

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: [modules/bitview-client/index.js:1364](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1364)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
