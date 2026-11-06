[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddressValidation

# Interface: AddressValidation

Defined in: [modules/bitview-client/index.js:59](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L59)

## Properties

### address?

> `optional` **address?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:61](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L61)

The validated address

***

### error?

> `optional` **error?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:69](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L69)

Error message for invalid addresses

***

### errorLocations?

> `optional` **errorLocations?**: `number`[] \| `null`

Defined in: [modules/bitview-client/index.js:67](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L67)

Positions of the characters at fault in an invalid address (Bitcoin Core's), possibly none;
  omitted for valid ones

***

### isscript?

> `optional` **isscript?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:63](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L63)

Whether the address is P2SH, P2WSH or P2TR

***

### isvalid

> **isvalid**: `boolean`

Defined in: [modules/bitview-client/index.js:60](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L60)

***

### iswitness?

> `optional` **iswitness?**: `boolean` \| `null`

Defined in: [modules/bitview-client/index.js:64](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L64)

***

### scriptPubKey?

> `optional` **scriptPubKey?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:62](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L62)

The scriptPubKey in hex

***

### witnessProgram?

> `optional` **witnessProgram?**: `string` \| `null`

Defined in: [modules/bitview-client/index.js:66](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L66)

Witness program in hex

***

### witnessVersion?

> `optional` **witnessVersion?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:65](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L65)

Witness version (0 for P2WPKH/P2WSH, 1 for P2TR, 2 to 16 for future versions; none for P2A)
