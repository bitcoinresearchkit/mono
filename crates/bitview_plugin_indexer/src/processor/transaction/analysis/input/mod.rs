mod facts;
mod script_sig_facts;
mod witness_facts;

pub use facts::Facts;
pub use script_sig_facts::ScriptSigFacts;
pub use witness_facts::WitnessFacts;

pub mod redeem;
pub mod revealed;
pub mod script_sig;
pub mod witness;

use bitcoin::TxIn;
use brk_types::OutputType;

use super::scan::TxScan;

/// An input's scripts and witness, each walked once, what they hold handed to `scan`.
pub fn analyze<'a>(input: &'a TxIn, output_type: OutputType, scan: &mut TxScan<'_>) -> Facts<'a> {
    scan.flags.insert_type(output_type);

    let script_sig = if input.script_sig.is_empty() {
        ScriptSigFacts::EMPTY
    } else {
        script_sig::analyze(&input.script_sig, output_type, scan)
    };
    let redeem = redeem::Facts::analyze(script_sig.last_push, output_type, scan);
    let witness = witness::analyze(
        &input.witness,
        redeem.effective_output_type(output_type, input.witness.len()),
        scan,
    );

    Facts {
        script_sig,
        redeem,
        witness,
    }
}
