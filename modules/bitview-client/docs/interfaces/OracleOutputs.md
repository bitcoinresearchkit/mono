[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OracleOutputs

# Interface: OracleOutputs

Defined in: [modules/bitview-client/index.js:870](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L870)

## Properties

### coinbase

> **coinbase**: `number`

Defined in: [modules/bitview-client/index.js:871](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L871)

The coinbase's: new coins, not payments.

***

### dust

> **dust**: `number`

Defined in: [modules/bitview-client/index.js:876](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L876)

Under 1,000 sats.

***

### fanOut

> **fanOut**: `number`

Defined in: [modules/bitview-client/index.js:872](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L872)

Those of transactions with more outputs than the fan-out cap (250; 100 before height
  630,000): batch payouts.

***

### opReturn

> **opReturn**: `number`

Defined in: [modules/bitview-client/index.js:874](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L874)

Those of transactions carrying an OP_RETURN output: data, not payments.

***

### payments

> **payments**: `number`

Defined in: [modules/bitview-client/index.js:878](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L878)

The rest: the payments it counted.

***

### roundBtc

> **roundBtc**: `number`

Defined in: [modules/bitview-client/index.js:877](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L877)

Round amounts in bitcoin (within 0.1% of 1, 2, 3, 5 or 6 times a power of ten sats).

***

### taproot

> **taproot**: `number`

Defined in: [modules/bitview-client/index.js:875](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L875)

Taproot outputs: mostly protocol traffic.
