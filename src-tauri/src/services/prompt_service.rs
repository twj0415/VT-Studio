use crate::db::Database;
use crate::domain::prompt::{
    CreativeRuleDto, CreativeRuleIdRequest, CreativeRuleRefDto, CreativeRuleReferenceCountsDto,
    ListCreativeRulesRequest, SaveCreativeRuleRequest, SetCreativeRuleEnabledRequest,
};
use crate::security::secret_guard::{reject_json_secrets, reject_text_secrets};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

const RULE_ROOT: &str = "prompts";
const BUILTIN_SOURCE: &str = "builtin";
const USER_SOURCE: &str = "user";

pub fn list_creative_rules(
    database: &Database,
    workspace_root: &Path,
    request: ListCreativeRulesRequest,
) -> Result<Vec<CreativeRuleDto>, String> {
    initialize_creative_rules(workspace_root)?;
    let mut rules = Vec::new();
    for source_type in [BUILTIN_SOURCE, USER_SOURCE] {
        if request
            .source_type
            .as_deref()
            .is_some_and(|requested| requested != source_type)
        {
            continue;
        }
        let source_root = creative_rule_root(workspace_root).join(source_type);
        if !source_root.exists() {
            continue;
        }
        collect_rules(&source_root, &mut rules)?;
    }
    if let Some(module) = request.module.as_deref() {
        rules.retain(|rule| rule.module == module);
    }
    rules.sort_by(|left, right| {
        left.module
            .cmp(&right.module)
            .then_with(|| left.source_type.cmp(&right.source_type))
            .then_with(|| left.key.cmp(&right.key))
    });
    rules
        .into_iter()
        .map(|rule| attach_reference_counts(database, rule))
        .collect()
}

pub fn get_creative_rule(
    database: &Database,
    workspace_root: &Path,
    request: CreativeRuleIdRequest,
) -> Result<CreativeRuleDto, String> {
    initialize_creative_rules(workspace_root)?;
    let path = resolve_rule_id(workspace_root, &request.rule_id)?;
    attach_reference_counts(database, parse_rule_file(&path)?)
}

pub fn clone_creative_rule_to_user(
    database: &Database,
    workspace_root: &Path,
    request: CreativeRuleIdRequest,
) -> Result<CreativeRuleDto, String> {
    initialize_creative_rules(workspace_root)?;
    let source_path = resolve_rule_id(workspace_root, &request.rule_id)?;
    let mut rule = parse_rule_file(&source_path)?;
    if rule.source_type != BUILTIN_SOURCE {
        return Err("only builtin creative rules can be cloned.".to_string());
    }

    rule.source_type = USER_SOURCE.to_string();
    rule.key = format!("user.{}", rule.key);
    rule.name = format!("{} Copy", rule.name);
    rule.enabled = false;
    validate_creative_rule(&rule)?;
    let target_path = rule_path(
        workspace_root,
        USER_SOURCE,
        &rule.module,
        &key_file_name(&rule.key),
    )?;
    if target_path.exists() {
        return Err("user creative rule already exists.".to_string());
    }
    write_rule_file(&target_path, &rule)?;
    attach_reference_counts(database, parse_rule_file(&target_path)?)
}

pub fn save_user_creative_rule(
    database: &Database,
    workspace_root: &Path,
    request: SaveCreativeRuleRequest,
) -> Result<CreativeRuleDto, String> {
    initialize_creative_rules(workspace_root)?;
    let rule = CreativeRuleDto {
        rule_id: format!(
            "{USER_SOURCE}:{}:{}",
            request.module,
            key_file_name(&request.key)
        ),
        key: request.key,
        name: request.name,
        module: request.module,
        rule_type: request.rule_type,
        provider_kind: request.provider_kind,
        version: request.version.unwrap_or_else(|| "1.0.0".to_string()),
        output_schema: request.output_schema,
        params_schema: request.params_schema.unwrap_or_else(|| json!({})),
        description: request.description,
        source_type: USER_SOURCE.to_string(),
        enabled: request.enabled,
        body: request.body,
        relative_path: String::new(),
        content_hash: String::new(),
        schema_hash: String::new(),
        reference_counts: CreativeRuleReferenceCountsDto::default(),
    };
    validate_creative_rule(&rule)?;
    let target_path = rule_path(
        workspace_root,
        USER_SOURCE,
        &rule.module,
        &key_file_name(&rule.key),
    )?;
    write_rule_file(&target_path, &rule)?;
    attach_reference_counts(database, parse_rule_file(&target_path)?)
}

pub fn set_user_creative_rule_enabled(
    database: &Database,
    workspace_root: &Path,
    request: SetCreativeRuleEnabledRequest,
) -> Result<CreativeRuleDto, String> {
    initialize_creative_rules(workspace_root)?;
    let path = resolve_rule_id(workspace_root, &request.rule_id)?;
    let mut rule = parse_rule_file(&path)?;
    if rule.source_type != USER_SOURCE {
        return Err("builtin creative rules cannot be modified directly.".to_string());
    }
    if !request.enabled {
        let counts = reference_counts(database, &rule)?;
        if reference_total(&counts) > 0 {
            return Err(format_reference_in_use(&counts));
        }
    }
    rule.enabled = request.enabled;
    validate_creative_rule(&rule)?;
    write_rule_file(&path, &rule)?;
    attach_reference_counts(database, parse_rule_file(&path)?)
}

