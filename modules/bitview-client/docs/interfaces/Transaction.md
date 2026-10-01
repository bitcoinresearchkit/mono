[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: [Developer/mono/modules/bitview-client/index.js:1239](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1239)

## Properties

### fee

> **fee**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1249](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1249)

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: [Developer/mono/modules/bitview-client/index.js:1240](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1240)

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1243](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1243)

Transaction lock time

***

### sigops

> **sigops**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1248](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1248)

Number of signature operations

***

### size

> **size**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1246](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1246)

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:1250](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1250)

Confirmation status (confirmed, block height/hash/time)

***

### txid

> **txid**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1241](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1241)

Transaction ID

***

### version

> **version**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1242](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1242)

Transaction version (raw i32 from Bitcoin protocol, may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: [Developer/mono/modules/bitview-client/index.js:1244](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1244)

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: [Developer/mono/modules/bitview-client/index.js:1245](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1245)

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1247](https://github.com/bitcoinresearchkit/mono/blob/ea62dc574077b34b3299e636a16f81ea96849ead/modules/bitview-client/index.js#L1247)

Transaction weight
