[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DifficultyAdjustment

# Interface: DifficultyAdjustment

Defined in: [modules/bitview-client/index.js:501](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L501)

## Properties

### adjustedTimeAvg

> **adjustedTimeAvg**: `number`

Defined in: [modules/bitview-client/index.js:511](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L511)

Block time the projection uses (milliseconds): `timeAvg`, except in an epoch's first 503
blocks, where it spans the last 503 blocks, those before the retarget scaled by it

***

### difficultyChange

> **difficultyChange**: `number`

Defined in: [modules/bitview-client/index.js:503](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L503)

Estimated difficulty change at next retarget at the `adjustedTimeAvg` pace (%)

***

### estimatedRetargetDate

> **estimatedRetargetDate**: `number`

Defined in: [modules/bitview-client/index.js:504](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L504)

Estimated timestamp of next retarget (milliseconds)

***

### expectedBlocks

> **expectedBlocks**: `number`

Defined in: [modules/bitview-client/index.js:515](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L515)

Blocks expected at the 10-minute target between the epoch's first block and the tip block

***

### nextRetargetHeight

> **nextRetargetHeight**: `number`

Defined in: [modules/bitview-client/index.js:509](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L509)

***

### previousRetarget

> **previousRetarget**: `number`

Defined in: [modules/bitview-client/index.js:507](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L507)

Previous difficulty adjustment (%)

***

### previousTime

> **previousTime**: `number`

Defined in: [modules/bitview-client/index.js:508](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L508)

Timestamp of most recent retarget (seconds)

***

### progressPercent

> **progressPercent**: `number`

Defined in: [modules/bitview-client/index.js:502](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L502)

Progress through current difficulty epoch (0-100%)

***

### remainingBlocks

> **remainingBlocks**: `number`

Defined in: [modules/bitview-client/index.js:505](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L505)

Blocks remaining until retarget

***

### remainingTime

> **remainingTime**: `number`

Defined in: [modules/bitview-client/index.js:506](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L506)

Estimated time until retarget at `adjustedTimeAvg` (milliseconds)

***

### timeAvg

> **timeAvg**: `number`

Defined in: [modules/bitview-client/index.js:510](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L510)

Average block time in current epoch (milliseconds)

***

### timeOffset

> **timeOffset**: `number`

Defined in: [modules/bitview-client/index.js:513](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L513)

Testnet's offset for its 20-minute minimum-difficulty rule (milliseconds): always 0 on
mainnet, as mempool.space
