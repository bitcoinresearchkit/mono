use brk_error::{Error, Result};

pub use crate::step::Step;

pub struct Path {
    pub raw: String,
    pub steps: Vec<Step>,
}

impl Path {
    pub fn parse(s: &str) -> Result<Self> {
        let mut parts = s.split('.').peekable();
        let mut steps = Vec::new();
        while let Some(name) = parts.next() {
            if name.is_empty() {
                return Err(Error::Parse(format!("bad path '{s}': empty segment")));
            }
            if name.parse::<usize>().is_ok() {
                return Err(Error::Parse(format!(
                    "bad path '{s}': '{name}' must follow a field name"
                )));
            }
            let index = parts.peek().and_then(|p| p.parse::<usize>().ok());
            if index.is_some() {
                parts.next();
            }
            steps.push(Step {
                name: name.to_string(),
                index,
            });
        }
        Ok(Self {
            raw: s.to_string(),
            steps,
        })
    }
}
