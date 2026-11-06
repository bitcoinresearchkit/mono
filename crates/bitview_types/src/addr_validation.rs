use bitcoin::{
    base58,
    bech32::{Bech32, Bech32m, primitives::decode::UncheckedHrpstring},
    hex::DisplayHex,
};
use brk_types::{Address, Hex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressValidation {
    #[schemars(example = true)]
    pub isvalid: bool,

    /// The validated address
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "address")]
    pub addr: Option<Address>,

    /// The scriptPubKey in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: Option<Hex>,

    /// Whether the address is P2SH, P2WSH or P2TR
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isscript: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub iswitness: Option<bool>,

    /// Witness version (0 for P2WPKH/P2WSH, 1 for P2TR, 2 to 16 for future versions; none for P2A)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_version: Option<u8>,

    /// Witness program in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness_program: Option<Hex>,

    /// Positions of the characters at fault in an invalid address (Bitcoin Core's), possibly none;
    /// omitted for valid ones
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_locations: Option<Vec<usize>>,

    /// Error message for invalid addresses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl AddressValidation {
    /// Bitcoin Core's `validateaddress` (mempool.space answers with it): the same decoding rules
    /// in the same order (`DecodeDestination`), the same errors, and the same fields per type.
    /// One difference: Core locates up to two errors in a Bech32 checksum; this one, with a
    /// single wrong character, says where it is, and with more, only that the checksum fails.
    pub fn from_addr(addr: &str) -> Self {
        match decode(addr) {
            Ok(destination) => destination.describe(),
            Err((error, error_locations)) => Self {
                isvalid: false,
                addr: None,
                script_pub_key: None,
                isscript: None,
                iswitness: None,
                witness_version: None,
                witness_program: None,
                error_locations: Some(error_locations),
                error: Some(error),
            },
        }
    }
}

/// An address Core accepts, as it describes it.
struct Destination {
    /// As Core writes it: Bech32 in lowercase.
    address: String,
    script: Vec<u8>,
    kind: Kind,
}

enum Kind {
    PubkeyHash,
    ScriptHash,
    /// P2WPKH (0, 20 bytes), P2WSH (0, 32), P2TR (1, 32) and versions Core doesn't know yet.
    Witness {
        version: u8,
        program: Vec<u8>,
    },
    Anchor,
}

impl Destination {
    fn describe(self) -> AddressValidation {
        let (isscript, iswitness, witness_version, witness_program) = match self.kind {
            Kind::PubkeyHash => (Some(false), Some(false), None, None),
            Kind::ScriptHash => (Some(true), Some(false), None, None),
            Kind::Anchor => (Some(true), Some(true), None, None),
            Kind::Witness { version, program } => {
                let isscript = match (version, program.len()) {
                    (0, 20) => Some(false),
                    (0, 32) | (1, 32) => Some(true),
                    _ => None,
                };
                let program = Hex::from(program.to_lower_hex_string());
                (isscript, Some(true), Some(version), Some(program))
            }
        };
        AddressValidation {
            isvalid: true,
            addr: Some(Address::from(self.address)),
            script_pub_key: Some(Hex::from(self.script.to_lower_hex_string())),
            isscript,
            iswitness,
            witness_version,
            witness_program,
            error_locations: None,
            error: None,
        }
    }
}

type Invalid = (String, Vec<usize>);

const HRP: &str = "bc";
const PUBKEY_PREFIX: u8 = 0x00;
const SCRIPT_PREFIX: u8 = 0x05;
/// Bech32's character set, a 5-bit value's character at its index.
const CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
const CHECKSUM_LEN: usize = 6;
const BECH32_MAX_LEN: usize = 90;
const WITNESS_PROGRAM_MAX: usize = 40;

fn invalid(error: impl Into<String>) -> Invalid {
    (error.into(), Vec::new())
}

