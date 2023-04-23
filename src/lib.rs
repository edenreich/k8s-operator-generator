

use kube::{CustomResource};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "example.com",
    version = "v1alpha1",
    kind = "Cat",
    plural = "Cats",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct CatSpec {
    name: String,
    breed: String,
    age: u32,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CatStatus {
    status: String,
}
