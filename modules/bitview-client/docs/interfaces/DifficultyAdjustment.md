[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / DifficultyAdjustment

# Interface: DifficultyAdjustment

Defined in: [modules/bitview-client/index.js:452](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L452)

## Properties

### adjustedTimeAvg

> **adjustedTimeAvg**: `number`

Defined in: [modules/bitview-client/index.js:462](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L462)

Block time the projection uses (milliseconds): `timeAvg`, except in an epoch's first 503
  blocks, where it spans the last 503 blocks, those before the retarget scaled by it

***

### difficultyChange

> **difficultyChange**: `number`

Defined in: [modules/bitview-client/index.js:454](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L454)

Estimated difficulty change at next retarget at the `adjustedTimeAvg` pace (%)

***

### estimatedRetargetDate

> **estimatedRetargetDate**: `number`

Defined in: [modules/bitview-client/index.js:455](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L455)

Estimated timestamp of next retarget (milliseconds)

***

### expectedBlocks

> **expectedBlocks**: `number`

Defined in: [modules/bitview-client/index.js:466](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L466)

Blocks expected at the 10-minute target between the epoch's first block and the tip block

***

### nextRetargetHeight

> **nextRetargetHeight**: `number`

Defined in: [modules/bitview-client/index.js:460](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L460)

***

### previousRetarget

> **previousRetarget**: `number`

Defined in: [modules/bitview-client/index.js:458](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L458)

Previous difficulty adjustment (%)

***

### previousTime

> **previousTime**: `number`

Defined in: [modules/bitview-client/index.js:459](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L459)

Timestamp of most recent retarget (seconds)

***

### progressPercent

> **progressPercent**: `number`

Defined in: [modules/bitview-client/index.js:453](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L453)

Progress through current difficulty epoch (0-100%)

***

### remainingBlocks

> **remainingBlocks**: `number`

Defined in: [modules/bitview-client/index.js:456](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L456)

Blocks remaining until retarget

***

### remainingTime

> **remainingTime**: `number`

Defined in: [modules/bitview-client/index.js:457](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L457)

Estimated time until retarget at `adjustedTimeAvg` (milliseconds)

***

### timeAvg

> **timeAvg**: `number`

Defined in: [modules/bitview-client/index.js:461](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L461)

Average block time in current epoch (milliseconds)

***

### timeOffset

> **timeOffset**: `number`

Defined in: [modules/bitview-client/index.js:464](https://github.com/bitcoinresearchkit/mono/blob/main/modules/bitview-client/index.js#L464)

Testnet's offset for its 20-minute minimum-difficulty rule (milliseconds): always 0 on
  mainnet, as mempool.space
