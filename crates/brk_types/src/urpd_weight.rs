use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;

/// Weighting applied to a URPD: raw (unweighted), cointime, or coinflow.
#[derive(
    Debug, Display, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UrpdWeight {
    #[default]
    Raw,
    Cointime,
    Coinflow,
}

#[cfg(test)]
mod tests {
    use schemars::schema_for;
    use serde_json::to_string;

    use super::UrpdWeight;

    #[test]
    fn names_and_schema_match() {
        for (weight, name) in [
            (UrpdWeight::Raw, "raw"),
            (UrpdWeight::Cointime, "cointime"),
            (UrpdWeight::Coinflow, "coinflow"),
        ] {
            assert_eq!(weight.to_string(), name);
            assert_eq!(to_string(&weight).unwrap(), format!("\"{name}\""));
        }

        let schema = to_string(&schema_for!(UrpdWeight)).unwrap();
        assert!(schema.contains("\"raw\""), "{schema}");
    }
}
