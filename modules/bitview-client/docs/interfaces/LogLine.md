[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / LogLine

# Interface: LogLine

Defined in: [modules/bitview-client/index.js:760](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L760)

## Properties

### level

> **level**: [`LogLevel`](../type-aliases/LogLevel.md)

Defined in: [modules/bitview-client/index.js:763](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L763)

***

### seq

> **seq**: `number`

Defined in: [modules/bitview-client/index.js:761](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L761)

Its place in this process's log, from 0 with no gaps.

***

### text

> **text**: `string`

Defined in: [modules/bitview-client/index.js:764](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L764)

What the log files show after the level: a request's status, path and duration, or a message and its fields.

***

### unixMs

> **unixMs**: `number`

Defined in: [modules/bitview-client/index.js:762](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L762)

When it was logged, in Unix milliseconds.
