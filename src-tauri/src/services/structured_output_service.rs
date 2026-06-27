use crate::domain::structured_output::{
    StructuredOutputValidationResult, ValidateStructuredOutputRequest,
};
use serde_json::Value;

pub fn validate_structured_output(
    request: ValidateStructuredOutputRequest,
) -> Result<StructuredOutputValidationResult, String> {
    if !request.output_schema.is_object() {
        return Err("output_schema must be a JSON object.".to_string());
    }

    let attempt_count = request.repair_attempt_count.unwrap_or(0);
    let max_attempts = request.max_repair_attempts.unwrap_or(2).min(2);
    let mut errors = Vec::new();
    let parsed_json = match parse_llm_json(&request.raw_output) {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(error);
            None
        }
    };

    if let Some(value) = parsed_json.as_ref() {
        validate_schema(value, &request.output_schema, "$", &mut errors);
        if let Some(expected_count) = request.expected_count {
            validate_expected_count(value, expected_count, &mut errors);
        }
    }

    let valid = errors.is_empty();
    let repair_needed = !valid && attempt_count < max_attempts;
    Ok(StructuredOutputValidationResult {
        valid,
        parsed_json,
        errors,
        repair_needed,
        attempt_count,
        max_attempts,
    })
}

fn parse_llm_json(raw: &str) -> Result<Value, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("output.empty: LLM output is empty.".to_string());
    }
    let candidate = extract_fenced_json(trimmed).unwrap_or(trimmed);
    serde_json::from_str::<Value>(candidate)
        .map_err(|error| format!("output.invalid_json: {error}"))
}

fn extract_fenced_json(raw: &str) -> Option<&str> {
    let content = raw.strip_prefix("```")?;
    let after_language = content
        .strip_prefix("json")
        .or_else(|| content.strip_prefix("JSON"))
        .unwrap_or(content);
    let after_newline = after_language.strip_prefix('\n')?;
    let end = after_newline.rfind("\n```")?;
    Some(after_newline[..end].trim())
}

fn validate_schema(value: &Value, schema: &Value, path: &str, errors: &mut Vec<String>) {
    if let Some(schema_type) = schema.get("type").and_then(Value::as_str) {
        if !matches_type(value, schema_type) {
            errors.push(format!("{path}: expected type {schema_type}"));
            return;
        }
    }

    if let Some(enum_values) = schema.get("enum").and_then(Value::as_array) {
        if !enum_values.iter().any(|item| item == value) {
            errors.push(format!("{path}: value is not in enum"));
        }
    }

    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        if let Some(object) = value.as_object() {
            for field in required.iter().filter_map(Value::as_str) {
                if !object.contains_key(field) {
                    errors.push(format!("{path}.{field}: required field missing"));
                }
            }
        }
    }

    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        if let Some(object) = value.as_object() {
            for (field, child_schema) in properties {
                if let Some(child) = object.get(field) {
                    validate_schema(child, child_schema, &format!("{path}.{field}"), errors);
                }
            }
        }
    }

    if let Some(items_schema) = schema.get("items") {
        if let Some(items) = value.as_array() {
            for (index, item) in items.iter().enumerate() {
                validate_schema(item, items_schema, &format!("{path}[{index}]"), errors);
            }
        }
    }

    if let Some(items) = value.as_array() {
        if let Some(min_items) = schema.get("minItems").and_then(Value::as_u64) {
            if items.len() < min_items as usize {
                errors.push(format!("{path}: array length is less than minItems"));
            }
        }
        if let Some(max_items) = schema.get("maxItems").and_then(Value::as_u64) {
            if items.len() > max_items as usize {
                errors.push(format!("{path}: array length exceeds maxItems"));
            }
        }
    }
}

fn validate_expected_count(value: &Value, expected_count: usize, errors: &mut Vec<String>) {
    let count = if let Some(items) = value.as_array() {
        Some(items.len())
    } else if let Some(items) = value.get("items").and_then(Value::as_array) {
        Some(items.len())
    } else if let Some(items) = value.get("prompts").and_then(Value::as_array) {
        Some(items.len())
    } else if let Some(items) = value.get("narrations").and_then(Value::as_array) {
        Some(items.len())
    } else {
        None
    };

    match count {
        Some(actual) if actual == expected_count => {}
        Some(actual) => errors.push(format!(
            "$: expected {expected_count} items but received {actual}"
        )),
        None => errors.push("$.expected_count: no countable array found".to_string()),
    }
}

