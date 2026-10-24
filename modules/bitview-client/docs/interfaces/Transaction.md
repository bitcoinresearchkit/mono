[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [modules/bitview-client/index.js:1343](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1343)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1353](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1353)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1344](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1344)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [modules/bitview-client/index.js:1347](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1347)

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:1352](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1352)

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:1350](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1350)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [modules/bitview-client/index.js:1354](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1354)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1345](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1345)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1346](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1346)

Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [modules/bitview-client/index.js:1348](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1348)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [modules/bitview-client/index.js:1349](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1349)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:1351](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1351)
