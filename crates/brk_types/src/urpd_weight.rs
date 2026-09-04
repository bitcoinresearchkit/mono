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

impl UrpdWeight {
    pub const WEIGHTED: [Self; 2] = [Self::Cointime, Self::Coinflow];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Cointime => "cointime",
            Self::Coinflow => "coinflow",
        }
    }

    pub const fn is_weighted(self) -> bool {
        !matches!(self, Self::Raw)
    }
}

#[cfg(test)]
mod tests {
    use super::UrpdWeight;

    #[test]
    fn names_and_schema_match() {
        for weight in [UrpdWeight::Raw, UrpdWeight::Cointime, UrpdWeight::Coinflow] {
            assert_eq!(weight.to_string(), weight.as_str());
            assert_eq!(
                serde_json::to_string(&weight).unwrap(),
                format!("\"{}\"", weight.as_str())
            );
        }

        let schema = serde_json::to_string(&schemars::schema_for!(UrpdWeight)).unwrap();
        assert!(schema.contains("\"raw\""), "{schema}");
    }
}
