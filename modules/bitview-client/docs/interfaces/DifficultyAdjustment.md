[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DifficultyAdjustment

# Interface: DifficultyAdjustment

Defined in: [modules/bitview-client/index.js:541](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L541)

## Properties

### adjustedTimeAvg

> **adjustedTimeAvg**: `number`

Defined in: [modules/bitview-client/index.js:551](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L551)

Block time the projection uses (milliseconds): `timeAvg`, except in an epoch's first 503
blocks, where it spans the last 503 blocks, those before the retarget scaled by it

***

### difficultyChange

> **difficultyChange**: `number`

Defined in: [modules/bitview-client/index.js:543](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L543)

Estimated difficulty change at next retarget at the `adjustedTimeAvg` pace (%)

***

### estimatedRetargetDate

> **estimatedRetargetDate**: `number`

Defined in: [modules/bitview-client/index.js:544](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L544)

Estimated timestamp of next retarget (milliseconds)

***

### expectedBlocks

> **expectedBlocks**: `number`

Defined in: [modules/bitview-client/index.js:555](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L555)

Blocks expected at the 10-minute target between the epoch's first block and the tip block

***

### nextRetargetHeight

> **nextRetargetHeight**: `number`

Defined in: [modules/bitview-client/index.js:549](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L549)

***

### previousRetarget

> **previousRetarget**: `number`

Defined in: [modules/bitview-client/index.js:547](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L547)

Previous difficulty adjustment (%)

***

### previousTime

> **previousTime**: `number`

Defined in: [modules/bitview-client/index.js:548](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L548)

Timestamp of most recent retarget (seconds)

***

### progressPercent

> **progressPercent**: `number`

Defined in: [modules/bitview-client/index.js:542](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L542)

Progress through current difficulty epoch (0-100%)

***

### remainingBlocks

> **remainingBlocks**: `number`

Defined in: [modules/bitview-client/index.js:545](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L545)

Blocks remaining until retarget

***

### remainingTime

> **remainingTime**: `number`

Defined in: [modules/bitview-client/index.js:546](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L546)

Estimated time until retarget at `adjustedTimeAvg` (milliseconds)

***

### timeAvg

> **timeAvg**: `number`

Defined in: [modules/bitview-client/index.js:550](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L550)

Average block time in current epoch (milliseconds)

***

### timeOffset

> **timeOffset**: `number`

Defined in: [modules/bitview-client/index.js:553](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L553)

Testnet's offset for its 20-minute minimum-difficulty rule (milliseconds): always 0 on
mainnet, as mempool.space
