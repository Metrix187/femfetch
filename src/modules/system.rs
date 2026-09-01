use crate::modules::Module;
use crate::platform;
use crate::util::format_uptime;

pub fn os_module() -> Module {
    Module {
        name: "os",
        key: "OS ✿",
        run: |_| platform::os_name_version().ok_or_else(|| "OS not detected".to_string()),
        default: true,
        dynamic: false,
        gate: None,
    }
}

pub fn host_module() -> Module {
    Module {
        name: "host",
        key: "Host ◍",
        run: |_| {
            let host = platform::host();
            let model = platform::model();
            match (host, model) {
                (Some(host), Some(model)) => Ok(format!("{host} ({model})")),
                (Some(host), None) => Ok(host),
                (None, Some(model)) => Ok(model),
                _ => Err("Host not detected".to_string()),
            }
        },
        default: true,
        dynamic: false,
        gate: None,
    }
}

pub fn kernel_module() -> Module {
    Module {
        name: "kernel",
        key: "Kernel ❀",
        run: |_| platform::kernel().ok_or_else(|| "Kernel not detected".to_string()),
        default: true,
        dynamic: false,
        gate: None,
    }
}

pub fn uptime_module() -> Module {
    Module {
        name: "uptime",
        key: "Uptime ♡",
        run: |_| {
            platform::uptime_seconds()
                .map(format_uptime)
                .ok_or_else(|| "Uptime not detected".to_string())
        },
        default: true,
        dynamic: true,
        gate: None,
    }
}