fn matches_type(value: &Value, schema_type: &str) -> bool {
    match schema_type {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::validate_structured_output;
    use crate::domain::structured_output::ValidateStructuredOutputRequest;
    use serde_json::json;

    #[test]
    fn accepts_fenced_json_when_schema_matches() {
        let result = validate_structured_output(ValidateStructuredOutputRequest {
            raw_output: "```json\n{\"items\":[{\"text\":\"a\"}]}\n```".to_string(),
            output_schema: json!({
                "type": "object",
                "required": ["items"],
                "properties": {
                    "items": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["text"],
                            "properties": { "text": { "type": "string" } }
                        }
                    }
                }
            }),
            expected_count: Some(1),
            repair_attempt_count: Some(0),
            max_repair_attempts: Some(2),
        })
        .expect("validation should run");

        assert!(result.valid);
        assert!(!result.repair_needed);
    }

    #[test]
    fn rejects_non_json_and_marks_repair_until_limit() {
        let result = validate_structured_output(ValidateStructuredOutputRequest {
            raw_output: "plain text".to_string(),
            output_schema: json!({ "type": "object" }),
            expected_count: None,
            repair_attempt_count: Some(1),
            max_repair_attempts: Some(2),
        })
        .expect("validation should run");

        assert!(!result.valid);
        assert!(result.repair_needed);
        assert!(result.parsed_json.is_none());

        let exhausted = validate_structured_output(ValidateStructuredOutputRequest {
            raw_output: "plain text".to_string(),
            output_schema: json!({ "type": "object" }),
            expected_count: None,
            repair_attempt_count: Some(2),
            max_repair_attempts: Some(2),
        })
        .expect("validation should run");
        assert!(!exhausted.repair_needed);
    }

    #[test]
    fn reports_required_enum_items_and_count_errors() {
        let result = validate_structured_output(ValidateStructuredOutputRequest {
            raw_output: json!({
                "items": [
                    { "kind": "bad" },
                    { "kind": "ok", "text": "b" }
                ]
            })
            .to_string(),
            output_schema: json!({
                "type": "object",
                "required": ["items"],
                "properties": {
                    "items": {
                        "type": "array",
                        "minItems": 1,
                        "maxItems": 3,
                        "items": {
                            "type": "object",
                            "required": ["text", "kind"],
                            "properties": {
                                "text": { "type": "string" },
                                "kind": { "type": "string", "enum": ["ok"] }
                            }
                        }
                    }
                }
            }),
            expected_count: Some(3),
            repair_attempt_count: Some(0),
            max_repair_attempts: Some(2),
        })
        .expect("validation should run");

        assert!(!result.valid);
        assert!(result.repair_needed);
        assert!(result
            .errors
            .iter()
            .any(|error| error.contains("$.items[0].text")));
        assert!(result
            .errors
            .iter()
            .any(|error| error.contains("value is not in enum")));
        assert!(result
            .errors
            .iter()
            .any(|error| error.contains("expected 3 items")));
    }

    #[test]
    fn reports_array_item_min_and_max_errors() {
        let result = validate_structured_output(ValidateStructuredOutputRequest {
            raw_output: json!([1, "bad", 3]).to_string(),
            output_schema: json!({
                "type": "array",
                "minItems": 4,
                "maxItems": 2,
                "items": { "type": "integer" }
            }),
            expected_count: None,
            repair_attempt_count: Some(0),
            max_repair_attempts: Some(2),
        })
        .expect("validation should run");

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|error| error.contains("$[1]: expected type integer")));
        assert!(result
            .errors
            .iter()
            .any(|error| error.contains("less than minItems")));
        assert!(result
            .errors
            .iter()
            .any(|error| error.contains("exceeds maxItems")));
    }

    #[test]
    fn caps_repair_attempts_at_two() {
        let result = validate_structured_output(ValidateStructuredOutputRequest {
            raw_output: "plain text".to_string(),
            output_schema: json!({ "type": "object" }),
            expected_count: None,
            repair_attempt_count: Some(2),
            max_repair_attempts: Some(5),
        })
        .expect("validation should run");

        assert_eq!(result.max_attempts, 2);
        assert!(!result.repair_needed);
    }
}
