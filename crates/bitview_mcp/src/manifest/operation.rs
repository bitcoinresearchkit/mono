use super::HttpOperation;

/// A tool by its name, and the API route it calls.
#[derive(Debug)]
pub struct Operation {
    pub name: String,
    pub http: HttpOperation,
}
