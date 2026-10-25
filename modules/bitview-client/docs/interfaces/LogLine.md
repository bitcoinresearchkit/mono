[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / LogLine

# Interface: LogLine

Defined in: [modules/bitview-client/index.js:735](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L735)

## Properties

### level

> **level**: [`LogLevel`](../type-aliases/LogLevel.md)

Defined in: [modules/bitview-client/index.js:738](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L738)

***

### seq

> **seq**: `number`

Defined in: [modules/bitview-client/index.js:736](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L736)

Its place in this process's log, from 0 with no gaps.

***

### text

> **text**: `string`

Defined in: [modules/bitview-client/index.js:739](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L739)

What the log files show after the level: a request's status, path and duration, or a message and its fields.

***

### unixMs

> **unixMs**: `number`

Defined in: [modules/bitview-client/index.js:737](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L737)

When it was logged, in Unix milliseconds.
