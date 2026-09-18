//! Ayarlar (koyu/açık mod + renk override'ları) → %APPDATA%\SistemBakimAraci\config.json

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default)]
    pub overrides: HashMap<String, HashMap<String, String>>,
}

fn default_mode() -> String {
    "dark".to_string()
}

fn default_lang() -> String {
    "tr".to_string()
}

impl Config {
    pub fn load() -> Config {
        let path = Self::path();
        match std::fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Config>(&text) {
                Ok(mut c) => {
                    if c.mode != "dark" && c.mode != "light" {
                        c.mode = "dark".to_string();
                    }
                    if c.lang != "tr" && c.lang != "en" {
                        c.lang = "tr".to_string();
                    }
                    c
                }
                Err(_) => Config::default(),
            },
            Err(_) => Config::default(),
        }
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, text);
        }
    }

    pub fn path() -> PathBuf {
        let base = std::env::var("APPDATA")
            .unwrap_or_else(|_| std::env::var("HOME").unwrap_or_else(|_| ".".to_string()));
        PathBuf::from(base).join("SistemBakimAraci").join("config.json")
    }
}
