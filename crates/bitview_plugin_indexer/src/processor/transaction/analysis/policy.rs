use bitcoin::{
    Amount, Script, Transaction, TxIn, TxOut, WitnessVersion, constants::WITNESS_SCALE_FACTOR,
    policy::MAX_STANDARD_TX_SIGOPS_COST, taproot::LeafVersion,
};
use brk_types::{Height, OutputType, SigOps};

use super::{super::ComputedTx, input, sigops::ComputedSigOps};
use crate::{TxFeatureFlags, processor::txout::ProcessedOutput};

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
const MAX_STANDARD_BARE_MULTISIG_SIGOP_COST: u32 = 3 * WITNESS_SCALE_FACTOR as u32;

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
                .redeem
                .sigops()
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
            if facts.redeem.sigops().is_none() {
                return true;
            }
            if facts.redeem.is_p2wsh() {
                has_nonstandard_p2wsh_witness(&facts.witness)
            } else {
                !facts.redeem.is_witness_program()
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
    use bitcoin::{Amount, ScriptBuf};
    use brk_types::{
        AddrBytes, OutputType, P2ABytes, P2PK33Bytes, P2PK65Bytes, P2PKHBytes, P2SHBytes,
        P2TRBytes, P2WPKHBytes, P2WSHBytes,
    };

    use super::is_dust;

    #[test]
    fn uses_exact_dust_thresholds_for_fixed_scripts() {
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
}
