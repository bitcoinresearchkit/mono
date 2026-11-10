//! A signature's sighash type as feature flags.

use bitcoin::sighash::{EcdsaSighashType, TapSighashType};

use crate::TxFeatureFlags;

pub fn ecdsa_sighash(sighash_type: EcdsaSighashType) -> u32 {
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

pub fn taproot_sighash(sighash_type: TapSighashType) -> u32 {
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
