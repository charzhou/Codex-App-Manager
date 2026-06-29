use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Local, TimeZone, Utc};
use serde::Serialize;
use serde_json::json;
use toml_edit::{value, DocumentMut, Item, Table};

use crate::app::paths;
use crate::errors::AppError;

const WARNING_TEXT: &str =
    "This preset does not force cli_auth_credentials_store = \"file\"; environments pinned to keyring auth may ignore auth.json.";

const CONFIG_SNIPPET: &str = r#"model_provider = "OpenAI"
model = "gpt-5.5"
review_model = "gpt-5.5"
model_reasoning_effort = "xhigh"
disable_response_storage = true
network_access = "enabled"
windows_wsl_setup_acknowledged = true

[model_providers.OpenAI]
name = "OpenAI"
base_url = "https://sub2api.tegical.com"
wire_api = "responses"
requires_openai_auth = true

[features]
goals = true
"#;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexConfigLogStep {
    pub step: String,
    pub message: String,
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexConfigBackupInfo {
    pub target_path: String,
    pub backup_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexCliPresetPreview {
    pub codex_home_path: String,
    pub config_toml_path: String,
    pub auth_json_path: String,
    pub backup_dir_path: String,
    pub config_snippet: String,
    pub auth_json_preview: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexCliPresetApplyResult {
    pub codex_home_path: String,
    pub config_toml_path: String,
    pub auth_json_path: String,
    pub backup_dir_path: String,
    pub backups: Vec<CodexConfigBackupInfo>,
    pub logs: Vec<CodexConfigLogStep>,
    pub warning: Option<String>,
}

fn codex_home_from_env() -> Option<PathBuf> {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(paths::codex_home_dir)
}

fn config_path_for_home(home: &Path) -> PathBuf {
    home.join("config.toml")
}

fn auth_path_for_home(home: &Path) -> PathBuf {
    home.join("auth.json")
}

fn backup_dir_for_home(home: &Path) -> PathBuf {
    home.join("backups")
}

fn masked_auth_preview() -> String {
    serde_json::to_string_pretty(&json!({
        "OPENAI_API_KEY": "sk-***"
    }))
    .unwrap_or_else(|_| "{\n  \"OPENAI_API_KEY\": \"sk-***\"\n}".to_string())
}

fn render_auth_json(api_key: &str) -> Result<String, AppError> {
    serde_json::to_string_pretty(&json!({
        "OPENAI_API_KEY": api_key
    }))
    .map_err(|e| AppError::Internal(format!("render auth.json: {e}")))
}

fn timestamp_string() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Utc.timestamp_opt(now as i64, 0)
        .single()
        .map(|dt| dt.with_timezone(&Local).format("%Y%m%d-%H%M%S").to_string())
        .unwrap_or_else(|| now.to_string())
}

fn copy_backup_if_exists(
    target: &Path,
    backup_dir: &Path,
) -> Result<Option<CodexConfigBackupInfo>, AppError> {
    if !target.exists() {
        return Ok(None);
    }

    fs::create_dir_all(backup_dir)
        .map_err(|e| AppError::Internal(format!("create backups dir: {e}")))?;
    let name = target
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::Internal("target file has no name".to_string()))?;
    let backup_path = backup_dir.join(format!("{name}.{}.bak", timestamp_string()));
    fs::copy(target, &backup_path)
        .map_err(|e| AppError::Internal(format!("backup {}: {e}", target.display())))?;
    Ok(Some(CodexConfigBackupInfo {
        target_path: target.display().to_string(),
        backup_path: backup_path.display().to_string(),
    }))
}

fn write_atomic_bytes(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Internal("target path has no parent".to_string()))?;
    fs::create_dir_all(parent)
        .map_err(|e| AppError::Internal(format!("create parent dir: {e}")))?;
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&tmp, bytes).map_err(|e| AppError::Internal(format!("write temp file: {e}")))?;
    fs::rename(&tmp, path).map_err(|e| AppError::Internal(format!("replace target file: {e}")))?;
    Ok(())
}

fn ensure_table<'a>(doc: &'a mut DocumentMut, key: &str) -> &'a mut Table {
    let root = doc.as_table_mut();
    let item = root.entry(key).or_insert(Item::Table(Table::new()));
    if !item.is_table() {
        *item = Item::Table(Table::new());
    }
    item.as_table_mut().expect("table must exist")
}

