use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;
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
    namespaced,
    printcolumn = r#"{"name": "Status", "type": "string", "jsonPath": ".status.conditions[0].status", "description": "The current status of the resource"}"#,
    printcolumn = r#"{"name": "Reference ID", "type": "string", "jsonPath": ".status.uuid", "description": "The reference ID of the resource"}"#,
    printcolumn = r#"{"name": "Age", "type": "date", "jsonPath": ".metadata.creationTimestamp", "description": "The creation time of the resource"}"#
)]
pub struct CatSpec {
    pub name: String,
    pub breed: String,
    pub age: i32,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema, PartialEq)]
pub struct CatStatus {
    pub uuid: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schemars(schema_with = "conditions")]
    pub conditions: Vec<Condition>,
    #[serde(rename = "observedGeneration")]
    pub observed_generation: Option<i64>,
}

fn conditions(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    serde_json::from_value(serde_json::json!({
        "type": "array",
        "x-kubernetes-list-type": "map",
        "x-kubernetes-list-map-keys": ["type"],
        "items": {
            "type": "object",
            "properties": {
                "lastTransitionTime": { "format": "date-time", "type": "string" },
                "message": { "type": "string" },
                "observedGeneration": { "type": "integer", "format": "int64", "default": 0 },
                "reason": { "type": "string" },
                "status": { "type": "string" },
                "type": { "type": "string" }
            },
            "required": [
                "lastTransitionTime",
                "message",
                "reason",
                "status",
                "type"
            ],
        },
    }))
    .unwrap()
}