pub fn delete_user_creative_rule(
    database: &Database,
    workspace_root: &Path,
    request: CreativeRuleIdRequest,
) -> Result<CreativeRuleDto, String> {
    initialize_creative_rules(workspace_root)?;
    let path = resolve_rule_id(workspace_root, &request.rule_id)?;
    let rule = parse_rule_file(&path)?;
    if rule.source_type != USER_SOURCE {
        return Err("builtin creative rules cannot be deleted.".to_string());
    }
    let counts = reference_counts(database, &rule)?;
    if reference_total(&counts) > 0 {
        return Err(format_reference_in_use(&counts));
    }
    fs::remove_file(&path).map_err(|error| error.to_string())?;
    Ok(CreativeRuleDto {
        reference_counts: counts,
        ..rule
    })
}

pub fn resolve_creative_rule_refs(workspace_root: &Path, refs: &Value) -> Result<Value, String> {
    initialize_creative_rules(workspace_root)?;
    let object = refs
        .as_object()
        .ok_or_else(|| "rule_refs must be a JSON object keyed by rule slot.".to_string())?;
    let mut resolved = serde_json::Map::new();
    for (slot, value) in object {
        validate_rule_slot(slot)?;
        let ref_object = value.as_object().ok_or_else(|| {
            format!(
                "rule_refs.{slot} must be an object with ruleId/ruleKey; string refs are not accepted."
            )
        })?;
        let rule_id = ref_object
            .get("ruleId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("rule_refs.{slot}.ruleId is required."))?;
        let rule_key = ref_object
            .get("ruleKey")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("rule_refs.{slot}.ruleKey is required."))?;
        let rule = parse_rule_file(&resolve_rule_id(workspace_root, rule_id)?)?;
        if rule.key != rule_key {
            return Err(format!(
                "rule_refs.{slot}.ruleKey does not match ruleId; expected {}.",
                rule.key
            ));
        }
        if rule.rule_type != expected_rule_type_for_slot(slot) {
            return Err(format!(
                "rule_refs.{slot} expects {}, got {}.",
                expected_rule_type_for_slot(slot),
                rule.rule_type
            ));
        }
        if !rule.enabled {
            return Err(format!(
                "rule_refs.{slot} references disabled creative rule {}.",
                rule.rule_id
            ));
        }
        let rule_ref = rule_ref_from_rule(slot, &rule);
        resolved.insert(
            slot.to_string(),
            serde_json::to_value(rule_ref).map_err(|error| error.to_string())?,
        );
    }
    Ok(Value::Object(resolved))
}

pub fn project_rule_snapshot(
    database: &Database,
    workspace_root: &Path,
    project_id: &str,
) -> Result<Value, String> {
    initialize_creative_rules(workspace_root)?;
    let (active_pack_id, rule_refs) = database
        .with_connection(|connection| {
            connection.query_row(
                "SELECT active_pack_id, rule_refs_json FROM projects WHERE project_id = ?1",
                [project_id],
                |row| {
                    let active_pack_id: Option<String> = row.get(0)?;
                    let rule_refs_json: String = row.get(1)?;
                    let rule_refs = serde_json::from_str::<Value>(&rule_refs_json)
                        .unwrap_or_else(|_| json!({}));
                    Ok((active_pack_id, rule_refs))
                },
            )
        })
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "activePackId": active_pack_id,
        "ruleRefs": rule_refs,
        "skillSnapshots": build_rule_snapshots(workspace_root, &rule_refs)?,
    }))
}

fn initialize_creative_rules(workspace_root: &Path) -> Result<(), String> {
    let root = creative_rule_root(workspace_root);
    for source_type in [BUILTIN_SOURCE, USER_SOURCE] {
        for module in allowed_modules() {
            fs::create_dir_all(root.join(source_type).join(module))
                .map_err(|error| error.to_string())?;
        }
    }

    for seed in builtin_rule_seeds() {
        let path = rule_path(workspace_root, BUILTIN_SOURCE, seed.module, seed.file_name)?;
        fs::write(&path, seed.content).map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn collect_rules(root: &Path, rules: &mut Vec<CreativeRuleDto>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_rules(&path, rules)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            rules.push(parse_rule_file(&path)?);
        }
    }
    Ok(())
}

fn attach_reference_counts(
    database: &Database,
    mut rule: CreativeRuleDto,
) -> Result<CreativeRuleDto, String> {
    rule.reference_counts = reference_counts(database, &rule)?;
    Ok(rule)
}

fn reference_counts(
    database: &Database,
    rule: &CreativeRuleDto,
) -> Result<CreativeRuleReferenceCountsDto, String> {
    let rule_key = rule.key.clone();
    let rule_id = rule.rule_id.clone();
    database
        .with_connection(|connection| {
            let video_packs = count_json_references(
                connection,
                "video_packs",
                "rule_refs_json",
                &rule_key,
                &rule_id,
            )?;
            let projects = count_json_references(
                connection,
                "projects",
                "rule_refs_json",
                &rule_key,
                &rule_id,
            )?;
            let task_steps =
                count_json_references(connection, "task_steps", "input_json", &rule_key, &rule_id)?;
            let image_contexts = count_json_references(
                connection,
                "image_candidates",
                "generation_context_snapshot_json",
                &rule_key,
                &rule_id,
            )?;
            let video_contexts = count_json_references(
                connection,
                "video_segments",
                "generation_context_snapshot_json",
                &rule_key,
                &rule_id,
            )?;
            Ok(CreativeRuleReferenceCountsDto {
                video_packs,
                projects,
                task_steps,
                generation_contexts: image_contexts + video_contexts,
            })
        })
        .map_err(|error| error.to_string())
}

