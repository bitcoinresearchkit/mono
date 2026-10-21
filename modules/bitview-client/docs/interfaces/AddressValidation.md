[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressValidation

# Interface: AddressValidation

Defined in: [modules/bitview-client/index.js:69](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L69)

## Properties

### address?

> `optional` **address?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:71](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L71)

The validated address

***

### error?

> `optional` **error?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:78](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L78)

Error message for invalid addresses

***

### errorLocations?

> `optional` **errorLocations?**: `number`[] \| `null`

Defined in: [modules/bitview-client/index.js:77](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L77)

Error locations: an empty array for invalid addresses, omitted for valid ones

***

### isscript?

> `optional` **isscript?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:73](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L73)

Whether the address is P2SH, P2WSH or P2TR

***

### isvalid

> **isvalid**: `boolean`

Defined in: [modules/bitview-client/index.js:70](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L70)

***

### iswitness?

> `optional` **iswitness?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:74](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L74)

***

### scriptPubKey?

> `optional` **scriptPubKey?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:72](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L72)

The scriptPubKey in hex

***

### witnessProgram?

> `optional` **witnessProgram?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:76](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L76)

Witness program in hex

***

### witnessVersion?

> `optional` **witnessVersion?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:75](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L75)

Witness version (0 for P2WPKH/P2WSH, 1 for P2TR and P2A)
