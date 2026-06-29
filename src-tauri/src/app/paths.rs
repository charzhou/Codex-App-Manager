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
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| directories::UserDirs::new().map(|dirs| dirs.home_dir().join(".codex")))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_home_prefers_env_override() {
        let original = std::env::var_os("CODEX_HOME");
        let expected = std::env::temp_dir().join(format!("cam-codex-home-{}", std::process::id()));
        std::env::set_var("CODEX_HOME", &expected);

        let actual = codex_home_dir().expect("codex home");

        assert_eq!(actual, expected);
        assert_eq!(
            codex_config_toml_path().expect("config path"),
            expected.join("config.toml")
        );
        assert_eq!(
            codex_auth_json_path().expect("auth path"),
            expected.join("auth.json")
        );
        assert_eq!(
            codex_backups_dir().expect("backups dir"),
            expected.join("backups")
        );

        match original {
            Some(value) => std::env::set_var("CODEX_HOME", value),
            None => std::env::remove_var("CODEX_HOME"),
        }
    }
}
