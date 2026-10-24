[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ValidateAddressParam

# Interface: ValidateAddressParam

Defined in: [modules/bitview-client/index.js:1580](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1580)

## Properties

### address

> **address**: `string`

Defined in: [modules/bitview-client/index.js:1581](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1581)

Bitcoin address to validate: 2 to 130 letters and digits (a P2PK public key in hex is 130);
anything else answers 400, as mempool.space does
