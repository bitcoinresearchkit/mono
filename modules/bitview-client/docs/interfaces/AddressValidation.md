[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressValidation

# Interface: AddressValidation

Defined in: [modules/bitview-client/index.js:89](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L89)

## Properties

### address?

> `optional` **address?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:91](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L91)

The validated address

***

### error?

> `optional` **error?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:98](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L98)

Error message for invalid addresses

***

### errorLocations?

> `optional` **errorLocations?**: `number`[] \| `null`

Defined in: [modules/bitview-client/index.js:97](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L97)

Error locations: an empty array for invalid addresses, omitted for valid ones

***

### isscript?

> `optional` **isscript?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:93](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L93)

Whether the address is P2SH, P2WSH or P2TR

***

### isvalid

> **isvalid**: `boolean`

Defined in: [modules/bitview-client/index.js:90](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L90)

***

### iswitness?

> `optional` **iswitness?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:94](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L94)

***

### scriptPubKey?

> `optional` **scriptPubKey?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:92](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L92)

The scriptPubKey in hex

***

### witnessProgram?

> `optional` **witnessProgram?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:96](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L96)

Witness program in hex

***

### witnessVersion?

> `optional` **witnessVersion?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:95](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L95)

Witness version (0 for P2WPKH/P2WSH, 1 for P2TR and P2A)
