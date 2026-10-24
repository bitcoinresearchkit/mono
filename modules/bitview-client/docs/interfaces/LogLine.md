[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / LogLine

# Interface: LogLine

Defined in: [modules/bitview-client/index.js:745](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L745)

## Properties

### level

> **level**: [`LogLevel`](../type-aliases/LogLevel.md)

Defined in: [modules/bitview-client/index.js:748](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L748)

***

### seq

> **seq**: `number`

Defined in: [modules/bitview-client/index.js:746](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L746)

Its place in this process's log, from 0 with no gaps.

***

### text

> **text**: `string`

Defined in: [modules/bitview-client/index.js:749](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L749)

What the log files show after the level: a request's status, path and duration, or a message and its fields.

***

### unixMs

> **unixMs**: `number`

Defined in: [modules/bitview-client/index.js:747](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L747)

When it was logged, in Unix milliseconds.
