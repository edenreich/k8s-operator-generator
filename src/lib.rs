use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "example.com",
    version = "v1alpha1",
    kind = "Dog",
    plural = "dogs",
    namespaced
)]
#[kube(status = "DogStatus")]
pub struct DogSpec {
    name: String,
    breed: String,
    age: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct DogStatus {
    is_ok: bool,
}
