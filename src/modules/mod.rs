mod hardware;
mod network;
mod packages;
mod system;
mod theme;
mod user;

use crate::{AppConfig, ModuleResult};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gate {
    LocalIp,
    Theme,
    Icons,
    Font,
    Cursor,
}

impl Gate {
    pub fn enabled(self, config: &AppConfig) -> bool {
        match self {
            Self::LocalIp => config.local_ip,
            Self::Theme => config.show_theme,
            Self::Icons => config.show_icons,
            Self::Font => config.show_font,
            Self::Cursor => config.show_cursor,
        }
    }

    pub fn flag(self) -> &'static str {
        match self {
            Self::LocalIp => "--local-ip",
            Self::Theme => "--show-theme",
            Self::Icons => "--show-icons",
            Self::Font => "--show-font",
            Self::Cursor => "--show-cursor",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Module {
    pub name: &'static str,
    pub key: &'static str,
    pub run: fn(&ModuleContext) -> Result<String, String>,
    pub default: bool,
    pub dynamic: bool,
    pub gate: Option<Gate>,
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
        hardware::cpu_temperature_module(),
        hardware::cpu_utilization_module(),
        hardware::cpu_frequency_module(),
        hardware::gpu_temperature_module(),
        hardware::gpu_utilization_module(),
        hardware::vram_module(),
        hardware::battery_module(),
        hardware::battery_health_module(),
        hardware::swap_module(),
        hardware::motherboard_module(),
        hardware::bios_module(),
        hardware::storage_model_module(),
        hardware::filesystem_module(),
        network::local_ip_module(),
        network::interface_module(),
        network::wifi_module(),
        network::network_status_module(),
        user::monitor_module(),
        theme::theme_module(),
        theme::icons_module(),
        theme::font_module(),
        theme::cursor_module(),
    ]
}

pub fn module_names() -> Vec<&'static str> {
    available_modules()
        .into_iter()
        .map(|module| module.name)
        .collect()
}

pub fn resolve_modules(requested: Option<&[String]>) -> Result<Vec<Module>, String> {
    let available = available_modules();
    if let Some(names) = requested {
        let mut resolved = Vec::with_capacity(names.len());
        for name in names.iter().filter(|name| !name.trim().is_empty()) {
            let module = available
                .iter()
                .find(|module| module.name.eq_ignore_ascii_case(name))
                .ok_or_else(|| {
                    format!(
                        "Unknown module: {name}. Available modules: {}",
                        available
                            .iter()
                            .map(|module| module.name)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })?;
            resolved.push(*module);
        }
        return Ok(resolved);
    }
    Ok(available
        .into_iter()
        .filter(|module| module.default)
        .collect())
}

pub fn append_enabled_optional(modules: &mut Vec<Module>, config: &AppConfig) {
    for module in available_modules() {
        if module.gate.is_some_and(|gate| gate.enabled(config))
            && !modules.iter().any(|selected| selected.name == module.name)
        {
            modules.push(module);
        }
    }
}

pub fn validate_gates(modules: &[Module], config: &AppConfig) -> Result<(), String> {
    for module in modules {
        if let Some(gate) = module.gate {
            if !gate.enabled(config) {
                return Err(format!(
                    "{} module requires {} for explicit opt-in.",
                    module.name,
                    gate.flag()
                ));
            }
        }
    }
    Ok(())
}

pub fn run_modules(modules: &[Module], config: &AppConfig) -> Vec<ModuleResult> {
    let context = Arc::new(ModuleContext {
        config: config.clone(),
    });
    let (tx, rx) = mpsc::channel();
    for (index, module) in modules.iter().copied().enumerate() {
        let tx = tx.clone();
        let context = Arc::clone(&context);
        thread::spawn(move || {
            let start = Instant::now();
            let result = (module.run)(&context);
            let took_ms = start.elapsed().as_secs_f64() * 1_000.0;
            let (value, error) = match result {
                Ok(value) => (value, None),
                Err(error) => ("N/A".to_string(), Some(error)),
            };
            let _ = tx.send((
                index,
                ModuleResult {
                    name: module.name.to_string(),
                    key: module.key.to_string(),
                    value,
                    took_ms: Some(took_ms),
                    error,
                },
            ));
        });
    }
    drop(tx);
    let mut results = vec![None; modules.len()];
    for (index, result) in rx {
        results[index] = Some(result);
    }
    modules
        .iter()
        .enumerate()
        .map(|(index, module)| {
            results[index].take().unwrap_or_else(|| ModuleResult {
                name: module.name.to_string(),
                key: module.key.to_string(),
                value: "N/A".to_string(),
                took_ms: None,
                error: Some("Module did not complete".to_string()),
            })
        })
        .collect()
}

pub fn merge_dynamic(all: &mut [ModuleResult], updated: Vec<ModuleResult>) {
    for result in updated {
        if let Some(existing) = all.iter_mut().find(|item| item.name == result.name) {
            *existing = result;
        }
    }
}