fn count_json_references(
    connection: &rusqlite::Connection,
    table: &str,
    column: &str,
    rule_key: &str,
    rule_id: &str,
) -> Result<u32, rusqlite::Error> {
    let sql = format!(
        "SELECT {column} FROM {table} WHERE {column} LIKE '%' || ?1 || '%' OR {column} LIKE '%' || ?2 || '%'"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([rule_key, rule_id], |row| row.get::<_, Option<String>>(0))?;
    let mut count = 0u32;
    for row in rows {
        let Some(text) = row? else {
            continue;
        };
        if text.contains(rule_key) || text.contains(rule_id) {
            count += 1;
        }
    }
    Ok(count)
}

fn reference_total(counts: &CreativeRuleReferenceCountsDto) -> u32 {
    counts.video_packs + counts.projects + counts.task_steps + counts.generation_contexts
}

fn format_reference_in_use(counts: &CreativeRuleReferenceCountsDto) -> String {
    format!(
        "creative_rule.in_use: rule is referenced by videoPacks={}, projects={}, taskSteps={}, generationContexts={}.",
        counts.video_packs, counts.projects, counts.task_steps, counts.generation_contexts
    )
}

fn build_rule_snapshots(workspace_root: &Path, rule_refs: &Value) -> Result<Value, String> {
    let Some(object) = rule_refs.as_object() else {
        return Ok(json!({}));
    };
    let mut snapshots = serde_json::Map::new();
    for (slot, value) in object {
        let Some(rule_id) = value.get("ruleId").and_then(Value::as_str) else {
            continue;
        };
        let rule = parse_rule_file(&resolve_rule_id(workspace_root, rule_id)?)?;
        snapshots.insert(
            slot.clone(),
            json!({
                "slot": slot,
                "ruleKey": rule.key,
                "ruleId": rule.rule_id,
                "sourceType": rule.source_type,
                "ruleType": rule.rule_type,
                "module": rule.module,
                "name": rule.name,
                "version": rule.version,
                "contentHash": rule.content_hash,
                "schemaHash": rule.schema_hash,
                "relativePath": rule.relative_path,
                "enabled": rule.enabled
            }),
        );
    }
    Ok(Value::Object(snapshots))
}

fn rule_ref_from_rule(slot: &str, rule: &CreativeRuleDto) -> CreativeRuleRefDto {
    CreativeRuleRefDto {
        slot: slot.to_string(),
        rule_key: rule.key.clone(),
        rule_id: rule.rule_id.clone(),
        source_type: rule.source_type.clone(),
        rule_type: rule.rule_type.clone(),
        module: rule.module.clone(),
        name: rule.name.clone(),
        version: rule.version.clone(),
        content_hash: rule.content_hash.clone(),
        schema_hash: rule.schema_hash.clone(),
        enabled: rule.enabled,
    }
}

fn creative_rule_content_hash(rule: &CreativeRuleDto) -> Result<String, String> {
    stable_hash_json(&json!({
        "key": rule.key,
        "module": rule.module,
        "ruleType": rule.rule_type,
        "providerKind": rule.provider_kind,
        "version": rule.version,
        "body": rule.body,
        "paramsSchema": rule.params_schema,
    }))
}

fn creative_rule_schema_hash(rule: &CreativeRuleDto) -> Result<String, String> {
    stable_hash_json(&json!({
        "outputSchema": rule.output_schema,
        "paramsSchema": rule.params_schema,
    }))
}

fn stable_hash_json(value: &Value) -> Result<String, String> {
    let serialized = serde_json::to_string(value).map_err(|error| error.to_string())?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    serialized.hash(&mut hasher);
    Ok(format!("hash_{:016x}", hasher.finish()))
}

fn parse_rule_file(path: &Path) -> Result<CreativeRuleDto, String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let (frontmatter, body) = split_frontmatter(&content)?;
    let frontmatter = parse_frontmatter(frontmatter)?;
    let source_type = frontmatter_string(&frontmatter, "source_type")?;
    let module = frontmatter_string(&frontmatter, "module")?;
    let file_name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "creative rule file name is invalid.".to_string())?;
    let mut rule = CreativeRuleDto {
        rule_id: format!("{source_type}:{module}:{file_name}"),
        key: frontmatter_string(&frontmatter, "key")?,
        name: frontmatter_string(&frontmatter, "name")?,
        module,
        rule_type: frontmatter_string(&frontmatter, "rule_type")?,
        provider_kind: frontmatter_string(&frontmatter, "provider_kind")?,
        version: frontmatter
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("1.0.0")
            .to_string(),
        output_schema: frontmatter
            .get("output_schema")
            .cloned()
            .ok_or_else(|| "output_schema is required.".to_string())?,
        params_schema: frontmatter
            .get("params_schema")
            .cloned()
            .ok_or_else(|| "params_schema is required.".to_string())?,
        description: frontmatter_string(&frontmatter, "description")?,
        source_type,
        enabled: frontmatter
            .get("enabled")
            .and_then(Value::as_bool)
            .ok_or_else(|| "enabled must be true or false.".to_string())?,
        body: body.trim().to_string(),
        relative_path: String::new(),
        content_hash: String::new(),
        schema_hash: String::new(),
        reference_counts: CreativeRuleReferenceCountsDto::default(),
    };
    validate_creative_rule(&rule)?;
    rule.relative_path = relative_rule_path(path)?;
    rule.content_hash = creative_rule_content_hash(&rule)?;
    rule.schema_hash = creative_rule_schema_hash(&rule)?;
    Ok(rule)
}

