use serde_json::Value;

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

pub fn find_json_secrets(value: &Value) -> Vec<SecretFinding> {
    let mut findings = Vec::new();
    scan_json(value, "$", &mut findings);
    findings
}

pub fn redact_text(value: &str) -> String {
    if looks_like_secret_value(value) {
        return "***REDACTED***".to_string();
    }

    value.to_string()
}

fn scan_json(value: &Value, path: &str, findings: &mut Vec<SecretFinding>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let child_path = format!("{path}.{key}");
                if is_secret_key(key) {
                    findings.push(SecretFinding {
                        path: child_path,
                        reason: "secret-like key name".to_string(),
                    });
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
        Value::String(text) if looks_like_secret_value(text) => findings.push(SecretFinding {
            path: path.to_string(),
            reason: "secret-like string value".to_string(),
        }),
        _ => {}
    }
}

fn is_secret_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    SECRET_KEYWORDS
        .iter()
        .any(|keyword| normalized.contains(keyword))
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

    let alpha_numeric_count = trimmed
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .count();

    alpha_numeric_count >= 24
        && trimmed
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_-.:/+=".contains(character))
}

#[cfg(test)]
mod tests {
    use super::{find_json_secrets, redact_text, reject_json_secrets};
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
            redact_text("Bearer abcdefghijklmnopqrstuvwxyz"),
            "***REDACTED***"
        );
        assert_eq!(redact_text("normal value"), "normal value");
    }
}
