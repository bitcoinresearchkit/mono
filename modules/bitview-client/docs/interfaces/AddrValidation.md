[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / AddrValidation

# Interface: AddrValidation

Defined in: [Developer/brk/modules/bitview-client/index.js:87](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L87)

## Properties

### address?

> `optional` **address?**: `string` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:89](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L89)

The validated address

***

### error?

> `optional` **error?**: `string` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:96](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L96)

Error message for invalid addresses

***

### errorLocations?

> `optional` **errorLocations?**: `number`[] \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:95](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L95)

Error locations (empty array for most errors)

***

### isscript?

> `optional` **isscript?**: `boolean` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:91](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L91)

Whether this is a script address (P2SH)

***

### isvalid

> **isvalid**: `boolean`

Defined in: [Developer/brk/modules/bitview-client/index.js:88](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L88)

Whether the address is valid

***

### iswitness?

> `optional` **iswitness?**: `boolean` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:92](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L92)

Whether this is a witness address

***

### scriptPubKey?

> `optional` **scriptPubKey?**: `string` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:90](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L90)

The scriptPubKey in hex

***

### witnessProgram?

> `optional` **witnessProgram?**: `string` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:94](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L94)

Witness program in hex

***

### witnessVersion?

> `optional` **witnessVersion?**: `number` \| `null`

Defined in: [Developer/brk/modules/bitview-client/index.js:93](https://github.com/bitcoinresearchkit/brk/blob/a6ece0db382228669ac4d1a9c673190fe7a4897b/modules/bitview-client/index.js#L93)

Witness version (0 for P2WPKH/P2WSH, 1 for P2TR)