/// Core's `DecodeDestination` for mainnet.
fn decode(s: &str) -> Result<Destination, Invalid> {
    let is_bech32 = s
        .get(..HRP.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(HRP));
    if !is_bech32 {
        return decode_base58(s);
    }
    let Some((checksum, data)) = bech32_data(s) else {
        return Err(locate_errors(s));
    };
    let Some(&version) = data.first() else {
        return Err(invalid("Empty Bech32 data section"));
    };
    let hrp = &s[..s.rfind('1').expect("a separator")].to_ascii_lowercase();
    if hrp != HRP {
        return Err(invalid(format!(
            "Invalid or unsupported prefix for Segwit (Bech32) address (expected {HRP}, got {hrp})."
        )));
    }
    match (version, checksum) {
        (0, Checksum::Bech32m) => {
            return Err(invalid(
                "Version 0 witness address must use Bech32 checksum",
            ));
        }
        (1.., Checksum::Bech32) => {
            return Err(invalid(
                "Version 1+ witness address must use Bech32m checksum",
            ));
        }
        _ => {}
    }
    let program = convert_bits(&data[1..])
        .ok_or_else(|| invalid("Invalid padding in Bech32 data section"))?;
    let bytes = |n: usize| if n == 1 { "byte" } else { "bytes" };
    if version == 0 && !matches!(program.len(), 20 | 32) {
        return Err(invalid(format!(
            "Invalid Bech32 v0 address program size ({} {}), per BIP141",
            program.len(),
            bytes(program.len())
        )));
    }
    let address = s.to_ascii_lowercase();
    if version == 1 && program == [0x4e, 0x73] {
        let script = vec![0x51, 0x02, 0x4e, 0x73];
        return Ok(Destination {
            address,
            script,
            kind: Kind::Anchor,
        });
    }
    if version > 16 {
        return Err(invalid("Invalid Bech32 address witness version"));
    }
    if !(2..=WITNESS_PROGRAM_MAX).contains(&program.len()) {
        return Err(invalid(format!(
            "Invalid Bech32 address program size ({} {})",
            program.len(),
            bytes(program.len())
        )));
    }
    let op = if version == 0 { 0x00 } else { 0x50 + version };
    let mut script = vec![op, program.len() as u8];
    script.extend_from_slice(&program);
    Ok(Destination {
        address,
        script,
        kind: Kind::Witness { version, program },
    })
}

fn decode_base58(s: &str) -> Result<Destination, Invalid> {
    // Core reads a payload of at most 21 bytes with its checksum, and otherwise tries again
    // without one, at most 100 bytes, only to word the error.
    match base58::decode_check(s) {
        Ok(data) if data.len() <= 21 => {
            let script = match data.as_slice() {
                [PUBKEY_PREFIX, hash @ ..] if hash.len() == 20 => {
                    [&[0x76, 0xa9, 0x14][..], hash, &[0x88, 0xac]].concat()
                }
                [SCRIPT_PREFIX, hash @ ..] if hash.len() == 20 => {
                    [&[0xa9, 0x14][..], hash, &[0x87]].concat()
                }
                [PUBKEY_PREFIX | SCRIPT_PREFIX, ..] => {
                    return Err(invalid("Invalid length for Base58 address (P2PKH or P2SH)"));
                }
                _ => return Err(invalid("Invalid or unsupported Base58-encoded address.")),
            };
            let kind = if data[0] == PUBKEY_PREFIX {
                Kind::PubkeyHash
            } else {
                Kind::ScriptHash
            };
            Ok(Destination {
                address: s.to_owned(),
                script,
                kind,
            })
        }
        _ if base58::decode(s).is_ok_and(|data| data.len() <= 100) => Err(invalid(
            "Invalid checksum or length of Base58 address (P2PKH or P2SH)",
        )),
        _ => Err(invalid(
            "Invalid or unsupported Segwit (Bech32) or Base58 encoding.",
        )),
    }
}

#[derive(Clone, Copy)]
enum Checksum {
    Bech32,
    Bech32m,
}

