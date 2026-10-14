use serde::Deserialize;

use super::Parameter;

#[derive(Debug, Clone, Deserialize)]
pub struct HttpOperation {
    pub path: String,
    pub parameters: Vec<Parameter>,
}
