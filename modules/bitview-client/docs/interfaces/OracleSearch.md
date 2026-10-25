[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OracleSearch

# Interface: OracleSearch

Defined in: [modules/bitview-client/index.js:945](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L945)

## Properties

### bin

> **bin**: `number`

Defined in: [modules/bitview-client/index.js:951](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L951)

The bin it picked, refined between bins: the block's price.

***

### firstBin

> **firstBin**: `number`

Defined in: [modules/bitview-client/index.js:947](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L947)

The window's first candidate bin, on the oracle's log scale (200 bins a decade; a bin is
where $100 lands in sats: the price is `10^(10 - bin / 200)` dollars).

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:946](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L946)

***

### histogram

> **histogram**: `number`[]

Defined in: [modules/bitview-client/index.js:952](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L952)

The smoothed histogram it scored: the payments of the block and the 11 before it (and,
live, the mempool's as the next block's), counted by amount in sats on the same scale (a
bin is `round(log10(sats) * 200)`), the newest weighing most.

***

### mempool

> **mempool**: `boolean`

Defined in: [modules/bitview-client/index.js:956](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L956)

Whether it's the live search with the mempool blended in as the next block (`height` the
next block's, `outputs` the mempool's), rather than a block's.

***

### outputs

> **outputs**: [`FilteredOutputs`](FilteredOutputs.md)

Defined in: [modules/bitview-client/index.js:955](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L955)

The block's own outputs (live, the mempool's) as the oracle's filter judged them.

***

### scores

> **scores**: `number`[]

Defined in: [modules/bitview-client/index.js:949](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L949)

Each candidate's score, from `first_bin` on: its round-dollar arms' counts, each over that
arm's peak in the window, summed. The highest is the pick's bin.
