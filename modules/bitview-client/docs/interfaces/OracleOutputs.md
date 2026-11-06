[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OracleOutputs

# Interface: OracleOutputs

Defined in: [modules/bitview-client/index.js:962](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L962)

## Properties

### coinbase

> **coinbase**: `number`

Defined in: [modules/bitview-client/index.js:963](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L963)

The coinbase's: new coins, not payments.

***

### dust

> **dust**: `number`

Defined in: [modules/bitview-client/index.js:968](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L968)

Under 1,000 sats.

***

### fanOut

> **fanOut**: `number`

Defined in: [modules/bitview-client/index.js:964](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L964)

Those of transactions with more outputs than the fan-out cap (250; 100 before height
  630,000): batch payouts.

***

### opReturn

> **opReturn**: `number`

Defined in: [modules/bitview-client/index.js:966](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L966)

Those of transactions carrying an OP_RETURN output: data, not payments.

***

### payments

> **payments**: `number`

Defined in: [modules/bitview-client/index.js:970](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L970)

The rest: the payments it counted.

***

### roundBtc

> **roundBtc**: `number`

Defined in: [modules/bitview-client/index.js:969](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L969)

Round amounts in bitcoin (within 0.1% of 1, 2, 3, 5 or 6 times a power of ten sats).

***

### taproot

> **taproot**: `number`

Defined in: [modules/bitview-client/index.js:967](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L967)

Taproot outputs: mostly protocol traffic.
