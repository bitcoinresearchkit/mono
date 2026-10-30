[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressValidation

# Interface: AddressValidation

Defined in: modules/bitview-client/index.js:94

## Properties

### address?

> `optional` **address?**: `string` \| `null`

Defined in: modules/bitview-client/index.js:96

The validated address

***

### error?

> `optional` **error?**: `string` \| `null`

Defined in: modules/bitview-client/index.js:103

Error message for invalid addresses

***

### errorLocations?

> `optional` **errorLocations?**: `number`[] \| `null`

Defined in: modules/bitview-client/index.js:102

Error locations: an empty array for invalid addresses, omitted for valid ones

***

### isscript?

> `optional` **isscript?**: `boolean` \| `null`

Defined in: modules/bitview-client/index.js:98

Whether the address is P2SH, P2WSH or P2TR

***

### isvalid

> **isvalid**: `boolean`

Defined in: modules/bitview-client/index.js:95

***

### iswitness?

> `optional` **iswitness?**: `boolean` \| `null`

Defined in: modules/bitview-client/index.js:99

***

### scriptPubKey?

> `optional` **scriptPubKey?**: `string` \| `null`

Defined in: modules/bitview-client/index.js:97

The scriptPubKey in hex

***

### witnessProgram?

> `optional` **witnessProgram?**: `string` \| `null`

Defined in: modules/bitview-client/index.js:101

Witness program in hex

***

### witnessVersion?

> `optional` **witnessVersion?**: `number` \| `null`

Defined in: modules/bitview-client/index.js:100

Witness version (0 for P2WPKH/P2WSH, 1 for P2TR and P2A)
