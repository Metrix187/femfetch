use crate::modules::Module;
use crate::platform;
use crate::util::format_bytes;

fn module(
    name: &'static str,
    key: &'static str,
    dynamic: bool,
    run: fn(&crate::modules::ModuleContext) -> Result<String, String>,
) -> Module {
    Module {
        name,
        key,
        run,
        default: false,
        dynamic,
        gate: None,
    }
}

pub fn cpu_module() -> Module {
    Module {
        name: "cpu",
        key: "CPU ✿",
        run: |_| platform::cpu().ok_or_else(|| "CPU not detected".to_string()),
        default: true,
        dynamic: false,
        gate: None,
    }
}

pub fn gpu_module() -> Module {
    Module {
        name: "gpu",
        key: "GPU ✿",
        run: |_| platform::gpu().ok_or_else(|| "GPU not detected".to_string()),
        default: true,
        dynamic: false,
        gate: None,
    }
}

pub fn memory_module() -> Module {
    Module {
        name: "memory",
        key: "Memory ♡",
        run: |_| {
            platform::memory()
                .map(|(total, used)| format!("{} / {}", format_bytes(used), format_bytes(total)))
                .ok_or_else(|| "Memory not detected".to_string())
        },
        default: true,
        dynamic: true,
        gate: None,
    }
}

pub fn disk_module() -> Module {
    Module {
        name: "disk",
        key: "Disk ❀",
        run: |context| {
            platform::disk(context.config.all_disks)
                .map(|(total, used)| format!("{} / {}", format_bytes(used), format_bytes(total)))
                .ok_or_else(|| "Disk not detected".to_string())
        },
        default: true,
        dynamic: true,
        gate: None,
    }
}

pub fn cpu_temperature_module() -> Module {
    module("cpu-temp", "CPU Temp ♨", true, |_| {
        platform::cpu_temperature()
            .map(|value| format!("{value:.1} °C"))
            .ok_or_else(|| "CPU temperature not detected".to_string())
    })
}

pub fn cpu_utilization_module() -> Module {
    module("cpu-usage", "CPU Usage ◌", true, |_| {
        platform::cpu_utilization()
            .map(|value| format!("{value:.1}%"))
            .ok_or_else(|| "CPU utilization not detected".to_string())
    })
}

pub fn cpu_frequency_module() -> Module {
    module("cpu-frequency", "CPU Freq ♫", true, |_| {
        platform::cpu_frequency()
            .map(|mhz| {
                if mhz >= 1_000.0 {
                    format!("{:.2} GHz", mhz / 1_000.0)
                } else {
                    format!("{mhz:.0} MHz")
                }
            })
            .ok_or_else(|| "CPU frequency not detected".to_string())
    })
}

pub fn gpu_temperature_module() -> Module {
    module("gpu-temp", "GPU Temp ♨", true, |_| {
        platform::gpu_temperature()
            .map(|value| format!("{value:.1} °C"))
            .ok_or_else(|| "GPU temperature not detected".to_string())
    })
}

pub fn gpu_utilization_module() -> Module {
    module("gpu-usage", "GPU Usage ◌", true, |_| {
        platform::gpu_utilization()
            .map(|value| format!("{value:.1}%"))
            .ok_or_else(|| "GPU utilization not detected".to_string())
    })
}

pub fn vram_module() -> Module {
    module("vram", "VRAM ♡", true, |_| {
        platform::vram()
            .map(|(total, used)| format!("{} / {}", format_bytes(used), format_bytes(total)))
            .ok_or_else(|| "VRAM not detected".to_string())
    })
}

pub fn battery_module() -> Module {
    module("battery", "Battery ♡", true, |_| {
        platform::battery()
            .map(|battery| {
                let status = battery.status.as_deref().unwrap_or("Unknown");
                format!("{}% ({status})", battery.capacity)
            })
            .ok_or_else(|| "Battery not detected".to_string())
    })
}

pub fn battery_health_module() -> Module {
    module("battery-health", "Battery Health ✿", true, |_| {
        platform::battery_health()
            .map(|value| format!("{value:.1}%"))
            .ok_or_else(|| "Battery health not detected".to_string())
    })
}

pub fn swap_module() -> Module {
    module("swap", "Swap ◍", true, |_| {
        platform::swap()
            .map(|(total, used)| format!("{} / {}", format_bytes(used), format_bytes(total)))
            .ok_or_else(|| "Swap not detected".to_string())
    })
}

pub fn motherboard_module() -> Module {
    module("motherboard", "Motherboard ✿", false, |_| {
        platform::motherboard().ok_or_else(|| "Motherboard not detected".to_string())
    })
}

pub fn bios_module() -> Module {
    module("bios", "BIOS / UEFI ❀", false, |_| {
        platform::bios().ok_or_else(|| "BIOS/UEFI not detected".to_string())
    })
}

pub fn storage_model_module() -> Module {
    module("storage-model", "Storage ✿", false, |_| {
        platform::storage_model().ok_or_else(|| "Storage model not detected".to_string())
    })
}

pub fn filesystem_module() -> Module {
    module("filesystem", "Filesystem ◍", false, |_| {
        platform::filesystem().ok_or_else(|| "Filesystem not detected".to_string())
    })
}
