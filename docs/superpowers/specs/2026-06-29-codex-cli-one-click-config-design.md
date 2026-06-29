# Codex CLI One-Click Config Design

## Goal

Add a cross-platform "Codex CLI one-click config" feature to Codex App Manager that writes a fixed global Codex CLI preset into `~/.codex/config.toml` and `~/.codex/auth.json`.

The user only inputs an API key. Everything else in the preset is fixed.

This feature is intentionally limited to:

- Codex CLI only
- Global user config only: `~/.codex/config.toml` and `~/.codex/auth.json`
- One fixed preset only
- `config.toml` merge write
- `auth.json` full overwrite
- Timestamped backup files for manual recovery

This feature does not include:

- Codex CLI WebSocket config
- OpenCode config
- Project-local `.codex/config.toml`
- Multiple presets
- Built-in restore UI
- Manager-side persistence of the API key

## Product Scope

The current `CodexConfig` page is only a placeholder shell. This feature turns it into a working Codex CLI preset writer.

The preset is applied to the user's Codex home directory, not to the manager's own settings store. This keeps the feature aligned with the user's stated goal: configure Codex itself, not Codex App Manager.

The feature targets Windows, macOS, and Linux.

## Fixed Preset

The applied Codex CLI preset is fixed to the following values:

```toml
model_provider = "OpenAI"
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
```

`auth.json` is written as:

```json
{
  "OPENAI_API_KEY": "<user input>"
}
```

This design intentionally does not force `cli_auth_credentials_store = "file"` into `config.toml`, because the user explicitly chose not to add that key. The UI must warn that environments pinned to keyring-based auth may not immediately honor `auth.json`.

## User Experience

The existing [src/app/views/CodexConfig.tsx](/Users/charilezhou/.codex/worktrees/62a3/Codex-App-Manager/src/app/views/CodexConfig.tsx) placeholder should become a single-page preset writer.

### Page structure

1. Intro section
   Explains that this page configures Codex CLI global files under `~/.codex`.
2. API key input
   One password-style input field with optional show/hide affordance.
3. Target path section
   Shows the resolved target files:
   - Windows: `%USERPROFILE%\\.codex\\config.toml` and `%USERPROFILE%\\.codex\\auth.json`
   - macOS/Linux: `~/.codex/config.toml` and `~/.codex/auth.json`
4. Collapsible preview section
   Default collapsed. Shows the fixed config snippet and the resulting `auth.json`.
5. Collapsible write log section
   Default collapsed. Shows step-by-step write progress and any failure details.
6. Action section
   Primary button: apply preset.

### Interaction rules

- The API key is required.
- The apply button is disabled while the input is empty.
- The API key is not persisted by the manager after the page unmounts or after the app restarts.
- The preview is read-only.
- The log is cleared on a new attempt, then rebuilt step by step.
- The page should remain usable after a failed attempt so the user can correct the key or retry.

### Success output

On success, the page shows:

- The resolved `config.toml` path
- The resolved `auth.json` path
- The backup file paths created for this run
- A reminder to restart Codex CLI or open a new Codex session
- A warning that `auth.json` may not be used if the user's Codex environment is explicitly pinned to keyring auth

## File Resolution

Backend file resolution should follow Codex's home-directory semantics:

- Prefer `CODEX_HOME` if it is set
- Otherwise use the default user Codex directory under the user's home directory

Target files:

- `<codex_home>/config.toml`
- `<codex_home>/auth.json`
- `<codex_home>/backups/`

The backend should expose the resolved paths to the frontend so the page can show the exact locations for the current platform.

## Write Flow

Add a dedicated backend command for this feature rather than extending the manager's existing generic settings/provenance commands.

Suggested command shape:

- `preview_codex_cli_preset()`
- `apply_codex_cli_preset(api_key: String)`

### Preview

The preview command returns:

- Resolved Codex home path
- Resolved `config.toml` path
- Resolved `auth.json` path
- The fixed config snippet
- The rendered `auth.json` body
- Warning text about file-based auth not being enforced

### Apply

The apply command performs:

1. Resolve Codex home paths
2. Ensure the Codex home directory exists
3. Ensure the backup directory exists
4. If `config.toml` exists, copy it to a timestamped backup file
5. If `auth.json` exists, copy it to a timestamped backup file
6. Load and parse `config.toml` if it exists, otherwise start from an empty document
7. Merge in the fixed preset values
8. Atomically write `config.toml`
9. Render the fixed `auth.json`
10. Atomically write `auth.json`
11. Return success payload including created backup paths and step log

## TOML Merge Rules

`config.toml` must not be replaced wholesale.

Use `toml_edit` so the implementation can update only the keys owned by this feature and preserve unrelated user config as much as possible.

### Keys managed by this feature

Top-level keys:

- `model_provider`
- `model`
- `review_model`
- `model_reasoning_effort`
- `disable_response_storage`
- `network_access`
- `windows_wsl_setup_acknowledged`

Nested tables:

- `[model_providers.OpenAI]`
  - `name`
  - `base_url`
  - `wire_api`
  - `requires_openai_auth`
- `[features]`
  - `goals`

### Merge behavior

- If a managed key already exists, overwrite it with the fixed preset value.
- If a required table does not exist, create it.
- If unrelated keys or tables exist, preserve them.
- If `[features]` already exists, update only `goals`.
- If `[model_providers.OpenAI]` already exists, update only the managed keys in that table.

