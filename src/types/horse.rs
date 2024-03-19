use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, CustomResource)]
#[kube(
    group = "example.com",
    version = "v1",
    kind = "Horse",
    plural = "Horse",
    status = "HorseStatus",
    namespaced
)]
pub struct HorseSpec {
    pub uuid: Option<String>,
    pub name: String,
    pub breed: String,
    pub age: i32,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct HorseStatus {
    pub uuid: Option<String>,
}
