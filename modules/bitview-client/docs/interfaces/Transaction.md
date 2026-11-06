[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [modules/bitview-client/index.js:1445](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1445)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1455](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1455)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1446](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1446)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [modules/bitview-client/index.js:1449](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1449)

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:1454](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1454)

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:1452](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1452)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [modules/bitview-client/index.js:1456](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1456)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1447](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1447)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1448](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1448)

Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [modules/bitview-client/index.js:1450](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1450)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [modules/bitview-client/index.js:1451](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1451)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:1453](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1453)
