use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, PartialEq, CustomResource)]
#[kube(
    group = "example.com",
    version = "v1",
    kind = "Horse",
    plural = "horses",
    derive = "PartialEq",
    status = "HorseStatus",
    namespaced
)]
pub struct HorseSpec {
    pub name: String,
    pub breed: String,
    pub age: i32,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, PartialEq)]
pub struct HorseStatus {
    pub uuid: Option<String>,
}
