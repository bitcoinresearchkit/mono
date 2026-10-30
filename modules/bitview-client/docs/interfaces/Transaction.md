[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Transaction

# Interface: Transaction

Defined in: modules/bitview-client/index.js:1437

## Properties

### fee

> **fee**: `number`

Defined in: modules/bitview-client/index.js:1447

Transaction fee in satoshis

***

### index?

> `optional` **index?**: `number` \| `null`

Defined in: modules/bitview-client/index.js:1438

Internal transaction index (brk-specific, not in mempool.space)

***

### locktime

> **locktime**: `number`

Defined in: modules/bitview-client/index.js:1441

***

### sigops

> **sigops**: `number`

Defined in: modules/bitview-client/index.js:1446

BIP-141 signature-operation cost

***

### size

> **size**: `number`

Defined in: modules/bitview-client/index.js:1444

Transaction size in bytes

***

### status

> **status**: [`TxStatus`](TxStatus.md)

Defined in: modules/bitview-client/index.js:1448

***

### txid

> **txid**: `string`

Defined in: modules/bitview-client/index.js:1439

***

### version

> **version**: `number`

Defined in: modules/bitview-client/index.js:1440

Transaction version (the protocol's 32 bits read unsigned, as Esplora reports them; may contain non-standard values in coinbase txs)

***

### vin

> **vin**: [`TxIn`](TxIn.md)[]

Defined in: modules/bitview-client/index.js:1442

Transaction inputs

***

### vout

> **vout**: [`TxOut`](TxOut.md)[]

Defined in: modules/bitview-client/index.js:1443

Transaction outputs

***

### weight

> **weight**: `number`

Defined in: modules/bitview-client/index.js:1445
