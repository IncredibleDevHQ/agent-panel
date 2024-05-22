use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub parameters: Parameters,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Parameters {
    #[serde(rename = "type")]
    pub _type: String,
    pub properties: HashMap<String, Parameter>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Parameter>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Functions {
    pub functions: Vec<Function>,
}
