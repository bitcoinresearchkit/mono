[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewError

# Class: BitviewError

Defined in: [Developer/brk/modules/bitview-client/index.js:1624](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L1624)

Custom error class for Bitview client errors

## Extends

- `Error`

## Constructors

### Constructor

> **new BitviewError**(`message`, `status?`): `BitviewError`

Defined in: [Developer/brk/modules/bitview-client/index.js:1629](https://github.com/bitcoinresearchkit/brk/blob/70a3e90b82b397fa1d5ca46c4ed613066ed3d195/modules/bitview-client/index.js#L1629)

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
