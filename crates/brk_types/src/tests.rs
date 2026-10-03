use crate::{
    AddrBytes, AddrHash, OutputType, P2ABytes, P2PK33Bytes, P2PK65Bytes, P2PKHBytes, P2SHBytes,
    P2TRBytes, P2WPKHBytes, P2WSHBytes,
};

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
        assert_eq!(
            AddrBytes::try_from((&script, output_type)).unwrap(),
            address
        );
    }
}
