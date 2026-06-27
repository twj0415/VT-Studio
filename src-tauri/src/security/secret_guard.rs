use serde_json::Value;

const REDACTION: &str = "***REDACTED***";

const SECRET_KEYWORDS: [&str; 13] = [
    "api_key",
    "apikey",
    "api-key",
    "secret",
    "token",
    "access_token",
    "refresh_token",
    "authorization",
    "bearer",
    "password",
    "credential",
    "private_key",
    "client_secret",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretFinding {
    pub path: String,
    pub reason: String,
}

pub struct SecretScanInput<'a> {
    pub name: &'a str,
    pub content: &'a str,
}

pub fn reject_json_secrets(value: &Value) -> Result<(), String> {
    let findings = find_json_secrets(value);
    if findings.is_empty() {
        return Ok(());
    }

    let paths = findings
        .iter()
        .map(|finding| finding.path.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!("secret-like fields are not allowed here: {paths}"))
}

#[allow(dead_code)]
pub fn reject_text_secrets(context: &str, value: &str) -> Result<(), String> {
    let findings = find_text_secrets(context, value);
    if findings.is_empty() {
        return Ok(());
    }

    let paths = findings
        .iter()
        .map(|finding| finding.path.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!("secret-like text is not allowed here: {paths}"))
}

#[allow(dead_code)]
pub fn reject_secret_scan(inputs: &[SecretScanInput<'_>]) -> Result<(), String> {
    let findings = inputs
        .iter()
        .flat_map(|input| find_text_secrets(input.name, input.content))
        .collect::<Vec<_>>();

    if findings.is_empty() {
        return Ok(());
    }

    let paths = findings
        .iter()
        .map(|finding| finding.path.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "secret-like content was found during export or diagnostics scan: {paths}"
    ))
}

pub fn find_json_secrets(value: &Value) -> Vec<SecretFinding> {
    let mut findings = Vec::new();
    scan_json(value, "$", &mut findings);
    findings
}

#[allow(dead_code)]
pub fn redact_json(value: &Value) -> Value {
    redact_json_value(value, "$", None)
}

pub fn redact_text(value: &str) -> String {
    let mut redacted = value.to_string();
    let mut tokens = collect_secret_tokens(value);
    tokens.sort_by_key(|token| std::cmp::Reverse(token.len()));
    tokens.dedup();

    for token in tokens {
        if token.is_empty() {
            continue;
        }

        redacted = redacted.replace(&token, REDACTION);
    }

    redacted
}

#[allow(dead_code)]
pub fn redact_header_value(header_name: &str, header_value: &str) -> String {
    if is_secret_key(header_name) {
        return REDACTION.to_string();
    }

    redact_text(header_value)
}

#[allow(dead_code)]
pub fn find_text_secrets(context: &str, value: &str) -> Vec<SecretFinding> {
    collect_secret_tokens(value)
        .into_iter()
        .enumerate()
        .map(|(index, _)| SecretFinding {
            path: format!("{context}#secret[{index}]"),
            reason: "secret-like text".to_string(),
        })
        .collect()
}

fn scan_json(value: &Value, path: &str, findings: &mut Vec<SecretFinding>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{path}.{key}");
                if is_secret_key(key) {
                    if secret_key_value_requires_block(child) {
                        findings.push(SecretFinding {
                            path: child_path,
                            reason: "secret-like key name".to_string(),
                        });
                    }
                    continue;
                }

                scan_json(child, &child_path, findings);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                scan_json(child, &format!("{path}[{index}]"), findings);
            }
        }
        Value::String(text) => {
            let nested_findings = find_text_secrets(path, text);
            if !nested_findings.is_empty() {
                findings.extend(nested_findings);
            } else if looks_like_secret_value(text) && !looks_like_safe_identifier_value(text) {
                findings.push(SecretFinding {
                    path: path.to_string(),
                    reason: "secret-like string value".to_string(),
                });
            }
        }
        _ => {}
    }
}

fn redact_json_value(value: &Value, path: &str, parent_key: Option<&str>) -> Value {
    if parent_key.is_some_and(is_secret_key) {
        return Value::String(REDACTION.to_string());
    }

    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, child)| {
                    (
                        key.clone(),
                        redact_json_value(child, &format!("{path}.{key}"), Some(key)),
                    )
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .enumerate()
                .map(|(index, child)| redact_json_value(child, &format!("{path}[{index}]"), None))
                .collect(),
        ),
        Value::String(text) if looks_like_secret_value(text) => {
            Value::String(REDACTION.to_string())
        }
        _ => value.clone(),
    }
}

fn is_secret_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    SECRET_KEYWORDS
        .iter()
        .any(|keyword| normalized.contains(keyword))
}

fn secret_key_value_requires_block(value: &Value) -> bool {
    match value {
        Value::Null | Value::Bool(_) => false,
        Value::String(text) => !text.trim().is_empty() && !looks_like_safe_identifier_value(text),
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => !map.is_empty(),
        Value::Number(_) => true,
    }
}

