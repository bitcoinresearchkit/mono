[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OracleOutputs

# Interface: OracleOutputs

Defined in: modules/bitview-client/index.js:954

## Properties

### coinbase

> **coinbase**: `number`

Defined in: modules/bitview-client/index.js:955

The coinbase's: new coins, not payments.

***

### dust

> **dust**: `number`

Defined in: modules/bitview-client/index.js:960

Under 1,000 sats.

***

### fanOut

> **fanOut**: `number`

Defined in: modules/bitview-client/index.js:956

Those of transactions with more outputs than the fan-out cap (250; 100 before height
630,000): batch payouts.

***

### opReturn

> **opReturn**: `number`

Defined in: modules/bitview-client/index.js:958

Those of transactions carrying an OP_RETURN output: data, not payments.

***

### payments

> **payments**: `number`

Defined in: modules/bitview-client/index.js:962

The rest: the payments it counted.

***

### roundBtc

> **roundBtc**: `number`

Defined in: modules/bitview-client/index.js:961

Round amounts in bitcoin (within 0.1% of 1, 2, 3, 5 or 6 times a power of ten sats).

***

### taproot

> **taproot**: `number`

Defined in: modules/bitview-client/index.js:959

Taproot outputs: mostly protocol traffic.
