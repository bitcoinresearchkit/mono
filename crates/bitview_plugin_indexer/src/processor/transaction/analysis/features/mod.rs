mod sighash;

use crate::TxFeatureFlags;

pub fn scan_ecdsa_signature(bytes: &[u8], flags: &mut TxFeatureFlags) {
    sighash::scan_ecdsa_signature(bytes, flags);
}

pub fn scan_taproot_signature(bytes: &[u8], flags: &mut TxFeatureFlags) {
    sighash::scan_taproot_signature(bytes, flags);
}

pub fn record_validated_ecdsa_sighash(bytes: &[u8], flags: &mut TxFeatureFlags) {
    sighash::record_validated_ecdsa_sighash(bytes, flags);
}

pub fn record_validated_taproot_sighash(bytes: &[u8], flags: &mut TxFeatureFlags) {
    sighash::record_validated_taproot_sighash(bytes, flags);
}
