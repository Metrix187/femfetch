use crate::themes::Theme;
use crate::{AppConfig, ModuleResult, RunOutput};
use serde_json::json;

pub mod ascii;

pub fn render_pretty(
    modules: &[ModuleResult],
    config: &AppConfig,
    theme: &Theme,
    ascii_art: Option<String>,
) -> String {
    let mut info_lines = Vec::new();
    for module in modules {
        let mut line = format!("{} {}", theme.label(&module.key), theme.value(&module.value));
        if config.speed {
            line.push_str(&format!(" {}", theme.muted(&format!("({} ms)", module.took_ms))));
        }
        info_lines.push(line);
    }

    if config.plain {
        return info_lines.join("\n");
    }

    let box_lines = boxed_lines(&info_lines, theme);
    let left_lines: Vec<String> = ascii_art
        .unwrap_or_default()
        .lines()
        .map(|line| theme.accent(line))
        .collect();
    stitch_columns(&left_lines, &box_lines)
}

pub fn render_json(modules: &[ModuleResult]) -> String {
    let output = RunOutput {
        modules: modules.to_vec(),
    };
    serde_json::to_string_pretty(&output).unwrap_or_else(|_| json!({ "modules": [] }).to_string())
}

pub fn resolve_ascii(config: &AppConfig) -> Option<String> {
    if config.plain || config.json {
        return None;
    }
    if let Some(path) = &config.ascii_file {
        if let Ok(contents) = std::fs::read_to_string(path) {
            return Some(contents);
        }
    }
    let preset = config
        .ascii_preset
        .clone()
        .unwrap_or_else(|| "bunny".to_string());
    ascii::get_ascii(&preset, &config.ascii_size)
}

fn boxed_lines(lines: &[String], theme: &Theme) -> Vec<String> {
    let mut max_len = 0usize;
    for line in lines {
        let len = visible_len(line);
        if len > max_len {
            max_len = len;
        }
    }
    let top = format!("╭─{}─╮", "─".repeat(max_len));
    let bottom = format!("╰─{}─╯", "─".repeat(max_len));
    let mut boxed = Vec::new();
    boxed.push(theme.border(&top));
    for line in lines {
        let padded = pad_to_width(line, max_len);
        boxed.push(theme.border("│") + " " + &padded + " " + &theme.border("│"));
    }
    boxed.push(theme.border(&bottom));
    boxed
}

fn stitch_columns(left: &[String], right: &[String]) -> String {
    let left_width = left.iter().map(|s| visible_len(s)).max().unwrap_or(0);
    let rows = left.len().max(right.len());
    let mut lines = Vec::new();
    for i in 0..rows {
        let left_line = left.get(i).cloned().unwrap_or_default();
        let right_line = right.get(i).cloned().unwrap_or_default();
        let left_padded = pad_to_width(&left_line, left_width);
        if right_line.is_empty() {
            lines.push(left_padded);
        } else {
            lines.push(format!("{left_padded}  {right_line}"));
        }
    }
    lines.join("\n")
}

fn visible_len(value: &str) -> usize {
    strip_ansi(value).chars().count()
}

fn pad_to_width(value: &str, width: usize) -> String {
    let len = visible_len(value);
    if len >= width {
        return value.to_string();
    }
    let mut out = value.to_string();
    out.push_str(&" ".repeat(width - len));
    out
}

fn strip_ansi(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            while let Some(next) = chars.next() {
                if next == 'm' {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}
