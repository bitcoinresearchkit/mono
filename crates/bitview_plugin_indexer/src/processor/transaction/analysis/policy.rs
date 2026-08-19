use bitcoin::{
    Amount, Script, Transaction, TxIn, TxOut, WitnessVersion, policy::MAX_STANDARD_TX_SIGOPS_COST,
    taproot::LeafVersion,
};
use brk_types::{Height, OutputType, SigOps};

use super::super::ComputedTx;
use super::{input, sigops::ComputedSigOps};
use crate::TxFeatureFlags;
use crate::processor::txout::ProcessedOutput;

// Deterministic policy snapshots, not consensus activation heights.
const LAST_V2_POLICY_HEIGHT: u32 = 863_500;
const FIRST_V29_POLICY_HEIGHT: u32 = 892_500;
const FIRST_V30_POLICY_HEIGHT: u32 = 921_000;
// rust-bitcoin 0.32 still exposes Core's previous 82-byte policy value.
const MIN_STANDARD_TX_NONWITNESS_SIZE: u32 = 65;
const MAX_EXECUTED_LEGACY_SIGOP_COST: u32 = 10_000;
const MAX_SCRIPT_SIG_SIZE: usize = 1_650;
const MAX_P2SH_SIGOPS: usize = 15;
const MAX_V29_OP_RETURN_SCRIPT_BYTES: usize = 83;
const MAX_V30_OP_RETURN_SCRIPT_BYTES: usize = 100_000;
const MAX_P2WSH_SCRIPT_BYTES: usize = 3_600;
const MAX_P2WSH_STACK_ITEMS: usize = 100;
const MAX_WITNESS_STACK_ITEM_BYTES: usize = 80;
const MAX_STANDARD_BARE_MULTISIG_SIGOP_COST: u32 =
    3 * bitcoin::constants::WITNESS_SCALE_FACTOR as u32;

#[derive(Default)]
pub struct Accumulator {
    height: u32,
    nonstandard: bool,
    dust_output_count: usize,
    op_return_count: usize,
    op_return_script_bytes: usize,
}

impl Accumulator {
    pub fn new(height: Height) -> Self {
        Self {
            height: height.into(),
            ..Self::default()
        }
    }

    pub fn scan_input(&mut self, input: &TxIn, output_type: OutputType, facts: &input::Facts<'_>) {
        if input.script_sig.len() > MAX_SCRIPT_SIG_SIZE || !facts.script_sig.push_only {
            self.nonstandard = true;
            return;
        }

        self.nonstandard |= match output_type {
            OutputType::P2SH => facts
                .redeem_sigops()
                .is_some_and(|count| count > MAX_P2SH_SIGOPS),
            OutputType::P2A => p2a_spend_is_nonstandard(self.height),
            OutputType::P2TR => facts.witness.has_annex,
            OutputType::OpReturn | OutputType::Empty | OutputType::Unknown => true,
            _ => false,
        } || has_nonstandard_witness(output_type, facts);
    }

    pub fn scan_output(&mut self, txout: &TxOut, output: &ProcessedOutput) {
        let script = &txout.script_pubkey;
        self.nonstandard |= match output.output_type {
            OutputType::Empty => true,
            OutputType::Unknown => !is_standard_unknown_witness(script),
            OutputType::P2MS => has_too_many_bare_multisig_keys(output.legacy_sigops),
            OutputType::OpReturn => {
                self.op_return_count += 1;
                self.op_return_script_bytes += script.len();
                false
            }
            _ => false,
        };
        self.dust_output_count += is_dust(txout.value, output.output_type, script) as usize;
    }

    pub fn finish(
        mut self,
        tx: &ComputedTx<'_>,
        sigops: ComputedSigOps,
        flags: &mut TxFeatureFlags,
    ) {
        self.nonstandard |= op_return_is_nonstandard(
            self.height,
            self.op_return_count,
            self.op_return_script_bytes,
        );
        self.nonstandard |=
            has_unconditionally_nonstandard_dust(self.height, self.dust_output_count);
        self.nonstandard |= has_nonstandard_header(tx, sigops, self.height);

        if self.nonstandard {
            flags.insert(TxFeatureFlags::UNCONDITIONALLY_NONSTANDARD);
        }
        if self.dust_output_count > 0 {
            flags.insert(TxFeatureFlags::DUST_OUTPUT);
        }
    }
}

