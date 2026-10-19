use rmcp::model::Tool;
use serde::Deserialize;

use super::HttpOperation;

#[derive(Debug, Deserialize)]
pub struct Operation {
    pub tool: Tool,
    pub http: HttpOperation,
}