fn split_frontmatter(content: &str) -> Result<(&str, &str), String> {
    let Some(rest) = content.strip_prefix("---\n") else {
        return Err("creative rule must start with frontmatter.".to_string());
    };
    let Some(end_index) = rest.find("\n---\n") else {
        return Err("creative rule frontmatter must be closed.".to_string());
    };
    let frontmatter = &rest[..end_index];
    let body = &rest[end_index + "\n---\n".len()..];
    Ok((frontmatter, body))
}

fn parse_frontmatter(raw: &str) -> Result<BTreeMap<String, Value>, String> {
    let mut map = BTreeMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(split_index) = line.find(':') else {
            return Err("frontmatter lines must use key: value.".to_string());
        };
        let key = line[..split_index].trim().to_string();
        let value = line[split_index + 1..].trim();
        if key == "output_schema" || key == "params_schema" {
            let schema = serde_json::from_str::<Value>(value)
                .map_err(|_| format!("{key} must be a JSON object."))?;
            map.insert(key, schema);
        } else if key == "enabled" {
            match value {
                "true" => {
                    map.insert(key, Value::Bool(true));
                }
                "false" => {
                    map.insert(key, Value::Bool(false));
                }
                _ => return Err("enabled must be true or false.".to_string()),
            };
        } else {
            map.insert(key, Value::String(unquote(value).to_string()));
        }
    }
    Ok(map)
}

fn validate_creative_rule(rule: &CreativeRuleDto) -> Result<(), String> {
    validate_identifier("key", &rule.key)?;
    validate_identifier("version", &rule.version)?;
    validate_one_of("module", &rule.module, allowed_modules())?;
    validate_one_of("rule_type", &rule.rule_type, allowed_rule_types())?;
    validate_one_of(
        "provider_kind",
        &rule.provider_kind,
        &["llm", "image", "video", "tts", "vlm", "workflow"],
    )?;
    validate_one_of(
        "source_type",
        &rule.source_type,
        &[BUILTIN_SOURCE, USER_SOURCE],
    )?;
    if rule.name.trim().is_empty() {
        return Err("name cannot be empty.".to_string());
    }
    if rule.description.trim().is_empty() {
        return Err("description cannot be empty.".to_string());
    }
    if rule.body.trim().is_empty() {
        return Err("creative rule body cannot be empty.".to_string());
    }
    if !rule.output_schema.is_object() {
        return Err("output_schema must be a JSON object.".to_string());
    }
    if !rule.params_schema.is_object() {
        return Err("params_schema must be a JSON object.".to_string());
    }
    reject_json_secrets(&rule.output_schema)?;
    reject_json_secrets(&rule.params_schema)?;
    reject_text_secrets("creative_rule.body", &rule.body)?;
    reject_text_secrets("creative_rule.description", &rule.description)?;
    Ok(())
}

fn write_rule_file(path: &Path, rule: &CreativeRuleDto) -> Result<(), String> {
    validate_creative_rule(rule)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let output_schema =
        serde_json::to_string(&rule.output_schema).map_err(|error| error.to_string())?;
    let params_schema =
        serde_json::to_string(&rule.params_schema).map_err(|error| error.to_string())?;
    let content = format!(
        "---\nkey: {}\nname: {}\nmodule: {}\nrule_type: {}\nprovider_kind: {}\nversion: {}\noutput_schema: {}\nparams_schema: {}\ndescription: {}\nsource_type: {}\nenabled: {}\n---\n\n{}\n",
        rule.key,
        rule.name,
        rule.module,
        rule.rule_type,
        rule.provider_kind,
        rule.version,
        output_schema,
        params_schema,
        rule.description,
        rule.source_type,
        rule.enabled,
        rule.body.trim()
    );
    fs::write(path, content).map_err(|error| error.to_string())
}

fn resolve_rule_id(workspace_root: &Path, rule_id: &str) -> Result<PathBuf, String> {
    let parts = rule_id.split(':').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err("rule_id must use source:module:file format.".to_string());
    }
    let source_type = parts[0];
    let module = parts[1];
    let file_name = parts[2];
    validate_one_of("source_type", source_type, &[BUILTIN_SOURCE, USER_SOURCE])?;
    validate_one_of("module", module, allowed_modules())?;
    validate_file_name(file_name)?;
    let path = creative_rule_root(workspace_root)
        .join(source_type)
        .join(module)
        .join(format!("{file_name}.md"));
    if !path.is_file() {
        return Err("creative_rule.not_found: creative rule was not found.".to_string());
    }
    ensure_inside_rule_root(workspace_root, &path)?;
    Ok(path)
}

