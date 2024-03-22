use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, PartialEq, CustomResource)]
#[kube(
    group = "example.com",
    version = "v1",
    kind = "Cat",
    plural = "cats",
    derive = "PartialEq",
    status = "CatStatus",
    namespaced
)]
pub struct CatSpec {
    pub name: String,
    pub breed: String,
    pub age: i32,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, PartialEq)]
pub struct CatStatus {
    pub uuid: Option<String>,
}
