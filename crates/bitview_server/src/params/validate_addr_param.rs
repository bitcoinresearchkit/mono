use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ValidateAddrParam {
    /// Bitcoin address to validate (can be any string)
    #[schemars(
        example = &"04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
        example = &"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        example = &"not-a-valid-addr",
        example = &"bc1qinvalid"
    )]
    #[serde(rename = "address")]
    pub addr: String,
}
