use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Map, Value, from_str, to_string};

mod http_operation;
mod operation;
mod parameter;

pub use http_operation::HttpOperation;
pub use operation::Operation;
pub use parameter::{Parameter, ParameterLocation};

const MANIFEST_JSON: &str = include_str!("../../mcp/manifest.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    instructions: String,
    operations: Vec<ManifestOperation>,
}

#[derive(Debug, Deserialize)]
struct ManifestOperation {
    tool: Map<String, Value>,
    http: HttpOperation,
}

#[derive(Debug)]
pub struct Catalog {
    instructions: String,
    /// Every tool as one compact JSON array, written once: `tools/list` sends it as it is.
    tools_json: String,
    operations: BTreeMap<String, Operation>,
}

impl Catalog {
    /// The manifest is generated and checked by bitview_bindgen; here it is only indexed.
    pub fn embedded() -> Self {
        let manifest: Manifest =
            from_str(MANIFEST_JSON).expect("generated LLM manifest is valid; run `cargo bindgen`");
        debug_assert_eq!(manifest.schema_version, 2);
        debug_assert!(!manifest.operations.is_empty());

        // `Mcp-Param-*` headers are not checked: no tool may ask for one.
        debug_assert!(!MANIFEST_JSON.contains("x-mcp-header"));
        let names = manifest
            .operations
            .iter()
            .map(|operation| operation.tool["name"].as_str().expect("a tool has a name"))
            .collect::<Vec<_>>();
        debug_assert!(names.is_sorted(), "generated tools are sorted by name");
        let tools = manifest
            .operations
            .iter()
            .map(|operation| &operation.tool)
            .collect::<Vec<_>>();
        let tools_json = to_string(&tools).expect("a JSON value serializes");
        let tool_count = tools.len();
        let operations = manifest
            .operations
            .into_iter()
            .map(|ManifestOperation { tool, http }| {
                let name = tool["name"].as_str().unwrap().to_owned();
                (name.clone(), Operation { name, http })
            })
            .collect::<BTreeMap<_, _>>();
        debug_assert_eq!(operations.len(), tool_count, "duplicate generated tool name");

        Self {
            instructions: manifest.instructions,
            tools_json,
            operations,
        }
    }

    /// The server's instructions, generated with the skill and llms.txt from one series flow.
    pub fn instructions(&self) -> &str {
        &self.instructions
    }

    pub fn tools_json(&self) -> &str {
        &self.tools_json
    }

    pub fn tool_count(&self) -> usize {
        self.operations.len()
    }

    pub fn operation(&self, name: &str) -> Option<&Operation> {
        self.operations.get(name)
    }
}
