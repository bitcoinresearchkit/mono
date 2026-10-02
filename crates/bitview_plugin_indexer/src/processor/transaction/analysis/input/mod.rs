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
