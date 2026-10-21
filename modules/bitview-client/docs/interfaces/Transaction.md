[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [modules/bitview-client/index.js:1242](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1242)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1252](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1252)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1243](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1243)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [modules/bitview-client/index.js:1246](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1246)

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:1251](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1251)

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:1249](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1249)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [modules/bitview-client/index.js:1253](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1253)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1244](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1244)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1245](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1245)

Transaction version (raw i32 from Bitcoin protocol, may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [modules/bitview-client/index.js:1247](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1247)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [modules/bitview-client/index.js:1248](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1248)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:1250](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1250)
