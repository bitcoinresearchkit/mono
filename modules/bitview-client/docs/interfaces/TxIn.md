[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [modules/bitview-client/index.js:1460](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1460)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1469](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1469)

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1470](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1470)

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [modules/bitview-client/index.js:1467](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1467)

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [modules/bitview-client/index.js:1463](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1463)

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [modules/bitview-client/index.js:1464](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1464)

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [modules/bitview-client/index.js:1465](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1465)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [modules/bitview-client/index.js:1468](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1468)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1461](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1461)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [modules/bitview-client/index.js:1462](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1462)

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: [modules/bitview-client/index.js:1466](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1466)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
