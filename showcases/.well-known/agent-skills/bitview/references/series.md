# Bitview series by topic

22901 series. Find one with `GET https://bitview.space/api/series/search?q=<plain words>`; this map is for browsing. Each branch lists its series count and first ids; a family's later members are left out of the branches, as they repeat the first one's series.

## Branches

### indexer

- `indexer.blocks` (9): `blockhash`, `coinbase_tag`, `timestamp`, `median_time`, `block_size`
- `indexer.transactions` (61): `first_tx_index`, `txid`, `tx_version`, `tx_locktime`, `tx_weight`
- `indexer.inputs` (7): `first_txin_index`, `outpoint`, `spent_txout_index`, `input_tx_index`, `output_type`
- `indexer.outputs` (4): `first_txout_index`, `output_value`, `output_type`, `type_index`
- `indexer.addresses` (16): `first_p2pk65_address_index`, `p2pk65_bytes`, `first_p2pk33_address_index`, `p2pk33_bytes`, `first_p2pkh_address_index`
- `indexer.scripts` (8): `first_p2ms_output_index`, `p2ms_output_tx_index`, `p2ms_legacy_sigop_cost`, `first_empty_script_output_index`, `empty_script_output_tx_index`
- `indexer.op_return` (4): `first_op_return_output_index`, `op_return_output_tx_index`, `op_return_protocol`, `post_op_return_bytes`

### mappings

- `mappings.addresses` (16): `p2pk65_address_index`, `p2pk65_address`, `p2pk33_address_index`, `p2pk33_address`, `p2pkh_address_index`
- `mappings.scripts` (4): `p2ms_output_index`, `empty_script_output_index`, `unknown_script_output_index`, `op_return_output_index`
- `mappings.height` (15): `block_minute10`, `block_minute30`, `block_hour1`, `block_hour4`, `block_hour12`
- `mappings.difficulty_epoch` (1): `first_height`
- `mappings.halving_epoch` (1): `first_height`
- `mappings.minute10` (1): `first_height`
- `mappings.minute30` (1): `first_height`
- `mappings.hour1` (1): `first_height`
- `mappings.hour4` (1): `first_height`
- `mappings.hour12` (1): `first_height`
- `mappings.day1` (2): `date`, `first_height`
- `mappings.day3` (2): `date`, `first_height`
- `mappings.week1` (2): `date`, `first_height`
- `mappings.month1` (2): `date`, `first_height`
- `mappings.month3` (2): `date`, `first_height`
- `mappings.month6` (2): `date`, `first_height`
- `mappings.year1` (2): `date`, `first_height`
- `mappings.year10` (2): `date`, `first_height`
- `mappings.tx_index` (3): `tx_index`, `tx_input_count`, `tx_output_count`
- `mappings.txin_index` (1): `txin_index`
- `mappings.txout_index` (1): `txout_index`
- `mappings.timestamp` (2): `timestamp_monotonic`, `timestamp`

### price

- `price.open` (1): `price_open`
- `price.high` (1): `price_high`
- `price.low` (1): `price_low`
- `price.close` (1): `price_close`
- `price.ohlc` (1): `price_ohlc`
- `price.spot` (1): `price`
- `price.sats_per_dollar` (1): `sats_per_dollar`

### blocks

- `blocks.count` (5): `block_count_cumulative`, `block_count_sum_24h`, `block_count_sum_1w`, `block_count_sum_1m`, `block_count_sum_1y`
- `blocks.interval` (10): `block_interval`, `block_interval_cumulative`, `block_interval_sum_24h`, `block_interval_sum_1w`, `block_interval_sum_1m`
- `blocks.size` (38): `block_size`, `block_size_cumulative`, `block_size_sum_24h`, `block_size_sum_1w`, `block_size_sum_1m`
- `blocks.weight` (38): `block_weight`, `block_weight_cumulative`, `block_weight_sum_24h`, `block_weight_sum_1w`, `block_weight_sum_1m`
- `blocks.sigop_cost` (6): `block_sigop_cost`, `block_sigop_cost_cumulative`, `block_sigop_cost_sum_24h`, `block_sigop_cost_sum_1w`, `block_sigop_cost_sum_1m`
- `blocks.fullness` (33): `block_fullness`, `block_fullness_avg_24h`, `block_fullness_avg_1w`, `block_fullness_avg_1m`, `block_fullness_avg_1y`
- `blocks.difficulty` (6): `difficulty`, `difficulty_hashrate`, `difficulty_adjustment`, `difficulty_epoch`, `blocks_to_retarget`
- `blocks.halving` (3): `halving_epoch`, `blocks_to_halving`, `days_to_halving`
- `blocks.empty_count` (5): `empty_block_count_cumulative`, `empty_block_count_sum_24h`, `empty_block_count_sum_1w`, `empty_block_count_sum_1m`, `empty_block_count_sum_1y`
- `blocks.work` (6): `block_work`, `block_work_cumulative`, `block_work_sum_24h`, `block_work_sum_1w`, `block_work_sum_1m`
- `blocks.version_bits` (5): `version_bit_4_block_count_cumulative`, `version_bit_4_block_count_sum_24h`, `version_bit_4_block_count_sum_1w`, `version_bit_4_block_count_sum_1m`, `version_bit_4_block_count_sum_1y`
- `blocks.coinbase_locktime` (10): `nonzero_coinbase_locktime_block_count_cumulative`, `nonzero_coinbase_locktime_block_count_sum_24h`, `nonzero_coinbase_locktime_block_count_sum_1w`, `nonzero_coinbase_locktime_block_count_sum_1m`, `nonzero_coinbase_locktime_block_count_sum_1y`

### inputs

- `inputs.count` (38): `input_count`, `input_count_cumulative`, `input_count_sum_24h`, `input_count_sum_1w`, `input_count_sum_1m`
- `inputs.per_second` (4): `inputs_per_second_24h`, `inputs_per_second_1w`, `inputs_per_second_1m`, `inputs_per_second_1y`
- `inputs.types` (220): `p2pk_input_count`, `p2pk_input_count_cumulative`, `p2pk_input_count_sum_24h`, `p2pk_input_count_sum_1w`, `p2pk_input_count_sum_1m`
- `inputs.signatures` (120): `schnorr_signature_count`, `schnorr_signature_count_cumulative`, `schnorr_signature_count_sum_24h`, `schnorr_signature_count_sum_1w`, `schnorr_signature_count_sum_1m`
- `inputs.pubkeys` (12): `compressed_pubkey_in_input_count`, `compressed_pubkey_in_input_count_cumulative`, `compressed_pubkey_in_input_count_sum_24h`, `compressed_pubkey_in_input_count_sum_1w`, `compressed_pubkey_in_input_count_sum_1m`
- `inputs.nested_segwit` (12): `p2sh_p2wpkh_input_count`, `p2sh_p2wpkh_input_count_cumulative`, `p2sh_p2wpkh_input_count_sum_24h`, `p2sh_p2wpkh_input_count_sum_1w`, `p2sh_p2wpkh_input_count_sum_1m`
- `inputs.multisig` (18): `p2sh_multisig_input_count`, `p2sh_multisig_input_count_cumulative`, `p2sh_multisig_input_count_sum_24h`, `p2sh_multisig_input_count_sum_1w`, `p2sh_multisig_input_count_sum_1m`
- `inputs.opcodes` (60): `op_checksig_in_input_count`, `op_checksig_in_input_count_cumulative`, `op_checksig_in_input_count_sum_24h`, `op_checksig_in_input_count_sum_1w`, `op_checksig_in_input_count_sum_1m`
- `inputs.p2tr_script_path` (6): `p2tr_script_path_input_count`, `p2tr_script_path_input_count_cumulative`, `p2tr_script_path_input_count_sum_24h`, `p2tr_script_path_input_count_sum_1w`, `p2tr_script_path_input_count_sum_1m`
- `inputs.spent_within` (24): `within_1_block_spent_utxo_count`, `within_1_block_spent_utxo_count_cumulative`, `within_1_block_spent_utxo_count_sum_24h`, `within_1_block_spent_utxo_count_sum_1w`, `within_1_block_spent_utxo_count_sum_1m`
- `inputs.coinbase_witness` (5): `coinbase_witness_block_count_cumulative`, `coinbase_witness_block_count_sum_24h`, `coinbase_witness_block_count_sum_1w`, `coinbase_witness_block_count_sum_1m`, `coinbase_witness_block_count_sum_1y`

### outputs

