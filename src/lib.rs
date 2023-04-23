

use kube::CustomResource;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "example.com",
    version = "v1alpha1",
    kind = "Cat",
    plural = "cats",
    namespaced
)]
#[kube(status = "CatStatus")]
pub struct CatSpec {
    name: String,
    breed: String,
    age: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct CatStatus {
    is_ok: bool,
}
