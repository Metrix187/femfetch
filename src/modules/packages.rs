use crate::modules::Module;
use crate::platform;

pub fn packages_module() -> Module {
    Module {
        name: "packages",
        key: "Packages ♡",
        run: |_| platform::packages().ok_or_else(|| "Packages not detected".to_string()),
        default: true,
        dynamic: false,
        gate: None,
    }
}
