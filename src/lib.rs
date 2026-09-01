pub mod cache;
pub mod config;
pub mod export;
pub mod modules;
pub mod platform;
pub mod render;
pub mod signals;
pub mod themes;
pub mod util;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub no_color: bool,
    pub theme: String,
    pub ascii_preset: Option<String>,
    pub ascii_file: Option<String>,
    pub ascii_size: String,
    pub plain: bool,
    pub compact: bool,
    pub json: bool,
    pub speed: bool,
    pub mascot_state: bool,
    pub modules: Option<Vec<String>>,
    pub cache_ttl_seconds: Option<u64>,
    pub all_disks: bool,
    pub local_ip: bool,
    pub show_theme: bool,
    pub show_icons: bool,
    pub show_font: bool,
    pub show_cursor: bool,
    pub terminal_width: Option<usize>,
    pub watch_interval_seconds: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            no_color: false,
            theme: "pastel".to_string(),
            ascii_preset: None,
            ascii_file: None,
            ascii_size: "medium".to_string(),
            plain: false,
            compact: false,
            json: false,
            speed: false,
            mascot_state: false,
            modules: None,
            cache_ttl_seconds: None,
            all_disks: false,
            local_ip: false,
            show_theme: false,
            show_icons: false,
            show_font: false,
            show_cursor: false,
            terminal_width: None,
            watch_interval_seconds: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResult {
    #[serde(default)]
    pub name: String,
    pub key: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub took_ms: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunOutput {
    pub version: u32,
    pub modules: Vec<ModuleResult>,
}
