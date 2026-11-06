[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Logs

# Interface: Logs

Defined in: [modules/bitview-client/index.js:675](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L675)

## Properties

### lines

> **lines**: [`LogLine`](LogLine.md)[]

Defined in: [modules/bitview-client/index.js:678](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L678)

***

### next

> **next**: `string`

Defined in: [modules/bitview-client/index.js:679](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L679)

Where this answer ends: sent back as `after`, the lines since. `{started_unix_ms}-{seq}`, the process and the
  `seq` of the next line it will log; to pass back as it is, not to read.

***

### startedUnixMs

> **startedUnixMs**: `number`

Defined in: [modules/bitview-client/index.js:676](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L676)

When this server process began its log, in Unix milliseconds: another than the last answer's shows the
  server restarted.
