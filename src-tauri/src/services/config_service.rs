use crate::domain::config::AppConfigDto;

pub fn get_app_config() -> Result<AppConfigDto, String> {
    Ok(AppConfigDto {
        app_locale: "zh-CN".to_string(),
        theme_preset: "graphite".to_string(),
        layout_density: "comfortable".to_string(),
    })
}

pub fn update_app_config(config: AppConfigDto) -> Result<AppConfigDto, String> {
    Ok(config)
}
