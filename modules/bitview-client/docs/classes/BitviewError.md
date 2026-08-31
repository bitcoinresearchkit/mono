[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewError

# Class: BitviewError

Defined in: [Developer/mono/modules/bitview-client/index.js:1675](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1675)

Custom error class for Bitview client errors

## Extends

- `Error`

## Constructors

### Constructor

> **new BitviewError**(`message`, `status?`): `BitviewError`

Defined in: [Developer/mono/modules/bitview-client/index.js:1680](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1680)

#### Parameters

##### message

`string`

##### status?

`number`

#### Returns

`BitviewError`

#### Overrides

`Error.constructor`

## Methods

### isError()

> `static` **isError**(`error`): `error is Error`

Defined in: .npm/\_npx/940582f83630445a/node\_modules/typescript/lib/lib.esnext.error.d.ts:23

Indicates whether the argument provided is a built-in Error instance or not.

#### Parameters

##### error

`unknown`

#### Returns

`error is Error`

#### Inherited from

`Error.isError`
