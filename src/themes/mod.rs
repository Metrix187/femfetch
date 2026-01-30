#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    no_color: bool,
    accent: &'static str,
    label: &'static str,
    value: &'static str,
    border: &'static str,
    muted: &'static str,
}

impl Theme {
    pub fn accent(&self, text: &str) -> String {
        self.paint(self.accent, text)
    }

    pub fn label(&self, text: &str) -> String {
        self.paint(self.label, text)
    }

    pub fn value(&self, text: &str) -> String {
        self.paint(self.value, text)
    }

    pub fn border(&self, text: &str) -> String {
        self.paint(self.border, text)
    }

    pub fn muted(&self, text: &str) -> String {
        self.paint(self.muted, text)
    }

    fn paint(&self, color: &str, text: &str) -> String {
        if self.no_color {
            text.to_string()
        } else {
            format!("{color}{text}\u{1b}[0m")
        }
    }
}

pub fn get_theme(name: &str, no_color: bool) -> Theme {
    let name = name.to_lowercase();
    match name.as_str() {
        "mint" => Theme {
            name: "mint",
            no_color,
            accent: "\u{1b}[38;2;176;255;227m",
            label: "\u{1b}[38;2;166;224;212m",
            value: "\u{1b}[38;2;240;255;250m",
            border: "\u{1b}[38;2;183;255;200m",
            muted: "\u{1b}[38;2;176;198;192m",
        },
        "sunset" => Theme {
            name: "sunset",
            no_color,
            accent: "\u{1b}[38;2;255;186;209m",
            label: "\u{1b}[38;2;255;208;170m",
            value: "\u{1b}[38;2;255;245;230m",
            border: "\u{1b}[38;2;255;198;214m",
            muted: "\u{1b}[38;2;231;175;186m",
        },
        _ => Theme {
            name: "pastel",
            no_color,
            accent: "\u{1b}[38;2;203;189;255m",
            label: "\u{1b}[38;2;255;209;227m",
            value: "\u{1b}[38;2;245;245;255m",
            border: "\u{1b}[38;2;198;199;255m",
            muted: "\u{1b}[38;2;199;199;214m",
        },
    }
}