- `outputs.spending_txin_index` (1): `spending_txin_index`
- `outputs.count` (38): `output_count`, `output_count_cumulative`, `output_count_sum_24h`, `output_count_sum_1w`, `output_count_sum_1m`
- `outputs.per_second` (4): `outputs_per_second_24h`, `outputs_per_second_1w`, `outputs_per_second_1m`, `outputs_per_second_1y`
- `outputs.spendable_count` (6): `spendable_output_count`, `spendable_output_count_cumulative`, `spendable_output_count_sum_24h`, `spendable_output_count_sum_1w`, `spendable_output_count_sum_1m`
- `outputs.types` (362): `p2pk_output_count`, `p2pk_output_count_cumulative`, `p2pk_output_count_sum_24h`, `p2pk_output_count_sum_1w`, `p2pk_output_count_sum_1m`
- `outputs.op_return_value` (12): `op_return_value`, `op_return_value_usd`, `op_return_value_cumulative`, `op_return_value_cumulative_usd`, `op_return_value_sum_24h`
- `outputs.coinbase` (72): `coinbase_output_count`, `coinbase_output_count_cumulative`, `coinbase_output_count_sum_24h`, `coinbase_output_count_sum_1w`, `coinbase_output_count_sum_1m`
- `outputs.pubkeys` (12): `compressed_pubkey_in_output_count`, `compressed_pubkey_in_output_count_cumulative`, `compressed_pubkey_in_output_count_sum_24h`, `compressed_pubkey_in_output_count_sum_1w`, `compressed_pubkey_in_output_count_sum_1m`

### transactions

- `transactions.count` (38): `tx_count`, `tx_count_cumulative`, `tx_count_sum_24h`, `tx_count_sum_1w`, `tx_count_sum_1m`
- `transactions.features` (717): `one_input_one_output_tx_count`, `one_input_one_output_tx_count_cumulative`, `one_input_one_output_tx_count_sum_24h`, `one_input_one_output_tx_count_sum_1w`, `one_input_one_output_tx_count_sum_1m`
- `transactions.vsize` (13): `tx_vsize`, `tx_vsize_min`, `tx_vsize_max`, `tx_vsize_p10`, `tx_vsize_p25`
- `transactions.fees` (249): `tx_fee`, `tx_fee_min`, `tx_fee_max`, `tx_fee_p10`, `tx_fee_p25`
- `transactions.inscription` (46): `has_inscription`, `inscription_tx_count`, `inscription_tx_count_cumulative`, `inscription_tx_count_sum_24h`, `inscription_tx_count_sum_1w`
- `transactions.patterns` (87): `is_coinjoin`, `coinjoin_tx_count`, `coinjoin_tx_count_cumulative`, `coinjoin_tx_count_sum_24h`, `coinjoin_tx_count_sum_1w`
- `transactions.nonstandard` (29): `is_nonstandard`, `nonstandard_tx_count`, `nonstandard_tx_count_cumulative`, `nonstandard_tx_count_sum_24h`, `nonstandard_tx_count_sum_1w`
- `transactions.versions` (112): `v1_tx_count`, `v1_tx_count_cumulative`, `v1_tx_count_sum_24h`, `v1_tx_count_sum_1w`, `v1_tx_count_sum_1m`
- `transactions.transfer_volume` (12): `transfer_volume`, `transfer_volume_usd`, `transfer_volume_cumulative`, `transfer_volume_cumulative_usd`, `transfer_volume_sum_24h`
- `transactions.payments` (18): `payment_count`, `payment_count_cumulative`, `payment_count_sum_24h`, `payment_count_sum_1w`, `payment_count_sum_1m`
- `transactions.timelocks` (30): `height_locktime_tx_count`, `height_locktime_tx_count_cumulative`, `height_locktime_tx_count_sum_24h`, `height_locktime_tx_count_sum_1w`, `height_locktime_tx_count_sum_1m`
- `transactions.spending` (18): `only_segwit_spending_tx_count`, `only_segwit_spending_tx_count_cumulative`, `only_segwit_spending_tx_count_sum_24h`, `only_segwit_spending_tx_count_sum_1w`, `only_segwit_spending_tx_count_sum_1m`
- `transactions.ephemeral_dust` (6): `ephemeral_dust_spending_tx_count`, `ephemeral_dust_spending_tx_count_cumulative`, `ephemeral_dust_spending_tx_count_sum_24h`, `ephemeral_dust_spending_tx_count_sum_1w`, `ephemeral_dust_spending_tx_count_sum_1m`
- `transactions.same_block_spending` (6): `same_block_spending_tx_count`, `same_block_spending_tx_count_cumulative`, `same_block_spending_tx_count_sum_24h`, `same_block_spending_tx_count_sum_1w`, `same_block_spending_tx_count_sum_1m`
- `transactions.per_second` (4): `tx_per_second_24h`, `tx_per_second_1w`, `tx_per_second_1m`, `tx_per_second_1y`

### mining

- `mining.rewards` (122): `coinbase`, `coinbase_usd`, `coinbase_cumulative`, `coinbase_cumulative_usd`, `coinbase_sum_24h`
- `mining.hashrate` (7): `hashrate`, `hashrate_sma_1w`, `hashrate_sma_1m`, `hashrate_sma_2m`, `hashrate_sma_1y`
- `mining.hash_price` (3): `hash_price`, `hash_price_atl`, `hash_price_rebound`
- `mining.hash_value` (3): `hash_value`, `hash_value_atl`, `hash_value_rebound`

### op_return

- `op_return.flag` (1): `has_op_return`
- `op_return.output_count` (6): `op_return_output_count`, `op_return_output_count_cumulative`, `op_return_output_count_sum_24h`, `op_return_output_count_sum_1w`, `op_return_output_count_sum_1m`
- `op_return.data_bytes` (11): `op_return_data_bytes`, `op_return_data_bytes_cumulative`, `op_return_data_bytes_sum_24h`, `op_return_data_bytes_sum_1w`, `op_return_data_bytes_sum_1m`
- `op_return.tx_share` (5): `op_return_tx_share_cumulative`, `op_return_tx_share_24h`, `op_return_tx_share_1w`, `op_return_tx_share_1m`, `op_return_tx_share_1y`
- `op_return.fees` (17): `op_return_fees`, `op_return_fees_usd`, `op_return_fees_cumulative`, `op_return_fees_cumulative_usd`, `op_return_fees_sum_24h`
- `op_return.protocols` (1173): `op_return_runes_output_count`, `op_return_runes_output_count_cumulative`, `op_return_runes_output_count_sum_24h`, `op_return_runes_output_count_sum_1w`, `op_return_runes_output_count_sum_1m`
- `op_return.policies` (204): `op_return_pre_v30_standard_output_count`, `op_return_pre_v30_standard_output_count_cumulative`, `op_return_pre_v30_standard_output_count_sum_24h`, `op_return_pre_v30_standard_output_count_sum_1w`, `op_return_pre_v30_standard_output_count_sum_1m`
- `op_return.sizes` (18): `op_return_data_over_42_bytes_output_count`, `op_return_data_over_42_bytes_output_count_cumulative`, `op_return_data_over_42_bytes_output_count_sum_24h`, `op_return_data_over_42_bytes_output_count_sum_1w`, `op_return_data_over_42_bytes_output_count_sum_1m`

### pools

- `pools.block_pool` (1): `block_pool`
- `pools.unknown` (32): `pool_unknown_block_count_cumulative`, `pool_unknown_block_count_sum_24h`, `pool_unknown_block_count_sum_1w`, `pool_unknown_block_count_sum_1m`, `pool_unknown_block_count_sum_1y`
- `pools.concentration` (5): `top_2_pools_block_count`, `top_3_pools_block_count`, `top_4_pools_block_count`, `top_5_pools_block_count`, `top_6_pools_block_count`
- `pools.concentration_grouped` (5): `top_2_pools_antpool_group_as_one_block_count`, `top_3_pools_antpool_group_as_one_block_count`, `top_4_pools_antpool_group_as_one_block_count`, `top_5_pools_antpool_group_as_one_block_count`, `top_6_pools_antpool_group_as_one_block_count`
- `pools.groups` (5): `pool_antpool_group_block_count_cumulative`, `pool_antpool_group_block_count_sum_24h`, `pool_antpool_group_block_count_sum_1w`, `pool_antpool_group_block_count_sum_1m`, `pool_antpool_group_block_count_sum_1y`

### utxo_set

- `utxo_set.supply` (1): `circulating_supply`
- `utxo_set.count` (1): `utxo_count`

### market

