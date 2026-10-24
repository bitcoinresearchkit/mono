[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [modules/bitview-client/index.js:1361](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1361)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1371](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1371)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1362](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1362)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [modules/bitview-client/index.js:1365](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1365)

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:1370](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1370)

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:1368](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1368)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [modules/bitview-client/index.js:1372](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1372)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1363](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1363)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1364](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1364)

Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [modules/bitview-client/index.js:1366](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1366)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [modules/bitview-client/index.js:1367](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1367)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:1369](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1369)
