use crate::modules::Module;
use crate::platform;
use crate::util::format_uptime;

pub fn os_module() -> Module {
    Module {
        name: "os",
        key: "OS ✿",
        run: |_| {
            platform::os_name_version().ok_or_else(|| "OS not detected".to_string())
        },
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
                (Some(h), Some(m)) => Ok(format!("{h} ({m})")),
                (Some(h), None) => Ok(h),
                (None, Some(m)) => Ok(m),
                _ => Err("Host not detected".to_string()),
            }
        },
    }
}

pub fn kernel_module() -> Module {
    Module {
        name: "kernel",
        key: "Kernel ❀",
        run: |_| platform::kernel().ok_or_else(|| "Kernel not detected".to_string()),
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
    }
}
