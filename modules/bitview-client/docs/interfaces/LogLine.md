[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / LogLine

# Interface: LogLine

Defined in: [modules/bitview-client/index.js:741](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L741)

## Properties

### level

> **level**: [`LogLevel`](../type-aliases/LogLevel.md)

Defined in: [modules/bitview-client/index.js:744](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L744)

***

### seq

> **seq**: `number`

Defined in: [modules/bitview-client/index.js:742](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L742)

Its place in this process's log, from 0 with no gaps.

***

### text

> **text**: `string`

Defined in: [modules/bitview-client/index.js:745](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L745)

What the log files show after the level: a request's status, path and duration, or a message and its fields.

***

### unixMs

> **unixMs**: `number`

Defined in: [modules/bitview-client/index.js:743](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L743)

When it was logged, in Unix milliseconds.