fn merge_fixed_preset(doc: &mut DocumentMut) {
    doc["model_provider"] = value("OpenAI");
    doc["model"] = value("gpt-5.5");
    doc["review_model"] = value("gpt-5.5");
    doc["model_reasoning_effort"] = value("xhigh");
    doc["disable_response_storage"] = value(true);
    doc["network_access"] = value("enabled");
    doc["windows_wsl_setup_acknowledged"] = value(true);

    let providers = ensure_table(doc, "model_providers");
    let openai_item = providers
        .entry("OpenAI")
        .or_insert(Item::Table(Table::new()));
    if !openai_item.is_table() {
        *openai_item = Item::Table(Table::new());
    }
    let openai = openai_item
        .as_table_mut()
        .expect("OpenAI provider table");
    openai["name"] = value("OpenAI");
    openai["base_url"] = value("https://sub2api.tegical.com");
    openai["wire_api"] = value("responses");
    openai["requires_openai_auth"] = value(true);

    let features = ensure_table(doc, "features");
    features["goals"] = value(true);
}

fn parse_or_empty_config(path: &Path) -> Result<DocumentMut, AppError> {
    if !path.exists() {
        return Ok(DocumentMut::new());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| AppError::Internal(format!("read config.toml: {e}")))?;
    raw.parse::<DocumentMut>()
        .map_err(|e| AppError::Internal(format!("parse config.toml: {e}")))
}

fn push_log(
    logs: &mut Vec<CodexConfigLogStep>,
    step: &str,
    message: &str,
    status: &str,
    detail: Option<String>,
) {
    logs.push(CodexConfigLogStep {
        step: step.to_string(),
        message: message.to_string(),
        status: status.to_string(),
        detail,
    });
}

pub fn preview_for_home(home: &Path) -> Result<CodexCliPresetPreview, AppError> {
    Ok(CodexCliPresetPreview {
        codex_home_path: home.display().to_string(),
        config_toml_path: config_path_for_home(home).display().to_string(),
        auth_json_path: auth_path_for_home(home).display().to_string(),
        backup_dir_path: backup_dir_for_home(home).display().to_string(),
        config_snippet: CONFIG_SNIPPET.to_string(),
        auth_json_preview: masked_auth_preview(),
        warning: WARNING_TEXT.to_string(),
    })
}

pub fn preview() -> Result<CodexCliPresetPreview, AppError> {
    let home = codex_home_from_env()
        .ok_or_else(|| AppError::Internal("Unable to resolve CODEX_HOME".to_string()))?;
    preview_for_home(&home)
}

pub fn apply_for_home(home: &Path, api_key: &str) -> Result<CodexCliPresetApplyResult, AppError> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(AppError::Internal("API key is required".to_string()));
    }

    let mut logs = Vec::new();
    let mut backups = Vec::new();
    let config_path = config_path_for_home(home);
    let auth_path = auth_path_for_home(home);
    let backup_dir = backup_dir_for_home(home);

    push_log(
        &mut logs,
        "resolve_codex_home",
        "Resolved Codex home.",
        "success",
        Some(home.display().to_string()),
    );
    fs::create_dir_all(home).map_err(|e| AppError::Internal(format!("create CODEX_HOME: {e}")))?;
    fs::create_dir_all(&backup_dir)
        .map_err(|e| AppError::Internal(format!("create backups dir: {e}")))?;
    push_log(
        &mut logs,
        "ensure_directories",
        "Ensured config directories exist.",
        "success",
        None,
    );

    if let Some(backup) = copy_backup_if_exists(&config_path, &backup_dir)? {
        push_log(
            &mut logs,
            "backup_config_toml",
            "Created config.toml backup.",
            "success",
            Some(backup.backup_path.clone()),
        );
        backups.push(backup);
    }
    if let Some(backup) = copy_backup_if_exists(&auth_path, &backup_dir)? {
        push_log(
            &mut logs,
            "backup_auth_json",
            "Created auth.json backup.",
            "success",
            Some(backup.backup_path.clone()),
        );
        backups.push(backup);
    }

    let mut doc = parse_or_empty_config(&config_path)?;
    push_log(
        &mut logs,
        "parse_config_toml",
        "Parsed config.toml.",
        "success",
        None,
    );
    merge_fixed_preset(&mut doc);
    push_log(
        &mut logs,
        "merge_config_toml",
        "Merged fixed preset into config.toml.",
        "success",
        None,
    );

    write_atomic_bytes(&config_path, doc.to_string().as_bytes())?;
    push_log(
        &mut logs,
        "write_config_toml",
        "Wrote config.toml.",
        "success",
        Some(config_path.display().to_string()),
    );

    let auth_json = render_auth_json(trimmed)?;
    push_log(
        &mut logs,
        "render_auth_json",
        "Rendered auth.json.",
        "success",
        None,
    );
    write_atomic_bytes(&auth_path, auth_json.as_bytes())?;
    push_log(
        &mut logs,
        "write_auth_json",
        "Wrote auth.json.",
        "success",
        Some(auth_path.display().to_string()),
    );
    push_log(
        &mut logs,
        "done",
        "Applied Codex CLI preset.",
        "success",
        None,
    );

    Ok(CodexCliPresetApplyResult {
        codex_home_path: home.display().to_string(),
        config_toml_path: config_path.display().to_string(),
        auth_json_path: auth_path.display().to_string(),
        backup_dir_path: backup_dir.display().to_string(),
        backups,
        logs,
        warning: Some(WARNING_TEXT.to_string()),
    })
}

