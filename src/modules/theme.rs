use crate::modules::Module;
use crate::platform;

pub fn theme_module() -> Module {
    Module {
        name: "theme",
        key: "Theme ✿",
        run: |_| {
            let info = platform::theme_info();
            info.theme.ok_or_else(|| "Theme not detected".to_string())
        },
    }
}

pub fn icons_module() -> Module {
    Module {
        name: "icons",
        key: "Icons ♡",
        run: |_| {
            let info = platform::theme_info();
            info.icons.ok_or_else(|| "Icons not detected".to_string())
        },
    }
}

pub fn font_module() -> Module {
    Module {
        name: "font",
        key: "Font ❀",
        run: |_| {
            let info = platform::theme_info();
            info.font.ok_or_else(|| "Font not detected".to_string())
        },
    }
}

pub fn cursor_module() -> Module {
    Module {
        name: "cursor",
        key: "Cursor ♡",
        run: |_| {
            let info = platform::theme_info();
            info.cursor.ok_or_else(|| "Cursor not detected".to_string())
        },
    }
}
