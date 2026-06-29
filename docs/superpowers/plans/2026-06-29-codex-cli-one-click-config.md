# Codex CLI One-Click Config Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-platform Codex CLI one-click config feature that writes a fixed global preset into `~/.codex/config.toml` via TOML merge and overwrites `~/.codex/auth.json` from a user-supplied API key, with timestamped backups, preview, and collapsible write logs.

**Architecture:** Add a dedicated Rust backend module for Codex-home path resolution, timestamped backups, TOML merge via `toml_edit`, and structured step logs, then expose preview/apply commands to a new frontend Codex Config page. Keep this feature isolated from the manager's own settings/provenance config system and do not generalize it into a multi-provider editor.

**Tech Stack:** React 19, TypeScript, Vitest, Tauri v2, Rust, `serde`, `serde_json`, `toml_edit`

---

## File Structure

- Create: `src-tauri/src/app/codex_cli_config.rs`
  Dedicated backend module for Codex CLI preset preview/apply logic, path resolution, timestamped backups, TOML merge, `auth.json` rendering, and structured write logs.
- Modify: `src-tauri/src/app/mod.rs`
  Export the new backend module.
- Modify: `src-tauri/src/app/paths.rs`
  Add Codex CLI file path helpers for `config.toml`, `auth.json`, and `backups/`.
- Modify: `src-tauri/src/commands.rs`
  Add Tauri commands for previewing and applying the Codex CLI preset.
- Modify: `src-tauri/src/lib.rs`
  Register the new commands.
- Modify: `src-tauri/Cargo.toml`
  Add `toml_edit`.
- Modify: `src/shared/types.ts`
  Add frontend/backend payload types for preview, apply results, backup info, and structured log steps.
- Modify: `src/services/managerApi.ts`
  Add preview/apply client methods and browser-safe fallbacks.
- Modify: `src/services/managerApi.test.ts`
  Add tests for the new API methods.
- Modify: `src/app/views/CodexConfig.tsx`
  Replace the placeholder page with the working one-click config UI.
- Modify: `src/app/i18n.tsx`
  Add strings for intro copy, warnings, field labels, preview, logs, success, and failure states.
- Create: `src/app/views/CodexConfig.test.tsx`
  Frontend tests for disabled state, expand/collapse behavior, success rendering, and failure rendering.

### Task 1: Add shared types for Codex CLI preset preview and apply results

**Files:**
- Modify: `src/shared/types.ts`
- Test: `src/services/managerApi.test.ts`

- [ ] **Step 1: Add the new shared TypeScript interfaces**

Add the following block near the other shared result types in `src/shared/types.ts`:

```ts
export type CodexConfigLogStatus = "pending" | "running" | "success" | "failure";

export interface CodexConfigLogStep {
  step: string;
  message: string;
  status: CodexConfigLogStatus;
  detail: string | null;
}

export interface CodexConfigBackupInfo {
  targetPath: string;
  backupPath: string;
}

export interface CodexCliPresetPreview {
  codexHomePath: string;
  configTomlPath: string;
  authJsonPath: string;
  backupDirPath: string;
  configSnippet: string;
  authJsonPreview: string;
  warning: string;
}

export interface CodexCliPresetApplyResult {
  codexHomePath: string;
  configTomlPath: string;
  authJsonPath: string;
  backupDirPath: string;
  backups: CodexConfigBackupInfo[];
  logs: CodexConfigLogStep[];
  warning: string | null;
}
```

- [ ] **Step 2: Run TypeScript type-check to verify the new interfaces compile**

Run:

```bash
npm run check
```

Expected: TypeScript completes successfully with exit code `0`.

- [ ] **Step 3: Commit the shared type additions**

```bash
git add src/shared/types.ts
git commit -m "feat: add codex cli config shared result types"
```

### Task 2: Add manager API methods and tests for preview/apply

**Files:**
- Modify: `src/services/managerApi.ts`
- Modify: `src/services/managerApi.test.ts`
- Modify: `src/shared/types.ts`

- [ ] **Step 1: Add imports for the new shared types**

Update the `src/services/managerApi.ts` type import block to include:

```ts
  CodexCliPresetApplyResult,
  CodexCliPresetPreview,
```

- [ ] **Step 2: Add browser fallbacks and Tauri invoke methods**

Insert these methods into the exported `managerApi` object in `src/services/managerApi.ts` near `openCodexHome()`:

