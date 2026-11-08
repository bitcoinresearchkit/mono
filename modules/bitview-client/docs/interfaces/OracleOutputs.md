[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OracleOutputs

# Interface: OracleOutputs

Defined in: [modules/bitview-client/index.js:936](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L936)

## Properties

### coinbase

> **coinbase**: `number`

Defined in: [modules/bitview-client/index.js:937](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L937)

The coinbase's: new coins, not payments.

***

### dust

> **dust**: `number`

Defined in: [modules/bitview-client/index.js:942](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L942)

Under 1,000 sats.

***

### fanOut

> **fanOut**: `number`

Defined in: [modules/bitview-client/index.js:938](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L938)

Those of transactions with more outputs than the fan-out cap (250; 100 before height
  630,000): batch payouts.

***

### opReturn

> **opReturn**: `number`

Defined in: [modules/bitview-client/index.js:940](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L940)

Those of transactions carrying an OP_RETURN output: data, not payments.

***

### payments

> **payments**: `number`

Defined in: [modules/bitview-client/index.js:944](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L944)

The rest: the payments it counted.

***

### roundBtc

> **roundBtc**: `number`

Defined in: [modules/bitview-client/index.js:943](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L943)

Round amounts in bitcoin (within 0.1% of 1, 2, 3, 5 or 6 times a power of ten sats).

***

### taproot

> **taproot**: `number`

Defined in: [modules/bitview-client/index.js:941](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L941)

Taproot outputs: mostly protocol traffic.
