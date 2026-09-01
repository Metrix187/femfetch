use crate::modules::{Gate, Module};
use crate::platform;

pub fn local_ip_module() -> Module {
    Module {
        name: "local-ip",
        key: "Local IP ♡",
        run: |_| platform::local_ip().ok_or_else(|| "Local IP not detected".to_string()),
        default: false,
        dynamic: true,
        gate: Some(Gate::LocalIp),
    }
}

pub fn interface_module() -> Module {
    Module {
        name: "interface",
        key: "Interface ✿",
        run: |_| {
            platform::network_interface()
                .ok_or_else(|| "Network interface not detected".to_string())
        },
        default: false,
        dynamic: true,
        gate: None,
    }
}

pub fn wifi_module() -> Module {
    Module {
        name: "wifi",
        key: "Wi-Fi ♡",
        run: |_| platform::wifi().ok_or_else(|| "Wi-Fi not detected".to_string()),
        default: false,
        dynamic: true,
        gate: None,
    }
}

pub fn network_status_module() -> Module {
    Module {
        name: "network",
        key: "Network ❀",
        run: |_| {
            platform::network_connected()
                .then_some("Connected".to_string())
                .ok_or_else(|| "Disconnected".to_string())
        },
        default: false,
        dynamic: true,
        gate: None,
    }
}
