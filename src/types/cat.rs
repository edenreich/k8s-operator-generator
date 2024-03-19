use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, CustomResource)]
#[kube(
    group = "example.com",
    version = "v1",
    kind = "Cat",
    plural = "Cat",
    status = "CatStatus",
    namespaced
)]
pub struct CatSpec {
    pub uuid: Option<String>,
    pub name: String,
    pub breed: String,
    pub age: i32,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CatStatus {
    pub uuid: Option<String>,
}