```ts
  getCodexCliPresetPreview(): Promise<CodexCliPresetPreview> {
    if (!hasTauriRuntime()) {
      return Promise.resolve({
        codexHomePath: "~/.codex",
        configTomlPath: "~/.codex/config.toml",
        authJsonPath: "~/.codex/auth.json",
        backupDirPath: "~/.codex/backups",
        configSnippet: [
          'model_provider = "OpenAI"',
          'model = "gpt-5.5"',
          'review_model = "gpt-5.5"',
          'model_reasoning_effort = "xhigh"',
          "disable_response_storage = true",
          'network_access = "enabled"',
          "windows_wsl_setup_acknowledged = true",
          "",
          "[model_providers.OpenAI]",
          'name = "OpenAI"',
          'base_url = "https://sub2api.tegical.com"',
          'wire_api = "responses"',
          "requires_openai_auth = true",
          "",
          "[features]",
          "goals = true",
        ].join("\n"),
        authJsonPreview: JSON.stringify({ OPENAI_API_KEY: "sk-***" }, null, 2),
        warning:
          'This preset does not force cli_auth_credentials_store = "file"; environments pinned to keyring auth may ignore auth.json.',
      });
    }
    return invoke<CodexCliPresetPreview>("preview_codex_cli_preset");
  },
  applyCodexCliPreset(apiKey: string): Promise<CodexCliPresetApplyResult> {
    const trimmed = apiKey.trim();
    if (!trimmed) {
      return Promise.reject(new Error("API key is required"));
    }
    if (!hasTauriRuntime()) {
      return Promise.resolve({
        codexHomePath: "~/.codex",
        configTomlPath: "~/.codex/config.toml",
        authJsonPath: "~/.codex/auth.json",
        backupDirPath: "~/.codex/backups",
        backups: [],
        logs: [
          {
            step: "done",
            message: "Browser preview mode did not write files.",
            status: "success",
            detail: null,
          },
        ],
        warning:
          'This preset does not force cli_auth_credentials_store = "file"; environments pinned to keyring auth may ignore auth.json.',
      });
    }
    return invoke<CodexCliPresetApplyResult>("apply_codex_cli_preset", { apiKey: trimmed });
  },
```

- [ ] **Step 3: Add failing tests for the new manager API methods**

Append this block to `src/services/managerApi.test.ts`:

```ts
describe("codex cli config API", () => {
  it("returns browser preview fallback without invoking Tauri", async () => {
    const preview = await managerApi.getCodexCliPresetPreview();

    expect(preview.configTomlPath).toContain(".codex/config.toml");
    expect(preview.authJsonPath).toContain(".codex/auth.json");
    expect(preview.configSnippet).toContain('model_provider = "OpenAI"');
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("rejects empty API keys before invoking Tauri", async () => {
    await expect(managerApi.applyCodexCliPreset("   ")).rejects.toThrow("API key is required");
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("invokes preview and apply commands inside Tauri", async () => {
    window.__TAURI_INTERNALS__ = {};
    const preview = {
      codexHomePath: "/Users/demo/.codex",
      configTomlPath: "/Users/demo/.codex/config.toml",
      authJsonPath: "/Users/demo/.codex/auth.json",
      backupDirPath: "/Users/demo/.codex/backups",
      configSnippet: 'model_provider = "OpenAI"',
      authJsonPreview: '{\n  "OPENAI_API_KEY": "sk-***"\n}',
      warning: "warning",
    };
    const result = {
      codexHomePath: "/Users/demo/.codex",
      configTomlPath: "/Users/demo/.codex/config.toml",
      authJsonPath: "/Users/demo/.codex/auth.json",
      backupDirPath: "/Users/demo/.codex/backups",
      backups: [
        {
          targetPath: "/Users/demo/.codex/config.toml",
          backupPath: "/Users/demo/.codex/backups/config.toml.20260629-190000.bak",
        },
      ],
      logs: [
        {
          step: "done",
          message: "Applied Codex CLI preset.",
          status: "success",
          detail: null,
        },
      ],
      warning: null,
    };

    invokeMock.mockResolvedValueOnce(preview).mockResolvedValueOnce(result);

    await expect(managerApi.getCodexCliPresetPreview()).resolves.toEqual(preview);
    await expect(managerApi.applyCodexCliPreset("sk-test")).resolves.toEqual(result);

    expect(invokeMock).toHaveBeenNthCalledWith(1, "preview_codex_cli_preset");
    expect(invokeMock).toHaveBeenNthCalledWith(2, "apply_codex_cli_preset", {
      apiKey: "sk-test",
    });
  });
});
```

- [ ] **Step 4: Run the targeted manager API tests and confirm they pass**

Run:

```bash
npm test -- src/services/managerApi.test.ts
```