pub fn tracks_executed_legacy_sigops(height: Height) -> bool {
    u32::from(height) >= FIRST_V30_POLICY_HEIGHT
}

pub fn has_nonstandard_header(tx: &ComputedTx, sigops: ComputedSigOps, height: u32) -> bool {
    has_nonstandard_version(tx.tx.version.0, height)
        || tx.weight() > Transaction::MAX_STANDARD_WEIGHT
        || tx.base_size < MIN_STANDARD_TX_NONWITNESS_SIZE
        || u32::from(sigops.total) > MAX_STANDARD_TX_SIGOPS_COST
        || height >= FIRST_V30_POLICY_HEIGHT
            && u32::from(sigops.executed_legacy) > MAX_EXECUTED_LEGACY_SIGOP_COST
}

#[inline]
pub fn p2a_spend_is_nonstandard(height: u32) -> bool {
    height <= LAST_V2_POLICY_HEIGHT
}

#[inline]
pub fn op_return_is_nonstandard(height: u32, count: usize, script_bytes: usize) -> bool {
    if height < FIRST_V30_POLICY_HEIGHT {
        count > 1 || script_bytes > MAX_V29_OP_RETURN_SCRIPT_BYTES
    } else {
        script_bytes > MAX_V30_OP_RETURN_SCRIPT_BYTES
    }
}

pub fn has_unconditionally_nonstandard_dust(height: u32, dust_output_count: usize) -> bool {
    if height < FIRST_V29_POLICY_HEIGHT {
        dust_output_count > 0
    } else {
        dust_output_count > 1
    }
}

pub fn is_dust(value: Amount, output_type: OutputType, script: &Script) -> bool {
    let threshold = match output_type {
        OutputType::P2PK65 => 672,
        OutputType::P2PK33 => 576,
        OutputType::P2PKH => 546,
        OutputType::P2MS | OutputType::Unknown => {
            return value < script.minimal_non_dust();
        }
        OutputType::P2SH => 540,
        OutputType::OpReturn => return false,
        OutputType::P2WPKH => 294,
        OutputType::P2WSH | OutputType::P2TR => 330,
        OutputType::P2A => 240,
        OutputType::Empty => 471,
    };

    value.to_sat() < threshold
}

pub fn has_nonstandard_witness(output_type: OutputType, facts: &input::Facts<'_>) -> bool {
    if facts.witness.stack_items == 0 {
        return false;
    }

    match output_type {
        OutputType::P2A => true,
        OutputType::P2WPKH => false,
        OutputType::P2WSH => has_nonstandard_p2wsh_witness(&facts.witness),
        OutputType::P2TR => {
            facts.witness.has_annex || has_nonstandard_taproot_witness(&facts.witness)
        }
        OutputType::P2SH => {
            if facts.redeem_sigops().is_none() {
                return true;
            }
            if facts.redeem_is_p2wsh() {
                has_nonstandard_p2wsh_witness(&facts.witness)
            } else {
                !facts.redeem_is_witness_program()
            }
        }
        _ => true,
    }
}

pub fn has_nonstandard_p2wsh_witness(witness: &input::WitnessFacts<'_>) -> bool {
    let stack_items = witness.stack_items - 1;
    witness.last.unwrap().len() > MAX_P2WSH_SCRIPT_BYTES
        || stack_items > MAX_P2WSH_STACK_ITEMS
        || witness.max_argument_bytes > MAX_WITNESS_STACK_ITEM_BYTES
}

pub fn has_nonstandard_taproot_witness(witness: &input::WitnessFacts<'_>) -> bool {
    if witness.stack_items < 2 {
        return false;
    }

    witness.leaf_version.is_none()
        || witness.leaf_version == Some(LeafVersion::TapScript)
            && witness.max_argument_bytes > MAX_WITNESS_STACK_ITEM_BYTES
}

pub fn has_nonstandard_version(version: i32, height: u32) -> bool {
    let max = if height <= LAST_V2_POLICY_HEIGHT {
        2
    } else {
        3
    };
    !(1..=max).contains(&version)
}

pub fn is_standard_unknown_witness(script: &Script) -> bool {
    script
        .witness_version()
        .is_some_and(|version| version != WitnessVersion::V0)
}

