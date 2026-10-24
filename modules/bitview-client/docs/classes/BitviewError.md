[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewError

# Class: BitviewError

Defined in: [modules/bitview-client/index.js:1707](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1707)

A failed request: the HTTP status and the server's error code and message, when it sent them.

## Extends

- `Error`

## Constructors

### Constructor

> **new BitviewError**(`message`, `status?`, `code?`): `BitviewError`

Defined in: [modules/bitview-client/index.js:1713](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L1713)

#### Parameters

##### message

`string`

##### status?

`number`

##### code?

[`ErrorCode`](../type-aliases/ErrorCode.md)

#### Returns

`BitviewError`

#### Overrides

`Error.constructor`
