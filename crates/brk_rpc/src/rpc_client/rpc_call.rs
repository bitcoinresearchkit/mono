use serde::Serialize;
use serde_json::{
    Result as JsonResult,
    value::{RawValue, to_raw_value},
};

/// One mixed batch RPC call with parameters serialized exactly once.
pub struct RpcCall {
    pub method: &'static str,
    pub params: Box<RawValue>,
}

impl RpcCall {
    pub fn new(method: &'static str, params: &(impl Serialize + ?Sized)) -> JsonResult<Self> {
        Ok(Self {
            method,
            params: to_raw_value(params)?,
        })
    }

    pub fn empty(method: &'static str) -> JsonResult<Self> {
        Self::new(method, &[] as &[()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_parameter_arrays() {
        let call = RpcCall::new("method", &("abc", false)).unwrap();
        assert_eq!(call.params.get(), r#"["abc",false]"#);

        let empty = RpcCall::empty("method").unwrap();
        assert_eq!(empty.params.get(), "[]");
    }
}
