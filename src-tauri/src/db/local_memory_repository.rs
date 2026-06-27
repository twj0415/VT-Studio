use crate::db::{Database, Repository};
use crate::domain::local_memory::{
    LocalMemoryEntryDto, LocalMemoryRetrievalCandidateDto, LocalMemoryRetrievalDto,
};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LocalMemoryRepository<'db> {
    database: &'db Database,
}

impl<'db> LocalMemoryRepository<'db> {
    pub fn new(database: &'db Database) -> Self {
        Self { database }
    }

    pub fn upsert_entry(
        &self,
        record: LocalMemoryEntryRecord,
    ) -> Result<LocalMemoryEntryDto, String> {
        let metadata_json =
            serde_json::to_string(&record.metadata).map_err(|error| error.to_string())?;
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    INSERT INTO local_memory_entries (
                        memory_id, project_id, source_kind, source_id, source_label,
                        content_summary, content_hash, embedding_provider_id,
                        embedding_model_id, embedding_vector_path, metadata_json,
                        lifecycle, expires_at, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'active', ?12, CURRENT_TIMESTAMP)
                    ON CONFLICT(memory_id) DO UPDATE SET
                        project_id = excluded.project_id,
                        source_kind = excluded.source_kind,
                        source_id = excluded.source_id,
                        source_label = excluded.source_label,
                        content_summary = excluded.content_summary,
                        content_hash = excluded.content_hash,
                        embedding_provider_id = excluded.embedding_provider_id,
                        embedding_model_id = excluded.embedding_model_id,
                        embedding_vector_path = excluded.embedding_vector_path,
                        metadata_json = excluded.metadata_json,
                        lifecycle = 'active',
                        expires_at = excluded.expires_at,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    params![
                        record.memory_id,
                        record.project_id,
                        record.source_kind,
                        record.source_id,
                        record.source_label,
                        record.content_summary,
                        record.content_hash,
                        record.embedding_provider_id,
                        record.embedding_model_id,
                        record.embedding_vector_path,
                        metadata_json,
                        record.expires_at,
                    ],
                )?;
                read_entry(connection, &record.memory_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Local memory entry was saved but cannot be read.".to_string())
    }

    pub fn list_entries(
        &self,
        project_id: Option<&str>,
        include_global: bool,
    ) -> Result<Vec<LocalMemoryEntryDto>, String> {
        self.database
            .with_connection(|connection| list_entries(connection, project_id, include_global))
            .map_err(|error| error.to_string())
    }

    pub fn read_entry(&self, memory_id: &str) -> Result<Option<LocalMemoryEntryDto>, String> {
        self.database
            .with_connection(|connection| read_entry(connection, memory_id))
            .map_err(|error| error.to_string())
    }

