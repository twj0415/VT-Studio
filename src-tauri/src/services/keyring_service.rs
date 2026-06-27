use keyring::Entry;
#[cfg(test)]
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;

const SERVICE_NAME: &str = "vt-ai-short-video-maker";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretHandle {
    pub provider_id: String,
    pub auth_type: String,
    pub key_alias: String,
    pub has_secret: bool,
}

pub trait KeyringBackend: Send + Sync {
    fn set_secret(&self, service: &str, account: &str, secret: &str) -> Result<(), String>;
    fn get_secret(&self, service: &str, account: &str) -> Result<Option<String>, String>;
    fn delete_secret(&self, service: &str, account: &str) -> Result<(), String>;
}

pub struct KeyringService {
    backend: Arc<dyn KeyringBackend>,
}

impl KeyringService {
    pub fn system() -> Self {
        Self {
            backend: Arc::new(SystemKeyringBackend),
        }
    }

    #[cfg(test)]
    pub(crate) fn memory() -> Self {
        Self {
            backend: Arc::new(MemoryKeyringBackend::default()),
        }
    }

    pub fn save_provider_secret(
        &self,
        provider_id: &str,
        auth_type: &str,
        key_alias: Option<&str>,
        secret: &str,
    ) -> Result<SecretHandle, String> {
        validate_provider_id(provider_id)?;
        validate_auth_type(auth_type)?;
        validate_secret(secret)?;
        let key_alias = normalize_key_alias(provider_id, auth_type, key_alias)?;
        let account = account_name(&key_alias);

        self.backend.set_secret(SERVICE_NAME, &account, secret)?;

        Ok(SecretHandle {
            provider_id: provider_id.to_string(),
            auth_type: auth_type.to_string(),
            key_alias,
            has_secret: true,
        })
    }

    pub fn read_provider_secret(&self, key_alias: &str) -> Result<Option<String>, String> {
        let key_alias = validate_key_alias(key_alias)?;
        self.backend
            .get_secret(SERVICE_NAME, &account_name(&key_alias))
    }

    pub fn delete_provider_secret(&self, key_alias: &str) -> Result<(), String> {
        let key_alias = validate_key_alias(key_alias)?;
        self.backend
            .delete_secret(SERVICE_NAME, &account_name(&key_alias))
    }

    pub fn has_provider_secret(&self, key_alias: &str) -> Result<bool, String> {
        self.read_provider_secret(key_alias)
            .map(|secret| secret.is_some())
    }
}

struct SystemKeyringBackend;

impl KeyringBackend for SystemKeyringBackend {
    fn set_secret(&self, service: &str, account: &str, secret: &str) -> Result<(), String> {
        Entry::new(service, account)
            .map_err(map_keyring_error)?
            .set_password(secret)
            .map_err(map_keyring_error)
    }

    fn get_secret(&self, service: &str, account: &str) -> Result<Option<String>, String> {
        match Entry::new(service, account)
            .map_err(map_keyring_error)?
            .get_password()
        {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(map_keyring_error(error)),
        }
    }

    fn delete_secret(&self, service: &str, account: &str) -> Result<(), String> {
        match Entry::new(service, account)
            .map_err(map_keyring_error)?
            .delete_credential()
        {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(map_keyring_error(error)),
        }
    }
}

#[cfg(test)]
#[derive(Default)]
struct MemoryKeyringBackend {
    secrets: Mutex<HashMap<String, String>>,
}

#[cfg(test)]
impl KeyringBackend for MemoryKeyringBackend {
    fn set_secret(&self, service: &str, account: &str, secret: &str) -> Result<(), String> {
        self.secrets
            .lock()
            .map_err(|_| "memory keyring lock poisoned.".to_string())?
            .insert(memory_key(service, account), secret.to_string());
        Ok(())
    }

    fn get_secret(&self, service: &str, account: &str) -> Result<Option<String>, String> {
        Ok(self
            .secrets
            .lock()
            .map_err(|_| "memory keyring lock poisoned.".to_string())?
            .get(&memory_key(service, account))
            .cloned())
    }

    fn delete_secret(&self, service: &str, account: &str) -> Result<(), String> {
        self.secrets
            .lock()
            .map_err(|_| "memory keyring lock poisoned.".to_string())?
            .remove(&memory_key(service, account));
        Ok(())
    }
}

fn normalize_key_alias(
    provider_id: &str,
    auth_type: &str,
    key_alias: Option<&str>,
) -> Result<String, String> {
    match key_alias {
        Some(alias) if !alias.trim().is_empty() => validate_key_alias(alias),
        _ => Ok(format!("{provider_id}:{auth_type}")),
    }
}

fn account_name(key_alias: &str) -> String {
    format!("provider:{key_alias}")
}

fn validate_provider_id(provider_id: &str) -> Result<(), String> {
    validate_identifier("provider_id", provider_id)
}

