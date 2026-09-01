use crate::modules::{Gate, Module};
use crate::platform;

pub fn theme_module() -> Module {
    Module {
        name: "theme",
        key: "Theme ✿",
        run: |_| {
            platform::theme_info()
                .theme
                .ok_or_else(|| "Theme not detected".to_string())
        },
        default: false,
        dynamic: false,
        gate: Some(Gate::Theme),
    }
}

pub fn icons_module() -> Module {
    Module {
        name: "icons",
        key: "Icons ♡",
        run: |_| {
            platform::theme_info()
                .icons
                .ok_or_else(|| "Icons not detected".to_string())
        },
        default: false,
        dynamic: false,
        gate: Some(Gate::Icons),
    }
}

pub fn font_module() -> Module {
    Module {
        name: "font",
        key: "Font ❀",
        run: |_| {
            platform::theme_info()
                .font
                .ok_or_else(|| "Font not detected".to_string())
        },
        default: false,
        dynamic: false,
        gate: Some(Gate::Font),
    }
}

pub fn cursor_module() -> Module {
    Module {
        name: "cursor",
        key: "Cursor ♡",
        run: |_| {
            platform::theme_info()
                .cursor
                .ok_or_else(|| "Cursor not detected".to_string())
        },
        default: false,
        dynamic: false,
        gate: Some(Gate::Cursor),
    }
}