    pub fn create_retrieval(
        &self,
        record: LocalMemoryRetrievalRecord,
        candidates: Vec<LocalMemoryCandidateRecord>,
    ) -> Result<LocalMemoryRetrievalDto, String> {
        self.database
            .transaction(|transaction| {
                transaction.execute(
                    r#"
                    INSERT INTO local_memory_retrievals (
                        retrieval_id, project_id, query_text, min_similarity, max_results, status
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, 'waiting_user')
                    "#,
                    params![
                        record.retrieval_id,
                        record.project_id,
                        record.query_text,
                        record.min_similarity,
                        record.max_results,
                    ],
                )?;
                for candidate in candidates {
                    let citation_json = serde_json::to_string(&candidate.citation)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
                    transaction.execute(
                        r#"
                        INSERT INTO local_memory_retrieval_candidates (
                            candidate_id, retrieval_id, memory_id, similarity, status, reason, citation_json
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                        "#,
                        params![
                            candidate.candidate_id,
                            candidate.retrieval_id,
                            candidate.memory_id,
                            candidate.similarity,
                            candidate.status,
                            candidate.reason,
                            citation_json,
                        ],
                    )?;
                }
                read_retrieval(transaction, &record.retrieval_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Local memory retrieval was saved but cannot be read.".to_string())
    }

    pub fn read_retrieval(
        &self,
        retrieval_id: &str,
    ) -> Result<Option<LocalMemoryRetrievalDto>, String> {
        self.database
            .with_connection(|connection| read_retrieval(connection, retrieval_id))
            .map_err(|error| error.to_string())
    }

    pub fn set_candidate_status(
        &self,
        candidate_id: &str,
        status: &str,
        reason: Option<&str>,
    ) -> Result<LocalMemoryRetrievalCandidateDto, String> {
        self.database
            .with_connection(|connection| {
                connection.execute(
                    r#"
                    UPDATE local_memory_retrieval_candidates
                    SET status = ?1,
                        reason = COALESCE(?2, reason),
                        updated_at = CURRENT_TIMESTAMP
                    WHERE candidate_id = ?3
                    "#,
                    params![status, reason, candidate_id],
                )?;
                read_candidate(connection, candidate_id)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("Local memory retrieval candidate not found: {candidate_id}"))
    }
}

impl Repository for LocalMemoryRepository<'_> {
    fn database(&self) -> &Database {
        self.database
    }
}

#[derive(Debug, Clone)]
pub struct LocalMemoryEntryRecord {
    pub memory_id: String,
    pub project_id: Option<String>,
    pub source_kind: String,
    pub source_id: String,
    pub source_label: String,
    pub content_summary: String,
    pub content_hash: String,
    pub embedding_provider_id: Option<String>,
    pub embedding_model_id: Option<String>,
    pub embedding_vector_path: Option<String>,
    pub metadata: Value,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LocalMemoryRetrievalRecord {
    pub retrieval_id: String,
    pub project_id: String,
    pub query_text: String,
    pub min_similarity: f64,
    pub max_results: i64,
}

impl LocalMemoryRetrievalRecord {
    pub fn new(
        project_id: String,
        query_text: String,
        min_similarity: f64,
        max_results: i64,
    ) -> Self {
        Self {
            retrieval_id: create_id("memory_retrieval"),
            project_id,
            query_text,
            min_similarity,
            max_results,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalMemoryCandidateRecord {
    pub candidate_id: String,
    pub retrieval_id: String,
    pub memory_id: String,
    pub similarity: f64,
    pub status: String,
    pub reason: Option<String>,
    pub citation: Value,
}

impl LocalMemoryCandidateRecord {
    pub fn new(
        retrieval_id: String,
        memory_id: String,
        similarity: f64,
        status: String,
        reason: Option<String>,
        citation: Value,
    ) -> Self {
        Self {
            candidate_id: create_id("memory_candidate"),
            retrieval_id,
            memory_id,
            similarity,
            status,
            reason,
            citation,
        }
    }
}

fn list_entries(
    connection: &Connection,
    project_id: Option<&str>,
    include_global: bool,
) -> Result<Vec<LocalMemoryEntryDto>, rusqlite::Error> {
    let mut entries = Vec::new();
    match (project_id, include_global) {
        (Some(project_id), true) => {
            let mut statement = connection.prepare(
                r#"
                SELECT memory_id, project_id, source_kind, source_id, source_label,
                       content_summary, content_hash, embedding_provider_id,
                       embedding_model_id, embedding_vector_path, metadata_json,
                       lifecycle, expires_at, created_at, updated_at
                FROM local_memory_entries
                WHERE lifecycle = 'active'
                  AND (project_id = ?1 OR project_id IS NULL)
                  AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
                ORDER BY updated_at DESC
                "#,
            )?;
            for row in statement.query_map([project_id], row_to_entry)? {
                entries.push(row?);
            }
        }
        (Some(project_id), false) => {
            let mut statement = connection.prepare(
                r#"
                SELECT memory_id, project_id, source_kind, source_id, source_label,
                       content_summary, content_hash, embedding_provider_id,
                       embedding_model_id, embedding_vector_path, metadata_json,
                       lifecycle, expires_at, created_at, updated_at
                FROM local_memory_entries
                WHERE lifecycle = 'active'
                  AND project_id = ?1
                  AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
                ORDER BY updated_at DESC
                "#,
            )?;
            for row in statement.query_map([project_id], row_to_entry)? {
                entries.push(row?);
            }
        }
        (None, _) => {
            let mut statement = connection.prepare(
                r#"
                SELECT memory_id, project_id, source_kind, source_id, source_label,
                       content_summary, content_hash, embedding_provider_id,
                       embedding_model_id, embedding_vector_path, metadata_json,
                       lifecycle, expires_at, created_at, updated_at
                FROM local_memory_entries
                WHERE lifecycle = 'active'
                  AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
                ORDER BY updated_at DESC
                "#,
            )?;
            for row in statement.query_map([], row_to_entry)? {
                entries.push(row?);
            }
        }
    }
    Ok(entries)
}

fn read_entry(
    connection: &Connection,
    memory_id: &str,
) -> Result<Option<LocalMemoryEntryDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT memory_id, project_id, source_kind, source_id, source_label,
                   content_summary, content_hash, embedding_provider_id,
                   embedding_model_id, embedding_vector_path, metadata_json,
                   lifecycle, expires_at, created_at, updated_at
            FROM local_memory_entries
            WHERE memory_id = ?1
            "#,
            [memory_id],
            row_to_entry,
        )
        .optional()
}

fn row_to_entry(row: &Row<'_>) -> Result<LocalMemoryEntryDto, rusqlite::Error> {
    let metadata_json: String = row.get(10)?;
    Ok(LocalMemoryEntryDto {
        memory_id: row.get(0)?,
        project_id: row.get(1)?,
        source_kind: row.get(2)?,
        source_id: row.get(3)?,
        source_label: row.get(4)?,
        content_summary: row.get(5)?,
        content_hash: row.get(6)?,
        embedding_provider_id: row.get(7)?,
        embedding_model_id: row.get(8)?,
        embedding_vector_path: row.get(9)?,
        metadata: serde_json::from_str(&metadata_json).unwrap_or_default(),
        lifecycle: row.get(11)?,
        expires_at: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

fn read_retrieval(
    connection: &Connection,
    retrieval_id: &str,
) -> Result<Option<LocalMemoryRetrievalDto>, rusqlite::Error> {
    let retrieval = connection
        .query_row(
            r#"
            SELECT retrieval_id, project_id, query_text, min_similarity, max_results,
                   status, created_at, updated_at
            FROM local_memory_retrievals
            WHERE retrieval_id = ?1
            "#,
            [retrieval_id],
            |row| {
                Ok(LocalMemoryRetrievalDto {
                    retrieval_id: row.get(0)?,
                    project_id: row.get(1)?,
                    query_text: row.get(2)?,
                    min_similarity: row.get(3)?,
                    max_results: row.get(4)?,
                    status: row.get(5)?,
                    candidates: Vec::new(),
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .optional()?;

    let Some(mut retrieval) = retrieval else {
        return Ok(None);
    };
    retrieval.candidates = list_candidates(connection, retrieval_id)?;
    Ok(Some(retrieval))
}

fn list_candidates(
    connection: &Connection,
    retrieval_id: &str,
) -> Result<Vec<LocalMemoryRetrievalCandidateDto>, rusqlite::Error> {
    let mut statement = connection.prepare(
        r#"
        SELECT candidate_id, retrieval_id, memory_id, similarity, status,
               reason, citation_json, created_at, updated_at
        FROM local_memory_retrieval_candidates
        WHERE retrieval_id = ?1
        ORDER BY similarity DESC, created_at ASC
        "#,
    )?;
    let rows = statement.query_map([retrieval_id], row_to_candidate)?;
    rows.collect()
}

fn read_candidate(
    connection: &Connection,
    candidate_id: &str,
) -> Result<Option<LocalMemoryRetrievalCandidateDto>, rusqlite::Error> {
    connection
        .query_row(
            r#"
            SELECT candidate_id, retrieval_id, memory_id, similarity, status,
                   reason, citation_json, created_at, updated_at
            FROM local_memory_retrieval_candidates
            WHERE candidate_id = ?1
            "#,
            [candidate_id],
            row_to_candidate,
        )
        .optional()
}

fn row_to_candidate(row: &Row<'_>) -> Result<LocalMemoryRetrievalCandidateDto, rusqlite::Error> {
    let citation_json: String = row.get(6)?;
    Ok(LocalMemoryRetrievalCandidateDto {
        candidate_id: row.get(0)?,
        retrieval_id: row.get(1)?,
        memory_id: row.get(2)?,
        similarity: row.get(3)?,
        status: row.get(4)?,
        reason: row.get(5)?,
        citation: serde_json::from_str(&citation_json).unwrap_or_default(),
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub fn create_entry_id() -> String {
    create_id("memory")
}

fn create_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}_{nanos}")
}
