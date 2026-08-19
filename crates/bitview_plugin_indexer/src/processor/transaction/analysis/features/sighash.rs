use bitcoin::{
    secp256k1::{ecdsa::Signature as EcdsaSignature, schnorr::Signature as SchnorrSignature},
    sighash::{EcdsaSighashType, TapSighashType},
};

use crate::TxFeatureFlags;

const MIN_ECDSA_SIGNATURE_BYTES: usize = 9;
const MAX_ECDSA_SIGNATURE_BYTES: usize = 73;

pub fn scan_ecdsa_signature(bytes: &[u8], flags: &mut TxFeatureFlags) {
    if !is_ecdsa_signature_candidate(bytes) {
        return;
    }

    let (sighash_byte, der) = bytes.split_last().unwrap();
    let Ok(sighash_type) = EcdsaSighashType::from_standard(u32::from(*sighash_byte)) else {
        return;
    };
    let candidate_flags = ecdsa_feature_flags(sighash_type);
    if flags.contains_all(candidate_flags) {
        return;
    }

    if EcdsaSignature::from_der(der).is_err() {
        return;
    }
    flags.insert(candidate_flags);
}

pub fn scan_taproot_signature(bytes: &[u8], flags: &mut TxFeatureFlags) {
    let (sighash_type, signature) = match bytes.len() {
        64 => (TapSighashType::Default, bytes),
        65 => {
            let Ok(sighash_type) = TapSighashType::from_consensus_u8(bytes[64]) else {
                return;
            };
            (sighash_type, &bytes[..64])
        }
        _ => return,
    };
    let candidate_flags = taproot_feature_flags(sighash_type);
    if flags.contains_all(candidate_flags) {
        return;
    }

    if SchnorrSignature::from_slice(signature).is_err() {
        return;
    }
    flags.insert(candidate_flags);
}

pub fn record_validated_ecdsa_sighash(bytes: &[u8], flags: &mut TxFeatureFlags) {
    let Ok(sighash_type) = EcdsaSighashType::from_standard(u32::from(*bytes.last().unwrap()))
    else {
        return;
    };
    flags.insert(ecdsa_feature_flags(sighash_type));
}

pub fn record_validated_taproot_sighash(bytes: &[u8], flags: &mut TxFeatureFlags) {
    let sighash_type = match bytes.len() {
        64 => TapSighashType::Default,
        65 => {
            let Ok(sighash_type) = TapSighashType::from_consensus_u8(bytes[64]) else {
                return;
            };
            sighash_type
        }
        _ => return,
    };
    flags.insert(taproot_feature_flags(sighash_type));
}

pub fn ecdsa_feature_flags(sighash_type: EcdsaSighashType) -> u32 {
    let mut flags = match sighash_type {
        EcdsaSighashType::All | EcdsaSighashType::AllPlusAnyoneCanPay => {
            TxFeatureFlags::SIGHASH_ALL
        }
        EcdsaSighashType::None | EcdsaSighashType::NonePlusAnyoneCanPay => {
            TxFeatureFlags::SIGHASH_NONE
        }
        EcdsaSighashType::Single | EcdsaSighashType::SinglePlusAnyoneCanPay => {
            TxFeatureFlags::SIGHASH_SINGLE
        }
    };
    if sighash_type.to_u32() & 0x80 != 0 {
        flags |= TxFeatureFlags::SIGHASH_ANYONE_CAN_PAY;
    }
    flags
}

#[inline]
pub fn is_ecdsa_signature_candidate(bytes: &[u8]) -> bool {
    (MIN_ECDSA_SIGNATURE_BYTES..=MAX_ECDSA_SIGNATURE_BYTES).contains(&bytes.len())
        && bytes.first() == Some(&0x30)
}

pub fn taproot_feature_flags(sighash_type: TapSighashType) -> u32 {
    let mut flags = match sighash_type {
        TapSighashType::Default => TxFeatureFlags::SIGHASH_DEFAULT,
        TapSighashType::All | TapSighashType::AllPlusAnyoneCanPay => TxFeatureFlags::SIGHASH_ALL,
        TapSighashType::None | TapSighashType::NonePlusAnyoneCanPay => TxFeatureFlags::SIGHASH_NONE,
        TapSighashType::Single | TapSighashType::SinglePlusAnyoneCanPay => {
            TxFeatureFlags::SIGHASH_SINGLE
        }
    };
    if matches!(
        sighash_type,
        TapSighashType::AllPlusAnyoneCanPay
            | TapSighashType::NonePlusAnyoneCanPay
            | TapSighashType::SinglePlusAnyoneCanPay
    ) {
        flags |= TxFeatureFlags::SIGHASH_ANYONE_CAN_PAY;
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::is_ecdsa_signature_candidate;

    #[test]
    pub fn rejects_impossible_ecdsa_signature_encodings() {
        assert!(!is_ecdsa_signature_candidate(&[0x30; 8]));
        assert!(is_ecdsa_signature_candidate(&[0x30; 9]));
        assert!(is_ecdsa_signature_candidate(&[0x30; 73]));
        assert!(!is_ecdsa_signature_candidate(&[0x30; 74]));
        assert!(!is_ecdsa_signature_candidate(&[0x02; 33]));
    }
}
