pub mod cache;
pub mod modules;
pub mod platform;
pub mod render;
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
    pub json: bool,
    pub speed: bool,
    pub modules: Option<Vec<String>>,
    pub cache_ttl_seconds: Option<u64>,
    pub all_disks: bool,
    pub local_ip: bool,
    pub show_theme: bool,
    pub show_icons: bool,
    pub show_font: bool,
    pub show_cursor: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResult {
    pub key: String,
    pub value: String,
    pub took_ms: u128,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunOutput {
    pub modules: Vec<ModuleResult>,
}
