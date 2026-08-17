[**brk-client**](../README.md)

***

[brk-client](../globals.md) / addressPayloadHashPrefix

# Function: addressPayloadHashPrefix()

> **addressPayloadHashPrefix**(`payload`, `nibbles`): `string`

Defined in: [Developer/brk/modules/brk-client/index.js:2417](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/brk-client/index.js#L2417)

Compute the RapidHash v3 hash-prefix used by `/api/address/hash-prefix/{addr_type}/{prefix}`.

## Parameters

### payload

`number`[] \| `ArrayBuffer` \| `Uint8Array`\<`ArrayBufferLike`\> \| `ArrayBufferView`\<`ArrayBufferLike`\>

Raw address payload bytes

### nibbles

`number`

Prefix length from 1 to 16 hex nibbles

## Returns

`string`
