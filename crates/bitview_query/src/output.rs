use bitview_types::Format;

/// Series data output format
#[derive(Debug)]
pub enum Output {
    Json(Vec<u8>),
    CSV(String),
}

impl Output {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(self) -> String {
        match self {
            Output::CSV(s) => s,
            Output::Json(v) => unsafe { String::from_utf8_unchecked(v) },
        }
    }

    pub fn default(format: Format) -> Self {
        match format {
            Format::CSV => Output::CSV(String::new()),
            Format::JSON => {
                Output::Json(br#"{"version":0,"total":0,"start":0,"end":0,"data":[]}"#.to_vec())
            }
        }
    }
}

/// Deprecated: Raw JSON without metadata wrapper
#[derive(Debug)]
pub enum OutputLegacy {
    Json(LegacyValue),
    CSV(String),
}

impl OutputLegacy {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(self) -> String {
        match self {
            OutputLegacy::CSV(s) => s,
            OutputLegacy::Json(v) => unsafe { String::from_utf8_unchecked(v.to_vec()) },
        }
    }

    pub fn default(format: Format) -> Self {
        match format {
            Format::CSV => OutputLegacy::CSV(String::new()),
            Format::JSON => OutputLegacy::Json(LegacyValue::List(b"[]".to_vec())),
        }
    }
}

/// Deprecated: Raw JSON without metadata wrapper.
#[derive(Debug)]
pub enum LegacyValue {
    Matrix(Vec<Vec<u8>>),
    List(Vec<u8>),
    Value(Vec<u8>),
}

impl LegacyValue {
    pub fn to_vec(self) -> Vec<u8> {
        match self {
            LegacyValue::Value(v) | LegacyValue::List(v) => v,
            LegacyValue::Matrix(m) => {
                let total_size = m.iter().map(|v| v.len()).sum::<usize>() + m.len() + 1;
                let mut buf = Vec::with_capacity(total_size);
                buf.push(b'[');
                for (i, vec) in m.into_iter().enumerate() {
                    if i > 0 {
                        buf.push(b',');
                    }
                    buf.extend(vec);
                }
                buf.push(b']');
                buf
            }
        }
    }
}
