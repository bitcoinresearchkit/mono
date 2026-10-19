use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub location: ParameterLocation,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterLocation {
    Path,
    Query,
}
