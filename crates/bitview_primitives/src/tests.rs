use brk_types::{
    AddrBytes, OutputType, P2ABytes, P2PK33Bytes, P2PK65Bytes, P2PKHBytes, P2SHBytes, P2TRBytes,
    P2WPKHBytes, P2WSHBytes,
};

use crate::AddrHash;

#[test]
fn hashes_borrowed_script_payloads_like_owned_addresses() {
    let addresses = [
        AddrBytes::from(P2PK65Bytes::from(&[0x04; 65][..])),
        AddrBytes::from(P2PK33Bytes::from(&[0x02; 33][..])),
        AddrBytes::from(P2PKHBytes::from(&[0x03; 20][..])),
        AddrBytes::from(P2SHBytes::from(&[0x04; 20][..])),
        AddrBytes::from(P2WPKHBytes::from(&[0x05; 20][..])),
        AddrBytes::from(P2WSHBytes::from(&[0x06; 32][..])),
        AddrBytes::from(P2TRBytes::from(&[0x07; 32][..])),
        AddrBytes::from(P2ABytes::from(&[78, 115][..])),
    ];

    for address in addresses {
        let script = address.to_script_pubkey();
        let output_type = OutputType::from(&script);

        assert_eq!(
            AddrHash::from_script(&script, output_type).unwrap(),
            AddrHash::from(&address)
        );
    }
}

/// The `scalar!` values: NaN-first float order, undefined floats in JSON/CSV.
#[test]
fn scalar_values() {
    use crate::Float32;

    assert_eq!(Float32::new(f32::NAN), Float32::new(f32::NAN));
    assert!(Float32::new(f32::NAN) < Float32::new(f32::MIN));

    #[cfg(feature = "storage")]
    {
        use vecdb::Formattable;
        let json = |value: Float32| {
            let mut buf = Vec::new();
            value.fmt_json(&mut buf);
            String::from_utf8(buf).unwrap()
        };
        assert_eq!(json(Float32::new(38.2)), "38.2");
        assert_eq!(json(Float32::new(f32::INFINITY)), "null");
        let mut csv = String::new();
        Float32::new(f32::NAN).fmt_csv(&mut csv).unwrap();
        assert!(csv.is_empty());
    }
}

/// Date indexes label each bucket where it starts, the clients' contract: a label maps back to its index and the
/// second before it to the previous one (none before the first).
#[test]
fn date_labels_start_their_buckets() {
    use brk_types::Timestamp;

    use crate::Index;

    for index in Index::all().into_iter().filter(Index::is_date_based) {
        for i in 0..=10 {
            let label = index.index_to_timestamp(i).unwrap();
            assert_eq!(index.timestamp_to_index(label), Some(i), "{index:?} {i}");
            if let Some(date) = index.index_to_date(i) {
                assert_eq!(index.date_to_index(date), Some(i), "{index:?} {i}");
            }
            assert_eq!(
                index.timestamp_to_index(Timestamp::new(*label - 1)),
                i.checked_sub(1),
                "{index:?} {i}"
            );
        }
    }
}