- `market.ath` (4): `price_ath`, `price_drawdown`, `days_since_price_ath`, `days_between_price_ath_max`
- `market.returns` (13): `price_return_24h`, `price_return_1w`, `price_return_1m`, `price_return_3m`, `price_return_6m`
- `market.range` (6): `price_min_1w`, `price_min_1m`, `price_min_1y`, `price_max_1w`, `price_max_1m`
- `market.sma` (31): `price_sma_1w`, `price_sma_1w_ratio`, `price_sma_8d`, `price_sma_8d_ratio`, `price_sma_13d`
- `market.ema` (30): `price_ema_1w`, `price_ema_1w_ratio`, `price_ema_8d`, `price_ema_8d_ratio`, `price_ema_13d`

### age

- `age.ranges` (3036): `age_under_1h_supply`, `age_under_1h_supply_usd`, `age_under_1h_supply_delta_24h`, `age_under_1h_supply_delta_1w`, `age_under_1h_supply_delta_1m`
- `age.halving_epochs` (540): `halving_epoch_0_supply`, `halving_epoch_0_supply_usd`, `halving_epoch_0_supply_delta_24h`, `halving_epoch_0_supply_delta_1w`, `halving_epoch_0_supply_delta_1m`
- `age.vintages` (1710): `vintage_2009_supply`, `vintage_2009_supply_usd`, `vintage_2009_supply_delta_24h`, `vintage_2009_supply_delta_1w`, `vintage_2009_supply_delta_1m`

### utxos

- `utxos.amounts` (897): `amount_0_sats_utxo_count`, `amount_0_sats_utxo_count_delta_24h`, `amount_0_sats_utxo_count_delta_1w`, `amount_0_sats_utxo_count_delta_1m`, `amount_0_sats_utxo_count_delta_1y`
- `utxos.types` (630): `p2pk_supply`, `p2pk_supply_usd`, `p2pk_supply_delta_24h`, `p2pk_supply_delta_1w`, `p2pk_supply_delta_1m`

### addresses

- `addresses.funded` (9): `funded_address_count`, `funded_address_count_delta_24h`, `funded_address_count_delta_1w`, `funded_address_count_delta_1m`, `funded_address_count_delta_1y`
- `addresses.empty` (1): `empty_address_count`
- `addresses.total` (1): `total_address_count`
- `addresses.new` (5): `new_address_count`, `new_address_count_sum_24h`, `new_address_count_sum_1w`, `new_address_count_sum_1m`, `new_address_count_sum_1y`
- `addresses.activity` (50): `reactivated_address_count`, `reactivated_address_count_cumulative`, `reactivated_address_count_sum_24h`, `reactivated_address_count_sum_1w`, `reactivated_address_count_sum_1m`
- `addresses.avg_balance` (2): `address_balance_avg`, `address_balance_avg_usd`
- `addresses.reused` (48): `funded_reused_address_count`, `total_reused_address_count`, `reused_address_supply`, `reused_address_supply_usd`, `reused_address_supply_share`
- `addresses.respent` (42): `funded_respent_address_count`, `total_respent_address_count`, `respent_address_supply`, `respent_address_supply_usd`, `respent_address_supply_share`
- `addresses.exposed` (5): `funded_exposed_address_count`, `total_exposed_address_count`, `exposed_address_supply`, `exposed_address_supply_usd`, `exposed_address_supply_share`
- `addresses.supply` (11): `address_supply`, `address_supply_usd`, `address_supply_delta_24h`, `address_supply_delta_1w`, `address_supply_delta_1m`
- `addresses.capital` (11): `address_capital`, `address_realized_cap`, `address_capital_delta_24h`, `address_capital_delta_1w`, `address_capital_delta_1m`
- `addresses.types` (942): `p2pk_funded_address_count`, `p2pk_funded_address_count_delta_24h`, `p2pk_funded_address_count_delta_1w`, `p2pk_funded_address_count_delta_1m`, `p2pk_funded_address_count_delta_1y`
- `addresses.balances` (914): `balance_0_sats_address_count`, `balance_0_sats_address_count_delta_24h`, `balance_0_sats_address_count_delta_1w`, `balance_0_sats_address_count_delta_1m`, `balance_0_sats_address_count_delta_1y`

### holders

- `holders.all` (215): `circulating_supply`, `market_cap`, `supply_in_profit`, `supply_in_profit_usd`, `supply_in_profit_share`
- `holders.sth` (217): `sth_supply`, `sth_supply_usd`, `sth_supply_in_profit`, `sth_supply_in_profit_usd`, `sth_supply_in_profit_share`

### entry

- `entry.discount` (90): `entry_discount_supply`, `entry_discount_supply_usd`, `entry_discount_supply_delta_24h`, `entry_discount_supply_delta_1w`, `entry_discount_supply_delta_1m`
- `entry.premium` (90): `entry_premium_supply`, `entry_premium_supply_usd`, `entry_premium_supply_delta_24h`, `entry_premium_supply_delta_1w`, `entry_premium_supply_delta_1m`

### profitability

- `profitability.all` (150): `profit_over_1000_percent_supply`, `profit_over_1000_percent_supply_usd`, `profit_over_1000_percent_supply_share`, `profit_over_1000_percent_capital`, `profit_over_1000_percent_realized_cap`

### supply

- `supply.circulating` (1): `circulating_supply`
- `supply.burned` (12): `burned`, `burned_usd`, `burned_cumulative`, `burned_cumulative_usd`, `burned_sum_24h`
- `supply.inflation_rate` (1): `inflation_rate`
- `supply.velocity` (2): `velocity_btc`, `velocity_usd`
- `supply.market_cap` (9): `market_cap`, `market_cap_delta_24h`, `market_cap_delta_1w`, `market_cap_delta_1m`, `market_cap_delta_1y`

### cointime

- `cointime.activity` (25): `coinblocks_created`, `coinblocks_created_cumulative`, `coinblocks_created_sum_24h`, `coinblocks_created_sum_1w`, `coinblocks_created_sum_1m`
- `cointime.age_ranges` (414): `age_under_1h_coindays_created`, `age_under_1h_coindays_created_cumulative`, `age_under_1h_coindays_created_sum_24h`, `age_under_1h_coindays_created_sum_1w`, `age_under_1h_coindays_created_sum_1m`
- `cointime.cohorts` (420): `awake_supply`, `awake_supply_usd`, `awake_supply_in_loss_share`, `awake_capital`, `awake_realized_cap`
- `cointime.supply` (6): `vaulted_supply`, `vaulted_supply_usd`, `hodled_or_lost_supply`, `hodled_or_lost_supply_usd`, `active_supply`
- `cointime.value` (18): `cointime_value_destroyed`, `cointime_value_destroyed_cumulative`, `cointime_value_destroyed_sum_24h`, `cointime_value_destroyed_sum_1w`, `cointime_value_destroyed_sum_1m`
- `cointime.caps` (7): `thermo_cap`, `investor_cap`, `active_cap`, `vaulted_cap`, `cointime_cap`
- `cointime.prices` (13): `vaulted_price`, `vaulted_price_ratio`, `vaulted_mvrv`, `active_price`, `active_price_ratio`
- `cointime.adjusted` (3): `cointime_adjusted_inflation_rate`, `cointime_adjusted_velocity_btc`, `cointime_adjusted_velocity_usd`
- `cointime.reserve_risk` (8): `reserve_risk`, `hodl_bank`, `vocdd`, `vocdd_cumulative`, `vocdd_sum_24h`

### coinflow

- `coinflow.age_ranges` (161): `age_under_1h_spending_rate`, `age_under_1h_spending_exposure`, `age_under_1h_mobility`, `age_under_1h_mobile_supply`, `age_under_1h_mobile_supply_usd`
- `coinflow.cohorts` (420): `mobile_supply`, `mobile_supply_usd`, `mobile_supply_in_loss_share`, `mobile_capital`, `mobile_realized_cap`

### bedrock

- `bedrock.awake` (34): `bedrock_awake_supply_in_loss_share_threshold_p95`, `bedrock_awake_supply_in_loss_share_threshold_p98`, `bedrock_awake_supply_in_loss_share_threshold_p99`, `bedrock_awake_supply_in_loss_share_threshold_p99_5`, `bedrock_awake_supply_in_loss_share_threshold_p99_9`

### rarity_meter

