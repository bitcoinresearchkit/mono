[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [modules/bitview-client/index.js:1381](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1381)

## Properties

### fee

> **fee**: `number`

Defined in: [modules/bitview-client/index.js:1391](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1391)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:1382](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1382)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [modules/bitview-client/index.js:1385](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1385)

***

### sigops

> **sigops**: `number`

Defined in: [modules/bitview-client/index.js:1390](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1390)

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: [modules/bitview-client/index.js:1388](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1388)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [modules/bitview-client/index.js:1392](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1392)

***

### txid

> **txid**: `string`

Defined in: [modules/bitview-client/index.js:1383](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1383)

***

### version

> **version**: `number`

Defined in: [modules/bitview-client/index.js:1384](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1384)

Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [modules/bitview-client/index.js:1386](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1386)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [modules/bitview-client/index.js:1387](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1387)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [modules/bitview-client/index.js:1389](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1389)
