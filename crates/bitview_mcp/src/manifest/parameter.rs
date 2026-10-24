use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Parameter {
    /// The wire name (`txId[]`).
    pub name: String,
    /// The tool argument's name (`txids`): MCP clients take only `[a-zA-Z0-9_.-]` in names.
    pub argument: String,
    pub location: ParameterLocation,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterLocation {
    Path,
    Query,
}
