#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, kube::CustomResource)]
#[kube(group = "example.com", version = "v1", kind = "Cat", plural = "cats", namespaced)]
pub struct CatSpec {
    id: String,
    name: String,
    breed: String,
    age: u32,
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, kube::CustomResource)]
#[kube(group = "example.com", version = "v1", kind = "Dog", plural = "dogs", namespaced)]
pub struct DogSpec {
    id: String,
    name: String,
    breed: String,
    age: u32,
}
