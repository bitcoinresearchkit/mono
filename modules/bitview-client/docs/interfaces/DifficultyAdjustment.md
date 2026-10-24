[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DifficultyAdjustment

# Interface: DifficultyAdjustment

Defined in: [modules/bitview-client/index.js:516](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L516)

## Properties

### adjustedTimeAvg

> **adjustedTimeAvg**: `number`

Defined in: [modules/bitview-client/index.js:526](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L526)

Block time the projection uses (milliseconds): `timeAvg`, except in an epoch's first 503
blocks, where it spans the last 503 blocks, those before the retarget scaled by it

***

### difficultyChange

> **difficultyChange**: `number`

Defined in: [modules/bitview-client/index.js:518](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L518)

Estimated difficulty change at next retarget at the `adjustedTimeAvg` pace (%)

***

### estimatedRetargetDate

> **estimatedRetargetDate**: `number`

Defined in: [modules/bitview-client/index.js:519](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L519)

Estimated timestamp of next retarget (milliseconds)

***

### expectedBlocks

> **expectedBlocks**: `number`

Defined in: [modules/bitview-client/index.js:530](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L530)

Blocks expected at the 10-minute target between the epoch's first block and the tip block

***

### nextRetargetHeight

> **nextRetargetHeight**: `number`

Defined in: [modules/bitview-client/index.js:524](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L524)

***

### previousRetarget

> **previousRetarget**: `number`

Defined in: [modules/bitview-client/index.js:522](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L522)

Previous difficulty adjustment (%)

***

### previousTime

> **previousTime**: `number`

Defined in: [modules/bitview-client/index.js:523](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L523)

Timestamp of most recent retarget (seconds)

***

### progressPercent

> **progressPercent**: `number`

Defined in: [modules/bitview-client/index.js:517](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L517)

Progress through current difficulty epoch (0-100%)

***

### remainingBlocks

> **remainingBlocks**: `number`

Defined in: [modules/bitview-client/index.js:520](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L520)

Blocks remaining until retarget

***

### remainingTime

> **remainingTime**: `number`

Defined in: [modules/bitview-client/index.js:521](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L521)

Estimated time until retarget at `adjustedTimeAvg` (milliseconds)

***

### timeAvg

> **timeAvg**: `number`

Defined in: [modules/bitview-client/index.js:525](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L525)

Average block time in current epoch (milliseconds)

***

### timeOffset

> **timeOffset**: `number`

Defined in: [modules/bitview-client/index.js:528](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L528)

Testnet's offset for its 20-minute minimum-difficulty rule (milliseconds): always 0 on
mainnet, as mempool.space
