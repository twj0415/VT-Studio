use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigDto {
    pub app_locale: String,
    pub theme_preset: String,
    pub layout_density: String,
}
