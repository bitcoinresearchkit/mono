[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / Health

# Interface: Health

Defined in: modules/bitview-client/index.js:646

## Properties

### blocksBehind

> **blocksBehind**: `number`

Defined in: modules/bitview-client/index.js:657

Number of blocks behind the tip

***

### computedHeight

> **computedHeight**: `number`

Defined in: modules/bitview-client/index.js:654

Height of the last computed block (series)

***

### indexedHeight

> **indexedHeight**: `number`

Defined in: modules/bitview-client/index.js:653

Height of the last indexed block

***

### lastIndexedAt

> **lastIndexedAt**: `string`

Defined in: modules/bitview-client/index.js:658

Human-readable timestamp of the last indexed block (ISO 8601)

***

### lastIndexedAtUnix

> **lastIndexedAtUnix**: `number`

Defined in: modules/bitview-client/index.js:659

Unix timestamp of the last indexed block

***

### service

> **service**: `"brk"`

Defined in: modules/bitview-client/index.js:648

***

### startedAt

> **startedAt**: `string`

Defined in: modules/bitview-client/index.js:651

Server start time (ISO 8601)

***

### status

> **status**: `"healthy"`

Defined in: modules/bitview-client/index.js:647

***

### timestamp

> **timestamp**: `string`

Defined in: modules/bitview-client/index.js:650

Current server time (ISO 8601)

***

### tipHeight

> **tipHeight**: `number`

Defined in: modules/bitview-client/index.js:655

Height of the Bitcoin node's chain tip; `/health` reports `indexed_height` instead, without
asking the node

***

### uptimeSeconds

> **uptimeSeconds**: `number`

Defined in: modules/bitview-client/index.js:652

Uptime in seconds

***

### version

> **version**: `string`

Defined in: modules/bitview-client/index.js:649
