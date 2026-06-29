use std::path::PathBuf;

/// Manager data directory shared by settings, provenance, and operation locks.
pub fn data_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("io.github", "wangnov", "codexappmanager")
        .map(|dirs| dirs.data_dir().to_path_buf())
}

pub fn settings_path() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("settings.json"))
}

pub fn provenance_path() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("provenance.json"))
}

pub fn codex_home_dir() -> Option<PathBuf> {
    directories::UserDirs::new().map(|dirs| dirs.home_dir().join(".codex"))
}

pub fn codex_config_toml_path() -> Option<PathBuf> {
    codex_home_dir().map(|dir| dir.join("config.toml"))
}

pub fn codex_auth_json_path() -> Option<PathBuf> {
    codex_home_dir().map(|dir| dir.join("auth.json"))
}

pub fn codex_backups_dir() -> Option<PathBuf> {
    codex_home_dir().map(|dir| dir.join("backups"))
}
