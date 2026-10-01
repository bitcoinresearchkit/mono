mod facts;
mod script_sig_facts;
mod witness_facts;

pub use facts::Facts;
pub use script_sig_facts::ScriptSigFacts;
pub use witness_facts::WitnessFacts;

pub mod redeem;
pub mod script_sig;
pub mod witness;

use bitcoin::TxIn;
use brk_types::OutputType;

use crate::TxFeatureFlags;

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
    fn empty_script_sig_has_empty_facts() {
        let input = TxIn::default();
        let facts = analyze(&input, OutputType::Unknown, &mut TxFeatureFlags::default());

        assert_eq!(facts.script_sig.accurate_sigops, 0);
        assert_eq!(facts.script_sig.last_push, None);
        assert_eq!(facts.script_sig.legacy_sigops, 0);
        assert!(facts.script_sig.push_only);
    }
}
