use serde::Deserialize;

use super::Parameter;

#[derive(Debug, Deserialize)]
pub struct HttpOperation {
    pub path: String,
    pub parameters: Vec<Parameter>,
}
