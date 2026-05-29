use std::{fs, path::PathBuf};

use crate::adapters::types::{default_settings, AppSettings};

pub fn load_settings() -> AppSettings {
    let Some(path) = config_path() else {
        return default_settings();
    };

    let Ok(contents) = fs::read_to_string(path) else {
        return default_settings();
    };

    serde_json::from_str(&contents).unwrap_or_else(|_| default_settings())
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let Some(path) = config_path() else {
        return Ok(());
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let contents = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, contents).map_err(|error| error.to_string())
}

fn config_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;

    #[cfg(target_os = "macos")]
    {
        return Some(home.join("Library/Application Support/Agent Island/config.json"));
    }

    #[cfg(not(target_os = "macos"))]
    {
        Some(home.join(".config/agent-island/config.json"))
    }
}