pub fn apply(api_key: &str) -> Result<CodexCliPresetApplyResult, AppError> {
    let home = codex_home_from_env()
        .ok_or_else(|| AppError::Internal("Unable to resolve CODEX_HOME".to_string()))?;
    apply_for_home(&home, api_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

    fn temp_root(name: &str) -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("cam-{name}-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn preview_contains_expected_fixed_preset_values() {
        let home = temp_root("preview");
        let preview = preview_for_home(&home).expect("preview");
        assert!(preview.config_snippet.contains(r#"model_provider = "OpenAI""#));
        assert!(preview
            .config_snippet
            .contains(r#"base_url = "https://sub2api.tegical.com""#));
        assert!(preview.auth_json_preview.contains(r#""OPENAI_API_KEY": "sk-***""#));
    }

    #[test]
    fn apply_creates_config_and_auth_when_missing() {
        let home = temp_root("create");
        let result = apply_for_home(&home, "sk-test-key").expect("apply");
        assert!(home.join("config.toml").exists());
        assert!(home.join("auth.json").exists());
        assert!(result.backups.is_empty());
    }

    #[test]
    fn apply_merges_managed_keys_without_dropping_unrelated_settings() {
        let home = temp_root("merge");
        fs::write(
            home.join("config.toml"),
            r#"
model = "old-model"
approval_policy = "never"

[features]
hooks = true
"#,
        )
        .unwrap();

        apply_for_home(&home, "sk-test-key").expect("apply");
        let written = read(&home.join("config.toml"));

        assert!(written.contains(r#"model = "gpt-5.5""#));
        assert!(written.contains(r#"approval_policy = "never""#));
        assert!(written.contains(r#"hooks = true"#));
        assert!(written.contains(r#"goals = true"#));
    }

    #[test]
    fn apply_overwrites_auth_json_completely() {
        let home = temp_root("auth-overwrite");
        fs::write(home.join("auth.json"), r#"{ "old": "value" }"#).unwrap();

        apply_for_home(&home, "sk-test-key").expect("apply");
        let written = read(&home.join("auth.json"));

        assert_eq!(
            written.trim(),
            "{\n  \"OPENAI_API_KEY\": \"sk-test-key\"\n}"
        );
    }

    #[test]
    fn apply_creates_timestamped_backups_for_existing_files() {
        let home = temp_root("backups");
        fs::write(home.join("config.toml"), r#"model = "old""#).unwrap();
        fs::write(home.join("auth.json"), r#"{ "OPENAI_API_KEY": "old" }"#).unwrap();

        let result = apply_for_home(&home, "sk-test-key").expect("apply");

        assert_eq!(result.backups.len(), 2);
        for backup in result.backups {
            assert!(PathBuf::from(&backup.backup_path).exists());
            assert!(backup.backup_path.contains("/backups/") || backup.backup_path.contains("\\backups\\"));
        }
    }

    #[test]
    fn apply_rejects_empty_api_key() {
        let home = temp_root("empty-key");
        let err = apply_for_home(&home, "   ").expect_err("empty key should fail");
        assert!(err.to_string().contains("API key"));
    }
}
