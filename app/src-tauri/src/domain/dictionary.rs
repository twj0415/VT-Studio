use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryItemDto {
    pub label: String,
    pub value: String,
    pub disabled: Option<bool>,
    pub color_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryDto {
    pub code: String,
    pub items: Vec<DictionaryItemDto>,
}
