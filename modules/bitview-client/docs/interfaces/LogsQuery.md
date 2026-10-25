[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / LogsQuery

# Interface: LogsQuery

Defined in: [modules/bitview-client/index.js:751](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L751)

## Properties

### after?

> `optional` **after?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:753](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L753)

Only the lines after this `seq`, when `started_unix_ms` is this process's; otherwise every kept line.

***

### startedUnixMs?

> `optional` **startedUnixMs?**: `number` \| `null`

Defined in: [modules/bitview-client/index.js:752](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L752)

The `started_unix_ms` of the last answer: `after` counts in that process's log.