fn rule_path(
    workspace_root: &Path,
    source_type: &str,
    module: &str,
    file_name: &str,
) -> Result<PathBuf, String> {
    validate_one_of("source_type", source_type, &[BUILTIN_SOURCE, USER_SOURCE])?;
    validate_one_of("module", module, allowed_modules())?;
    validate_file_name(file_name)?;
    let path = creative_rule_root(workspace_root)
        .join(source_type)
        .join(module)
        .join(format!("{file_name}.md"));
    ensure_inside_rule_root(workspace_root, &path)?;
    Ok(path)
}

fn ensure_inside_rule_root(workspace_root: &Path, path: &Path) -> Result<(), String> {
    let root = creative_rule_root(workspace_root);
    if path.starts_with(&root) {
        return Ok(());
    }
    Err("creative rule path escaped workspace prompt root.".to_string())
}

fn creative_rule_root(workspace_root: &Path) -> PathBuf {
    workspace_root.join(RULE_ROOT)
}

fn relative_rule_path(path: &Path) -> Result<String, String> {
    let parts = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let Some(index) = parts.iter().position(|part| part == RULE_ROOT) else {
        return Ok(path.to_string_lossy().replace('\\', "/"));
    };
    Ok(parts[index..].join("/"))
}

fn frontmatter_string(map: &BTreeMap<String, Value>, key: &str) -> Result<String, String> {
    map.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{key} is required."))
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn key_file_name(key: &str) -> String {
    key.replace('.', "_")
}

fn validate_identifier(name: &str, value: &str) -> Result<(), String> {
    if value.trim() != value || value.is_empty() {
        return Err(format!("{name} cannot be empty or padded."));
    }
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "_-.:".contains(character))
    {
        return Ok(());
    }
    Err(format!(
        "{name} may only contain ASCII letters, numbers, underscore, hyphen, dot, or colon."
    ))
}

fn validate_file_name(value: &str) -> Result<(), String> {
    if value.trim() != value || value.is_empty() {
        return Err("file name cannot be empty or padded.".to_string());
    }
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "_-.".contains(character))
    {
        return Ok(());
    }
    Err(
        "file name may only contain ASCII letters, numbers, underscore, hyphen, or dot."
            .to_string(),
    )
}

fn validate_one_of(name: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(format!("{name} has unsupported value."))
}

fn validate_rule_slot(slot: &str) -> Result<(), String> {
    validate_one_of("rule_slot", slot, allowed_rule_slots())
}

fn allowed_rule_slots() -> &'static [&'static str] {
    &[
        "script",
        "storyboard",
        "character",
        "scene",
        "style",
        "image_prompt",
        "storyboard_image",
        "video_prompt",
        "review",
    ]
}

fn expected_rule_type_for_slot(slot: &str) -> &'static str {
    match slot {
        "script" => "script_rule",
        "storyboard" => "storyboard_rule",
        "character" => "character_rule",
        "scene" => "scene_rule",
        "style" => "style_rule",
        "image_prompt" => "image_prompt_rule",
        "storyboard_image" => "storyboard_image_rule",
        "video_prompt" => "video_prompt_rule",
        "review" => "review_rule",
        _ => "review_rule",
    }
}

fn allowed_modules() -> &'static [&'static str] {
    &[
        "script",
        "storyboard",
        "character",
        "scene",
        "style",
        "image_prompt",
        "storyboard_image",
        "video_prompt",
        "subtitle",
        "cover",
        "review",
    ]
}

fn allowed_rule_types() -> &'static [&'static str] {
    &[
        "storyboard_rule",
        "character_rule",
        "scene_rule",
        "style_rule",
        "image_prompt_rule",
        "storyboard_image_rule",
        "video_prompt_rule",
        "review_rule",
        "script_rule",
        "subtitle_rule",
        "cover_rule",
    ]
}

struct BuiltinRuleSeed {
    module: &'static str,
    file_name: &'static str,
    content: &'static str,
}

