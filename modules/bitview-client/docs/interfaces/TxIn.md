[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: modules/bitview-client/index.js:1452

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: modules/bitview-client/index.js:1461

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: modules/bitview-client/index.js:1462

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: modules/bitview-client/index.js:1459

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: modules/bitview-client/index.js:1455

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: modules/bitview-client/index.js:1456

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: modules/bitview-client/index.js:1457

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: modules/bitview-client/index.js:1460

***

### txid

> **txid**: `string`

Defined in: modules/bitview-client/index.js:1453

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: modules/bitview-client/index.js:1454

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: modules/bitview-client/index.js:1458

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
