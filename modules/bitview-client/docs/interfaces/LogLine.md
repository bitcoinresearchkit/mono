[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / LogLine

# Interface: LogLine

Defined in: [modules/bitview-client/index.js:730](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L730)

## Properties

### level

> **level**: [`LogLevel`](../type-aliases/LogLevel.md)

Defined in: [modules/bitview-client/index.js:733](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L733)

***

### seq

> **seq**: `number`

Defined in: [modules/bitview-client/index.js:731](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L731)

Its place in this process's log, from 0 with no gaps.

***

### text

> **text**: `string`

Defined in: [modules/bitview-client/index.js:734](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L734)

What the log files show after the level: a request's status, path and duration, or a message and its fields.

***

### unixMs

> **unixMs**: `number`

Defined in: [modules/bitview-client/index.js:732](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L732)

When it was logged, in Unix milliseconds.
