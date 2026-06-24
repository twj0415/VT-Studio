use crate::domain::media::ExecutableMediaOptionDto;

pub fn list_executable_media_options() -> Result<Vec<ExecutableMediaOptionDto>, String> {
    Ok(vec![
        ExecutableMediaOptionDto {
            option_id: "mock_image_model".to_string(),
            label: "Mock image model".to_string(),
            kind: "provider_model".to_string(),
            capability: "text_to_image".to_string(),
            provider_model_id: Some("mock/image-to-video-still".to_string()),
            workflow_preset_id: None,
            enabled: true,
        },
        ExecutableMediaOptionDto {
            option_id: "mock_i2v_workflow".to_string(),
            label: "Mock image-to-video workflow".to_string(),
            kind: "workflow_preset".to_string(),
            capability: "image_to_video".to_string(),
            provider_model_id: None,
            workflow_preset_id: Some("mock-i2v-v1".to_string()),
            enabled: true,
        },
    ])
}
