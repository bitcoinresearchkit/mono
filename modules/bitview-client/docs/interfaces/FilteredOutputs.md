[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / FilteredOutputs

# Interface: FilteredOutputs

Defined in: [modules/bitview-client/index.js:622](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L622)

## Properties

### coinbase

> **coinbase**: `number`

Defined in: [modules/bitview-client/index.js:623](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L623)

The coinbase's: new coins, not payments.

***

### dust

> **dust**: `number`

Defined in: [modules/bitview-client/index.js:628](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L628)

Under 1,000 sats.

***

### fanOut

> **fanOut**: `number`

Defined in: [modules/bitview-client/index.js:624](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L624)

Those of transactions with more outputs than the fan-out cap (250; 100 before height
630,000): batch payouts.

***

### opReturn

> **opReturn**: `number`

Defined in: [modules/bitview-client/index.js:626](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L626)

Those of transactions carrying an OP_RETURN output: data, not payments.

***

### payments

> **payments**: `number`

Defined in: [modules/bitview-client/index.js:630](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L630)

The rest: the payments it counted.

***

### roundBtc

> **roundBtc**: `number`

Defined in: [modules/bitview-client/index.js:629](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L629)

Round amounts in bitcoin (within 0.1% of 1, 2, 3, 5 or 6 times a power of ten sats).

***

### taproot

> **taproot**: `number`

Defined in: [modules/bitview-client/index.js:627](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L627)

Taproot outputs: mostly protocol traffic.
