use crate::modules::Module;
use crate::platform;

pub fn shell_module() -> Module {
    Module {
        name: "shell",
        key: "Shell ✿",
        run: |_| platform::shell().ok_or_else(|| "Shell not detected".to_string()),
    }
}

pub fn resolution_module() -> Module {
    Module {
        name: "resolution",
        key: "Resolution ♡",
        run: |_| {
            platform::resolutions().ok_or_else(|| "Resolution not detected".to_string())
        },
    }
}

pub fn de_wm_module() -> Module {
    Module {
        name: "dewm",
        key: "DE / WM ✿",
        run: |_| platform::de_wm().ok_or_else(|| "DE/WM not detected".to_string()),
    }
}

pub fn terminal_module() -> Module {
    Module {
        name: "terminal",
        key: "Terminal ❀",
        run: |_| platform::terminal().ok_or_else(|| "Terminal not detected".to_string()),
    }
}
