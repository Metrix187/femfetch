mod hardware;
mod network;
mod packages;
mod system;
mod theme;
mod user;

use crate::AppConfig;
use crate::ModuleResult;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;

#[derive(Clone, Copy)]
pub struct Module {
    pub name: &'static str,
    pub key: &'static str,
    pub run: fn(&ModuleContext) -> Result<String, String>,
}

#[derive(Clone)]
pub struct ModuleContext {
    pub config: AppConfig,
}

pub fn available_modules() -> Vec<Module> {
    vec![
        system::os_module(),
        system::host_module(),
        system::kernel_module(),
        system::uptime_module(),
        packages::packages_module(),
        user::shell_module(),
        user::resolution_module(),
        user::de_wm_module(),
        user::terminal_module(),
        hardware::cpu_module(),
        hardware::gpu_module(),
        hardware::memory_module(),
        hardware::disk_module(),
        network::local_ip_module(),
        theme::theme_module(),
        theme::icons_module(),
        theme::font_module(),
        theme::cursor_module(),
    ]
}

pub fn resolve_modules(requested: Option<&[String]>) -> Result<Vec<Module>, String> {
    let available = available_modules();
    if let Some(names) = requested {
        let mut resolved = Vec::new();
        for name in names {
            let module = available
                .iter()
                .find(|m| m.name.eq_ignore_ascii_case(name))
                .ok_or_else(|| format!("Unknown module: {name}"))?;
            resolved.push(module.clone());
        }
        return Ok(resolved);
    }
    Ok(available)
}

pub fn run_modules(modules: Vec<Module>, config: AppConfig) -> Vec<ModuleResult> {
    let context = Arc::new(ModuleContext { config });
    let (tx, rx) = mpsc::channel();
    for (idx, module) in modules.iter().enumerate() {
        let tx = tx.clone();
        let context = Arc::clone(&context);
        let module = *module;
        thread::spawn(move || {
            let start = Instant::now();
            let result = (module.run)(&context);
            let took_ms = start.elapsed().as_millis();
            let (value, error) = match result {
                Ok(value) => (value, None),
                Err(err) => ("N/A".to_string(), Some(err)),
            };
            let _ = tx.send((idx, ModuleResult {
                key: module.key.to_string(),
                value,
                took_ms,
                error,
            }));
        });
    }
    drop(tx);
    let mut results = vec![None; modules.len()];
    for (idx, result) in rx {
        results[idx] = Some(result);
    }
    results.into_iter().flatten().collect()
}