fn builtin_rule_seeds() -> Vec<BuiltinRuleSeed> {
    vec![
        BuiltinRuleSeed {
            module: "script",
            file_name: "script_topic_narration",
            content: builtin_rule(
                "script.topic_narration",
                "Topic Narration",
                "script",
                "script_rule",
                "Generate a short-video narration draft from a concise topic.",
                r#"{"type":"object","required":["title","narrations"],"properties":{"title":{"type":"string"},"narrations":{"type":"array"}}}"#,
                r#"{"type":"object","properties":{"tone":{"type":"string"},"targetCount":{"type":"integer"}}}"#,
                "Create a concise narration plan from {{project.topic}}. Return JSON only.",
            ),
        },
        BuiltinRuleSeed {
            module: "storyboard",
            file_name: "storyboard_default",
            content: builtin_rule(
                "storyboard.default",
                "Default Storyboard",
                "storyboard",
                "storyboard_rule",
                "Split narration into storyboard items with visual intent and duration.",
                r#"{"type":"object","required":["items"],"properties":{"items":{"type":"array"}}}"#,
                r#"{"type":"object","properties":{"targetSceneCount":{"type":"integer"},"durationSeconds":{"type":"number"}}}"#,
                "Create storyboard items from {{script.narrations}}. Keep order and return JSON only.",
            ),
        },
        BuiltinRuleSeed {
            module: "character",
            file_name: "character_default",
            content: builtin_rule(
                "character.default",
                "Default Character",
                "character",
                "character_rule",
                "Extract editable character bible fields from narration and storyboard context.",
                r#"{"type":"object","required":["characters"],"properties":{"characters":{"type":"array"}}}"#,
                r#"{"type":"object","properties":{"mergeSimilar":{"type":"boolean"},"language":{"type":"string"}}}"#,
                "Extract character profiles with stable ids, appearance, clothing, personality, and reference needs. Return JSON only.",
            ),
        },
        BuiltinRuleSeed {
            module: "scene",
            file_name: "scene_default",
            content: builtin_rule(
                "scene.default",
                "Default Scene",
                "scene",
                "scene_rule",
                "Extract reusable environment and location bible fields from storyboard context.",
                r#"{"type":"object","required":["locations"],"properties":{"locations":{"type":"array"}}}"#,
                r#"{"type":"object","properties":{"mergeSimilar":{"type":"boolean"},"includeVariants":{"type":"boolean"}}}"#,
                "Extract reusable locations with stable ids, space description, lighting, time of day, props, and variants. Return JSON only.",
            ),
        },
        BuiltinRuleSeed {
            module: "style",
            file_name: "style_default",
            content: builtin_rule(
                "style.default",
                "Default Style",
                "style",
                "style_rule",
                "Create structured Style Bible fields for the current project.",
                r#"{"type":"object","required":["style_prompt","color_palette","lighting","composition","negative_prompt"],"properties":{"style_prompt":{"type":"string"},"color_palette":{"type":"array"},"lighting":{"type":"string"},"composition":{"type":"string"},"negative_prompt":{"type":"string"}}}"#,
                r#"{"type":"object","properties":{"referencePolicy":{"type":"string"},"language":{"type":"string"}}}"#,
                "Create a reusable style bible. Do not copy people, logos, brands, or private details from references. Return JSON only.",
            ),
        },
        BuiltinRuleSeed {
            module: "image_prompt",
            file_name: "image_prompt_shot_frame",
            content: builtin_rule(
                "image_prompt.shot_frame",
                "Storyboard Image Prompt",
                "image_prompt",
                "image_prompt_rule",
                "Create image prompts from storyboard and bible context.",
                r#"{"type":"object","required":["prompts"],"properties":{"prompts":{"type":"array"}}}"#,
                r#"{"type":"object","properties":{"includeStyleBible":{"type":"boolean"},"includeCharacterBible":{"type":"boolean"},"includeLocationBible":{"type":"boolean"}}}"#,
                "Write image prompts for {{storyboard.items}} using style and character bibles. Return JSON only.",
            ),
        },
        BuiltinRuleSeed {
            module: "storyboard_image",
            file_name: "storyboard_image_default",
            content: builtin_rule(
                "storyboard_image.default",
                "Storyboard Image Generation",
                "storyboard_image",
                "storyboard_image_rule",
                "Apply final storyboard-image generation instructions and review constraints.",
                r#"{"type":"object","required":["prompt","negative_prompt"],"properties":{"prompt":{"type":"string"},"negative_prompt":{"type":"string"}}}"#,
                r#"{"type":"object","properties":{"referenceImagePolicy":{"type":"string"},"maxNegativePromptLength":{"type":"integer"}}}"#,
                "Assemble a final image prompt from storyboard, style, character, location, and model constraints. Return JSON only.",
            ),
        },
        BuiltinRuleSeed {
            module: "video_prompt",
            file_name: "video_prompt_image_to_video",
            content: builtin_rule(
                "video_prompt.image_to_video",
                "Image To Video Prompt",
                "video_prompt",
                "video_prompt_rule",
                "Create motion prompts for image-to-video generation.",
                r#"{"type":"object","required":["prompts"],"properties":{"prompts":{"type":"array"}}}"#,
                r#"{"type":"object","properties":{"durationSeconds":{"type":"number"},"cameraMotionAllowed":{"type":"boolean"}}}"#,
                "Write video motion prompts from selected images and storyboard intent. Return JSON only.",
            ),
        },
        BuiltinRuleSeed {
            module: "subtitle",
            file_name: "subtitle_default",
            content: builtin_rule(
                "subtitle.default",
                "Default Subtitle",
                "subtitle",
                "subtitle_rule",
                "Create subtitle chunks from final narration text.",
                r#"{"type":"object","required":["chunks"],"properties":{"chunks":{"type":"array"}}}"#,
                r#"{"type":"object","properties":{"maxCharsPerLine":{"type":"integer"},"language":{"type":"string"}}}"#,
                "Convert narration into subtitle chunks. Return JSON only.",
            ),
        },
        BuiltinRuleSeed {
            module: "cover",
            file_name: "cover_default",
            content: builtin_rule(
                "cover.default",
                "Default Cover",
                "cover",
                "cover_rule",
                "Create cover title and image prompt suggestions.",
                r#"{"type":"object","required":["title","image_prompt"],"properties":{"title":{"type":"string"},"image_prompt":{"type":"string"}}}"#,
                r#"{"type":"object","properties":{"titleCount":{"type":"integer"},"style":{"type":"string"}}}"#,
                "Create a concise cover title and image prompt from project context. Return JSON only.",
            ),
        },
        BuiltinRuleSeed {
            module: "review",
            file_name: "review_safety",
            content: builtin_rule(
                "review.safety",
                "Safety Review",
                "review",
                "review_rule",
                "Review generated text for safety and missing inputs.",
                r#"{"type":"object","required":["passed","issues"],"properties":{"passed":{"type":"boolean"},"issues":{"type":"array","items":{"type":"string"}}}}"#,
                r#"{"type":"object","properties":{"strictness":{"type":"string"},"language":{"type":"string"}}}"#,
                "Review the provided generation result for missing fields and unsafe wording. Return JSON only.",
            ),
        },
    ]
}

