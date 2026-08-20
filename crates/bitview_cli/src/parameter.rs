pub(crate) struct Parameter {
    pub api_name: &'static str,
    pub name: &'static str,
    pub required: bool,
    pub value_name: &'static str,
    pub repeatable: bool,
    pub description: Option<&'static str>,
}
