[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / BitviewError

# Class: BitviewError

Defined in: [modules/bitview-client/index.js:1607](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1607)

A failed request: the HTTP status and the server's error code and message, when it sent them.

## Extends

- `Error`

## Constructors

### Constructor

> **new BitviewError**(`message`, `status?`, `code?`): `BitviewError`

Defined in: [modules/bitview-client/index.js:1613](https://github.com/bitcoinresearchkit/mono/blob/35af38a95a9d8eab77d0bc6900ec332bc53accaf/modules/bitview-client/index.js#L1613)

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
