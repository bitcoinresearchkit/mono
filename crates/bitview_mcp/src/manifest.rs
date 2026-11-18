use std::collections::BTreeMap;

use rmcp::model::Tool;
use serde::Deserialize;
use serde_json::from_str;

mod http_operation;
mod operation;
mod parameter;

pub use http_operation::HttpOperation;
pub use operation::Operation;
pub use parameter::{Parameter, ParameterLocation};

const MANIFEST_JSON: &str = include_str!("../generated/manifest.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    instructions: String,
    operations: Vec<Operation>,
}

#[derive(Debug)]
pub struct Catalog {
    instructions: String,
    tools: Vec<Tool>,
    operations: BTreeMap<String, Operation>,
}

impl Catalog {
    /// The manifest is generated and checked by bitview_bindgen; here it is only indexed.
    pub fn embedded() -> Self {
        let manifest: Manifest =
            from_str(MANIFEST_JSON).expect("generated LLM manifest is valid; run `cargo bindgen`");
        debug_assert_eq!(manifest.schema_version, 2);
        debug_assert!(!manifest.operations.is_empty());

        let tools = manifest
            .operations
            .iter()
            .map(|operation| operation.tool.clone())
            .collect::<Vec<_>>();
        let operations = manifest
            .operations
            .into_iter()
            .map(|operation| (operation.tool.name.to_string(), operation))
            .collect::<BTreeMap<_, _>>();
        debug_assert_eq!(
            operations.len(),
            tools.len(),
            "duplicate generated tool name"
        );
        debug_assert!(tools.is_sorted_by(|left, right| left.name < right.name));

        Self {
            instructions: manifest.instructions,
            tools,
            operations,
        }
    }

    /// The server's instructions, generated with the skill and llms.txt from one series flow.
    pub fn instructions(&self) -> &str {
        &self.instructions
    }

    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    pub fn operation(&self, name: &str) -> Option<&Operation> {
        self.operations.get(name)
    }
}
