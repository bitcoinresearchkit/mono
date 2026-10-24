[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / OracleSearch

# Interface: OracleSearch

Defined in: [modules/bitview-client/index.js:930](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L930)

## Properties

### bin

> **bin**: `number`

Defined in: [modules/bitview-client/index.js:936](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L936)

The bin it picked, refined between bins: the block's price.

***

### firstBin

> **firstBin**: `number`

Defined in: [modules/bitview-client/index.js:932](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L932)

The window's first candidate bin, on the oracle's log scale (200 bins a decade; a bin is
where $100 lands in sats: the price is `10^(10 - bin / 200)` dollars).

***

### height

> **height**: `number`

Defined in: [modules/bitview-client/index.js:931](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L931)

***

### histogram

> **histogram**: `number`[]

Defined in: [modules/bitview-client/index.js:937](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L937)

The smoothed histogram it scored: the payments of the block and the 11 before it (and,
live, the mempool's as the next block's), counted by amount in sats on the same scale (a
bin is `round(log10(sats) * 200)`), the newest weighing most.

***

### scores

> **scores**: `number`[]

Defined in: [modules/bitview-client/index.js:934](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L934)

Each candidate's score, from `first_bin` on: its round-dollar arms' counts, each over that
arm's peak in the window, summed. The highest is the pick's bin.
