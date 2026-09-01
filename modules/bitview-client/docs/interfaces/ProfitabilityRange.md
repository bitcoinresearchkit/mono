[**bitview-client**](../README.md)

***

[bitview-client](../globals.md) / ProfitabilityRange

# Interface: ProfitabilityRange

Defined in: [Developer/mono/modules/bitview-client/index.js:1071](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1071)

## Properties

### \_0pctTo10pctInLoss

> **\_0pctTo10pctInLoss**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1102](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1102)

Uses UTXOs whose represented-block spot price equals creation price or
is less than 10% below it.

***

### \_0pctTo10pctInProfit

> **\_0pctTo10pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1100](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1100)

Uses UTXOs whose represented-block spot price is above creation price by
no more than 10%.

***

### \_100pctTo200pctInProfit

> **\_100pctTo200pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1080](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1080)

Uses UTXOs whose represented-block spot price is more than 100% and no
more than 200% above creation price.

***

### \_10pctTo20pctInLoss

> **\_10pctTo20pctInLoss**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1104](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1104)

Uses UTXOs whose represented-block spot price is at least 10% and less
than 20% below creation price.

***

### \_10pctTo20pctInProfit

> **\_10pctTo20pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1098](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1098)

Uses UTXOs whose represented-block spot price is more than 10% and no
more than 20% above creation price.

***

### \_200pctTo300pctInProfit

> **\_200pctTo300pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1078](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1078)

Uses UTXOs whose represented-block spot price is more than 200% and no
more than 300% above creation price.

***

### \_20pctTo30pctInLoss

> **\_20pctTo30pctInLoss**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1106](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1106)

Uses UTXOs whose represented-block spot price is at least 20% and less
than 30% below creation price.

***

### \_20pctTo30pctInProfit

> **\_20pctTo30pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1096](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1096)

Uses UTXOs whose represented-block spot price is more than 20% and no
more than 30% above creation price.

***

### \_300pctTo500pctInProfit

> **\_300pctTo500pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1076](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1076)

Uses UTXOs whose represented-block spot price is more than 300% and no
more than 500% above creation price.

***

### \_30pctTo40pctInLoss

> **\_30pctTo40pctInLoss**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1108](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1108)

Uses UTXOs whose represented-block spot price is at least 30% and less
than 40% below creation price.

***

### \_30pctTo40pctInProfit

> **\_30pctTo40pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1094](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1094)

Uses UTXOs whose represented-block spot price is more than 30% and no
more than 40% above creation price.

***

### \_40pctTo50pctInLoss

> **\_40pctTo50pctInLoss**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1110](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1110)

Uses UTXOs whose represented-block spot price is at least 40% and less
than 50% below creation price.

***

### \_40pctTo50pctInProfit

> **\_40pctTo50pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1092](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1092)

Uses UTXOs whose represented-block spot price is more than 40% and no
more than 50% above creation price.

***

### \_500pctTo1000pctInProfit

> **\_500pctTo1000pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1074](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1074)

Uses UTXOs whose represented-block spot price is more than 500% and no
more than 1,000% above creation price.

***

### \_50pctTo60pctInLoss

> **\_50pctTo60pctInLoss**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1112](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1112)

Uses UTXOs whose represented-block spot price is at least 50% and less
than 60% below creation price.

***

### \_50pctTo60pctInProfit

> **\_50pctTo60pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1090](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1090)

Uses UTXOs whose represented-block spot price is more than 50% and no
more than 60% above creation price.

***

### \_60pctTo70pctInLoss

> **\_60pctTo70pctInLoss**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1114](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1114)

Uses UTXOs whose represented-block spot price is at least 60% and less
than 70% below creation price.

***

### \_60pctTo70pctInProfit

> **\_60pctTo70pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1088](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1088)

Uses UTXOs whose represented-block spot price is more than 60% and no
more than 70% above creation price.

***

### \_70pctTo80pctInLoss

> **\_70pctTo80pctInLoss**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1116](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1116)

Uses UTXOs whose represented-block spot price is at least 70% and less
than 80% below creation price.

***

### \_70pctTo80pctInProfit

> **\_70pctTo80pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1086](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1086)

Uses UTXOs whose represented-block spot price is more than 70% and no
more than 80% above creation price.

***

### \_80pctTo90pctInLoss

> **\_80pctTo90pctInLoss**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1118](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1118)

Uses UTXOs whose represented-block spot price is at least 80% and less
than 90% below creation price.

***

### \_80pctTo90pctInProfit

> **\_80pctTo90pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1084](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1084)

Uses UTXOs whose represented-block spot price is more than 80% and no
more than 90% above creation price.

***

### \_90pctTo100pctInLoss

> **\_90pctTo100pctInLoss**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1120](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1120)

Uses UTXOs whose represented-block spot price is at least 90% below
creation price.

***

### \_90pctTo100pctInProfit

> **\_90pctTo100pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1082](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1082)

Uses UTXOs whose represented-block spot price is more than 90% and no
more than 100% above creation price.

***

### over1000pctInProfit

> **over1000pctInProfit**: `number`

Defined in: [Developer/mono/modules/bitview-client/index.js:1072](https://github.com/bitcoinresearchkit/brk/blob/5fc2a239df8aa10fda5a85d6a97dfb7da3f00c2f/modules/bitview-client/index.js#L1072)

Uses UTXOs whose represented-block spot price is more than 1,000% above
creation price.
