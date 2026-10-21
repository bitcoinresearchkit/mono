[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DifficultyAdjustment

# Interface: DifficultyAdjustment

Defined in: [modules/bitview-client/index.js:465](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L465)

## Properties

### adjustedTimeAvg

> **adjustedTimeAvg**: `number`

Defined in: [modules/bitview-client/index.js:475](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L475)

Block time the projection uses (milliseconds): `timeAvg`, except in an epoch's first 503
blocks, where it spans the last 503 blocks, those before the retarget scaled by it

***

### difficultyChange

> **difficultyChange**: `number`

Defined in: [modules/bitview-client/index.js:467](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L467)

Estimated difficulty change at next retarget at the `adjustedTimeAvg` pace (%)

***

### estimatedRetargetDate

> **estimatedRetargetDate**: `number`

Defined in: [modules/bitview-client/index.js:468](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L468)

Estimated timestamp of next retarget (milliseconds)

***

### expectedBlocks

> **expectedBlocks**: `number`

Defined in: [modules/bitview-client/index.js:479](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L479)

Blocks expected at the 10-minute target between the epoch's first block and the tip block

***

### nextRetargetHeight

> **nextRetargetHeight**: `number`

Defined in: [modules/bitview-client/index.js:473](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L473)

***

### previousRetarget

> **previousRetarget**: `number`

Defined in: [modules/bitview-client/index.js:471](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L471)

Previous difficulty adjustment (%)

***

### previousTime

> **previousTime**: `number`

Defined in: [modules/bitview-client/index.js:472](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L472)

Timestamp of most recent retarget (seconds)

***

### progressPercent

> **progressPercent**: `number`

Defined in: [modules/bitview-client/index.js:466](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L466)

Progress through current difficulty epoch (0-100%)

***

### remainingBlocks

> **remainingBlocks**: `number`

Defined in: [modules/bitview-client/index.js:469](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L469)

Blocks remaining until retarget

***

### remainingTime

> **remainingTime**: `number`

Defined in: [modules/bitview-client/index.js:470](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L470)

Estimated time until retarget at `adjustedTimeAvg` (milliseconds)

***

### timeAvg

> **timeAvg**: `number`

Defined in: [modules/bitview-client/index.js:474](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L474)

Average block time in current epoch (milliseconds)

***

### timeOffset

> **timeOffset**: `number`

Defined in: [modules/bitview-client/index.js:477](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L477)

Testnet's offset for its 20-minute minimum-difficulty rule (milliseconds): always 0 on
mainnet, as mempool.space