pub fn has_too_many_bare_multisig_keys(sigops: SigOps) -> bool {
    u32::from(sigops) > MAX_STANDARD_BARE_MULTISIG_SIGOP_COST
}

#[cfg(test)]
mod tests {
    use bitcoin::{Amount, ScriptBuf, TxIn, Witness};
    use brk_types::{
        AddrBytes, Height, OutputType, P2ABytes, P2PK33Bytes, P2PK65Bytes, P2PKHBytes, P2SHBytes,
        P2TRBytes, P2WPKHBytes, P2WSHBytes, SigOps,
    };

    use super::super::input;
    use super::{
        FIRST_V29_POLICY_HEIGHT, FIRST_V30_POLICY_HEIGHT, LAST_V2_POLICY_HEIGHT,
        MAX_V29_OP_RETURN_SCRIPT_BYTES, MAX_V30_OP_RETURN_SCRIPT_BYTES,
        has_nonstandard_p2wsh_witness, has_nonstandard_taproot_witness, has_nonstandard_version,
        has_nonstandard_witness, has_too_many_bare_multisig_keys,
        has_unconditionally_nonstandard_dust, is_dust, is_standard_unknown_witness,
        op_return_is_nonstandard, p2a_spend_is_nonstandard, tracks_executed_legacy_sigops,
    };

    #[test]
    pub fn tracks_executed_sigops_only_when_policy_uses_them() {
        assert!(!tracks_executed_legacy_sigops(Height::from(
            FIRST_V30_POLICY_HEIGHT - 1
        )));
        assert!(tracks_executed_legacy_sigops(Height::from(
            FIRST_V30_POLICY_HEIGHT
        )));
    }

    #[test]
    pub fn accepts_only_policy_transaction_versions() {
        assert!(has_nonstandard_version(0, LAST_V2_POLICY_HEIGHT));
        assert!(has_nonstandard_version(-1, LAST_V2_POLICY_HEIGHT));
        assert!(!has_nonstandard_version(1, LAST_V2_POLICY_HEIGHT));
        assert!(!has_nonstandard_version(2, LAST_V2_POLICY_HEIGHT));
        assert!(has_nonstandard_version(3, LAST_V2_POLICY_HEIGHT));
        assert!(!has_nonstandard_version(3, LAST_V2_POLICY_HEIGHT + 1));
    }

    #[test]
    pub fn p2a_spending_starts_after_the_v2_policy_snapshot() {
        assert!(p2a_spend_is_nonstandard(LAST_V2_POLICY_HEIGHT));
        assert!(!p2a_spend_is_nonstandard(LAST_V2_POLICY_HEIGHT + 1));
    }

    #[test]
    pub fn op_return_limits_switch_at_the_v30_policy_snapshot() {
        assert!(!op_return_is_nonstandard(
            FIRST_V30_POLICY_HEIGHT - 1,
            1,
            MAX_V29_OP_RETURN_SCRIPT_BYTES
        ));
        assert!(op_return_is_nonstandard(
            FIRST_V30_POLICY_HEIGHT - 1,
            2,
            MAX_V29_OP_RETURN_SCRIPT_BYTES
        ));
        assert!(op_return_is_nonstandard(
            FIRST_V30_POLICY_HEIGHT - 1,
            1,
            MAX_V29_OP_RETURN_SCRIPT_BYTES + 1
        ));
        assert!(!op_return_is_nonstandard(
            FIRST_V30_POLICY_HEIGHT,
            2,
            MAX_V30_OP_RETURN_SCRIPT_BYTES
        ));
        assert!(op_return_is_nonstandard(
            FIRST_V30_POLICY_HEIGHT,
            1,
            MAX_V30_OP_RETURN_SCRIPT_BYTES + 1
        ));
    }

    #[test]
    pub fn applies_ephemeral_dust_policy_from_v29() {
        assert!(!has_unconditionally_nonstandard_dust(
            FIRST_V29_POLICY_HEIGHT - 1,
            0
        ));
        assert!(has_unconditionally_nonstandard_dust(
            FIRST_V29_POLICY_HEIGHT - 1,
            1
        ));
        assert!(!has_unconditionally_nonstandard_dust(
            FIRST_V29_POLICY_HEIGHT,
            1
        ));
        assert!(has_unconditionally_nonstandard_dust(
            FIRST_V29_POLICY_HEIGHT,
            2
        ));
    }

