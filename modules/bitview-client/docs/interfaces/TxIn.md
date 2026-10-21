[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [modules/bitview-client/index.js:1247](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1247)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1256](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1256)

Inner redeemscript in assembly (P2SH inputs)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [modules/bitview-client/index.js:1257](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1257)

Inner witnessscript in assembly (P2WSH and P2SH-P2WSH: the witness script; P2TR script path: the leaf script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [modules/bitview-client/index.js:1254](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1254)

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [modules/bitview-client/index.js:1250](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1250)

The output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [modules/bitview-client/index.js:1251](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1251)

Signature script in hex (empty for native SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [modules/bitview-client/index.js:1252](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1252)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [modules/bitview-client/index.js:1255](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1255)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1248](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1248)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [modules/bitview-client/index.js:1249](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1249)

Output index being spent (coinbase: 4294967295)

***

### witness?

> `optional` **witness?**: `string`[]

Defined in: [modules/bitview-client/index.js:1253](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1253)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