Expected: `PASS` for the new `codex cli config API` tests and the existing test file.

- [ ] **Step 5: Commit the manager API slice**

```bash
git add src/services/managerApi.ts src/services/managerApi.test.ts src/shared/types.ts
git commit -m "feat: add codex cli config manager API"
```

### Task 3: Add Codex CLI config backend path helpers and TOML dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/app/paths.rs`

- [ ] **Step 1: Add `toml_edit` to the Rust dependencies**

Update the `[dependencies]` section in `src-tauri/Cargo.toml` with:

```toml
toml_edit = "0.22"
```

- [ ] **Step 2: Add Codex CLI config path helpers**

Append the following helpers to `src-tauri/src/app/paths.rs`:

```rust
pub fn codex_config_toml_path() -> Option<PathBuf> {
    codex_home_dir().map(|dir| dir.join("config.toml"))
}

pub fn codex_auth_json_path() -> Option<PathBuf> {
    codex_home_dir().map(|dir| dir.join("auth.json"))
}

pub fn codex_backups_dir() -> Option<PathBuf> {
    codex_home_dir().map(|dir| dir.join("backups"))
}
```

- [ ] **Step 3: Run cargo check to verify the dependency and helpers compile**

Run:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: Cargo resolves `toml_edit` successfully and exits with code `0`.

- [ ] **Step 4: Commit the backend path/dependency groundwork**

```bash
git add src-tauri/Cargo.toml src-tauri/src/app/paths.rs
git commit -m "feat: add codex cli config backend dependencies"
```

### Task 4: Add failing backend tests for preview, backups, merge, and auth overwrite

**Files:**
- Create: `src-tauri/src/app/codex_cli_config.rs`
- Modify: `src-tauri/src/app/mod.rs`

- [ ] **Step 1: Create the backend module skeleton and exported types**

Create `src-tauri/src/app/codex_cli_config.rs` with this initial skeleton:

```rust
use serde::Serialize;

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
```

- [ ] **Step 2: Export the new module**

Add this line to `src-tauri/src/app/mod.rs`:

```rust
pub mod codex_cli_config;
```

- [ ] **Step 3: Add failing backend tests that describe the required behavior**

Append this test module to `src-tauri/src/app/codex_cli_config.rs`:

```rust
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
        assert!(preview.config_snippet.contains(r#"base_url = "https://sub2api.tegical.com""#));
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
```

- [ ] **Step 4: Run cargo test and confirm the new backend tests fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml codex_cli_config -- --nocapture
```

Expected: FAIL with missing functions like `preview_for_home` / `apply_for_home`.

- [ ] **Step 5: Commit the failing backend tests**

```bash
git add src-tauri/src/app/codex_cli_config.rs src-tauri/src/app/mod.rs
git commit -m "test: add failing codex cli config backend tests"
```

### Task 5: Implement backend preview, timestamped backups, auth rendering, and TOML merge

**Files:**
- Modify: `src-tauri/src/app/codex_cli_config.rs`
- Modify: `src-tauri/src/app/paths.rs`

- [ ] **Step 1: Add the fixed preset constants and warning text**

At the top of `src-tauri/src/app/codex_cli_config.rs`, add:

```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
```

- [ ] **Step 2: Implement path resolution, preview rendering, and auth rendering helpers**

Add these functions to `src-tauri/src/app/codex_cli_config.rs`:

```rust
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
    .unwrap()
}

fn render_auth_json(api_key: &str) -> Result<String, AppError> {
    serde_json::to_string_pretty(&json!({
        "OPENAI_API_KEY": api_key
    }))
    .map_err(|e| AppError::Internal(format!("render auth.json: {e}")))
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
```

- [ ] **Step 3: Implement timestamped backups and atomic write helpers**

Add these helpers to `src-tauri/src/app/codex_cli_config.rs`:

```rust
fn timestamp_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let tm = chrono::DateTime::<chrono::Utc>::from_timestamp(now as i64, 0)
        .unwrap()
        .with_timezone(&chrono::Local);
    tm.format("%Y%m%d-%H%M%S").to_string()
}

