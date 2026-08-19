use brk_error::Error;

pub struct Step {
    pub name: String,
    pub index: Option<usize>,
}

pub struct Path {
    pub raw: String,
    pub steps: Vec<Step>,
}

impl Path {
    pub fn parse(s: &str) -> brk_error::Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        let mut steps = Vec::new();
        let mut i = 0;
        while i < parts.len() {
            let name = parts[i];
            if name.is_empty() {
                return Err(Error::Parse(format!("bad path '{s}': empty segment")));
            }
            if name.parse::<usize>().is_ok() {
                return Err(Error::Parse(format!(
                    "bad path '{s}': '{name}' must follow a field name"
                )));
            }
            let index = parts.get(i + 1).and_then(|p| p.parse::<usize>().ok());
            steps.push(Step {
                name: name.to_string(),
                index,
            });
            i += if index.is_some() { 2 } else { 1 };
        }
        Ok(Self {
            raw: s.to_string(),
            steps,
        })
    }
}
