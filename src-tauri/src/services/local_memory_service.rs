use crate::db::local_memory_repository::{
    create_entry_id, LocalMemoryCandidateRecord, LocalMemoryEntryRecord, LocalMemoryRepository,
    LocalMemoryRetrievalRecord,
};
use crate::db::project_repository::ProjectRepository;
use crate::db::Database;
use crate::domain::local_memory::{
    BuildLocalMemoryContextRequest, CreateLocalMemoryRetrievalRequest,
    ListLocalMemoryEntriesRequest, LocalMemoryCandidateDecisionRequest,
    LocalMemoryContextCitationDto, LocalMemoryContextDto, LocalMemoryEntryDto,
    LocalMemoryRetrievalCandidateDto, LocalMemoryRetrievalDto, UpsertLocalMemoryEntryRequest,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

const DEFAULT_MAX_RESULTS: i64 = 8;
const HARD_MAX_RESULTS: i64 = 20;

pub fn upsert_local_memory_entry(
    database: &Database,
    request: UpsertLocalMemoryEntryRequest,
) -> Result<LocalMemoryEntryDto, String> {
    if let Some(project_id) = request.project_id.as_ref() {
        ensure_project_exists(database, project_id)?;
    }
    let source_kind = required_trimmed("sourceKind", &request.source_kind)?;
    let source_id = required_trimmed("sourceId", &request.source_id)?;
    let content_summary = required_trimmed("contentSummary", &request.content_summary)?;
    let source_label = request
        .source_label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&source_id)
        .to_string();
    let memory_id = request
        .memory_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(create_entry_id);
    let content_hash =
        stable_content_hash(&format!("{source_kind}\n{source_id}\n{content_summary}"));

    LocalMemoryRepository::new(database).upsert_entry(LocalMemoryEntryRecord {
        memory_id,
        project_id: request.project_id,
        source_kind,
        source_id,
        source_label,
        content_summary,
        content_hash,
        embedding_provider_id: request.embedding_provider_id,
        embedding_model_id: request.embedding_model_id,
        embedding_vector_path: request.embedding_vector_path,
        metadata: request.metadata.unwrap_or_else(|| json!({})),
        expires_at: request.expires_at,
    })
}

pub fn list_local_memory_entries(
    database: &Database,
    request: ListLocalMemoryEntriesRequest,
) -> Result<Vec<LocalMemoryEntryDto>, String> {
    if let Some(project_id) = request.project_id.as_ref() {
        ensure_project_exists(database, project_id)?;
    }
    LocalMemoryRepository::new(database).list_entries(
        request.project_id.as_deref(),
        request.include_global.unwrap_or(true),
    )
}

