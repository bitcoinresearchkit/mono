[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressValidation

# Interface: AddressValidation

Defined in: [modules/bitview-client/index.js:94](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L94)

## Properties

### address?

> `optional` **address?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:96](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L96)

The validated address

***

### error?

> `optional` **error?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:103](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L103)

Error message for invalid addresses

***

### errorLocations?

> `optional` **errorLocations?**: `number`[] \| `null`

Defined in: [modules/bitview-client/index.js:102](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L102)

Error locations: an empty array for invalid addresses, omitted for valid ones

***

### isscript?

> `optional` **isscript?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:98](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L98)

Whether the address is P2SH, P2WSH or P2TR

***

### isvalid

> **isvalid**: `boolean`

Defined in: [modules/bitview-client/index.js:95](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L95)

***

### iswitness?

> `optional` **iswitness?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:99](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L99)

***

### scriptPubKey?

> `optional` **scriptPubKey?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:97](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L97)

The scriptPubKey in hex

***

### witnessProgram?

> `optional` **witnessProgram?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:101](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L101)

Witness program in hex

***

### witnessVersion?

> `optional` **witnessVersion?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:100](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L100)

Witness version (0 for P2WPKH/P2WSH, 1 for P2TR and P2A)