fn builtin_rule(
    key: &'static str,
    name: &'static str,
    module: &'static str,
    rule_type: &'static str,
    description: &'static str,
    output_schema: &'static str,
    params_schema: &'static str,
    body: &'static str,
) -> &'static str {
    Box::leak(
        format!(
            "---\nkey: {key}\nname: {name}\nmodule: {module}\nrule_type: {rule_type}\nprovider_kind: llm\nversion: 1.0.0\noutput_schema: {output_schema}\nparams_schema: {params_schema}\ndescription: {description}\nsource_type: builtin\nenabled: true\n---\n\n{body}\n"
        )
        .into_boxed_str(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        clone_creative_rule_to_user, delete_user_creative_rule, get_creative_rule,
        list_creative_rules, project_rule_snapshot, save_user_creative_rule,
        set_user_creative_rule_enabled,
    };
    use crate::db::project_repository::ProjectRepository;
    use crate::db::Database;
    use crate::domain::project::CreateProjectRequest;
    use crate::domain::prompt::{
        CreativeRuleIdRequest, ListCreativeRulesRequest, SaveCreativeRuleRequest,
        SetCreativeRuleEnabledRequest,
    };
    use crate::domain::video_pack::UpsertUserVideoPackRequest;
    use crate::services::video_pack_service::upsert_user_video_pack;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn initializes_and_lists_builtin_rules() {
        let root = test_root("builtin");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let rules = list_creative_rules(
            &database,
            &root,
            ListCreativeRulesRequest {
                source_type: Some("builtin".to_string()),
                module: None,
            },
        )
        .expect("builtin rules should list");

        assert!(rules
            .iter()
            .any(|rule| rule.key == "script.topic_narration"));
        let script_rule = rules
            .iter()
            .find(|rule| rule.key == "script.topic_narration")
            .expect("script rule should exist");
        assert_eq!(script_rule.version, "1.0.0");
        assert!(script_rule.content_hash.starts_with("hash_"));
        assert!(script_rule.schema_hash.starts_with("hash_"));
        assert!(root
            .join("prompts/builtin/script/script_topic_narration.md")
            .is_file());

        cleanup(root);
    }

    #[test]
    fn builtin_rule_cannot_be_deleted_or_modified_directly() {
        let root = test_root("builtin_readonly");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let rule = get_creative_rule(
            &database,
            &root,
            CreativeRuleIdRequest {
                rule_id: "builtin:script:script_topic_narration".to_string(),
            },
        )
        .expect("builtin rule should load");

        assert_eq!(rule.source_type, "builtin");
        assert!(delete_user_creative_rule(
            &database,
            &root,
            CreativeRuleIdRequest {
                rule_id: rule.rule_id.clone(),
            },
        )
        .is_err());
        assert!(set_user_creative_rule_enabled(
            &database,
            &root,
            SetCreativeRuleEnabledRequest {
                rule_id: rule.rule_id,
                enabled: false,
            },
        )
        .is_err());

        cleanup(root);
    }

    #[test]
    fn clones_saves_enables_and_deletes_user_rule() {
        let root = test_root("user_lifecycle");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let cloned = clone_creative_rule_to_user(
            &database,
            &root,
            CreativeRuleIdRequest {
                rule_id: "builtin:image_prompt:image_prompt_shot_frame".to_string(),
            },
        )
        .expect("builtin should clone");
        assert_eq!(cloned.source_type, "user");
        assert!(!cloned.enabled);

        let saved = save_user_creative_rule(
            &database,
            &root,
            SaveCreativeRuleRequest {
                key: "user.image_prompt.custom".to_string(),
                name: "Custom image prompt".to_string(),
                module: "image_prompt".to_string(),
                rule_type: "image_prompt_rule".to_string(),
                provider_kind: "llm".to_string(),
                version: Some("1.2.0".to_string()),
                output_schema: json!({
                    "type": "object",
                    "properties": { "prompt": { "type": "string" } }
                }),
                params_schema: Some(json!({ "type": "object" })),
                description: "Custom editable rule".to_string(),
                enabled: false,
                body: "Return JSON only from {{storyboard.items}}.".to_string(),
            },
        )
        .expect("user rule should save");
        assert_eq!(saved.rule_id, "user:image_prompt:user_image_prompt_custom");
        assert_eq!(saved.version, "1.2.0");
        assert!(saved.content_hash.starts_with("hash_"));
        assert!(saved.schema_hash.starts_with("hash_"));

        let enabled = set_user_creative_rule_enabled(
            &database,
            &root,
            SetCreativeRuleEnabledRequest {
                rule_id: saved.rule_id.clone(),
                enabled: true,
            },
        )
        .expect("user rule should toggle");
        assert!(enabled.enabled);

        let deleted = delete_user_creative_rule(
            &database,
            &root,
            CreativeRuleIdRequest {
                rule_id: saved.rule_id,
            },
        )
        .expect("user rule should delete");
        assert_eq!(deleted.source_type, "user");

        cleanup(root);
    }

    #[test]
    fn rejects_invalid_schema_and_secret_like_body() {
        let root = test_root("rejects_invalid");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        assert!(save_user_creative_rule(
            &database,
            &root,
            SaveCreativeRuleRequest {
                key: "user.script.bad".to_string(),
                name: "Bad".to_string(),
                module: "script".to_string(),
                rule_type: "script_rule".to_string(),
                provider_kind: "llm".to_string(),
                version: None,
                output_schema: json!(["not", "object"]),
                params_schema: Some(json!({ "type": "object" })),
                description: "Bad schema".to_string(),
                enabled: false,
                body: "Return JSON only.".to_string(),
            },
        )
        .is_err());
        assert!(save_user_creative_rule(
            &database,
            &root,
            SaveCreativeRuleRequest {
                key: "user.script.secret".to_string(),
                name: "Secret".to_string(),
                module: "script".to_string(),
                rule_type: "script_rule".to_string(),
                provider_kind: "llm".to_string(),
                version: None,
                output_schema: json!({ "type": "object" }),
                params_schema: Some(json!({ "type": "object" })),
                description: "Secret body".to_string(),
                enabled: false,
                body: "api_key = sk-abcdefghijklmnopqrstuvwxyz012345".to_string(),
            },
        )
        .is_err());

        cleanup(root);
    }

    #[test]
    fn user_rule_referenced_by_video_pack_cannot_be_disabled_or_deleted() {
        let root = test_root("referenced_rule_blocked");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let rule = save_user_creative_rule(
            &database,
            &root,
            SaveCreativeRuleRequest {
                key: "user.image_prompt.blocked".to_string(),
                name: "Blocked Image Prompt".to_string(),
                module: "image_prompt".to_string(),
                rule_type: "image_prompt_rule".to_string(),
                provider_kind: "llm".to_string(),
                version: None,
                output_schema: json!({ "type": "object" }),
                params_schema: Some(json!({ "type": "object" })),
                description: "Referenced by a video pack".to_string(),
                enabled: true,
                body: "Return JSON only.".to_string(),
            },
        )
        .expect("user rule should save");

        upsert_user_video_pack(
            &database,
            &root,
            UpsertUserVideoPackRequest {
                pack_id: Some("pack_rule_blocked".to_string()),
                name: "Rule blocked pack".to_string(),
                description: "References a user rule".to_string(),
                applicable_input_types: vec!["topic".to_string()],
                content_category: Some("knowledge".to_string()),
                default_tone: None,
                default_aspect_ratio: "9:16".to_string(),
                default_duration_seconds: 30,
                default_scene_count: 5,
                rule_refs: json!({
                    "image_prompt": {
                        "ruleKey": rule.key,
                        "ruleId": rule.rule_id
                    }
                }),
                recommended_executable_refs: json!({}),
                asset_refs: json!([]),
                is_enabled: true,
            },
        )
        .expect("video pack should save");

        let loaded = get_creative_rule(
            &database,
            &root,
            CreativeRuleIdRequest {
                rule_id: "user:image_prompt:user_image_prompt_blocked".to_string(),
            },
        )
        .expect("rule should load");
        assert_eq!(loaded.reference_counts.video_packs, 1);
        assert!(set_user_creative_rule_enabled(
            &database,
            &root,
            SetCreativeRuleEnabledRequest {
                rule_id: loaded.rule_id.clone(),
                enabled: false,
            },
        )
        .is_err());
        assert!(delete_user_creative_rule(
            &database,
            &root,
            CreativeRuleIdRequest {
                rule_id: loaded.rule_id,
            },
        )
        .is_err());

        cleanup(root);
    }

    #[test]
    fn project_rule_snapshot_records_rule_versions_and_hashes_without_body() {
        let root = test_root("project_rule_snapshot");
        let database = Database::open(root.join("app.sqlite3")).expect("database should open");
        let rules = list_creative_rules(
            &database,
            &root,
            ListCreativeRulesRequest {
                source_type: Some("builtin".to_string()),
                module: Some("script".to_string()),
            },
        )
        .expect("rules should list");
        let script_rule = rules
            .iter()
            .find(|rule| rule.key == "script.topic_narration")
            .expect("script rule should exist");
        ProjectRepository::new(&database)
            .create_with_id(
                "project_rule_snapshot".to_string(),
                CreateProjectRequest {
                    title: "Rule snapshot".to_string(),
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
                    rule_refs: Some(json!({
                        "script": {
                            "ruleKey": script_rule.key,
                            "ruleId": script_rule.rule_id
                        }
                    })),
                    executable_refs: None,
                    input_process_mode: "generate".to_string(),
                    input_options: Some(json!({})),
                },
            )
            .expect("project should save");

        let snapshot = project_rule_snapshot(&database, &root, "project_rule_snapshot")
            .expect("snapshot should build");
        assert_eq!(
            snapshot["skillSnapshots"]["script"]["version"],
            script_rule.version
        );
        assert_eq!(
            snapshot["skillSnapshots"]["script"]["contentHash"],
            script_rule.content_hash
        );
        assert_eq!(
            snapshot["skillSnapshots"]["script"]["schemaHash"],
            script_rule.schema_hash
        );
        let serialized = serde_json::to_string(&snapshot).expect("snapshot should serialize");
        assert!(!serialized.contains("Create a concise narration plan"));

        cleanup(root);
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-prompt-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }
}