pub fn create_local_memory_retrieval(
    database: &Database,
    request: CreateLocalMemoryRetrievalRequest,
) -> Result<LocalMemoryRetrievalDto, String> {
    ensure_project_exists(database, &request.project_id)?;
    let query_text = required_trimmed("queryText", &request.query_text)?;
    if !(0.0..=1.0).contains(&request.min_similarity) {
        return Err("min_similarity must be between 0 and 1.".to_string());
    }
    let max_results = request.max_results.unwrap_or(DEFAULT_MAX_RESULTS);
    if !(1..=HARD_MAX_RESULTS).contains(&max_results) {
        return Err(format!(
            "max_results must be between 1 and {HARD_MAX_RESULTS}."
        ));
    }
    if request.candidates.is_empty() {
        return Err(
            "retrieval candidates are required; this command does not run embedding search."
                .to_string(),
        );
    }

    let repository = LocalMemoryRepository::new(database);
    let retrieval = LocalMemoryRetrievalRecord::new(
        request.project_id.clone(),
        query_text,
        request.min_similarity,
        max_results,
    );
    let mut seen = HashSet::new();
    let mut candidate_records = Vec::new();
    let mut sorted_inputs = request.candidates;
    sorted_inputs.sort_by(|left, right| {
        right
            .similarity
            .partial_cmp(&left.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for input in sorted_inputs {
        if !(0.0..=1.0).contains(&input.similarity) {
            return Err("candidate similarity must be between 0 and 1.".to_string());
        }
        if !seen.insert(input.memory_id.clone()) {
            continue;
        }
        let entry = repository
            .read_entry(&input.memory_id)?
            .ok_or_else(|| format!("Local memory entry not found: {}", input.memory_id))?;
        if !entry_belongs_to_project(&entry, &request.project_id) {
            return Err(format!(
                "Local memory entry {} does not belong to project {}.",
                entry.memory_id, request.project_id
            ));
        }

        let below_threshold = input.similarity < request.min_similarity;
        let inactive = entry.lifecycle != "active";
        let status = if below_threshold || inactive {
            "rejected"
        } else {
            "waiting_user"
        };
        let reason = if below_threshold {
            Some(format!(
                "below_min_similarity:{:.4}<{}",
                input.similarity, request.min_similarity
            ))
        } else if inactive {
            Some("memory_inactive".to_string())
        } else {
            input.reason
        };
        candidate_records.push(LocalMemoryCandidateRecord::new(
            retrieval.retrieval_id.clone(),
            entry.memory_id.clone(),
            input.similarity,
            status.to_string(),
            reason,
            citation_for_entry(&entry),
        ));
    }

    candidate_records.truncate(max_results as usize);
    repository.create_retrieval(retrieval, candidate_records)
}

pub fn approve_local_memory_candidate(
    database: &Database,
    request: LocalMemoryCandidateDecisionRequest,
) -> Result<LocalMemoryRetrievalCandidateDto, String> {
    let repository = LocalMemoryRepository::new(database);
    let candidate = repository.set_candidate_status(
        &request.candidate_id,
        "approved",
        request.reason.as_deref(),
    )?;
    let retrieval = repository
        .read_retrieval(&candidate.retrieval_id)?
        .ok_or_else(|| {
            format!(
                "Local memory retrieval not found: {}",
                candidate.retrieval_id
            )
        })?;
    if candidate.similarity < retrieval.min_similarity {
        repository.set_candidate_status(
            &candidate.candidate_id,
            "rejected",
            Some("below_min_similarity_cannot_approve"),
        )?;
        return Err(
            "candidate similarity is below retrieval threshold and cannot be approved.".to_string(),
        );
    }
    Ok(candidate)
}

pub fn reject_local_memory_candidate(
    database: &Database,
    request: LocalMemoryCandidateDecisionRequest,
) -> Result<LocalMemoryRetrievalCandidateDto, String> {
    LocalMemoryRepository::new(database).set_candidate_status(
        &request.candidate_id,
        "rejected",
        request.reason.as_deref(),
    )
}

pub fn build_local_memory_context(
    database: &Database,
    request: BuildLocalMemoryContextRequest,
) -> Result<LocalMemoryContextDto, String> {
    let repository = LocalMemoryRepository::new(database);
    let retrieval = repository
        .read_retrieval(&request.retrieval_id)?
        .ok_or_else(|| format!("Local memory retrieval not found: {}", request.retrieval_id))?;

    let mut citations = Vec::new();
    for candidate in retrieval
        .candidates
        .iter()
        .filter(|item| item.status == "approved")
    {
        if candidate.similarity < retrieval.min_similarity {
            continue;
        }
        let entry = repository
            .read_entry(&candidate.memory_id)?
            .ok_or_else(|| format!("Local memory entry not found: {}", candidate.memory_id))?;
        if !entry_belongs_to_project(&entry, &retrieval.project_id) || entry.lifecycle != "active" {
            continue;
        }
        citations.push(LocalMemoryContextCitationDto {
            candidate_id: candidate.candidate_id.clone(),
            memory_id: entry.memory_id,
            source_kind: entry.source_kind,
            source_id: entry.source_id,
            source_label: entry.source_label,
            similarity: candidate.similarity,
            content_summary: entry.content_summary,
            citation: candidate.citation.clone(),
        });
    }

    Ok(LocalMemoryContextDto {
        retrieval_id: retrieval.retrieval_id,
        project_id: retrieval.project_id,
        min_similarity: retrieval.min_similarity,
        usable: !citations.is_empty(),
        citations,
    })
}

fn ensure_project_exists(database: &Database, project_id: &str) -> Result<(), String> {
    ProjectRepository::new(database)
        .get_detail(project_id)?
        .ok_or_else(|| format!("Project not found: {project_id}"))?;
    Ok(())
}

fn entry_belongs_to_project(entry: &LocalMemoryEntryDto, project_id: &str) -> bool {
    entry.project_id.as_deref().is_none() || entry.project_id.as_deref() == Some(project_id)
}

fn citation_for_entry(entry: &LocalMemoryEntryDto) -> Value {
    json!({
        "memoryId": entry.memory_id,
        "sourceKind": entry.source_kind,
        "sourceId": entry.source_id,
        "sourceLabel": entry.source_label,
        "contentHash": entry.content_hash
    })
}

fn required_trimmed(field: &str, value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{field} is required."));
    }
    Ok(trimmed.to_string())
}

fn stable_content_hash(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::project_repository::ProjectRepository;
    use crate::domain::project::CreateProjectRequest;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn retrieval_rejects_candidates_below_threshold_and_context_only_uses_approved_sources() {
        let root = test_root("local_memory_threshold");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_memory");
        let high = create_memory(&database, "project_memory", "scene_high", "角色设定稳定");
        let low = create_memory(&database, "project_memory", "scene_low", "无关设定");

        let retrieval = create_local_memory_retrieval(
            &database,
            CreateLocalMemoryRetrievalRequest {
                project_id: "project_memory".to_string(),
                query_text: "角色一致性".to_string(),
                min_similarity: 0.72,
                max_results: Some(5),
                candidates: vec![
                    crate::domain::local_memory::LocalMemoryRetrievalCandidateInput {
                        memory_id: high.memory_id.clone(),
                        similarity: 0.88,
                        reason: Some("角色相关".to_string()),
                    },
                    crate::domain::local_memory::LocalMemoryRetrievalCandidateInput {
                        memory_id: low.memory_id.clone(),
                        similarity: 0.31,
                        reason: Some("低相关".to_string()),
                    },
                ],
            },
        )
        .expect("retrieval should save");
        assert_eq!(retrieval.candidates.len(), 2);
        assert_eq!(retrieval.candidates[0].status, "waiting_user");
        assert_eq!(retrieval.candidates[1].status, "rejected");

        let empty_context = build_local_memory_context(
            &database,
            BuildLocalMemoryContextRequest {
                retrieval_id: retrieval.retrieval_id.clone(),
            },
        )
        .expect("context should build");
        assert!(!empty_context.usable);

        approve_local_memory_candidate(
            &database,
            LocalMemoryCandidateDecisionRequest {
                candidate_id: retrieval.candidates[0].candidate_id.clone(),
                reason: Some("用户确认可用".to_string()),
            },
        )
        .expect("high similarity candidate should approve");
        let context = build_local_memory_context(
            &database,
            BuildLocalMemoryContextRequest {
                retrieval_id: retrieval.retrieval_id,
            },
        )
        .expect("context should build");
        assert!(context.usable);
        assert_eq!(context.citations.len(), 1);
        assert_eq!(context.citations[0].memory_id, high.memory_id);

        cleanup(root);
    }

    #[test]
    fn below_threshold_candidate_cannot_be_approved() {
        let root = test_root("local_memory_cannot_approve_low");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_memory_low");
        let entry = create_memory(&database, "project_memory_low", "memory_low", "低相关记忆");

        let retrieval = create_local_memory_retrieval(
            &database,
            CreateLocalMemoryRetrievalRequest {
                project_id: "project_memory_low".to_string(),
                query_text: "主角".to_string(),
                min_similarity: 0.8,
                max_results: Some(3),
                candidates: vec![
                    crate::domain::local_memory::LocalMemoryRetrievalCandidateInput {
                        memory_id: entry.memory_id,
                        similarity: 0.4,
                        reason: None,
                    },
                ],
            },
        )
        .expect("retrieval should save");

        let error = approve_local_memory_candidate(
            &database,
            LocalMemoryCandidateDecisionRequest {
                candidate_id: retrieval.candidates[0].candidate_id.clone(),
                reason: None,
            },
        )
        .expect_err("low similarity candidate should reject approval");
        assert!(error.contains("below retrieval threshold"));

        cleanup(root);
    }

    #[test]
    fn retrieval_rejects_memory_from_other_project() {
        let root = test_root("local_memory_project_scope");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        create_project(&database, "project_a");
        create_project(&database, "project_b");
        let entry = create_memory(&database, "project_a", "memory_a", "A 项目记忆");

        let error = create_local_memory_retrieval(
            &database,
            CreateLocalMemoryRetrievalRequest {
                project_id: "project_b".to_string(),
                query_text: "查询".to_string(),
                min_similarity: 0.5,
                max_results: Some(3),
                candidates: vec![
                    crate::domain::local_memory::LocalMemoryRetrievalCandidateInput {
                        memory_id: entry.memory_id,
                        similarity: 0.9,
                        reason: None,
                    },
                ],
            },
        )
        .expect_err("cross project memory should reject");
        assert!(error.contains("does not belong to project"));

        cleanup(root);
    }

    fn create_memory(
        database: &Database,
        project_id: &str,
        source_id: &str,
        summary: &str,
    ) -> LocalMemoryEntryDto {
        upsert_local_memory_entry(
            database,
            UpsertLocalMemoryEntryRequest {
                memory_id: None,
                project_id: Some(project_id.to_string()),
                source_kind: "storyboard_item".to_string(),
                source_id: source_id.to_string(),
                source_label: Some(source_id.to_string()),
                content_summary: summary.to_string(),
                embedding_provider_id: None,
                embedding_model_id: None,
                embedding_vector_path: None,
                metadata: Some(json!({ "test": true })),
                expires_at: None,
            },
        )
        .expect("memory should save")
    }

    fn create_project(database: &Database, project_id: &str) {
        ProjectRepository::new(database)
            .create_with_id(
                project_id.to_string(),
                CreateProjectRequest {
                    title: "Local memory".to_string(),
                    workflow_type: "image_to_video".to_string(),
                    input_type: "topic".to_string(),
                    topic: Some("主题".to_string()),
                    source_text: None,
                    source_text_path: None,
                    content_language: "zh-CN".to_string(),
                    tone: None,
                    aspect_ratio: "9:16".to_string(),
                    target_scene_count: 1,
                    segment_duration_seconds: 4.0,
                    style_prompt: None,
                    active_pack_id: None,
                    rule_refs: None,
                    executable_refs: None,
                    input_process_mode: "generate".to_string(),
                    input_options: Some(json!({})),
                },
            )
            .expect("project should create");
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
