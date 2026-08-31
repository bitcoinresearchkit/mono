[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / TxIn

# Interface: TxIn

Defined in: [Developer/mono/modules/bitview-client/index.js:1395](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1395)

## Properties

### innerRedeemscriptAsm?

> `optional` **innerRedeemscriptAsm?**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1404](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1404)

Inner redeemscript in assembly (for P2SH-wrapped SegWit: scriptsig + witness both present)

***

### innerWitnessscriptAsm?

> `optional` **innerWitnessscriptAsm?**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1405](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1405)

Inner witnessscript in assembly (for P2WSH: last witness item decoded as script)

***

### isCoinbase

> **isCoinbase**: `boolean`

Defined in: [Developer/mono/modules/bitview-client/index.js:1402](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1402)

Whether this input is a coinbase (block reward) input

***

### prevout

> **prevout**: [`TxOut`](TxOut.md) \| `null`

Defined in: [Developer/mono/modules/bitview-client/index.js:1398](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1398)

Information about the previous output being spent

***

### scriptsig

> **scriptsig**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1399](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1399)

Signature script (hex, for non-SegWit inputs)

***

### scriptsigAsm

> **scriptsigAsm**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1400](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1400)

Signature script in assembly format

***

### sequence

> **sequence**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1403](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1403)

Input sequence number

***

### txid

> **txid**: `string`

Defined in: [Developer/mono/modules/bitview-client/index.js:1396](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1396)

Transaction ID of the output being spent

***

### vout

> **vout**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1397](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1397)

Output index being spent (u16: coinbase is 65535, mempool.space uses u32: 4294967295)

***

### witness?

> `optional` **witness?**: [`Witness`](../type-aliases/Witness.md)

Defined in: [Developer/mono/modules/bitview-client/index.js:1401](https://github.com/bitcoinresearchkit/brk/blob/2470b9cb6cd0af501e879a2573eeea5f2a3f3bd0/modules/bitview-client/index.js#L1401)

Witness data (stack items, present for SegWit inputs; hex-encoded on the wire)
