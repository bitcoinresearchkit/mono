//! The JSON Schema `pattern`s the manifest's inputs use: anchored (`^…$`) sequences of a
//! character class (`[0-9a-f]`), `\d` or a literal character, each optionally quantified (`{n}`,
//! `{m,n}`, `+`, `*`, `?`). Anything else fails to compile, which a test checks over the manifest.

/// One position of a pattern and how many characters it takes.
#[derive(Debug)]
struct Atom {
    /// Inclusive character ranges, one for a literal.
    ranges: Vec<(char, char)>,
    min: usize,
    max: usize,
}

impl Atom {
    fn accepts(&self, c: char) -> bool {
        self.ranges
            .iter()
            .any(|&(low, high)| (low..=high).contains(&c))
    }
}

#[derive(Debug)]
pub(crate) struct Pattern(Vec<Atom>);

impl Pattern {
    pub(crate) fn compile(pattern: &str) -> Result<Self, String> {
        let unsupported = || format!("unsupported pattern {pattern}");
        let body = pattern
            .strip_prefix('^')
            .and_then(|body| body.strip_suffix('$'))
            .ok_or_else(unsupported)?;
        let mut chars = body.chars().peekable();
        let mut atoms = Vec::new();
        while let Some(c) = chars.next() {
            let ranges = match c {
                '[' => {
                    let mut ranges = Vec::new();
                    loop {
                        match chars.next().ok_or_else(unsupported)? {
                            ']' if !ranges.is_empty() => break,
                            '\\' | '[' | '^' => return Err(unsupported()),
                            low => {
                                if chars.next_if_eq(&'-').is_some() {
                                    let high = chars.next().ok_or_else(unsupported)?;
                                    if high == ']' || high < low {
                                        return Err(unsupported());
                                    }
                                    ranges.push((low, high));
                                } else {
                                    ranges.push((low, low));
                                }
                            }
                        }
                    }
                    ranges
                }
                '\\' => match chars.next() {
                    Some('d') => vec![('0', '9')],
                    Some(c @ ('.' | '-' | '+' | '\\')) => vec![(c, c)],
                    _ => return Err(unsupported()),
                },
                '(' | ')' | '|' | '.' | '{' | '}' | '+' | '*' | '?' | '^' | '$' | ']' => {
                    return Err(unsupported());
                }
                c => vec![(c, c)],
            };
            let (min, max) = match chars.peek() {
                Some('+') => (1, usize::MAX),
                Some('*') => (0, usize::MAX),
                Some('?') => (0, 1),
                Some('{') => {
                    chars.next();
                    let mut bounds = String::new();
                    for c in chars.by_ref() {
                        if c == '}' {
                            break;
                        }
                        bounds.push(c);
                    }
                    let count = |text: &str| text.parse::<usize>().map_err(|_| unsupported());
                    let (min, max) = match bounds.split_once(',') {
                        Some((min, max)) => (count(min)?, count(max)?),
                        None => (count(&bounds)?, count(&bounds)?),
                    };
                    if min > max {
                        return Err(unsupported());
                    }
                    atoms.push(Atom { ranges, min, max });
                    continue;
                }
                _ => (1, 1),
            };
            if (min, max) != (1, 1) {
                chars.next();
            }
            atoms.push(Atom { ranges, min, max });
        }
        Ok(Self(atoms))
    }

    pub(crate) fn matches(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        matches_from(&self.0, &chars)
    }
}

/// Backtracking over each atom's counts, longest first.
fn matches_from(atoms: &[Atom], chars: &[char]) -> bool {
    let Some((atom, rest)) = atoms.split_first() else {
        return chars.is_empty();
    };
    let longest = chars
        .iter()
        .take(atom.max)
        .take_while(|&&c| atom.accepts(c))
        .count();
    if longest < atom.min {
        return false;
    }
    (atom.min..=longest)
        .rev()
        .any(|count| matches_from(rest, &chars[count..]))
}
