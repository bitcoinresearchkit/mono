[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [modules/bitview-client/index.js:1411](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1411)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1421](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1421)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1412](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1412)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [modules/bitview-client/index.js:1415](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1415)

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:1420](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1420)

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:1418](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1418)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [modules/bitview-client/index.js:1422](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1422)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1413](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1413)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1414](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1414)

Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [modules/bitview-client/index.js:1416](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1416)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [modules/bitview-client/index.js:1417](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1417)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:1419](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1419)
