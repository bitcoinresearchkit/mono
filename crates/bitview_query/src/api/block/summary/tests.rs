use super::*;
use bitcoin::Witness;
use bitview_primitives::Boolean;

fn facts(nonstandard: bool, coinjoin: bool, consolidation: bool) -> bitview_plugin_transactions::SummaryFacts {
    bitview_plugin_transactions::SummaryFacts {
        fee: Sats::ZERO, vsize: brk_types::VSize::new(1), rate: brk_types::FeeRate::ZERO,
        nonstandard, coinjoin, consolidation,
    }
}

#[test]
fn point_cold_selection_never_loads_block_classifications() {
    // The real resolve path calls select_point before summary_facts/feature/raw reads. Even a
    // huge enclosing block selects one row; a full-block txid scan is not called on this path.
    let selected = select_point(1_000..101_000, Some(12_345), || panic!("whole-block read")).unwrap();
    let reads: Vec<_> = selected.collect();
    assert_eq!(reads, [12_345]);
}

#[test]
fn membership_rejects_wrong_block_and_handles_another_txid_occurrence() {
    assert!(matches!(select_point(10..20, Some(22), || Ok(None)), Err(Error::NotFound(_))));
    assert!(matches!(select_point(10..20, None, || Ok(None)), Err(Error::NotFound(_))));
    assert_eq!(select_point(10..20, Some(22), || Ok(Some(10))).unwrap(), 10..11);
    assert_eq!(select_point(10..20, None, || Ok(Some(19))).unwrap(), 19..20);
    // A fallback cannot smuggle a transaction outside the supplied canonical block either.
    assert!(select_point(10..20, Some(22), || Ok(Some(20))).is_err());
}

#[test]
fn forced_flags_preserve_high_bits_and_exclude_observation_bits() {
    // Every feature bit is forced independently, then all together through JSON; >u32 bits
    // are essential, especially ANYONECANPAY (bit 44). These are literal wire positions.
    let bits = [0, 8, 9, 10, 11, 12, 13, 14, 24, 25, 26, 27, 28, 40, 41, 42, 43, 44];
    let mut combined = 0;
    for bit in bits {
        let expected = 1u64 << bit;
        let mut masks = [0, 0];
        or_column(&mut masks, expected, vec![Boolean::from(true), Boolean::from(false)]).unwrap();
        assert_eq!(masks, [expected, 0]);
        combined |= expected;
    }
    let mask = finish_flags(combined, TxVersion::THREE, &facts(true, true, true), 1, 5, true, true);
    assert_eq!(mask & 3, 1); // never RBF and NO_RBF simultaneously
    assert_ne!(mask & (1 << 44), 0);
    assert_ne!(mask & (1 << 34), 0);
    assert_eq!(mask & (1 << 32), 0); // envelope excludes CoinJoin
    assert_eq!(mask & (3 << 18), 0); // no invented replacement/acceleration
    let text = serde_json::to_string(&mask).unwrap();
    assert_eq!(serde_json::from_str::<u64>(&text).unwrap(), mask);
    assert!(mask < 1 << 53);
    assert!(or_column(&mut [0, 0], 1, vec![Boolean::from(true)]).is_err());
}

#[test]
fn forced_roles_and_native_policy_are_independent_of_fee_rate() {
    // Equal standalone/chunk rates do not erase structural relatives. Native policy is a
    // supplied, stored classification; the summary does not reinterpret its activation height.
    let mask = finish_flags(0, TxVersion::TWO, &facts(true, true, false), 5, 5, true, true);
    assert_eq!(mask, 2 | 8 | 32 | (3 << 16) | (1 << 32));
    let token = finish_flags(flags::OP_RETURN, TxVersion::ONE, &facts(false, true, false), 5, 5, false, false);
    assert_eq!(token & flags::COINJOIN, 0);
    let coinbase = finish_flags(0, TxVersion::ONE, &facts(false, false, false), 1, 5, false, false);
    assert_ne!(coinbase & flags::BATCH_PAYOUT, 0);
    let below = finish_flags(0, TxVersion::ONE, &facts(false, false, false), 1, 4, false, false);
    assert_eq!(below & flags::BATCH_PAYOUT, 0);
    let huge = finish_flags(0, TxVersion::ONE, &facts(false, false, false), usize::MAX, usize::MAX, false, false);
    assert_eq!(huge & flags::BATCH_PAYOUT, 0);
}

#[test]
fn sums_and_published_ranges_fail_instead_of_wrapping_or_truncating() {
    assert_eq!(add_output(17, Sats::from(23u64)).unwrap(), 40);
    assert!(add_output(u64::MAX, Sats::from(1u64)).is_err());
    assert!(add_output(2_100_000_000_000_000, Sats::from(1u64)).is_err());
    assert!(checked_end(usize::MAX, 1, usize::MAX).is_err());
    assert!(checked_end(10, 2, 11).is_err());
    assert!(checked_end(10, 0, 10).is_err());
    assert_eq!(checked_end(10, 2, 12).unwrap(), 12);
    assert!(exact(vec![1], 2).is_err());
}

#[test]
fn broader_envelope_scan_uses_script_instructions_and_witness_shape() {
    let witness = |script: &[u8], annex: bool| {
        let mut w = Witness::new();
        w.push([0u8; 64]);
        w.push(script);
        w.push([0xc0u8; 33]);
        if annex { w.push([0x50, 0x00]); }
        w
    };
    // Broad envelope has no ord payload; both witness forms must detect it.
    assert!(has_taproot_envelope(&witness(&[0x00, 0x63, 0x68], false)));
    assert!(has_taproot_envelope(&witness(&[0x00, 0x63, 0x68], true)));
    assert!(has_taproot_envelope(&witness(&[0x00, 0x63, 3, b'o', b'r', b'd'], false)));
    // Bytes inside pushed data are not opcodes; OP_IFDUP is not OP_IF.
    assert!(!has_taproot_envelope(&witness(&[2, 0x00, 0x63], false)));
    assert!(!has_taproot_envelope(&witness(&[0x00, 0x73], false)));
    assert!(!has_taproot_envelope(&Witness::new()));
    let mut keypath = Witness::new();
    keypath.push([0u8; 64]);
    assert!(!has_taproot_envelope(&keypath));
    keypath.push([0x50, 0x00]);
    assert!(!has_taproot_envelope(&keypath));
}

#[test]
fn summary_omits_unknown_observations_and_keeps_integer_vsize() {
    let summary = TransactionSummary {
        txid: Txid::COINBASE, fee: Sats::from(7u64), vsize: brk_types::VSize::from(101u64),
        value: Sats::from(123u64), rate: brk_types::FeeRate::from_milli(501),
        flags: flags::NO_RBF | flags::V1 | flags::SIGHASH_ACP,
    };
    let json = serde_json::to_value(summary).unwrap();
    assert_eq!(json.as_object().unwrap().len(), 6);
    assert_eq!(json["vsize"], 101);
    assert_eq!(json["rate"], 0.501);
    assert!(json.get("time").is_none());
    assert!(json.get("acc").is_none());
}
