[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DifficultyAdjustment

# Interface: DifficultyAdjustment

Defined in: [modules/bitview-client/index.js:526](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L526)

## Properties

### adjustedTimeAvg

> **adjustedTimeAvg**: `number`

Defined in: [modules/bitview-client/index.js:536](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L536)

Block time the projection uses (milliseconds): `timeAvg`, except in an epoch's first 503
blocks, where it spans the last 503 blocks, those before the retarget scaled by it

***

### difficultyChange

> **difficultyChange**: `number`

Defined in: [modules/bitview-client/index.js:528](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L528)

Estimated difficulty change at next retarget at the `adjustedTimeAvg` pace (%)

***

### estimatedRetargetDate

> **estimatedRetargetDate**: `number`

Defined in: [modules/bitview-client/index.js:529](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L529)

Estimated timestamp of next retarget (milliseconds)

***

### expectedBlocks

> **expectedBlocks**: `number`

Defined in: [modules/bitview-client/index.js:540](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L540)

Blocks expected at the 10-minute target between the epoch's first block and the tip block

***

### nextRetargetHeight

> **nextRetargetHeight**: `number`

Defined in: [modules/bitview-client/index.js:534](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L534)

***

### previousRetarget

> **previousRetarget**: `number`

Defined in: [modules/bitview-client/index.js:532](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L532)

Previous difficulty adjustment (%)

***

### previousTime

> **previousTime**: `number`

Defined in: [modules/bitview-client/index.js:533](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L533)

Timestamp of most recent retarget (seconds)

***

### progressPercent

> **progressPercent**: `number`

Defined in: [modules/bitview-client/index.js:527](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L527)

Progress through current difficulty epoch (0-100%)

***

### remainingBlocks

> **remainingBlocks**: `number`

Defined in: [modules/bitview-client/index.js:530](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L530)

Blocks remaining until retarget

***

### remainingTime

> **remainingTime**: `number`

Defined in: [modules/bitview-client/index.js:531](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L531)

Estimated time until retarget at `adjustedTimeAvg` (milliseconds)

***

### timeAvg

> **timeAvg**: `number`

Defined in: [modules/bitview-client/index.js:535](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L535)

Average block time in current epoch (milliseconds)

***

### timeOffset

> **timeOffset**: `number`

Defined in: [modules/bitview-client/index.js:538](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L538)

Testnet's offset for its 20-minute minimum-difficulty rule (milliseconds): always 0 on
mainnet, as mempool.space
