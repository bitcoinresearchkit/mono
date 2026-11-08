[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [modules/bitview-client/index.js:1402](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1402)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1411](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1411)

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1412](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1412)

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [modules/bitview-client/index.js:1409](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1409)

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [modules/bitview-client/index.js:1405](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1405)

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [modules/bitview-client/index.js:1406](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1406)

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [modules/bitview-client/index.js:1407](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1407)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [modules/bitview-client/index.js:1410](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1410)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1403](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1403)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [modules/bitview-client/index.js:1404](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1404)

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: [modules/bitview-client/index.js:1408](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1408)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
