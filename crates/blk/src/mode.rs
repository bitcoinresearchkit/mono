use brk_error::Error;

#[derive(Clone, Copy)]
pub enum Mode {
    Bare,
    Tsv,
    Json,
    Pretty,
}

impl Mode {
    pub fn pick(pretty: bool, compact: bool, n_fields: usize) -> brk_error::Result<Self> {
        if pretty && compact {
            return Err(Error::Parse(
                "--pretty and --compact are mutually exclusive".into(),
            ));
        }
        if compact && n_fields == 0 {
            return Err(Error::Parse("--compact requires at least one field".into()));
        }
        Ok(if pretty {
            Self::Pretty
        } else if n_fields == 0 {
            Self::Json
        } else if n_fields == 1 {
            Self::Bare
        } else if compact {
            Self::Tsv
        } else {
            Self::Json
        })
    }
}