fn copy_backup_if_exists(target: &Path, backup_dir: &Path) -> Result<Option<CodexConfigBackupInfo>, AppError> {
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
```

- [ ] **Step 4: Add the missing chrono dependency required by the timestamp helper**

Update `src-tauri/Cargo.toml` to include:

```toml
chrono = { version = "0.4", default-features = false, features = ["clock"] }
```

- [ ] **Step 5: Implement TOML merge and apply flow**

Add the following functions to `src-tauri/src/app/codex_cli_config.rs`:

```rust
fn ensure_table<'a>(doc: &'a mut DocumentMut, key: &str) -> &'a mut Table {
    if !doc[key].is_table() {
        doc[key] = Item::Table(Table::new());
    }
    doc[key].as_table_mut().unwrap()
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
    if !providers["OpenAI"].is_table() {
        providers["OpenAI"] = Item::Table(Table::new());
    }
    let openai = providers["OpenAI"].as_table_mut().unwrap();
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

fn push_log(logs: &mut Vec<CodexConfigLogStep>, step: &str, message: &str, status: &str, detail: Option<String>) {
    logs.push(CodexConfigLogStep {
        step: step.to_string(),
        message: message.to_string(),
        status: status.to_string(),
        detail,
    });
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

    push_log(&mut logs, "resolve_codex_home", "Resolved Codex home.", "success", Some(home.display().to_string()));
    fs::create_dir_all(home).map_err(|e| AppError::Internal(format!("create CODEX_HOME: {e}")))?;
    fs::create_dir_all(&backup_dir).map_err(|e| AppError::Internal(format!("create backups dir: {e}")))?;
    push_log(&mut logs, "ensure_directories", "Ensured config directories exist.", "success", None);

    if let Some(backup) = copy_backup_if_exists(&config_path, &backup_dir)? {
        push_log(&mut logs, "backup_config_toml", "Created config.toml backup.", "success", Some(backup.backup_path.clone()));
        backups.push(backup);
    }
    if let Some(backup) = copy_backup_if_exists(&auth_path, &backup_dir)? {
        push_log(&mut logs, "backup_auth_json", "Created auth.json backup.", "success", Some(backup.backup_path.clone()));
        backups.push(backup);
    }

    let mut doc = parse_or_empty_config(&config_path)?;
    push_log(&mut logs, "parse_config_toml", "Parsed config.toml.", "success", None);
    merge_fixed_preset(&mut doc);
    push_log(&mut logs, "merge_config_toml", "Merged fixed preset into config.toml.", "success", None);

    write_atomic_bytes(&config_path, doc.to_string().as_bytes())?;
    push_log(&mut logs, "write_config_toml", "Wrote config.toml.", "success", Some(config_path.display().to_string()));

    let auth_json = render_auth_json(trimmed)?;
    push_log(&mut logs, "render_auth_json", "Rendered auth.json.", "success", None);
    write_atomic_bytes(&auth_path, auth_json.as_bytes())?;
    push_log(&mut logs, "write_auth_json", "Wrote auth.json.", "success", Some(auth_path.display().to_string()));
    push_log(&mut logs, "done", "Applied Codex CLI preset.", "success", None);

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
```

- [ ] **Step 6: Run the targeted backend tests and confirm they pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml codex_cli_config -- --nocapture
```

Expected: PASS for all `codex_cli_config` tests.

- [ ] **Step 7: Commit the backend implementation**

```bash
git add src-tauri/Cargo.toml src-tauri/src/app/codex_cli_config.rs src-tauri/src/app/paths.rs
git commit -m "feat: implement codex cli config backend writer"
```

### Task 6: Expose backend preview/apply commands through Tauri

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/app/codex_cli_config.rs`

- [ ] **Step 1: Add imports for the new backend types and functions**

Near the other `crate::app::...` imports in `src-tauri/src/commands.rs`, add:

```rust
use crate::app::codex_cli_config::{
    apply as apply_codex_cli_preset_inner, preview as preview_codex_cli_preset_inner,
    CodexCliPresetApplyResult, CodexCliPresetPreview,
};
```

- [ ] **Step 2: Add preview and apply Tauri commands**

Insert these commands into `src-tauri/src/commands.rs` near the existing config-related commands:

```rust
#[tauri::command]
pub fn preview_codex_cli_preset() -> Result<CodexCliPresetPreview, CommandError> {
    preview_codex_cli_preset_inner().map_err(Into::into)
}

#[tauri::command]
pub fn apply_codex_cli_preset(api_key: String) -> Result<CodexCliPresetApplyResult, CommandError> {
    apply_codex_cli_preset_inner(&api_key).map_err(Into::into)
}
```

- [ ] **Step 3: Register the commands in the invoke handler**

Add these entries to the `tauri::generate_handler![]` list in `src-tauri/src/lib.rs`:

```rust
            commands::preview_codex_cli_preset,
            commands::apply_codex_cli_preset,
```

- [ ] **Step 4: Run cargo check to verify the Tauri surface compiles**

Run:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: Cargo exits with code `0`.

- [ ] **Step 5: Commit the Tauri command wiring**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: expose codex cli config tauri commands"
```

### Task 7: Build the frontend Codex Config page with API key input, preview, logs, and result state

**Files:**
- Modify: `src/app/views/CodexConfig.tsx`
- Modify: `src/services/managerApi.ts`
- Modify: `src/shared/types.ts`

- [ ] **Step 1: Replace the placeholder page with a working stateful component**

Replace `src/app/views/CodexConfig.tsx` with:

```tsx
import { useEffect, useMemo, useState } from "react";

import { errorMessage, managerApi } from "../../services/managerApi";
import type {
  CodexCliPresetApplyResult,
  CodexCliPresetPreview,
} from "../../shared/types";
import { Icon } from "../icons";
import { useI18n } from "../i18n";
import { NavBar, Ring } from "../components";

function maskApiKey(value: string): string {
  if (!value) return "";
  if (value.length <= 8) return "*".repeat(value.length);
  return `${value.slice(0, 4)}${"*".repeat(Math.max(4, value.length - 8))}${value.slice(-4)}`;
}

export function CodexConfig({ onBack }: { onBack: () => void }) {
  const { t } = useI18n();
  const [apiKey, setApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [preview, setPreview] = useState<CodexCliPresetPreview | null>(null);
  const [previewOpen, setPreviewOpen] = useState(false);
  const [logOpen, setLogOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<CodexCliPresetApplyResult | null>(null);

  useEffect(() => {
    void managerApi
      .getCodexCliPresetPreview()
      .then(setPreview)
      .catch((cause) => setError(errorMessage(cause)));
  }, []);

  const authPreview = useMemo(() => {
    if (!preview) return "";
    if (!apiKey.trim()) return preview.authJsonPreview;
    return JSON.stringify({ OPENAI_API_KEY: maskApiKey(apiKey.trim()) }, null, 2);
  }, [apiKey, preview]);

  const apply = async () => {
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      const next = await managerApi.applyCodexCliPreset(apiKey);
      setResult(next);
      setLogOpen(true);
    } catch (cause) {
      setError(errorMessage(cause));
      setLogOpen(true);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="pop">
      <NavBar title={t("nav.config")} onBack={onBack} />
      <div className="scroll view">
        <section className="hero" style={{ marginTop: 16 }}>
          <Ring icon="sliders" variant="muted" />
          <div className="headline" style={{ fontSize: 18 }}>
            {t("config.title")}
          </div>
          <div className="desc">{t("config.descV2")}</div>
        </section>

        {error ? (
          <div className="banner err">
            <Icon name="alert" />
            <span>{error}</span>
          </div>
        ) : null}

        {result ? (
          <div className="banner info">
            <Icon name="check" />
            <span>{t("config.applySuccess")}</span>
          </div>
        ) : null}

        <div className="group">
          <div className="group-h">{t("config.apiKeyHeader")}</div>
          <div className="list">
            <div className="row" style={{ display: "block" }}>
              <div className="rtitle" style={{ marginBottom: 8 }}>
                {t("config.apiKeyLabel")}
              </div>
              <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                <input
                  className="input mono"
                  type={showKey ? "text" : "password"}
                  value={apiKey}
                  placeholder={t("config.apiKeyPlaceholder")}
                  onChange={(e) => setApiKey(e.target.value)}
                  style={{ flex: 1 }}
                />
                <button className="mini-action" onClick={() => setShowKey((v) => !v)}>
                  {showKey ? t("config.hideKey") : t("config.showKey")}
                </button>
              </div>
            </div>
          </div>
        </div>

        {preview ? (
          <div className="group">
            <div className="group-h">{t("config.pathsHeader")}</div>
            <div className="list">
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.codexHome")}</span>
                  <span className="rsub mono">{preview.codexHomePath}</span>
                </span>
              </div>
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.configPath")}</span>
                  <span className="rsub mono">{preview.configTomlPath}</span>
                </span>
              </div>
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.authPath")}</span>
                  <span className="rsub mono">{preview.authJsonPath}</span>
                </span>
              </div>
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.backupDir")}</span>
                  <span className="rsub mono">{preview.backupDirPath}</span>
                </span>
              </div>
              <div className="row">
                <button className="mini-action" onClick={() => void managerApi.openCodexHome()}>
                  <Icon name="folder" />
                  {t("config.openCodexHome")}
                </button>
              </div>
            </div>
          </div>
        ) : null}

        <div className="group">
          <button className="row" onClick={() => setPreviewOpen((v) => !v)}>
            <span className="rtext">
              <span className="rtitle">{t("config.previewHeader")}</span>
              <span className="rsub">{t("config.previewSub")}</span>
            </span>
            <span className="rval">{previewOpen ? t("config.collapse") : t("config.expand")}</span>
          </button>
          {previewOpen && preview ? (
            <div className="list">
              <div className="row" style={{ display: "block" }}>
                <div className="rtitle">{t("config.previewConfigToml")}</div>
                <pre
                  className="mono"
                  style={{
                    whiteSpace: "pre-wrap",
                    overflowX: "auto",
                    padding: 12,
                    borderRadius: 16,
                    background: "rgba(8, 16, 32, 0.55)",
                    marginTop: 8,
                  }}
                >
                  {preview.configSnippet}
                </pre>
              </div>
              <div className="row" style={{ display: "block" }}>
                <div className="rtitle">{t("config.previewAuthJson")}</div>
                <pre
                  className="mono"
                  style={{
                    whiteSpace: "pre-wrap",
                    overflowX: "auto",
                    padding: 12,
                    borderRadius: 16,
                    background: "rgba(8, 16, 32, 0.55)",
                    marginTop: 8,
                  }}
                >
                  {authPreview}
                </pre>
              </div>
              <div className="banner info">
                <Icon name="info" />
                <span>{preview.warning}</span>
              </div>
            </div>
          ) : null}
        </div>

        <div className="group">
          <button className="row" onClick={() => setLogOpen((v) => !v)}>
            <span className="rtext">
              <span className="rtitle">{t("config.logHeader")}</span>
              <span className="rsub">{t("config.logSub")}</span>
            </span>
            <span className="rval">{logOpen ? t("config.collapse") : t("config.expand")}</span>
          </button>
          {logOpen ? (
            <div className="list">
              {(result?.logs ?? []).length === 0 ? (
                <div className="row">
                  <span className="rsub">{t("config.noLogsYet")}</span>
                </div>
              ) : (
                result!.logs.map((entry, index) => (
                  <div className="row" key={`${entry.step}-${index}`}>
                    <span className="rtext">
                      <span className="rtitle">{entry.message}</span>
                      <span className="rsub mono">
                        [{entry.status}] {entry.step}
                        {entry.detail ? ` · ${entry.detail}` : ""}
                      </span>
                    </span>
                  </div>
                ))
              )}
            </div>
          ) : null}
        </div>

        {result ? (
          <div className="group">
            <div className="group-h">{t("config.resultHeader")}</div>
            <div className="list">
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.configPath")}</span>
                  <span className="rsub mono">{result.configTomlPath}</span>
                </span>
              </div>
              <div className="row">
                <span className="rtext">
                  <span className="rtitle">{t("config.authPath")}</span>
                  <span className="rsub mono">{result.authJsonPath}</span>
                </span>
              </div>
              {result.backups.map((backup) => (
                <div className="row" key={backup.backupPath}>
                  <span className="rtext">
                    <span className="rtitle">{t("config.backupCreated")}</span>
                    <span className="rsub mono">{backup.backupPath}</span>
                  </span>
                </div>
              ))}
              {result.warning ? (
                <div className="banner info">
                  <Icon name="info" />
                  <span>{result.warning}</span>
                </div>
              ) : null}
            </div>
          </div>
        ) : null}

        <div className="actions">
          <button className="btn big" disabled={busy || !apiKey.trim()} onClick={() => void apply()}>
            {busy ? t("config.applying") : t("config.apply")}
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Run TypeScript check and fix any type/import errors**

Run:

```bash
npm run check
```

Expected: TypeScript exits with code `0`.

- [ ] **Step 3: Commit the working Codex Config page shell**

```bash
git add src/app/views/CodexConfig.tsx
git commit -m "feat: add codex cli config page"
```

### Task 8: Add i18n strings and frontend tests for the new page

**Files:**
- Modify: `src/app/i18n.tsx`
- Create: `src/app/views/CodexConfig.test.tsx`
- Modify: `package.json`

- [ ] **Step 1: Add the new English and Chinese i18n strings**

In the `zh-CN` catalog in `src/app/i18n.tsx`, replace the old placeholder keys and add:

```ts
  "config.title": "Codex CLI 一键配置",
  "config.descV2": "把固定的 Codex CLI 预设写入 ~/.codex/config.toml 和 auth.json。你只需要填写 API key。",
  "config.apiKeyHeader": "API 密钥",
  "config.apiKeyLabel": "OpenAI API Key",
  "config.apiKeyPlaceholder": "输入 API key",
  "config.showKey": "显示",
  "config.hideKey": "隐藏",
  "config.pathsHeader": "目标路径",
  "config.codexHome": "Codex 主目录",
  "config.configPath": "config.toml 路径",
  "config.authPath": "auth.json 路径",
  "config.backupDir": "备份目录",
  "config.openCodexHome": "打开配置目录",
  "config.previewHeader": "将写入的内容",
  "config.previewSub": "查看固定预设和 auth.json 预览",
  "config.previewConfigToml": "config.toml 片段",
  "config.previewAuthJson": "auth.json 预览",
  "config.logHeader": "写入日志",
  "config.logSub": "查看每一步执行和错误信息",
  "config.noLogsYet": "还没有写入日志。",
  "config.resultHeader": "应用结果",
  "config.backupCreated": "已创建备份",
  "config.apply": "一键应用",
  "config.applying": "正在写入…",
  "config.applySuccess": "Codex CLI 预设已写入。",
  "config.expand": "展开",
  "config.collapse": "收起",
```

In the `en` catalog in `src/app/i18n.tsx`, add:

```ts
  "config.title": "Codex CLI one-click config",
  "config.descV2": "Write a fixed Codex CLI preset into ~/.codex/config.toml and auth.json. You only need to provide the API key.",
  "config.apiKeyHeader": "API key",
  "config.apiKeyLabel": "OpenAI API key",
  "config.apiKeyPlaceholder": "Enter API key",
  "config.showKey": "Show",
  "config.hideKey": "Hide",
  "config.pathsHeader": "Target paths",
  "config.codexHome": "Codex home",
  "config.configPath": "config.toml path",
  "config.authPath": "auth.json path",
  "config.backupDir": "Backup directory",
  "config.openCodexHome": "Open config directory",
  "config.previewHeader": "What will be written",
  "config.previewSub": "Inspect the fixed preset and auth.json preview",
  "config.previewConfigToml": "config.toml snippet",
  "config.previewAuthJson": "auth.json preview",
  "config.logHeader": "Write log",
  "config.logSub": "Inspect step-by-step progress and errors",
  "config.noLogsYet": "No write logs yet.",
  "config.resultHeader": "Apply result",
  "config.backupCreated": "Backup created",
  "config.apply": "Apply preset",
  "config.applying": "Writing…",
  "config.applySuccess": "The Codex CLI preset was written.",
  "config.expand": "Expand",
  "config.collapse": "Collapse",
```

- [ ] **Step 2: Add frontend tests for the Codex Config page**

Create `src/app/views/CodexConfig.test.tsx` with:

```tsx
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

import { CodexConfig } from "./CodexConfig";

const getCodexCliPresetPreview = vi.fn();
const applyCodexCliPreset = vi.fn();
const openCodexHome = vi.fn();

vi.mock("../../services/managerApi", () => ({
  managerApi: {
    getCodexCliPresetPreview,
    applyCodexCliPreset,
    openCodexHome,
  },
  errorMessage: (cause: unknown) => (cause instanceof Error ? cause.message : String(cause)),
}));

vi.mock("../i18n", () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("../components", () => ({
  NavBar: ({ title }: { title: string }) => <div>{title}</div>,
  Ring: () => <div>ring</div>,
}));

vi.mock("../icons", () => ({
  Icon: () => <span>icon</span>,
}));

describe("CodexConfig", () => {
  beforeEach(() => {
    getCodexCliPresetPreview.mockReset();
    applyCodexCliPreset.mockReset();
    openCodexHome.mockReset();
    getCodexCliPresetPreview.mockResolvedValue({
      codexHomePath: "~/.codex",
      configTomlPath: "~/.codex/config.toml",
      authJsonPath: "~/.codex/auth.json",
      backupDirPath: "~/.codex/backups",
      configSnippet: 'model_provider = "OpenAI"',
      authJsonPreview: '{\n  "OPENAI_API_KEY": "sk-***"\n}',
      warning: "warning",
    });
  });

  it("disables apply until an API key is entered", async () => {
    render(<CodexConfig onBack={() => undefined} />);
    const button = await screen.findByRole("button", { name: "config.apply" });
    expect(button).toBeDisabled();
  });

  it("keeps preview and log collapsed by default and expands them on click", async () => {
    render(<CodexConfig onBack={() => undefined} />);
    await screen.findByText("config.pathsHeader");
    expect(screen.queryByText('model_provider = "OpenAI"')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /config.previewHeader/i }));
    expect(await screen.findByText('model_provider = "OpenAI"')).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /config.logHeader/i }));
    expect(await screen.findByText("config.noLogsYet")).toBeInTheDocument();
  });

  it("shows success state and logs after apply", async () => {
    applyCodexCliPreset.mockResolvedValue({
      codexHomePath: "~/.codex",
      configTomlPath: "~/.codex/config.toml",
      authJsonPath: "~/.codex/auth.json",
      backupDirPath: "~/.codex/backups",
      backups: [
        {
          targetPath: "~/.codex/config.toml",
          backupPath: "~/.codex/backups/config.toml.20260629-190000.bak",
        },
      ],
      logs: [
        {
          step: "done",
          message: "Applied Codex CLI preset.",
          status: "success",
          detail: null,
        },
      ],
      warning: null,
    });

    render(<CodexConfig onBack={() => undefined} />);
    const input = await screen.findByPlaceholderText("config.apiKeyPlaceholder");
    fireEvent.change(input, { target: { value: "sk-test" } });
    fireEvent.click(screen.getByRole("button", { name: "config.apply" }));

    expect(await screen.findByText("config.applySuccess")).toBeInTheDocument();
    expect(await screen.findByText("Applied Codex CLI preset.")).toBeInTheDocument();
    expect(await screen.findByText("~/.codex/backups/config.toml.20260629-190000.bak")).toBeInTheDocument();
  });

  it("shows failure when apply rejects", async () => {
    applyCodexCliPreset.mockRejectedValue(new Error("write auth.json failed"));

    render(<CodexConfig onBack={() => undefined} />);
    const input = await screen.findByPlaceholderText("config.apiKeyPlaceholder");
    fireEvent.change(input, { target: { value: "sk-test" } });
    fireEvent.click(screen.getByRole("button", { name: "config.apply" }));

    expect(await screen.findByText("write auth.json failed")).toBeInTheDocument();
  });
});
```

- [ ] **Step 3: Run the page test and verify it passes**

Run:

```bash
npm test -- src/app/views/CodexConfig.test.tsx
```

Expected: `PASS` for all `CodexConfig` tests.

- [ ] **Step 4: Run full frontend tests and type-check**

Run:

```bash
npm test
npm run check
```

Expected: both commands exit with code `0`.

- [ ] **Step 5: Commit the UI polish, i18n, and frontend tests**

```bash
git add src/app/views/CodexConfig.tsx src/app/views/CodexConfig.test.tsx src/app/i18n.tsx
git commit -m "feat: add codex cli config ui and tests"
```

### Task 9: Run final verification and review the diff against the spec

**Files:**
- Modify: none
- Test: repository commands only

- [ ] **Step 1: Run targeted backend tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml codex_cli_config -- --nocapture
```

