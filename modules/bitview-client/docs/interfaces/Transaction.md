[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [modules/bitview-client/index.js:1393](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1393)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1403](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1403)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1394](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1394)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [modules/bitview-client/index.js:1397](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1397)

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:1402](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1402)

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:1400](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1400)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [modules/bitview-client/index.js:1404](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1404)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1395](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1395)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1396](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1396)

Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [modules/bitview-client/index.js:1398](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1398)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [modules/bitview-client/index.js:1399](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1399)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:1401](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1401)
