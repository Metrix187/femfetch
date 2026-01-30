use crate::modules::Module;
use crate::platform;

pub fn local_ip_module() -> Module {
    Module {
        name: "local-ip",
        key: "Local IP ♡",
        run: |_| platform::local_ip().ok_or_else(|| "Local IP not detected".to_string()),
    }
}
