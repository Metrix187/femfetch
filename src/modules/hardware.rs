use crate::modules::Module;
use crate::platform;
use crate::util::format_bytes;

pub fn cpu_module() -> Module {
    Module {
        name: "cpu",
        key: "CPU ✿",
        run: |_| platform::cpu().ok_or_else(|| "CPU not detected".to_string()),
    }
}

pub fn gpu_module() -> Module {
    Module {
        name: "gpu",
        key: "GPU ✿",
        run: |_| platform::gpu().ok_or_else(|| "GPU not detected".to_string()),
    }
}

pub fn memory_module() -> Module {
    Module {
        name: "memory",
        key: "Memory ♡",
        run: |_| {
            if let Some((total, used)) = platform::memory() {
                Ok(format!("{} / {}", format_bytes(used), format_bytes(total)))
            } else {
                Err("Memory not detected".to_string())
            }
        },
    }
}

pub fn disk_module() -> Module {
    Module {
        name: "disk",
        key: "Disk ❀",
        run: |ctx| {
            if let Some((total, used)) = platform::disk(ctx.config.all_disks) {
                Ok(format!("{} / {}", format_bytes(used), format_bytes(total)))
            } else {
                Err("Disk not detected".to_string())
            }
        },
    }
}
