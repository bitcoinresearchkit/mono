[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / addressPayloadHashPrefix

# Function: addressPayloadHashPrefix()

> **addressPayloadHashPrefix**(`payload`, `nibbles`): `string`

Defined in: [modules/bitview-client/index.js:2458](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L2458)

Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{address_type}/{prefix}`.

## Parameters

### payload

`number`[] \| `ArrayBuffer` \| `Uint8Array`\<`ArrayBufferLike`\> \| `ArrayBufferView`\<`ArrayBufferLike`\>

### nibbles

`number`

Prefix length from 1 to 16 hex nibbles

## Returns

`string`
