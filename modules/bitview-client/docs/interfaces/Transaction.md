[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [modules/bitview-client/index.js:1304](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1304)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1314](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1314)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1305](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1305)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [modules/bitview-client/index.js:1308](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1308)

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:1313](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1313)

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:1311](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1311)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [modules/bitview-client/index.js:1315](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1315)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1306](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1306)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1307](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1307)

Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [modules/bitview-client/index.js:1309](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1309)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [modules/bitview-client/index.js:1310](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1310)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:1312](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1312)
