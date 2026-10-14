use std::io::{self, Write};

use brk_error::Result;
use serde_json::{Map, Value, to_writer, to_writer_pretty};

use crate::{fields::Ctx, mode::Mode, path::Path};

pub struct Formatter {
    mode: Mode,
    fields: Vec<Path>,
}

impl Formatter {
    pub fn new(mode: Mode, fields: Vec<Path>) -> Self {
        Self { mode, fields }
    }

    /// Writes one block's line (or pretty block) to `out`.
    pub fn write(&self, ctx: &Ctx, out: &mut impl Write) -> Result<()> {
        match self.mode {
            Mode::Bare => self.bare(ctx, false, out),
            Mode::Tsv => self.tsv(ctx, out),
            Mode::Json => json(out, &self.object(ctx)?, false),
            Mode::Pretty if self.fields.len() == 1 => self.bare(ctx, true, out),
            Mode::Pretty => json(out, &self.object(ctx)?, true),
        }
    }

    fn bare(&self, ctx: &Ctx, pretty: bool, out: &mut impl Write) -> Result<()> {
        match ctx.resolve(&self.fields[0])? {
            Value::String(s) if s.is_empty() => Ok(()),
            Value::String(s) => Ok(writeln!(out, "{s}")?),
            other => json(out, &other, pretty),
        }
    }

    fn tsv(&self, ctx: &Ctx, out: &mut impl Write) -> Result<()> {
        // Resolve every field first so a bad path never leaves a partial row.
        let values = self
            .fields
            .iter()
            .map(|path| ctx.resolve_str(path))
            .collect::<Result<Vec<_>>>()?;
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                out.write_all(b"\t")?;
            }
            for (j, part) in value.split(['\t', '\n', '\r']).enumerate() {
                if j > 0 {
                    out.write_all(b" ")?;
                }
                out.write_all(part.as_bytes())?;
            }
        }
        Ok(out.write_all(b"\n")?)
    }

    fn object(&self, ctx: &Ctx) -> Result<Value> {
        if self.fields.is_empty() {
            return Ok(ctx.full());
        }
        let mut obj = Map::with_capacity(self.fields.len());
        for path in &self.fields {
            obj.insert(path.raw.clone(), ctx.resolve(path)?);
        }
        Ok(Value::Object(obj))
    }
}

fn json(out: &mut impl Write, value: &Value, pretty: bool) -> Result<()> {
    // Through `io::Error` so a closed pipe keeps its `BrokenPipe` kind.
    if pretty {
        to_writer_pretty(&mut *out, value)
    } else {
        to_writer(&mut *out, value)
    }
    .map_err(io::Error::from)?;
    Ok(out.write_all(b"\n")?)
}
