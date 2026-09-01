use crate::util::now_epoch_seconds;
use crate::ModuleResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    timestamp: u64,
    key: String,
    modules: Vec<ModuleResult>,
}

pub fn load_cache(key: &str, ttl_seconds: u64) -> Option<Vec<ModuleResult>> {
    let path = cache_path()?;
    let contents = fs::read_to_string(path).ok()?;
    let entry: CacheEntry = serde_json::from_str(&contents).ok()?;
    let now = now_epoch_seconds();
    if entry.key != key {
        return None;
    }
    if now.saturating_sub(entry.timestamp) > ttl_seconds {
        return None;
    }
    Some(entry.modules)
}

pub fn write_cache(key: &str, modules: &[ModuleResult]) -> Result<(), String> {
    let path = cache_path().ok_or_else(|| "Cache path unavailable".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Cache dir error: {e}"))?;
    }
    let entry = CacheEntry {
        timestamp: now_epoch_seconds(),
        key: key.to_string(),
        modules: modules.to_vec(),
    };
    let data = serde_json::to_string(&entry).map_err(|e| format!("Cache encode error: {e}"))?;
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, data).map_err(|e| format!("Cache write error: {e}"))?;
    fs::rename(&temporary, &path).map_err(|e| {
        let _ = fs::remove_file(&temporary);
        format!("Cache replace error: {e}")
    })?;
    Ok(())
}

fn cache_path() -> Option<PathBuf> {
    if cfg!(windows) {
        let base = std::env::var("LOCALAPPDATA").ok()?;
        Some(PathBuf::from(base).join("femfetch").join("cache.json"))
    } else {
        let base = std::env::var("XDG_CACHE_HOME")
            .ok()
            .or_else(|| std::env::var("HOME").ok().map(|h| format!("{h}/.cache")))?;
        Some(PathBuf::from(base).join("femfetch").join("cache.json"))
    }
}
