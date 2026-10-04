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