- `rarity_meter.components` (620): `realized_price_ratio_rarity_p0_1`, `realized_price_rarity_p0_1`, `realized_price_ratio_rarity_p0_5`, `realized_price_rarity_p0_5`, `realized_price_ratio_rarity_p1`
- `rarity_meter.extremes` (100): `supply_in_loss`, `supply_in_loss_rarity_p0_1`, `supply_in_loss_rarity_p0_5`, `supply_in_loss_rarity_p1`, `supply_in_loss_rarity_p2`
- `rarity_meter.full` (21): `rarity_meter_p0_1`, `rarity_meter_p0_5`, `rarity_meter_p1`, `rarity_meter_p2`, `rarity_meter_p5`

## Families

Each member holds the family's series, its ids built on the member's base (`key → base`).

- `addresses.activity` (10 series each): `reactivated` → `reactivated_address_count`, `sending` → `sending_address_count`, `receiving` → `receiving_address_count`, `bidirectional` → `bidirectional_address_count`, `active` → `active_address_count`
- `addresses.balances` (64 series each): `1_to_10_sats` → `balance_1_to_10_sats`, `10_to_100_sats` → `balance_10_to_100_sats`, `100_to_1k_sats` → `balance_100_to_1k_sats`, `1k_to_10k_sats` → `balance_1k_to_10k_sats`, `10k_to_100k_sats` → `balance_10k_to_100k_sats`, `100k_to_1m_sats` → `balance_100k_to_1m_sats`, `1m_to_10m_sats` → `balance_1m_to_10m_sats`, `10m_to_100m_sats` → `balance_10m_to_100m_sats`, `1_to_10_btc` → `balance_1_to_10_btc`, `10_to_100_btc` → `balance_10_to_100_btc`, `100_to_1k_btc` → `balance_100_to_1k_btc`, `1k_to_10k_btc` → `balance_1k_to_10k_btc`, `10k_to_100k_btc` → `balance_10k_to_100k_btc`, `over_100k_btc` → `balance_over_100k_btc`
- `addresses.respent.events` (11 series each): `outputs` → `output_to_respent_address`, `inputs` → `input_from_respent_address`, `active` → `active_respent_address`
- `addresses.reused.events` (11 series each): `outputs` → `output_to_reused_address`, `inputs` → `input_from_reused_address`, `active` → `active_reused_address`
- `addresses.types` (157 series each): `p2pk` → `p2pk`, `p2pkh` → `p2pkh`, `p2sh` → `p2sh`, `p2wpkh` → `p2wpkh`, `p2wsh` → `p2wsh`, `p2tr` → `p2tr`
- `age.halving_epochs` (90 series each): `0` → `halving_epoch_0`, `1` → `halving_epoch_1`, `2` → `halving_epoch_2`, `3` → `halving_epoch_3`, `4` → `halving_epoch_4`, `5` → `halving_epoch_5`
- `age.ranges` (132 series each): `under_1h` → `age_under_1h`, `1h_to_1d` → `age_1h_to_1d`, `1d_to_1w` → `age_1d_to_1w`, `1w_to_1m` → `age_1w_to_1m`, `1m_to_2m` → `age_1m_to_2m`, `2m_to_3m` → `age_2m_to_3m`, `3m_to_4m` → `age_3m_to_4m`, `4m_to_5m` → `age_4m_to_5m`, `5m_to_6m` → `age_5m_to_6m`, `6m_to_9m` → `age_6m_to_9m`, `9m_to_1y` → `age_9m_to_1y`, `1y_to_18m` → `age_1y_to_18m`, `18m_to_2y` → `age_18m_to_2y`, `2y_to_3y` → `age_2y_to_3y`, `3y_to_4y` → `age_3y_to_4y`, `4y_to_5y` → `age_4y_to_5y`, `5y_to_6y` → `age_5y_to_6y`, `6y_to_7y` → `age_6y_to_7y`, `7y_to_8y` → `age_7y_to_8y`, `8y_to_10y` → `age_8y_to_10y`, `10y_to_12y` → `age_10y_to_12y`, `12y_to_15y` → `age_12y_to_15y`, `over_15y` → `age_over_15y`
- `age.vintages` (90 series each): `2009` → `vintage_2009`, `2010` → `vintage_2010`, `2011` → `vintage_2011`, `2012` → `vintage_2012`, `2013` → `vintage_2013`, `2014` → `vintage_2014`, `2015` → `vintage_2015`, `2016` → `vintage_2016`, `2017` → `vintage_2017`, `2018` → `vintage_2018`, `2019` → `vintage_2019`, `2020` → `vintage_2020`, `2021` → `vintage_2021`, `2022` → `vintage_2022`, `2023` → `vintage_2023`, `2024` → `vintage_2024`, `2025` → `vintage_2025`, `2026` → `vintage_2026`, `2027` → `vintage_2027`
- `bedrock` (34 series each): `awake` → `bedrock_awake`, `mobile` → `bedrock_mobile`, `unweighted` → `bedrock_unweighted`
- `blocks.fullness` (4 series each): `avg` → `block_fullness_avg`, `min` → `block_fullness_min`, `max` → `block_fullness_max`, `p10` → `block_fullness_p10`, `p25` → `block_fullness_p25`, `median` → `block_fullness_median`, `p75` → `block_fullness_p75`, `p90` → `block_fullness_p90`
- `blocks.size` (4 series each): `sum` → `block_size_sum`, `avg` → `block_size_avg`, `min` → `block_size_min`, `max` → `block_size_max`, `p10` → `block_size_p10`, `p25` → `block_size_p25`, `median` → `block_size_median`, `p75` → `block_size_p75`, `p90` → `block_size_p90`
- `blocks.weight` (4 series each): `sum` → `block_weight_sum`, `avg` → `block_weight_avg`, `min` → `block_weight_min`, `max` → `block_weight_max`, `p10` → `block_weight_p10`, `p25` → `block_weight_p25`, `median` → `block_weight_median`, `p75` → `block_weight_p75`, `p90` → `block_weight_p90`
- `coinflow.age_ranges` (7 series each): `under_1h` → `age_under_1h`, `1h_to_1d` → `age_1h_to_1d`, `1d_to_1w` → `age_1d_to_1w`, `1w_to_1m` → `age_1w_to_1m`, `1m_to_2m` → `age_1m_to_2m`, `2m_to_3m` → `age_2m_to_3m`, `3m_to_4m` → `age_3m_to_4m`, `4m_to_5m` → `age_4m_to_5m`, `5m_to_6m` → `age_5m_to_6m`, `6m_to_9m` → `age_6m_to_9m`, `9m_to_1y` → `age_9m_to_1y`, `1y_to_18m` → `age_1y_to_18m`, `18m_to_2y` → `age_18m_to_2y`, `2y_to_3y` → `age_2y_to_3y`, `3y_to_4y` → `age_3y_to_4y`, `4y_to_5y` → `age_4y_to_5y`, `5y_to_6y` → `age_5y_to_6y`, `6y_to_7y` → `age_6y_to_7y`, `7y_to_8y` → `age_7y_to_8y`, `8y_to_10y` → `age_8y_to_10y`, `10y_to_12y` → `age_10y_to_12y`, `12y_to_15y` → `age_12y_to_15y`, `over_15y` → `age_over_15y`
- `coinflow.cohorts` (60 series each): `all` → (none), `sth` → `sth`, `lth` → `lth`, `under_4m` → `age_under_4m`, `under_6m` → `age_under_6m`, `over_4m` → `age_over_4m`, `over_6m` → `age_over_6m`
- `cointime.activity` (6 series each): `coinblocks_created` → `coinblocks_created`, `coinblocks_destroyed` → `coinblocks_destroyed`, `coinblocks_stored` → `coinblocks_stored`
- `cointime.age_ranges` (18 series each): `under_1h` → `age_under_1h`, `1h_to_1d` → `age_1h_to_1d`, `1d_to_1w` → `age_1d_to_1w`, `1w_to_1m` → `age_1w_to_1m`, `1m_to_2m` → `age_1m_to_2m`, `2m_to_3m` → `age_2m_to_3m`, `3m_to_4m` → `age_3m_to_4m`, `4m_to_5m` → `age_4m_to_5m`, `5m_to_6m` → `age_5m_to_6m`, `6m_to_9m` → `age_6m_to_9m`, `9m_to_1y` → `age_9m_to_1y`, `1y_to_18m` → `age_1y_to_18m`, `18m_to_2y` → `age_18m_to_2y`, `2y_to_3y` → `age_2y_to_3y`, `3y_to_4y` → `age_3y_to_4y`, `4y_to_5y` → `age_4y_to_5y`, `5y_to_6y` → `age_5y_to_6y`, `6y_to_7y` → `age_6y_to_7y`, `7y_to_8y` → `age_7y_to_8y`, `8y_to_10y` → `age_8y_to_10y`, `10y_to_12y` → `age_10y_to_12y`, `12y_to_15y` → `age_12y_to_15y`, `over_15y` → `age_over_15y`
- `cointime.cohorts` (60 series each): `all` → (none), `sth` → `sth`, `lth` → `lth`, `under_4m` → `age_under_4m`, `under_6m` → `age_under_6m`, `over_4m` → `age_over_4m`, `over_6m` → `age_over_6m`
- `cointime.prices` (3 series each): `vaulted` → `vaulted`, `active` → `active`, `cointime` → `cointime`
- `cointime.supply` (2 series each): `vaulted` → `vaulted_supply`, `hodled_or_lost` → `hodled_or_lost_supply`, `active` → `active_supply`
- `cointime.value` (6 series each): `destroyed` → `cointime_value_destroyed`, `created` → `cointime_value_created`, `stored` → `cointime_value_stored`
- `entry.discount.activity.transfer_volume.sum` (2 series each): `24h` → `entry_discount_transfer_volume_sum_24h`, `1w` → `entry_discount_transfer_volume_sum_1w`, `1m` → `entry_discount_transfer_volume_sum_1m`, `1y` → `entry_discount_transfer_volume_sum_1y`
- `entry.discount.realized` (6 series each): `profit` → `entry_discount_realized_profit`, `loss` → `entry_discount_realized_loss`, `net_pnl` → `entry_discount_net_realized_pnl`, `value_destroyed` → `entry_discount_value_destroyed`
- `entry.discount.supply` (2 series each): `total` → `entry_discount_supply`, `in_profit` → `entry_discount_supply_in_profit`, `in_loss` → `entry_discount_supply_in_loss`
- `entry.premium.activity.transfer_volume.sum` (2 series each): `24h` → `entry_premium_transfer_volume_sum_24h`, `1w` → `entry_premium_transfer_volume_sum_1w`, `1m` → `entry_premium_transfer_volume_sum_1m`, `1y` → `entry_premium_transfer_volume_sum_1y`
- `entry.premium.realized` (6 series each): `profit` → `entry_premium_realized_profit`, `loss` → `entry_premium_realized_loss`, `net_pnl` → `entry_premium_net_realized_pnl`, `value_destroyed` → `entry_premium_value_destroyed`
- `entry.premium.supply` (2 series each): `total` → `entry_premium_supply`, `in_profit` → `entry_premium_supply_in_profit`, `in_loss` → `entry_premium_supply_in_loss`
- `holders` (217 series each): `sth` → `sth`, `lth` → `lth`, `under_4m` → `age_under_4m`, `under_6m` → `age_under_6m`, `over_4m` → `age_over_4m`, `over_6m` → `age_over_6m`
- `holders.all.activity.transfer_volume.in_loss.sum` (2 series each): `24h` → `transfer_volume_in_loss_sum_24h`, `1w` → `transfer_volume_in_loss_sum_1w`, `1m` → `transfer_volume_in_loss_sum_1m`, `1y` → `transfer_volume_in_loss_sum_1y`
- `holders.all.activity.transfer_volume.in_profit.sum` (2 series each): `24h` → `transfer_volume_in_profit_sum_24h`, `1w` → `transfer_volume_in_profit_sum_1w`, `1m` → `transfer_volume_in_profit_sum_1m`, `1y` → `transfer_volume_in_profit_sum_1y`
- `holders.all.activity.transfer_volume.sum` (2 series each): `24h` → `transfer_volume_sum_24h`, `1w` → `transfer_volume_sum_1w`, `1m` → `transfer_volume_sum_1m`, `1y` → `transfer_volume_sum_1y`
- `holders.all.realized` (4 series each): `profit_to_loss` → `realized_profit_to_loss`, `sopr` → `sopr`, `sell_side_risk_ratio` → `sell_side_risk_ratio`
- `holders.all.realized` (6 series each): `profit` → `realized_profit`, `loss` → `realized_loss`, `net_pnl` → `net_realized_pnl`, `gross_pnl` → `gross_realized_pnl`, `value_destroyed` → `value_destroyed`, `peak_regret` → `realized_peak_regret`
- `indexer.transactions.features` (2 series each): `p2pk` → `p2pk`, `p2pkh` → `p2pkh`, `p2ms` → `p2ms`, `p2sh` → `p2sh`, `p2wpkh` → `p2wpkh`, `p2wsh` → `p2wsh`, `p2tr` → `p2tr`, `p2a` → `p2a`, `empty_script` → `empty_script`, `unknown_script` → `unknown_script`, `op_return` → `op_return`, `fake_pubkey` → `fake_pubkey`, `fake_scripthash` → `fake_scripthash`, `inscription` → `inscription`, `annex` → `annex`, `sighash_all` → `sighash_all`, `sighash_none` → `sighash_none`, `sighash_single` → `sighash_single`, `sighash_default` → `sighash_default`, `sighash_anyone_can_pay` → `sighash_anyone_can_pay`, `dust_output` → `dust_output`
- `indexer.transactions.versions` (1 series each): `v1` → `v1_tx_count`, `v2` → `v2_tx_count`, `v3` → `v3_tx_count`, `other` → `other_version_tx_count`
- `inputs` (6 series each): `p2tr_script_path` → `p2tr_script_path_input_count`, `inscriptions` → `inscription_input_count`, `same_block` → `same_block_spent_utxo_count`
- `inputs.count` (4 series each): `sum` → `input_count_sum`, `avg` → `input_count_avg`, `min` → `input_count_min`, `max` → `input_count_max`, `p10` → `input_count_p10`, `p25` → `input_count_p25`, `median` → `input_count_median`, `p75` → `input_count_p75`, `p90` → `input_count_p90`
- `inputs.multisig` (6 series each): `p2sh` → `p2sh_multisig_input_count`, `p2sh_p2wsh` → `p2sh_p2wsh_multisig_input_count`, `p2wsh` → `p2wsh_multisig_input_count`
- `inputs.opcodes` (6 series each): `checksig` → `op_checksig_in_input_count`, `checksigverify` → `op_checksigverify_in_input_count`, `checkmultisig` → `op_checkmultisig_in_input_count`, `checksigadd` → `op_checksigadd_in_input_count`, `checklocktimeverify` → `op_checklocktimeverify_in_input_count`, `checksequenceverify` → `op_checksequenceverify_in_input_count`, `hash160` → `op_hash160_in_input_count`, `sha256` → `op_sha256_in_input_count`, `if` → `op_if_in_input_count`, `notif` → `op_notif_in_input_count`
- `inputs.signatures.ecdsa` (6 series each): `count` → `ecdsa_signature_count`, `bytes` → `ecdsa_signature_bytes`, `strict_der` → `strict_der_ecdsa_signature_count`, `low_r` → `low_r_ecdsa_signature_count`, `low_s` → `low_s_ecdsa_signature_count`, `low_r_low_s` → `low_r_low_s_ecdsa_signature_count`
- `inputs.signatures.ecdsa.length` (6 series each): `70_bytes` → `ecdsa_70_bytes_signature_count`, `71_bytes` → `ecdsa_71_bytes_signature_count`, `72_bytes` → `ecdsa_72_bytes_signature_count`, `73_bytes` → `ecdsa_73_bytes_signature_count`, `74_bytes` → `ecdsa_74_bytes_signature_count`, `over_74_bytes` → `ecdsa_over_74_bytes_signature_count`
- `inputs.signatures.ecdsa.sighash` (6 series each): `all` → `sighash_all_ecdsa_signature_count`, `none` → `sighash_none_ecdsa_signature_count`, `single` → `sighash_single_ecdsa_signature_count`, `all_anyone_can_pay` → `sighash_all_anyone_can_pay_ecdsa_signature_count`, `none_anyone_can_pay` → `sighash_none_anyone_can_pay_ecdsa_signature_count`, `single_anyone_can_pay` → `sighash_single_anyone_can_pay_ecdsa_signature_count`
- `inputs.spent_within` (6 series each): `1_block` → `within_1_block_spent_utxo_count`, `6_blocks` → `within_6_blocks_spent_utxo_count`, `144_blocks` → `within_144_blocks_spent_utxo_count`, `2016_blocks` → `within_2016_blocks_spent_utxo_count`
- `inputs.types` (22 series each): `p2pk` → `p2pk_input`, `p2pkh` → `p2pkh_input`, `p2ms` → `p2ms_input`, `p2sh` → `p2sh_input`, `p2wpkh` → `p2wpkh_input`, `p2wsh` → `p2wsh_input`, `p2tr` → `p2tr_input`, `p2a` → `p2a_input`, `empty_script` → `empty_script_input`, `unknown_script` → `unknown_script_input`
- `market.ema` (2 series each): `1w` → `price_ema_1w`, `8d` → `price_ema_8d`, `13d` → `price_ema_13d`, `21d` → `price_ema_21d`, `1m` → `price_ema_1m`, `34d` → `price_ema_34d`, `50d` → `price_ema_50d`, `55d` → `price_ema_55d`, `89d` → `price_ema_89d`, `144d` → `price_ema_144d`, `200d` → `price_ema_200d`, `1y` → `price_ema_1y`, `2y` → `price_ema_2y`, `200w` → `price_ema_200w`, `4y` → `price_ema_4y`
- `market.sma` (2 series each): `1w` → `price_sma_1w`, `8d` → `price_sma_8d`, `13d` → `price_sma_13d`, `21d` → `price_sma_21d`, `1m` → `price_sma_1m`, `34d` → `price_sma_34d`, `50d` → `price_sma_50d`, `55d` → `price_sma_55d`, `89d` → `price_sma_89d`, `144d` → `price_sma_144d`, `1y` → `price_sma_1y`, `2y` → `price_sma_2y`, `200w` → `price_sma_200w`, `4y` → `price_sma_4y`
- `mining.rewards.coinbase.sum` (2 series each): `24h` → `coinbase_sum_24h`, `1w` → `coinbase_sum_1w`, `1m` → `coinbase_sum_1m`, `1y` → `coinbase_sum_1y`
- `mining.rewards.fees` (8 series each): `sum` → `fees_sum`, `avg` → `fees_avg`, `min` → `fees_min`, `max` → `fees_max`, `p10` → `fees_p10`, `p25` → `fees_p25`, `median` → `fees_median`, `p75` → `fees_p75`, `p90` → `fees_p90`
- `mining.rewards.subsidy.sum` (2 series each): `24h` → `subsidy_sum_24h`, `1w` → `subsidy_sum_1w`, `1m` → `subsidy_sum_1m`, `1y` → `subsidy_sum_1y`
- `mining.rewards.unclaimed.sum` (2 series each): `24h` → `unclaimed_rewards_sum_24h`, `1w` → `unclaimed_rewards_sum_1w`, `1m` → `unclaimed_rewards_sum_1m`, `1y` → `unclaimed_rewards_sum_1y`
- `op_return` (6 series each): `output_count` → `op_return_output_count`, `tx_count` → `op_return_tx_count`, `tx_vsize` → `op_return_tx_vsize`
- `op_return.fees.sum` (2 series each): `24h` → `op_return_fees_sum_24h`, `1w` → `op_return_fees_sum_1w`, `1m` → `op_return_fees_sum_1m`, `1y` → `op_return_fees_sum_1y`
- `op_return.policies` (51 series each): `pre_v30_standard` → `op_return_pre_v30_standard`, `pre_v30_nonstandard` → `op_return_pre_v30_nonstandard`, `oversized` → `op_return_oversized`, `multiple` → `op_return_multiple`
- `op_return.protocols` (51 series each): `runes` → `op_return_runes`, `witness_commitment` → `op_return_witness_commitment`, `veriblock` → `op_return_veriblock`, `omni` → `op_return_omni`, `stacks` → `op_return_stacks`, `blockstack` → `op_return_blockstack`, `colu` → `op_return_colu`, `open_assets` → `op_return_open_assets`, `komodo` → `op_return_komodo`, `coinspark` → `op_return_coinspark`, `poet` → `op_return_poet`, `docproof` → `op_return_docproof`, `factom` → `op_return_factom`, `eternity_wall` → `op_return_eternity_wall`, `memo` → `op_return_memo`, `bitproof` → `op_return_bitproof`, `ascribe` → `op_return_ascribe`, `stampery` → `op_return_stampery`, `bip47` → `op_return_bip47`, `bare_hash` → `op_return_bare_hash`, `text` → `op_return_text`, `empty` → `op_return_empty`, `unknown` → `op_return_unknown`
- `op_return.sizes` (6 series each): `over_42_bytes` → `op_return_data_over_42_bytes_output_count`, `over_520_bytes` → `op_return_data_over_520_bytes_output_count`, `over_10k_bytes` → `op_return_data_over_10k_bytes_output_count`
- `outputs.coinbase.types` (6 series each): `p2pk` → `coinbase_p2pk_output_count`, `p2pkh` → `coinbase_p2pkh_output_count`, `p2ms` → `coinbase_p2ms_output_count`, `p2sh` → `coinbase_p2sh_output_count`, `p2wpkh` → `coinbase_p2wpkh_output_count`, `p2wsh` → `coinbase_p2wsh_output_count`, `p2tr` → `coinbase_p2tr_output_count`, `p2a` → `coinbase_p2a_output_count`, `empty_script` → `coinbase_empty_script_output_count`, `unknown_script` → `coinbase_unknown_script_output_count`, `op_return` → `coinbase_op_return_output_count`
- `outputs.count` (4 series each): `sum` → `output_count_sum`, `avg` → `output_count_avg`, `min` → `output_count_min`, `max` → `output_count_max`, `p10` → `output_count_p10`, `p25` → `output_count_p25`, `median` → `output_count_median`, `p75` → `output_count_p75`, `p90` → `output_count_p90`
- `outputs.op_return_value.sum` (2 series each): `24h` → `op_return_value_sum_24h`, `1w` → `op_return_value_sum_1w`, `1m` → `op_return_value_sum_1m`, `1y` → `op_return_value_sum_1y`
- `outputs.types` (34 series each): `p2pk` → `p2pk_output`, `p2pkh` → `p2pkh_output`, `p2ms` → `p2ms_output`, `p2sh` → `p2sh_output`, `p2wpkh` → `p2wpkh_output`, `p2wsh` → `p2wsh_output`, `p2tr` → `p2tr_output`, `p2a` → `p2a_output`, `empty_script` → `empty_script_output`, `unknown_script` → `unknown_script_output`
- `pools` (32 series each): `unknown` → `pool_unknown`, `blockfills` → `pool_blockfills`, `ultimuspool` → `pool_ultimuspool`, `terrapool` → `pool_terrapool`, `luxor` → `pool_luxor`, `1thash` → `pool_1thash`, `cloverpool` → `pool_cloverpool`, `bitfarms` → `pool_bitfarms`, `huobipool` → `pool_huobipool`, `wayicn` → `pool_wayicn`, `canoepool` → `pool_canoepool`, `btctop` → `pool_btctop`, `bitcoincom` → `pool_bitcoincom`, `175btc` → `pool_175btc`, `gbminers` → `pool_gbminers`, `axbt` → `pool_axbt`, `asicminer` → `pool_asicminer`, `bitminter` → `pool_bitminter`, `bitcoinrussia` → `pool_bitcoinrussia`, `btcserv` → `pool_btcserv`, `simplecoinus` → `pool_simplecoinus`, `btcguild` → `pool_btcguild`, `eligius` → `pool_eligius`, `ozcoin` → `pool_ozcoin`, `eclipsemc` → `pool_eclipsemc`, `maxbtc` → `pool_maxbtc`, `triplemining` → `pool_triplemining`, `coinlab` → `pool_coinlab`, `50btc` → `pool_50btc`, `ghashio` → `pool_ghashio`, `stminingcorp` → `pool_stminingcorp`, `bitparking` → `pool_bitparking`, `mmpool` → `pool_mmpool`, `polmine` → `pool_polmine`, `kncminer` → `pool_kncminer`, `bitalo` → `pool_bitalo`, `f2pool` → `pool_f2pool`, `hhtt` → `pool_hhtt`, `megabigpower` → `pool_megabigpower`, `mtred` → `pool_mtred`, `nmcbit` → `pool_nmcbit`, `yourbtcnet` → `pool_yourbtcnet`, `givemecoins` → `pool_givemecoins`, `braiinspool` → `pool_braiinspool`, `antpool` → `pool_antpool`, `multicoinco` → `pool_multicoinco`, `bcpoolio` → `pool_bcpoolio`, `cointerra` → `pool_cointerra`, `kanopool` → `pool_kanopool`, `solock` → `pool_solock`, `ckpool` → `pool_ckpool`, `nicehash` → `pool_nicehash`, `bitclub` → `pool_bitclub`, `bitcoinaffiliatenetwork` → `pool_bitcoinaffiliatenetwork`, `btcc` → `pool_btcc`, `bwpool` → `pool_bwpool`, `exxbw` → `pool_exxbw`, `bitsolo` → `pool_bitsolo`, `bitfury` → `pool_bitfury`, `21inc` → `pool_21inc`, `digitalbtc` → `pool_digitalbtc`, `8baochi` → `pool_8baochi`, `mybtccoinpool` → `pool_mybtccoinpool`, `tbdice` → `pool_tbdice`, `hashpool` → `pool_hashpool`, `nexious` → `pool_nexious`, `bravomining` → `pool_bravomining`, `hotpool` → `pool_hotpool`, `okexpool` → `pool_okexpool`, `bcmonster` → `pool_bcmonster`, `1hash` → `pool_1hash`, `bixin` → `pool_bixin`, `tatmaspool` → `pool_tatmaspool`, `viabtc` → `pool_viabtc`, `connectbtc` → `pool_connectbtc`, `batpool` → `pool_batpool`, `waterhole` → `pool_waterhole`, `dcexploration` → `pool_dcexploration`, `dcex` → `pool_dcex`, `btpool` → `pool_btpool`, `58coin` → `pool_58coin`, `bitcoinindia` → `pool_bitcoinindia`, `shawnp0wers` → `pool_shawnp0wers`, `phashio` → `pool_phashio`, `rigpool` → `pool_rigpool`, `haozhuzhu` → `pool_haozhuzhu`, `7pool` → `pool_7pool`, `miningkings` → `pool_miningkings`, `hashbx` → `pool_hashbx`, `dpool` → `pool_dpool`, `rawpool` → `pool_rawpool`, `haominer` → `pool_haominer`, `helix` → `pool_helix`, `bitcoinukraine` → `pool_bitcoinukraine`, `poolin` → `pool_poolin`, `secretsuperstar` → `pool_secretsuperstar`, `tigerpoolnet` → `pool_tigerpoolnet`, `sigmapoolcom` → `pool_sigmapoolcom`, `okpooltop` → `pool_okpooltop`, `hummerpool` → `pool_hummerpool`, `tangpool` → `pool_tangpool`, `bytepool` → `pool_bytepool`, `spiderpool` → `pool_spiderpool`, `novablock` → `pool_novablock`, `miningcity` → `pool_miningcity`, `binancepool` → `pool_binancepool`, `minerium` → `pool_minerium`, `lubiancom` → `pool_lubiancom`, `okkong` → `pool_okkong`, `aaopool` → `pool_aaopool`, `emcdpool` → `pool_emcdpool`, `foundryusa` → `pool_foundryusa`, `sbicrypto` → `pool_sbicrypto`, `arkpool` → `pool_arkpool`, `purebtccom` → `pool_purebtccom`, `marapool` → `pool_marapool`, `kucoinpool` → `pool_kucoinpool`, `entrustcharitypool` → `pool_entrustcharitypool`, `okminer` → `pool_okminer`, `titan` → `pool_titan`, `pegapool` → `pool_pegapool`, `btcnuggets` → `pool_btcnuggets`, `cloudhashing` → `pool_cloudhashing`, `digitalxmintsy` → `pool_digitalxmintsy`, `telco214` → `pool_telco214`, `btcpoolparty` → `pool_btcpoolparty`, `multipool` → `pool_multipool`, `transactioncoinmining` → `pool_transactioncoinmining`, `btcdig` → `pool_btcdig`, `trickysbtcpool` → `pool_trickysbtcpool`, `btcmp` → `pool_btcmp`, `eobot` → `pool_eobot`, `unomp` → `pool_unomp`, `patels` → `pool_patels`, `gogreenlight` → `pool_gogreenlight`, `bitcoinindiapool` → `pool_bitcoinindiapool`, `ekanembtc` → `pool_ekanembtc`, `canoe` → `pool_canoe`, `tiger` → `pool_tiger`, `1m1x` → `pool_1m1x`, `zulupool` → `pool_zulupool`, `secpool` → `pool_secpool`, `ocean` → `pool_ocean`, `whitepool` → `pool_whitepool`, `wiz` → `pool_wiz`, `wk057` → `pool_wk057`, `futurebitapollosolo` → `pool_futurebitapollosolo`, `carbonnegative` → `pool_carbonnegative`, `portlandhodl` → `pool_portlandhodl`, `phoenix` → `pool_phoenix`, `neopool` → `pool_neopool`, `maxipool` → `pool_maxipool`, `bitfufupool` → `pool_bitfufupool`, `gdpool` → `pool_gdpool`, `miningdutch` → `pool_miningdutch`, `publicpool` → `pool_publicpool`, `miningsquared` → `pool_miningsquared`, `innopolistech` → `pool_innopolistech`, `btclab` → `pool_btclab`, `parasite` → `pool_parasite`, `redrockpool` → `pool_redrockpool`, `est3lar` → `pool_est3lar`, `braiinssolo` → `pool_braiinssolo`, `solopoolcom` → `pool_solopoolcom`, `noderunners` → `pool_noderunners`, `dmnd` → `pool_dmnd`, `trustpool` → `pool_trustpool`
- `profitability` (150 series each): `all` → (none), `sth` → `sth`, `lth` → `lth`
- `rarity_meter` (21 series each): `full` → `rarity_meter`, `local` → `local_rarity_meter`, `cycle` → `cycle_rarity_meter`
- `rarity_meter.components.all` (38 series each): `realized_price` → `rarity`, `capitalized_price` → `rarity`, `vaulted_price` → `rarity`, `true_market_mean` → `rarity`
- `rarity_meter.components.all.median_capitalized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.all.median_realized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.awake.realized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.mobile.median_capitalized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.mobile.median_realized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.mobile.realized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.sth.capitalized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.sth.median_capitalized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.sth.median_realized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.sth.realized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.under_6m.capitalized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.components.under_6m.realized_price` (2 series each): `p0_1` → `rarity_p0_1`, `p0_5` → `rarity_p0_5`, `p1` → `rarity_p1`, `p2` → `rarity_p2`, `p5` → `rarity_p5`, `p10` → `rarity_p10`, `p20` → `rarity_p20`, `p30` → `rarity_p30`, `p40` → `rarity_p40`, `median` → `rarity_median`, `p60` → `rarity_p60`, `p70` → `rarity_p70`, `p80` → `rarity_p80`, `p90` → `rarity_p90`, `p95` → `rarity_p95`, `p98` → `rarity_p98`, `p99` → `rarity_p99`, `p99_5` → `rarity_p99_5`, `p99_9` → `rarity_p99_9`
- `rarity_meter.extremes` (20 series each): `supply_in_loss` → `supply_in_loss`, `realized_profit_sum_24h` → `realized_profit_sum_24h`, `realized_loss_sum_24h` → `realized_loss_sum_24h`, `realized_peak_regret_sum_24h` → `realized_peak_regret_sum_24h`, `sell_side_risk_ratio_24h` → `sell_side_risk_ratio_24h`
- `supply.burned.sum` (2 series each): `24h` → `burned_sum_24h`, `1w` → `burned_sum_1w`, `1m` → `burned_sum_1m`, `1y` → `burned_sum_1y`
- `transactions.count` (4 series each): `sum` → `tx_count_sum`, `avg` → `tx_count_avg`, `min` → `tx_count_min`, `max` → `tx_count_max`, `p10` → `tx_count_p10`, `p25` → `tx_count_p25`, `median` → `tx_count_median`, `p75` → `tx_count_p75`, `p90` → `tx_count_p90`
- `transactions.features` (6 series each): `one_input_one_output` → `one_input_one_output_tx_count`, `one_input_two_output` → `one_input_two_output_tx_count`, `bip69` → `bip69_tx_count`, `bip69_expected_by_chance` → `bip69_expected_by_chance_tx_count`
- `transactions.features` (28 series each): `segwit` → `segwit_tx`, `one_input` → `one_input_tx`, `one_output` → `one_output_tx`
- `transactions.features.dust_output.value.sum` (2 series each): `24h` → `dust_output_tx_value_sum_24h`, `1w` → `dust_output_tx_value_sum_1w`, `1m` → `dust_output_tx_value_sum_1m`, `1y` → `dust_output_tx_value_sum_1y`
- `transactions.features.epobc.value.sum` (2 series each): `24h` → `epobc_tx_value_sum_24h`, `1w` → `epobc_tx_value_sum_1w`, `1m` → `epobc_tx_value_sum_1m`, `1y` → `epobc_tx_value_sum_1y`
- `transactions.features.rbf_signaling.value.sum` (2 series each): `24h` → `rbf_signaling_tx_value_sum_24h`, `1w` → `rbf_signaling_tx_value_sum_1w`, `1m` → `rbf_signaling_tx_value_sum_1m`, `1y` → `rbf_signaling_tx_value_sum_1y`
- `transactions.features.scripts` (29 series each): `p2pk` → `p2pk`, `p2pkh` → `p2pkh`, `p2ms` → `p2ms`, `p2sh` → `p2sh`, `p2wpkh` → `p2wpkh`, `p2wsh` → `p2wsh`, `p2tr` → `p2tr`, `p2a` → `p2a`, `empty_script` → `empty_script`, `unknown_script` → `unknown_script`, `fake_pubkey` → `fake_pubkey`, `fake_scripthash` → `fake_scripthash`
- `transactions.features.signatures` (29 series each): `annex` → `annex`, `sighash_all` → `sighash_all`, `sighash_none` → `sighash_none`, `sighash_single` → `sighash_single`, `sighash_default` → `sighash_default`, `sighash_anyone_can_pay` → `sighash_anyone_can_pay`
- `transactions.fees.cpfp_child.value.sum` (2 series each): `24h` → `cpfp_child_tx_value_sum_24h`, `1w` → `cpfp_child_tx_value_sum_1w`, `1m` → `cpfp_child_tx_value_sum_1m`, `1y` → `cpfp_child_tx_value_sum_1y`
- `transactions.fees.cpfp_parent.value.sum` (2 series each): `24h` → `cpfp_parent_tx_value_sum_24h`, `1w` → `cpfp_parent_tx_value_sum_1w`, `1m` → `cpfp_parent_tx_value_sum_1m`, `1y` → `cpfp_parent_tx_value_sum_1y`
- `transactions.fees.tx_count_by_fee_rate` (6 series each): `under_0p1_sat_vb` → `fee_rate_under_0p1_sat_vb_tx_count`, `0p1_to_0p25_sat_vb` → `fee_rate_0p1_to_0p25_sat_vb_tx_count`, `0p25_to_0p5_sat_vb` → `fee_rate_0p25_to_0p5_sat_vb_tx_count`, `0p5_to_1_sat_vb` → `fee_rate_0p5_to_1_sat_vb_tx_count`, `1_to_2_sat_vb` → `fee_rate_1_to_2_sat_vb_tx_count`, `2_to_5_sat_vb` → `fee_rate_2_to_5_sat_vb_tx_count`, `5_to_10_sat_vb` → `fee_rate_5_to_10_sat_vb_tx_count`, `10_to_25_sat_vb` → `fee_rate_10_to_25_sat_vb_tx_count`, `25_to_50_sat_vb` → `fee_rate_25_to_50_sat_vb_tx_count`, `50_to_100_sat_vb` → `fee_rate_50_to_100_sat_vb_tx_count`, `100_to_250_sat_vb` → `fee_rate_100_to_250_sat_vb_tx_count`, `250_to_500_sat_vb` → `fee_rate_250_to_500_sat_vb_tx_count`, `500_to_1k_sat_vb` → `fee_rate_500_to_1k_sat_vb_tx_count`, `over_1k_sat_vb` → `fee_rate_over_1k_sat_vb_tx_count`
- `transactions.fees.vsize_by_effective_fee_rate` (6 series each): `under_0p1_sat_vb` → `effective_fee_rate_under_0p1_sat_vb_tx_vsize`, `0p1_to_0p25_sat_vb` → `effective_fee_rate_0p1_to_0p25_sat_vb_tx_vsize`, `0p25_to_0p5_sat_vb` → `effective_fee_rate_0p25_to_0p5_sat_vb_tx_vsize`, `0p5_to_1_sat_vb` → `effective_fee_rate_0p5_to_1_sat_vb_tx_vsize`, `1_to_2_sat_vb` → `effective_fee_rate_1_to_2_sat_vb_tx_vsize`, `2_to_5_sat_vb` → `effective_fee_rate_2_to_5_sat_vb_tx_vsize`, `5_to_10_sat_vb` → `effective_fee_rate_5_to_10_sat_vb_tx_vsize`, `10_to_25_sat_vb` → `effective_fee_rate_10_to_25_sat_vb_tx_vsize`, `25_to_50_sat_vb` → `effective_fee_rate_25_to_50_sat_vb_tx_vsize`, `50_to_100_sat_vb` → `effective_fee_rate_50_to_100_sat_vb_tx_vsize`, `100_to_250_sat_vb` → `effective_fee_rate_100_to_250_sat_vb_tx_vsize`, `250_to_500_sat_vb` → `effective_fee_rate_250_to_500_sat_vb_tx_vsize`, `500_to_1k_sat_vb` → `effective_fee_rate_500_to_1k_sat_vb_tx_vsize`, `over_1k_sat_vb` → `effective_fee_rate_over_1k_sat_vb_tx_vsize`
- `transactions.inscription.fees.sum` (2 series each): `24h` → `inscription_fees_sum_24h`, `1w` → `inscription_fees_sum_1w`, `1m` → `inscription_fees_sum_1m`, `1y` → `inscription_fees_sum_1y`
- `transactions.inscription.value.sum` (2 series each): `24h` → `inscription_tx_value_sum_24h`, `1w` → `inscription_tx_value_sum_1w`, `1m` → `inscription_tx_value_sum_1m`, `1y` → `inscription_tx_value_sum_1y`
- `transactions.nonstandard.value.sum` (2 series each): `24h` → `nonstandard_tx_value_sum_24h`, `1w` → `nonstandard_tx_value_sum_1w`, `1m` → `nonstandard_tx_value_sum_1m`, `1y` → `nonstandard_tx_value_sum_1y`
- `transactions.patterns` (29 series each): `coinjoin` → `coinjoin`, `consolidation` → `consolidation`, `batch_payout` → `batch_payout`
- `transactions.payments` (6 series each): `count` → `payment_count`, `segwit_spending` → `segwit_spending_payment_count`, `rbf_signaling` → `rbf_signaling_payment_count`
- `transactions.spending` (6 series each): `only_segwit` → `only_segwit_spending_tx_count`, `only_legacy` → `only_legacy_spending_tx_count`, `segwit_and_legacy` → `segwit_and_legacy_spending_tx_count`
- `transactions.timelocks` (6 series each): `height` → `height_locktime_tx_count`, `time` → `time_locktime_tx_count`, `unenforced` → `unenforced_locktime_tx_count`, `relative_height` → `relative_height_locktime_tx_count`, `relative_time` → `relative_time_locktime_tx_count`
- `transactions.transfer_volume.sum` (2 series each): `24h` → `transfer_volume_sum_24h`, `1w` → `transfer_volume_sum_1w`, `1m` → `transfer_volume_sum_1m`, `1y` → `transfer_volume_sum_1y`
- `transactions.versions` (28 series each): `v1` → `v1_tx`, `v2` → `v2_tx`, `v3` → `v3_tx`, `other` → `other_version_tx`
- `utxos.amounts` (63 series each): `1_to_10_sats` → `amount_1_to_10_sats`, `10_to_100_sats` → `amount_10_to_100_sats`, `100_to_1k_sats` → `amount_100_to_1k_sats`, `1k_to_10k_sats` → `amount_1k_to_10k_sats`, `10k_to_100k_sats` → `amount_10k_to_100k_sats`, `100k_to_1m_sats` → `amount_100k_to_1m_sats`, `1m_to_10m_sats` → `amount_1m_to_10m_sats`, `10m_to_100m_sats` → `amount_10m_to_100m_sats`, `1_to_10_btc` → `amount_1_to_10_btc`, `10_to_100_btc` → `amount_10_to_100_btc`, `100_to_1k_btc` → `amount_100_to_1k_btc`, `1k_to_10k_btc` → `amount_1k_to_10k_btc`, `10k_to_100k_btc` → `amount_10k_to_100k_btc`, `over_100k_btc` → `amount_over_100k_btc`
- `utxos.types` (63 series each): `p2pk` → `p2pk`, `p2pkh` → `p2pkh`, `p2ms` → `p2ms`, `p2sh` → `p2sh`, `p2wpkh` → `p2wpkh`, `p2wsh` → `p2wsh`, `p2tr` → `p2tr`, `p2a` → `p2a`, `empty_script` → `empty_script`, `unknown_script` → `unknown_script`