Expected: all `codex_cli_config` tests pass.

- [ ] **Step 2: Run targeted frontend tests**

Run:

```bash
npm test -- src/services/managerApi.test.ts src/app/views/CodexConfig.test.tsx
```

Expected: both test files pass.

- [ ] **Step 3: Run full verification commands**

Run:

```bash
npm run check
npm test
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: all commands exit with code `0`.

- [ ] **Step 4: Review the final diff for spec coverage**

Run:

```bash
git diff --stat 78a4d9f..HEAD
```

Expected: changes cover:

- frontend Codex Config page
- shared types
- manager API methods/tests
- Rust backend module
- Tauri command registration
- i18n strings
- tests

- [ ] **Step 5: Commit the final verification checkpoint**

```bash
git commit --allow-empty -m "chore: verify codex cli config feature"
```

## Self-Review

### Spec coverage

- Fixed preset values: covered in Task 5
- Global `~/.codex/*` target only: covered in Tasks 5 and 6
- `config.toml` merge write: covered in Task 5
- `auth.json` overwrite: covered in Task 5
- Timestamped backups in `backups/`: covered in Task 5
- Preview and logs default collapsed: covered in Task 7 and Task 8
- No manager-side API key persistence: covered in Task 7
- Cross-platform path display: covered in Tasks 5 and 7
- Warning about file auth not being enforced: covered in Tasks 5 and 7

### Placeholder scan

- No `TODO` or `TBD` markers included
- All tasks name exact files and commands
- Code-bearing steps contain code blocks

### Type consistency

- Frontend types use `CodexCliPresetPreview`, `CodexCliPresetApplyResult`, `CodexConfigLogStep`, and `CodexConfigBackupInfo`
- Backend exports matching camelCase-serialized Rust structs
- Manager API method names match page usage: `getCodexCliPresetPreview` and `applyCodexCliPreset`
