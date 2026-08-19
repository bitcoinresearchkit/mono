mod redeem;
mod script_sig;
mod witness;

use bitcoin::{TxIn, taproot::LeafVersion};
use brk_types::OutputType;

use crate::TxFeatureFlags;

pub struct Facts<'a> {
    pub script_sig: ScriptSigFacts<'a>,
    pub redeem: redeem::Facts,
    pub witness: WitnessFacts<'a>,
}

impl Facts<'_> {
    #[inline]
    pub fn redeem_sigops(&self) -> Option<usize> {
        self.redeem.sigops()
    }

    #[inline]
    pub fn redeem_is_p2wpkh(&self) -> bool {
        self.redeem.is_p2wpkh()
    }

    #[inline]
    pub fn redeem_is_p2wsh(&self) -> bool {
        self.redeem.is_p2wsh()
    }

    #[inline]
    pub fn redeem_is_witness_program(&self) -> bool {
        self.redeem.is_witness_program()
    }
}

pub struct ScriptSigFacts<'a> {
    pub accurate_sigops: usize,
    pub last_push: Option<&'a [u8]>,
    pub legacy_sigops: usize,
    pub push_only: bool,
}

impl ScriptSigFacts<'_> {
    pub const EMPTY: Self = Self {
        accurate_sigops: 0,
        last_push: None,
        legacy_sigops: 0,
        push_only: true,
    };
}

pub struct WitnessFacts<'a> {
    pub has_annex: bool,
    pub last: Option<&'a [u8]>,
    pub leaf_version: Option<LeafVersion>,
    pub max_argument_bytes: usize,
    pub stack_items: usize,
}

pub fn analyze<'a>(
    input: &'a TxIn,
    output_type: OutputType,
    flags: &mut TxFeatureFlags,
) -> Facts<'a> {
    flags.insert_type(output_type);

    let script_sig = if input.script_sig.is_empty() {
        ScriptSigFacts::EMPTY
    } else {
        script_sig::analyze(
            &input.script_sig,
            output_type,
            input.witness.is_empty(),
            flags,
        )
    };
    let redeem = redeem::Facts::analyze(script_sig.last_push, output_type);
    let witness = witness::analyze(
        &input.witness,
        redeem.effective_output_type(output_type, input.witness.len()),
        flags,
    );

    Facts {
        script_sig,
        redeem,
        witness,
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::TxIn;
    use brk_types::OutputType;

    use super::analyze;
    use crate::TxFeatureFlags;

    #[test]
    pub fn empty_script_sig_has_empty_facts() {
        let input = TxIn::default();
        let facts = analyze(&input, OutputType::Unknown, &mut TxFeatureFlags::default());

        assert_eq!(facts.script_sig.accurate_sigops, 0);
        assert_eq!(facts.script_sig.last_push, None);
        assert_eq!(facts.script_sig.legacy_sigops, 0);
        assert!(facts.script_sig.push_only);
    }
}
