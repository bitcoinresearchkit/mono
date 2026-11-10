use bitcoin::{Script, WitnessVersion};
use brk_types::OutputType;

use super::{super::scan::TxScan, revealed::Revealed};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    None,
    P2WPKH,
    P2WSH,
    Witness,
    Other,
}

#[derive(Clone, Copy)]
pub struct Facts {
    kind: Kind,
    script: Revealed,
}

impl Facts {
    pub fn analyze(
        last_push: Option<&[u8]>,
        output_type: OutputType,
        scan: &mut TxScan<'_>,
    ) -> Self {
        if output_type != OutputType::P2SH {
            return Self::NONE;
        }

        let Some(bytes) = last_push else {
            return Self::NONE;
        };
        let script = Script::from_bytes(bytes);
        let (kind, script) = match script.witness_version() {
            // A version-0 program, `OP_0` and a hash, reveals no opcode, key or signature
            // operation: not walked.
            Some(WitnessVersion::V0) if script.len() == 22 => (Kind::P2WPKH, Revealed::NONE),
            Some(WitnessVersion::V0) if script.len() == 34 => (Kind::P2WSH, Revealed::NONE),
            Some(_) => (Kind::Witness, Revealed::of(script, scan)),
            None => (Kind::Other, Revealed::of(script, scan)),
        };

        Self { kind, script }
    }

    /// The type whose rules the witness is read by: a nested witness program's, once it has its
    /// witness, else the spent output's own.
    pub fn effective_output_type(
        self,
        output_type: OutputType,
        witness_items: usize,
    ) -> OutputType {
        match self.kind {
            Kind::P2WPKH if witness_items == 2 => OutputType::P2WPKH,
            Kind::P2WSH if witness_items > 0 => OutputType::P2WSH,
            _ => output_type,
        }
    }

    pub fn is_present(self) -> bool {
        self.kind != Kind::None
    }

    pub fn is_p2wpkh(self) -> bool {
        self.kind == Kind::P2WPKH
    }

    pub fn is_p2wsh(self) -> bool {
        self.kind == Kind::P2WSH
    }

    pub fn is_witness_program(self) -> bool {
        matches!(self.kind, Kind::P2WPKH | Kind::P2WSH | Kind::Witness)
    }

    pub fn is_multisig(self) -> bool {
        self.script.multisig
    }

    pub fn sigops(self) -> Option<usize> {
        self.is_present().then_some(self.script.sigops)
    }

    pub const NONE: Self = Self {
        kind: Kind::None,
        script: Revealed::NONE,
    };
}
