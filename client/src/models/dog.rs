/*
 * Pets API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Dog {
    #[serde(rename = "id", deserialize_with = "Option::deserialize")]
    pub id: Option<serde_json::Value>,
    #[serde(rename = "name", deserialize_with = "Option::deserialize")]
    pub name: Option<serde_json::Value>,
    #[serde(rename = "breed", deserialize_with = "Option::deserialize")]
    pub breed: Option<serde_json::Value>,
    #[serde(rename = "age", deserialize_with = "Option::deserialize")]
    pub age: Option<serde_json::Value>,
}

impl Dog {
    pub fn new(id: Option<serde_json::Value>, name: Option<serde_json::Value>, breed: Option<serde_json::Value>, age: Option<serde_json::Value>) -> Dog {
        Dog {
            id,
            name,
            breed,
            age,
        }
    }
}