/// Core's `bech32::Decode`: the checksum that holds and the data's 5-bit values before it; none
/// when the string isn't valid Bech32 or Bech32m.
fn bech32_data(s: &str) -> Option<(Checksum, Vec<u8>)> {
    if s.len() > BECH32_MAX_LEN || !character_errors(s).is_empty() {
        return None;
    }
    let pos = s.rfind('1')?;
    if pos == 0 || pos + CHECKSUM_LEN >= s.len() {
        return None;
    }
    let unchecked = UncheckedHrpstring::new(s).ok()?;
    let checksum = if unchecked.has_valid_checksum::<Bech32>() {
        Checksum::Bech32
    } else if unchecked.has_valid_checksum::<Bech32m>() {
        Checksum::Bech32m
    } else {
        return None;
    };
    let data = s[pos + 1..s.len() - CHECKSUM_LEN]
        .bytes()
        .map(|c| symbol(c).expect("checked by the parse"))
        .collect();
    Some((checksum, data))
}

fn symbol(c: u8) -> Option<u8> {
    CHARSET
        .iter()
        .position(|&x| x == c.to_ascii_lowercase())
        .map(|i| i as u8)
}

/// Core's `CheckCharacters`: characters outside 33 to 126, and letters of the other case than
/// the first letter's.
fn character_errors(s: &str) -> Vec<usize> {
    let (mut lower, mut upper) = (false, false);
    let mut errors = Vec::new();
    for (i, c) in s.bytes().enumerate() {
        match c {
            b'a'..=b'z' if upper => errors.push(i),
            b'a'..=b'z' => lower = true,
            b'A'..=b'Z' if lower => errors.push(i),
            b'A'..=b'Z' => upper = true,
            _ if !(33..=126).contains(&c) => errors.push(i),
            _ => {}
        }
    }
    errors
}

/// Core's `LocateErrors`, but for checksums only a single wrong character is located.
fn locate_errors(s: &str) -> Invalid {
    if s.len() > BECH32_MAX_LEN {
        return ("Bech32 string too long".into(), vec![BECH32_MAX_LEN]);
    }
    let errors = character_errors(s);
    if !errors.is_empty() {
        return ("Invalid character or mixed case".into(), errors);
    }
    let Some(pos) = s.rfind('1') else {
        return invalid("Missing separator");
    };
    if pos == 0 || pos + CHECKSUM_LEN >= s.len() {
        return ("Invalid separator position".into(), vec![pos]);
    }
    let invalid_char = (pos + 1..s.len()).find(|&i| symbol(s.as_bytes()[i]).is_none());
    if let Some(i) = invalid_char {
        return ("Invalid Base 32 character".into(), vec![i]);
    }
    // One character replaced by each other one, data and checksum alike, under either checksum.
    let lowercase = !s.bytes().any(|c| c.is_ascii_uppercase());
    let mut candidate = s.as_bytes().to_vec();
    for i in pos + 1..s.len() {
        let original = candidate[i];
        for &c in CHARSET {
            let c = if lowercase { c } else { c.to_ascii_uppercase() };
            if c == original.to_ascii_lowercase() || c == original {
                continue;
            }
            candidate[i] = c;
            let fixed = std::str::from_utf8(&candidate)
                .ok()
                .and_then(|s| UncheckedHrpstring::new(s).ok());
            if let Some(fixed) = fixed {
                if fixed.has_valid_checksum::<Bech32>() {
                    return ("Invalid Bech32 checksum".into(), vec![i]);
                }
                if fixed.has_valid_checksum::<Bech32m>() {
                    return ("Invalid Bech32m checksum".into(), vec![i]);
                }
            }
        }
        candidate[i] = original;
    }
    invalid("Invalid checksum")
}

/// Core's `ConvertBits<5, 8, false>`: none when the padding isn't zero bits short of a byte.
fn convert_bits(data: &[u8]) -> Option<Vec<u8>> {
    let (mut acc, mut bits) = (0u32, 0u32);
    let mut out = Vec::with_capacity(data.len() * 5 / 8);
    for &value in data {
        acc = (acc << 5) | u32::from(value);
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    (bits < 5 && (acc << (8 - bits)) & 0xff == 0).then_some(out)
}
