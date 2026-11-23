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

/// A transaction's JSON reads back through serde as the same transaction, and its keys are the
/// schema's (what the clients are generated from): the writer, the `Deserialize` derives and the
/// schema each name the fields, so this keeps them one set. The sample has every optional key:
/// a coinbase, a P2SH-P2WSH spend, a taproot script-path spend, outputs with and without an
/// address, a malformed script and a confirmed status.
#[test]
fn transaction_json_round_trip() {
    use bitcoin::{ScriptBuf, Witness, absolute::LockTime, transaction::Version};

    use crate::{BlockHash, Chain, Height, SigOps, Timestamp, Transaction, TxIn, TxIndex, TxOut};
    use crate::{TxStatus, Txid, Vout, Weight};

    Chain::MAIN.activate().unwrap();
    let script = |bytes: Vec<u8>| ScriptBuf::from_bytes(bytes);
    let out = |bytes: Vec<u8>, sats: u64| TxOut::from((script(bytes), Sats::from(sats)));
    let with = |head: &[u8], body: &[u8], tail: &[u8]| [head, body, tail].concat();
    let input = |prevout, script_sig, witness: &[Vec<u8>]| TxIn {
        txid: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"
            .parse::<Txid>()
            .unwrap(),
        vout: Vout::from(1u32),
        prevout,
        script_sig,
        witness: Witness::from_slice(witness),
        is_coinbase: false,
        sequence: 0xffff_fffd,
    };
    let p2wsh = with(&[0x00, 0x20], &[7; 32], &[]);
    let tx = Transaction {
        index: Some(TxIndex::new(42)),
        txid: "2bb85f4b004be6da54f766c17c1e855187327112c231ef2ff35ebad0ea67c69e"
            .parse()
            .unwrap(),
        version: Version::TWO.into(),
        lock_time: LockTime::ZERO.into(),
        input: vec![
            TxIn {
                txid: Txid::COINBASE,
                vout: Vout::from(u32::MAX),
                prevout: None,
                is_coinbase: true,
                sequence: u32::MAX,
                ..input(None, script(vec![0x03, 1, 2, 3]), &[vec![0; 32]])
            },
            input(
                Some(out(with(&[0xa9, 0x14], &[1; 20], &[0x87]), 5_000)),
                script(with(&[0x22], &p2wsh, &[])),
                &[vec![0x30; 71], vec![0x51]],
            ),
            input(
                Some(out(with(&[0x51, 0x20], &[2; 32], &[]), 6_000)),
                ScriptBuf::new(),
                &[vec![1; 64], vec![0xac], with(&[0xc0], &[3; 32], &[])],
            ),
        ],
        output: vec![
            out(with(&[0x00, 0x14], &[4; 20], &[]), 1_000),
            out(with(&[0x76, 0xa9, 0x14], &[5; 20], &[0x88, 0xac]), 2_000),
            out(with(&[0x21, 0x02], &[6; 32], &[0xac]), 3_000),
            out(vec![0x6a, 0x04, 1, 2, 3, 4], 0),
            out(vec![0x4c], 0),
        ],
        total_size: 512,
        weight: Weight::new(1_337),
        total_sigop_cost: SigOps::new(5),
        fee: Sats::from(4_000u64),
        status: TxStatus::confirmed(
            Height::new(916_656),
            "000000000000000000012711f7e0d13e586752a42c66e25faf75f159b3d04911"
                .parse::<BlockHash>()
                .unwrap(),
            Timestamp::from(1_759_000_868usize),
        ),
    };
    let json = tx.to_json();
    let back: Transaction = serde_json::from_slice(&json).unwrap();
    assert_eq!(back.to_json(), json, "{}", String::from_utf8_lossy(&json));

    #[cfg(feature = "schemars")]
    {
        use std::collections::BTreeSet;

        use serde_json::Value;

        let schema = serde_json::to_value(schemars::schema_for!(Transaction)).unwrap();
        let properties = |schema: &Value| -> BTreeSet<String> {
            schema["properties"]
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect()
        };
        let keys = |objects: Vec<&Value>| -> BTreeSet<String> {
            objects
                .iter()
                .flat_map(|o| o.as_object().unwrap().keys().cloned())
                .collect()
        };
        let value: Value = serde_json::from_slice(&json).unwrap();
        let ins: Vec<&Value> = value["vin"].as_array().unwrap().iter().collect();
        let outs: Vec<&Value> = value["vout"].as_array().unwrap().iter().collect();
        assert_eq!(keys(vec![&value]), properties(&schema));
        assert_eq!(keys(ins), properties(&schema["$defs"]["TxIn"]));
        assert_eq!(keys(outs), properties(&schema["$defs"]["TxOut"]));
        assert_eq!(
            keys(vec![&value["status"]]),
            properties(&schema["$defs"]["TxStatus"])
        );
    }
}
