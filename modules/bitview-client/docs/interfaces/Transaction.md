[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [modules/bitview-client/index.js:1433](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1433)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1443](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1443)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1434](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1434)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [modules/bitview-client/index.js:1437](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1437)

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:1442](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1442)

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:1440](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1440)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [modules/bitview-client/index.js:1444](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1444)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1435](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1435)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1436](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1436)

Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [modules/bitview-client/index.js:1438](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1438)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [modules/bitview-client/index.js:1439](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1439)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:1441](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1441)
