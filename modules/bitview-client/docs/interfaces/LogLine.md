[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / LogLine

# Interface: LogLine

Defined in: [modules/bitview-client/index.js:666](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L666)

## Properties

### level

> **level**: [`LogLevel`](../type-aliases/LogLevel.md)

Defined in: [modules/bitview-client/index.js:669](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L669)

***

### seq

> **seq**: `number`

Defined in: [modules/bitview-client/index.js:667](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L667)

Its place in this process's log, from 0 with no gaps.

***

### text

> **text**: `string`

Defined in: [modules/bitview-client/index.js:670](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L670)

What the log files show after the level: a request's status, path and duration, or a message and its fields.

***

### unixMs

> **unixMs**: `number`

Defined in: [modules/bitview-client/index.js:668](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L668)

When it was logged, in Unix milliseconds.