The implementation does not need to preserve exact original formatting or comments perfectly, but it should preserve document structure as much as `toml_edit` reasonably allows.

## `auth.json` Rules

`auth.json` is overwritten fully on each successful run.

Rules:

- The file content is always exactly:
  - `{ "OPENAI_API_KEY": "<user input>" }`
- Use pretty JSON output
- Write atomically
- Do not merge with existing JSON content

## Backup Rules

This feature does not use a single rotating `.bak`.

Instead, every apply attempt that finds an existing target file creates a timestamped backup under `<codex_home>/backups/`.

Suggested format:

- `config.toml.YYYYMMDD-HHMMSS.bak`
- `auth.json.YYYYMMDD-HHMMSS.bak`

Example:

- `~/.codex/backups/config.toml.20260629-182530.bak`
- `~/.codex/backups/auth.json.20260629-182530.bak`

Rules:

- Only create a backup for a file that already exists
- Create the backup before attempting to mutate that file
- Return the concrete backup paths in the apply result
- Do not add restore UI in v1
- Do not implement backup pruning or multi-version management logic

This design intentionally favors manual recovery over automated rollback complexity.

## Logging Model

The frontend log panel should show a structured sequence of backend-reported steps.

Suggested steps:

- `resolve_codex_home`
- `ensure_directories`
- `backup_config_toml`
- `backup_auth_json`
- `parse_config_toml`
- `merge_config_toml`
- `write_config_toml`
- `render_auth_json`
- `write_auth_json`
- `done`

For each step, capture:

- Step id
- Human-readable message
- Status: pending, running, success, failure
- Optional detail string

On failure, the final result should include:

- Failing step
- Raw backend error message
- Backup paths that were already created

The log panel is diagnostic UI, not audit logging. It exists to help users understand where a write failed.

## Error Handling

This feature should be step-safe but not fully transactional.

### Pre-write failure

If the operation fails before any file write, return an error and leave existing files untouched.

### Config write failure

If backup succeeds but `config.toml` parse, merge, or write fails:

- Return an error
- Do not touch `auth.json`
- Surface the failure step and message in the log

### Auth write failure after config success

If `config.toml` is already written successfully but `auth.json` write fails:

- Return an error
- Do not auto-rollback `config.toml`
- Surface the created backup paths so the user can restore manually

This is an intentional v1 tradeoff. The design prefers simpler, observable failure behavior with manual recovery over cross-file rollback logic.

### Invalid input

- Empty API key is rejected in the frontend
- Empty API key is also rejected in the backend for defense in depth

## Security Notes

- The manager must not persist the API key in its own settings or provenance files
- The API key should be kept in memory only for the active page/session
- The API key should not be written into the manager's diagnostic logs
- The write log shown in UI should avoid echoing the full API key
- Preview may show a masked or redacted form of the key in `auth.json`

Because `auth.json` is plaintext by design in this flow, the feature should include a warning that the file contains credentials and should be protected like a password.

## Frontend Changes

Expected frontend changes:

- Replace the placeholder implementation in `src/app/views/CodexConfig.tsx`
- Extend `src/services/managerApi.ts` with preview/apply methods for the Codex CLI preset writer
- Extend `src/shared/types.ts` with new result/log payload types
- Add i18n strings for:
  - Intro copy
  - API key label and placeholder
  - Preview section
  - Log section
  - Success/error/result copy
  - Warning about file-based auth not being enforced

## Backend Changes

Expected backend changes:

- Add a dedicated application module for Codex CLI config writing
  - Suggested file: `src-tauri/src/app/codex_cli_config.rs`
- Add new Tauri commands in `src-tauri/src/commands.rs`
- Register the commands in `src-tauri/src/lib.rs`
- Add `toml_edit` dependency to `src-tauri/Cargo.toml`
- Reuse the existing atomic-write style already used elsewhere in the manager where appropriate

This feature should remain separate from:

- manager `settings.json`
- manager `provenance.json`
- existing config health/reset/restore commands

Those existing commands belong to the manager's own configuration, not Codex CLI's `~/.codex/config.toml`.

## Testing Plan

### Rust unit tests

1. Creates `config.toml` and `auth.json` when neither exists
2. Merges into an existing `config.toml` without dropping unrelated keys
3. Updates existing `[features]` and `[model_providers.OpenAI]` tables correctly
4. Creates timestamped backups when original files exist
5. Overwrites `auth.json` completely
6. Rejects empty API key
7. Reports step-specific failure when TOML parsing fails

### Frontend tests

1. Apply button disabled when API key is empty
2. Preview section is collapsed by default and can be expanded
3. Log section is collapsed by default and can be expanded
4. Successful apply renders result summary and backup paths
5. Failed apply renders error state and step log

### Manual verification

1. Windows path display uses `%USERPROFILE%\\.codex`
2. macOS/Linux path display uses `~/.codex`
3. Existing complex `config.toml` retains unrelated user settings
4. Multiple apply runs create multiple timestamped backups under `backups/`
5. Codex CLI reads the resulting files successfully on all three platforms

## Implementation Notes

This feature intentionally optimizes for a narrow real workflow:

- one preset
- one editable field
- one global target
- observable writes
- manual recovery via timestamped backups

Do not generalize this into a multi-provider configuration system in the initial implementation. That would add complexity without serving the current goal.
