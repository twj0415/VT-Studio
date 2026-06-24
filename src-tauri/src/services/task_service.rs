use crate::domain::task::{
    CompositionTaskDto, StartCompositionRequest, TaskDetailDto, TaskStepDto,
};

pub fn create_task(project_id: String) -> Result<TaskDetailDto, String> {
    get_task_detail(project_id)
}

pub fn get_task_detail(project_id: String) -> Result<TaskDetailDto, String> {
    Ok(TaskDetailDto {
        task_id: "task_draft".to_string(),
        project_id: project_id.clone(),
        task_status: "waiting_user".to_string(),
        current_step: Some("storyboard_review".to_string()),
        composition_task: None,
        steps: vec![
            TaskStepDto {
                step_id: "step_storyboard_review".to_string(),
                step_name: "storyboard_review".to_string(),
                status: "waiting_user".to_string(),
            },
            TaskStepDto {
                step_id: "step_image_review".to_string(),
                step_name: "image_review".to_string(),
                status: "pending".to_string(),
            },
            TaskStepDto {
                step_id: "step_video_review".to_string(),
                step_name: "video_review".to_string(),
                status: "pending".to_string(),
            },
            TaskStepDto {
                step_id: "step_final_composition".to_string(),
                step_name: "final_composition".to_string(),
                status: "pending".to_string(),
            },
        ],
    })
}

pub fn approve_task_step(project_id: String, step_name: String) -> Result<TaskDetailDto, String> {
    let current_step = match step_name.as_str() {
        "storyboard_review" => Some("image_review".to_string()),
        "image_review" => Some("video_review".to_string()),
        "video_review" => Some("final_composition".to_string()),
        _ => None,
    };

    Ok(TaskDetailDto {
        task_id: "task_draft".to_string(),
        project_id: project_id.clone(),
        task_status: if step_name == "final_composition" {
            "succeeded"
        } else {
            "waiting_user"
        }
        .to_string(),
        current_step,
        composition_task: None,
        steps: vec![
            TaskStepDto {
                step_id: "step_storyboard_review".to_string(),
                step_name: "storyboard_review".to_string(),
                status: if step_name == "storyboard_review" {
                    "succeeded"
                } else {
                    "waiting_user"
                }
                .to_string(),
            },
            TaskStepDto {
                step_id: "step_image_review".to_string(),
                step_name: "image_review".to_string(),
                status: if step_name == "image_review" {
                    "succeeded"
                } else if step_name == "storyboard_review" {
                    "waiting_user"
                } else {
                    "pending"
                }
                .to_string(),
            },
            TaskStepDto {
                step_id: "step_video_review".to_string(),
                step_name: "video_review".to_string(),
                status: if step_name == "video_review" {
                    "succeeded"
                } else if step_name == "image_review" {
                    "waiting_user"
                } else {
                    "pending"
                }
                .to_string(),
            },
            TaskStepDto {
                step_id: "step_final_composition".to_string(),
                step_name: "final_composition".to_string(),
                status: if step_name == "final_composition" {
                    "succeeded"
                } else if step_name == "video_review" {
                    "waiting_user"
                } else {
                    "pending"
                }
                .to_string(),
            },
        ],
    })
}

pub fn start_composition(request: StartCompositionRequest) -> Result<CompositionTaskDto, String> {
    Err(format!(
        "Project {} requires every StoryboardItem to have selected_video_segment_id before starting composition.",
        request.project_id
    ))
}
