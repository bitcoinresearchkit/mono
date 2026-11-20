use brk_error::{Error, Result};
use brk_types::Txid;

/// Parses Core's compact JSON transaction-ID array (`["<64 hex digits>","<...>"]`, no whitespace)
/// by its fixed layout: the generic JSON scanner and hex decoder were most of the listing's
/// parse time. Any other layout is an error naming the element.
pub fn parse_txid_array(json: &str) -> Result<Vec<Txid>> {
    /// A quoted txid and its separator.
    const ELEMENT: usize = 67;
    let element_error =
        |index: usize| Error::Parse(format!("txid array: element {index} is malformed"));
    let body = json
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .ok_or_else(|| Error::Parse("txid array: not an array".into()))?;
    let mut txids = Vec::with_capacity(body.len() / ELEMENT + 1);
    let mut rest = body.as_bytes();
    while !rest.is_empty() {
        let index = txids.len();
        let (element, tail) = rest
            .split_first_chunk::<{ ELEMENT - 1 }>()
            .ok_or_else(|| element_error(index))?;
        let [b'"', digits @ .., b'"'] = element else {
            return Err(element_error(index));
        };
        txids.push(Txid::from_hex_digits(digits).ok_or_else(|| element_error(index))?);
        rest = match tail {
            [] => tail,
            [b',', next @ ..] if !next.is_empty() => next,
            _ => return Err(element_error(index + 1)),
        };
    }
    Ok(txids)
}
