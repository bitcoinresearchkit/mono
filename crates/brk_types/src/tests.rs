use crate::{AddressOutputType, Cents, CentsSats, Cpfp, OutputType, Sats};

/// Each of mempool.space's CPFP shapes decodes to its own variant (untagged: first match wins).
#[test]
fn cpfp_shapes() {
    let decode = |json: &str| serde_json::from_str::<Cpfp>(json).unwrap();
    let txid = "\"965f866bf8623bbf956c1b2aeec1efc1ad162fd428ab7fb89f128a0754ebbc32\"";
    let entry = format!("{{\"txid\":{txid},\"weight\":616,\"fee\":155500}}");
    let cluster = format!(
        "{{\"txs\":[{{\"txid\":{txid},\"weight\":694,\"fee\":174000,\"parents\":[]}}],\"chunks\":[{{\"txs\":[0],\"feerate\":1000.0}}],\"chunkIndex\":0}}"
    );
    let mempool = format!(
        "{{\"ancestors\":[{entry}],\"bestDescendant\":null,\"descendants\":[],\"effectiveFeePerVsize\":1000.0,\"sigops\":0,\"fee\":174000,\"vsize\":174,\"adjustedVsize\":174,\"cluster\":{cluster}}}"
    );
    let confirmed = format!(
        "{{\"ancestors\":[{entry}],\"descendants\":[],\"effectiveFeePerVsize\":1000.0,\"cluster\":{cluster}}}"
    );
    assert!(matches!(decode(&mempool), Cpfp::Mempool(_)));
    assert!(matches!(decode(&confirmed), Cpfp::Confirmed(_)));
    assert!(matches!(decode("{\"ancestors\":[]}"), Cpfp::Empty(_)));
    assert_eq!(
        serde_json::to_string(&Cpfp::EMPTY).unwrap(),
        "{\"ancestors\":[]}"
    );
}

/// The address types are exactly the output types that have addresses.
#[test]
fn address_output_types() {
    let names = [
        "p2pk65",
        "p2pk33",
        "p2pkh",
        "p2ms",
        "p2sh",
        "op_return",
        "p2wpkh",
        "p2wsh",
        "p2tr",
        "p2a",
        "empty_script",
        "unknown_script",
    ];
    assert_eq!(names.len(), OutputType::COUNT);
    for name in names {
        let json = format!("\"{name}\"");
        let output_type = serde_json::from_str::<OutputType>(&json).unwrap();
        let parsed = serde_json::from_str::<AddressOutputType>(&json);
        assert_eq!(parsed.is_ok(), output_type.is_addr(), "{name}");
        if let Ok(address_type) = parsed {
            assert_eq!(serde_json::to_string(&address_type).unwrap(), json);
        }
    }
}

/// An amount's value rounds half a cent up; past u64 it widens to u128 and still agrees with the
/// exact product; a NaN price stays NaN, and so does a realized price over no sats.
#[test]
fn values_round_half_up_and_widen_past_u64() {
    let at = |sats: u64, cents_per_btc: u64| Sats::from(sats).value_at(Cents::from(cents_per_btc));
    assert_eq!(
        at(1, 50_000_000),
        Cents::from(1u64),
        "half a cent rounds up"
    );
    assert_eq!(at(1, 49_999_999), Cents::from(0u64));
    // 21M BTC at $100M: the u64 product overflows, the u128 path takes it.
    let (supply, price) = (2_100_000_000_000_000u64, 10_000_000_000u64);
    assert!(supply.checked_mul(price).is_none());
    assert_eq!(at(supply, price), Cents::from(210_000_000_000_000_000u64));
    for (sats, price) in [
        (1, 50_000_000),
        (123_456_789, 6_543_210_987),
        (u32::MAX as u64, 99),
    ] {
        let exact = CentsSats::from_price_sats(Cents::from(price), Sats::from(sats)).to_cents();
        assert_eq!(at(sats, price), exact, "both paths round alike");
    }
    assert!(Sats::from(1u64).value_at(Cents::NAN).is_nan());
    assert!(
        CentsSats::from_price_sats(Cents::from(1u64), Sats::from(1u64))
            .realized_price(Sats::ZERO)
            .is_nan()
    );
}