    #[test]
    pub fn uses_exact_dust_thresholds_for_fixed_scripts() {
        let scripts = [
            AddrBytes::from(P2PK65Bytes::from(&[0; 65][..])).to_script_pubkey(),
            AddrBytes::from(P2PK33Bytes::from(&[0; 33][..])).to_script_pubkey(),
            AddrBytes::from(P2PKHBytes::from(&[0; 20][..])).to_script_pubkey(),
            AddrBytes::from(P2SHBytes::from(&[0; 20][..])).to_script_pubkey(),
            AddrBytes::from(P2WPKHBytes::from(&[0; 20][..])).to_script_pubkey(),
            AddrBytes::from(P2WSHBytes::from(&[0; 32][..])).to_script_pubkey(),
            AddrBytes::from(P2TRBytes::from(&[0; 32][..])).to_script_pubkey(),
            AddrBytes::from(P2ABytes::from(&[0; 2][..])).to_script_pubkey(),
        ];
        let thresholds = [672, 576, 546, 540, 294, 330, 330, 240];

        for ((output_type, script), threshold) in OutputType::ADDR_TYPES
            .into_iter()
            .zip(&scripts)
            .zip(thresholds)
        {
            assert_eq!(script.minimal_non_dust().to_sat(), threshold);
            assert!(is_dust(
                Amount::from_sat(threshold - 1),
                output_type,
                script
            ));
            assert!(!is_dust(Amount::from_sat(threshold), output_type, script));
        }

        let empty = ScriptBuf::new();
        assert_eq!(empty.minimal_non_dust().to_sat(), 471);
        assert!(is_dust(Amount::from_sat(470), OutputType::Empty, &empty));
        assert!(!is_dust(Amount::from_sat(471), OutputType::Empty, &empty));
    }

    #[test]
    pub fn keeps_exact_dust_calculation_for_unknown_scripts() {
        let script = ScriptBuf::from_bytes(vec![0x61; 1_000]);
        let threshold = script.minimal_non_dust().to_sat();

        assert!(is_dust(
            Amount::from_sat(threshold - 1),
            OutputType::Unknown,
            &script
        ));
        assert!(!is_dust(
            Amount::from_sat(threshold),
            OutputType::Unknown,
            &script
        ));
    }

    #[test]
    pub fn recognizes_future_witness_programs() {
        assert!(is_standard_unknown_witness(
            &ScriptBuf::from_hex(
                "52200000000000000000000000000000000000000000000000000000000000000000"
            )
            .unwrap()
        ));
        assert!(!is_standard_unknown_witness(
            &ScriptBuf::from_hex(
                "00200000000000000000000000000000000000000000000000000000000000000000"
            )
            .unwrap()
        ));
    }

    #[test]
    pub fn limits_standard_bare_multisig_to_three_keys() {
        assert!(!has_too_many_bare_multisig_keys(SigOps::new(12)));
        assert!(has_too_many_bare_multisig_keys(SigOps::new(16)));
    }

    #[test]
    pub fn rejects_p2a_witness_stuffing() {
        let input = TxIn {
            witness: Witness::from_slice(&[b"stuffing"]),
            ..TxIn::default()
        };
        let facts = input::analyze(
            &input,
            OutputType::P2A,
            &mut crate::TxFeatureFlags::default(),
        );
        assert!(has_nonstandard_witness(OutputType::P2A, &facts,));
    }

    #[test]
    pub fn enforces_witness_stack_item_limits() {
        let oversized = [0_u8; 81];
        let input = TxIn {
            witness: Witness::from_slice(&[oversized.as_slice(), [0x51].as_slice()]),
            ..TxIn::default()
        };
        let facts = input::analyze(
            &input,
            OutputType::P2WSH,
            &mut crate::TxFeatureFlags::default(),
        );
        assert!(has_nonstandard_p2wsh_witness(&facts.witness));

        let control_block = [0xc0_u8; 33];
        let input = TxIn {
            witness: Witness::from_slice(&[
                oversized.as_slice(),
                [0x51].as_slice(),
                control_block.as_slice(),
            ]),
            ..TxIn::default()
        };
        let facts = input::analyze(
            &input,
            OutputType::P2TR,
            &mut crate::TxFeatureFlags::default(),
        );
        assert!(has_nonstandard_taproot_witness(&facts.witness));
    }
}
