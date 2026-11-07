//! The script payloads an address type indexes: fixed-size bytes, lowercase hex in JSON.

use crate::OutputType;

macro_rules! payload_bytes {
    ($($(#[$meta:meta])* $output_type:ident => $name:ident($len:literal),)+) => {
        crate::bytes::hex_bytes! {
            $($(#[$meta])* $name($len),)+
        }

        /// Each address type with its payload length in bytes.
        pub const ADDRESS_PAYLOAD_LENGTHS: &[(OutputType, usize)] = &[$((OutputType::$output_type, $len)),+];

        impl OutputType {
            /// The payload length an address of this type indexes; `None` for types without addresses.
            pub const fn payload_len(self) -> Option<usize> {
                match self {
                    $(Self::$output_type => Some($len),)+
                    _ => None,
                }
            }
        }
    };
}

payload_bytes! {
    /// An uncompressed (65-byte) public key.
    P2PK65 => P2PK65Bytes(65),
    /// A compressed (33-byte) public key.
    P2PK33 => P2PK33Bytes(33),
    /// The 20-byte public key hash of a P2PKH output.
    P2PKH => P2PKHBytes(20),
    /// The 20-byte script hash of a P2SH output.
    P2SH => P2SHBytes(20),
    /// The 20-byte public key hash of a P2WPKH output.
    P2WPKH => P2WPKHBytes(20),
    /// The 32-byte script hash of a P2WSH output.
    P2WSH => P2WSHBytes(32),
    /// The 32-byte output key of a P2TR output.
    P2TR => P2TRBytes(32),
    /// The 2-byte witness program of a P2A (pay-to-anchor) output.
    P2A => P2ABytes(2),
}
