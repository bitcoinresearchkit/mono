[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OracleBlock

# Interface: OracleBlock

Defined in: modules/bitview-client/index.js:919

## Properties

### bin

> **bin**: `number`

Defined in: modules/bitview-client/index.js:926

The bin it picked, refined between bins by a parabola through the best score and its
neighbours.

***

### firstBin

> **firstBin**: `number`

Defined in: modules/bitview-client/index.js:928

The search window's first candidate bin: 12 below the block before's answer, rounded, 24
candidates in all.

***

### hash

> **hash**: `string`

Defined in: modules/bitview-client/index.js:921

The confirmed block it was read at: this block's hash; live with the mempool, the tip's
(block `height - 1`), the one the mempool is blended onto.

***

### height

> **height**: `number`

Defined in: modules/bitview-client/index.js:920

The block's height; live with the mempool, the next block's.

***

### histogram

> **histogram**: `number`[]

Defined in: modules/bitview-client/index.js:935

The smoothed histogram it scored, 2,400 bins: the block's payments and the 11 blocks'
before it (live, the mempool's as the next block's), each block's counts weighted
`(2/7) * (5/7)^age`, the newest at age 0.

***

### mempool

> **mempool**: `boolean`

Defined in: modules/bitview-client/index.js:923

Whether it's the live reading with the mempool blended in as the next block (`outputs` the
mempool's), rather than a confirmed block's.

***

### outputs

> **outputs**: [`OracleOutputs`](OracleOutputs.md)

Defined in: modules/bitview-client/index.js:938

The block's own outputs (live, the mempool's) as the oracle's filter judged them.

***

### price

> **price**: `number`

Defined in: modules/bitview-client/index.js:925

The price it found, in dollars: `bin` as a price, to the cent.

***

### scores

> **scores**: `number`[]

Defined in: modules/bitview-client/index.js:930

Each candidate's score, from `first_bin` on, unitless: for each of the 19 round-dollar
amounts ($1, 2, 3, 5, 10, 15, 20, 25, 30, 50, 100, 150, 200, 300, 500, 1k, 2k, 5k, 10k),
the histogram's count where that amount lands if $100 is at the candidate, over that
amount's peak in the window, summed. The highest is the pick's bin; all zero when the window
held no payments at all, and the oracle kept the block before's answer.
