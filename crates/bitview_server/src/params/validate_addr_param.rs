use bitview_types::ErrorCode;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::Error;

#[derive(Deserialize, JsonSchema)]
pub struct ValidateAddressParam {
    /// Bitcoin address to validate: 2 to 130 letters and digits (a P2PK public key in hex is 130);
    /// anything else answers 400, as mempool.space does
    #[schemars(
        example = &"04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f",
        example = &"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
        example = &"notavalidaddr",
        example = &"bc1qinvalid"
    )]
    #[serde(rename = "address")]
    pub addr: String,
}

impl ValidateAddressParam {
    /// The address when it could be one by its characters, as mempool.space checks before asking
    /// its node; 400 otherwise.
    pub fn checked(&self) -> Result<&str, Error> {
        let len = self.addr.len();
        if (2..=130).contains(&len) && self.addr.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
            Ok(&self.addr)
        } else {
            Err(Error::new(
                axum::http::StatusCode::BAD_REQUEST,
                ErrorCode::InvalidAddress,
                "Invalid address",
            ))
        }
    }
}
