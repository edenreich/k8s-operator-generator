use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, CustomResource)]
#[kube(
    group = "example.com",
    version = "v1",
    kind = "Dog",
    plural = "dogs",
    status = "DogStatus",
    namespaced
)]
pub struct DogSpec {
    pub name: String,
    pub breed: String,
    pub age: i32,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DogStatus {
    pub uuid: Option<String>,
}
