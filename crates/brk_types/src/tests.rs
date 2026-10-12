use crate::{
    AddressBytes, OutputType, P2ABytes, P2PK33Bytes, P2PK65Bytes, P2PKHBytes, P2SHBytes, P2TRBytes,
    P2WPKHBytes, P2WSHBytes,
};

#[test]
fn script_pubkeys_round_trip_to_addresses() {
    let addresses = [
        AddressBytes::from(P2PK65Bytes::from(&[0x04; 65][..])),
        AddressBytes::from(P2PK33Bytes::from(&[0x02; 33][..])),
        AddressBytes::from(P2PKHBytes::from(&[0x03; 20][..])),
        AddressBytes::from(P2SHBytes::from(&[0x04; 20][..])),
        AddressBytes::from(P2WPKHBytes::from(&[0x05; 20][..])),
        AddressBytes::from(P2WSHBytes::from(&[0x06; 32][..])),
        AddressBytes::from(P2TRBytes::from(&[0x07; 32][..])),
        AddressBytes::from(P2ABytes::from(&[78, 115][..])),
    ];

    for address in addresses {
        let script = address.to_script_pubkey();
        let output_type = OutputType::from(&script);

        assert_eq!(
            AddressBytes::try_from((&script, output_type)).unwrap(),
            address
        );
    }
}