fn validate_key_alias(key_alias: &str) -> Result<String, String> {
    validate_identifier("key_alias", key_alias)?;
    Ok(key_alias.to_string())
}

fn validate_auth_type(auth_type: &str) -> Result<(), String> {
    match auth_type {
        "api_key" | "bearer_token" | "basic" | "custom_header" | "oauth" => Ok(()),
        "none" => Err("auth_type=none cannot store a secret.".to_string()),
        _ => Err("unsupported provider auth_type.".to_string()),
    }
}

fn validate_identifier(name: &str, value: &str) -> Result<(), String> {
    if value.trim() != value || value.is_empty() {
        return Err(format!("{name} cannot be empty or padded."));
    }

    if value.len() > 128 {
        return Err(format!("{name} is too long."));
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

fn validate_secret(secret: &str) -> Result<(), String> {
    if secret.trim() != secret || secret.is_empty() {
        return Err("secret cannot be empty or padded.".to_string());
    }

    if secret.len() < 8 {
        return Err("secret is too short.".to_string());
    }

    Ok(())
}

fn map_keyring_error(error: keyring::Error) -> String {
    match error {
        keyring::Error::NoEntry => "keyring entry was not found.".to_string(),
        _ => "keyring operation failed.".to_string(),
    }
}

#[cfg(test)]
fn memory_key(service: &str, account: &str) -> String {
    format!("{service}\n{account}")
}

#[cfg(test)]
mod tests {
    use super::KeyringService;
    use crate::db::provider_repository::{ProviderRecord, ProviderRepository};
    use crate::db::Database;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn stores_secret_only_in_keyring_backend() {
        let service = KeyringService::memory();
        let handle = service
            .save_provider_secret(
                "provider_deepseek",
                "api_key",
                Some("deepseek_main"),
                "sk-test-secret-123456",
            )
            .expect("secret should save");

        assert_eq!(handle.key_alias, "deepseek_main");
        assert!(handle.has_secret);
        assert_eq!(
            service
                .read_provider_secret("deepseek_main")
                .expect("secret should read")
                .as_deref(),
            Some("sk-test-secret-123456")
        );

        service
            .delete_provider_secret("deepseek_main")
            .expect("secret should delete");
        assert!(!service
            .has_provider_secret("deepseek_main")
            .expect("secret check should work"));
    }

    #[test]
    fn rejects_invalid_secret_inputs() {
        let service = KeyringService::memory();

        assert!(service
            .save_provider_secret("provider_a", "none", None, "secret-value")
            .is_err());
        assert!(service
            .save_provider_secret("provider a", "api_key", None, "secret-value")
            .is_err());
        assert!(service
            .save_provider_secret("provider_a", "api_key", None, "short")
            .is_err());
    }

    #[test]
    fn provider_repository_rejects_secret_like_config() {
        let path = test_database_path("provider_rejects_secret_config");
        let database = Database::open(&path).expect("database should open");
        let repository = ProviderRepository::new(&database);

        let result = repository.upsert_provider(&ProviderRecord {
            provider_id: "provider_secret".to_string(),
            vendor: "openai_compatible".to_string(),
            kind: "llm".to_string(),
            display_name: "Secret Provider".to_string(),
            auth_type: "api_key".to_string(),
            key_alias: Some("secret_alias".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            status: "disabled".to_string(),
            enabled: false,
            config_json: json!({ "api_key": "sk-abcdefghijklmnopqrstuvwxyz012345" }),
        });

        assert!(result.is_err());
        assert_eq!(
            repository
                .list_providers()
                .expect("providers should list")
                .len(),
            0
        );

        cleanup(path);
    }

    #[test]
    fn provider_repository_persists_only_key_alias() {
        let path = test_database_path("provider_key_alias_only");
        let database = Database::open(&path).expect("database should open");
        let repository = ProviderRepository::new(&database);
        let secret = "sk-abcdefghijklmnopqrstuvwxyz012345";

        repository
            .upsert_provider(&ProviderRecord {
                provider_id: "provider_alias".to_string(),
                vendor: "openai_compatible".to_string(),
                kind: "llm".to_string(),
                display_name: "Alias Provider".to_string(),
                auth_type: "api_key".to_string(),
                key_alias: Some("alias_main".to_string()),
                base_url: Some("https://api.example.com/v1".to_string()),
                status: "disabled".to_string(),
                enabled: false,
                config_json: json!({ "timeoutSeconds": 30 }),
            })
            .expect("provider should save");

        let providers = repository.list_providers().expect("providers should list");
        assert_eq!(providers[0].key_alias.as_deref(), Some("alias_main"));
        assert!(!format!("{providers:?}").contains(secret));

        let bytes = fs::read(&path).expect("database file should read");
        assert!(!String::from_utf8_lossy(&bytes).contains(secret));

        cleanup(path);
    }

    fn test_database_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-keyring-{name}-{}-{nanos}.sqlite3",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
    }
}
