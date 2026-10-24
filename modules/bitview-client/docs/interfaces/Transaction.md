[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [modules/bitview-client/index.js:1331](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1331)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1341](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1341)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1332](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1332)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [modules/bitview-client/index.js:1335](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1335)

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:1340](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1340)

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:1338](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1338)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [modules/bitview-client/index.js:1342](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1342)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1333](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1333)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1334](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1334)

Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [modules/bitview-client/index.js:1336](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1336)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [modules/bitview-client/index.js:1337](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1337)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:1339](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1339)
