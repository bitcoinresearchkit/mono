[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ValidateAddressParam

# Interface: ValidateAddressParam

Defined in: [modules/bitview-client/index.js:1586](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1586)

## Properties

### address

> **address**: `string`

Defined in: [modules/bitview-client/index.js:1587](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1587)

Bitcoin address to validate: 2 to 130 letters and digits (a P2PK public key in hex is 130);
anything else answers 400, as mempool.space does
