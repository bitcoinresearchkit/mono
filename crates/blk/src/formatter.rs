use serde_json::{Map, Value};

use crate::{fields::Ctx, mode::Mode, path::Path};

pub struct Formatter {
    mode: Mode,
    fields: Vec<Path>,
}

impl Formatter {
    pub fn new(mode: Mode, fields: Vec<Path>) -> Self {
        Self { mode, fields }
    }

    pub fn format(&self, ctx: &Ctx) -> brk_error::Result<String> {
        match self.mode {
            Mode::Bare => self.bare(ctx, false),
            Mode::Tsv => self.tsv(ctx),
            Mode::Json => Ok(serde_json::to_string(&self.object(ctx)?)?),
            Mode::Pretty if self.fields.len() == 1 => self.bare(ctx, true),
            Mode::Pretty => Ok(serde_json::to_string_pretty(&self.object(ctx)?)?),
        }
    }

    fn bare(&self, ctx: &Ctx, pretty: bool) -> brk_error::Result<String> {
        Ok(match ctx.resolve(&self.fields[0])? {
            Value::String(s) => s,
            other if pretty => serde_json::to_string_pretty(&other)?,
            other => other.to_string(),
        })
    }

    fn tsv(&self, ctx: &Ctx) -> brk_error::Result<String> {
        let mut row = String::new();
        for (i, path) in self.fields.iter().enumerate() {
            if i > 0 {
                row.push('\t');
            }
            for c in ctx.resolve_str(path)?.chars() {
                row.push(if matches!(c, '\t' | '\n' | '\r') {
                    ' '
                } else {
                    c
                });
            }
        }
        Ok(row)
    }

    fn object(&self, ctx: &Ctx) -> brk_error::Result<Value> {
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