fn looks_like_safe_identifier_value(value: &str) -> bool {
    let trimmed = value.trim();
    let lowered = trimmed.to_ascii_lowercase();
    if lowered.starts_with("sk-")
        || lowered.starts_with("rk-")
        || lowered.starts_with("bearer ")
        || lowered.starts_with("basic ")
    {
        return false;
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains(':') {
        return false;
    }
    let has_separator = trimmed.contains('_') || trimmed.contains('-') || trimmed.contains('.');
    has_separator
        && trimmed.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

fn looks_like_secret_value(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.len() < 16 {
        return false;
    }

    let lowered = trimmed.to_ascii_lowercase();
    if lowered.starts_with("bearer ") || lowered.starts_with("basic ") {
        return true;
    }

    if lowered.starts_with("http://") || lowered.starts_with("https://") {
        return lowered.contains("api_key=")
            || lowered.contains("apikey=")
            || lowered.contains("access_token=")
            || lowered.contains("token=");
    }

    if lowered.starts_with("sk-") || lowered.starts_with("rk-") {
        return trimmed.len() >= 20;
    }

    let alpha_numeric_count = trimmed
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .count();

    alpha_numeric_count >= 32
        && trimmed
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_-.+=".contains(character))
}

fn collect_secret_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();

    for line in value.lines() {
        if let Some((key, secret_value)) = split_secret_assignment(line) {
            if is_secret_key(key) {
                let secret_value = trim_secret_token(secret_value);
                if looks_like_secret_value(secret_value) {
                    tokens.push(secret_value.to_string());
                }
            }
        }

        let line_tokens = tokenize_line(line);
        for (index, token) in line_tokens.iter().enumerate() {
            let lowered = token.to_ascii_lowercase();
            if matches!(lowered.as_str(), "bearer" | "basic") {
                if let Some(next) = line_tokens.get(index + 1) {
                    tokens.push((*next).to_string());
                }
            }

            if looks_like_secret_value(token) && !looks_like_safe_identifier_value(token) {
                tokens.push((*token).to_string());
            }
        }
    }

    tokens.sort();
    tokens.dedup();
    tokens
}

fn split_secret_assignment(line: &str) -> Option<(&str, &str)> {
    let colon = line.find(':');
    let equals = line.find('=');
    let split_at = match (colon, equals) {
        (Some(left), Some(right)) => left.min(right),
        (Some(index), None) | (None, Some(index)) => index,
        (None, None) => return None,
    };

    let (key, value) = line.split_at(split_at);
    Some((key.trim(), value[1..].trim()))
}

fn tokenize_line(line: &str) -> Vec<&str> {
    line.split(|character: char| {
        character.is_whitespace()
            || matches!(
                character,
                '"' | '\'' | ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}'
            )
    })
    .map(trim_secret_token)
    .filter(|token| !token.is_empty())
    .collect()
}

fn trim_secret_token(value: &str) -> &str {
    value.trim_matches(|character: char| {
        character.is_whitespace()
            || matches!(
                character,
                '"' | '\'' | ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}'
            )
    })
}

#[cfg(test)]
mod tests {
    use super::{
        find_json_secrets, find_text_secrets, redact_header_value, redact_json, redact_text,
        reject_json_secrets, reject_secret_scan, reject_text_secrets, SecretScanInput,
    };
    use serde_json::json;

    #[test]
    fn detects_secret_keys_inside_json() {
        let findings = find_json_secrets(&json!({
            "headers": {
                "Authorization": "Bearer sk-live-secret"
            }
        }));

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].path, "$.headers.Authorization");
    }

    #[test]
    fn rejects_secret_like_values_inside_json() {
        assert!(reject_json_secrets(&json!({
            "baseUrl": "https://api.example.com",
            "opaque": "sk-abcdefghijklmnopqrstuvwxyz012345"
        }))
        .is_err());
    }

    #[test]
    fn redacts_secret_like_text() {
        assert_eq!(
            redact_text("Authorization: Bearer abcdefghijklmnopqrstuvwxyz"),
            "Authorization: ***REDACTED***"
        );
        assert_eq!(redact_text("normal value"), "normal value");
    }

    #[test]
    fn detects_and_rejects_secret_like_log_text() {
        let log = "request failed\nAuthorization: Bearer sk-live-secret-token-123456";
        let findings = find_text_secrets("provider.log", log);

        assert_eq!(findings.len(), 2);
        assert!(reject_text_secrets("provider.log", log).is_err());
    }

    #[test]
    fn redacts_secret_headers() {
        assert_eq!(
            redact_header_value("Authorization", "Bearer sk-live-secret-token-123456"),
            "***REDACTED***"
        );
        assert_eq!(redact_header_value("X-Trace-Id", "normal"), "normal");
    }

    #[test]
    fn redacts_json_secret_fields_and_values() {
        let redacted = redact_json(&json!({
            "headers": {
                "Authorization": "Bearer sk-live-secret-token-123456"
            },
            "opaque": "sk-abcdefghijklmnopqrstuvwxyz012345",
            "baseUrl": "https://api.example.com"
        }));

        assert_eq!(redacted["headers"]["Authorization"], "***REDACTED***");
        assert_eq!(redacted["opaque"], "***REDACTED***");
        assert_eq!(redacted["baseUrl"], "https://api.example.com");
    }

    #[test]
    fn export_scan_blocks_secret_content() {
        assert!(reject_secret_scan(&[SecretScanInput {
            name: "diagnostics/provider.log",
            content: "api_key = sk-abcdefghijklmnopqrstuvwxyz012345",
        }])
        .is_err());
        assert!(reject_secret_scan(&[SecretScanInput {
            name: "manifest.json",
            content: "{\"key_alias\":\"deepseek_main\"}",
        }])
        .is_ok());
    }
}
