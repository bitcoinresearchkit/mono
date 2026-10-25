[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [modules/bitview-client/index.js:1448](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1448)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1457](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1457)

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1458](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1458)

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [modules/bitview-client/index.js:1455](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1455)

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [modules/bitview-client/index.js:1451](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1451)

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [modules/bitview-client/index.js:1452](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1452)

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [modules/bitview-client/index.js:1453](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1453)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [modules/bitview-client/index.js:1456](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1456)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1449](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1449)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [modules/bitview-client/index.js:1450](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1450)

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: [modules/bitview-client/index.js:1454](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1454)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
