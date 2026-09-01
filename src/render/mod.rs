use crate::themes::Theme;
use crate::{AppConfig, ModuleResult, RunOutput};
use serde_json::json;
use std::fmt::Write;

pub mod ascii;

pub fn render_pretty(
    modules: &[ModuleResult],
    config: &AppConfig,
    theme: &Theme,
    ascii_art: Option<&str>,
) -> String {
    let width = config.terminal_width.unwrap_or_else(terminal_width);
    let mut info_lines = Vec::with_capacity(modules.len());
    for module in modules {
        let mut line = format!(
            "{} {}",
            theme.label(&module.key),
            theme.value(&module.value)
        );
        if config.speed {
            if let Some(took_ms) = module.took_ms {
                let _ = write!(line, " {}", theme.muted(&format!("({took_ms:.3} ms)")));
            }
        }
        info_lines.push(line);
    }

    if config.plain {
        return info_lines
            .iter()
            .map(|line| truncate_ansi(line, width))
            .collect::<Vec<_>>()
            .join("\n");
    }

    if config.compact {
        return render_compact(&info_lines, theme, ascii_art, width);
    }

    render_adaptive(&info_lines, theme, ascii_art, width)
}

pub fn render_json(modules: &[ModuleResult]) -> String {
    let output = RunOutput {
        version: 1,
        modules: modules.to_vec(),
    };
    serde_json::to_string_pretty(&output)
        .unwrap_or_else(|_| json!({ "version": 1, "modules": [] }).to_string())
}

pub fn resolve_ascii(config: &AppConfig, mascot: Option<&str>) -> Result<Option<String>, String> {
    if config.plain || config.json {
        return Ok(None);
    }
    if let Some(path) = &config.ascii_file {
        return std::fs::read_to_string(path)
            .map(Some)
            .map_err(|error| format!("Could not read ASCII file {path}: {error}"));
    }
    let preset = mascot.or(config.ascii_preset.as_deref()).unwrap_or("bunny");
    ascii::get_ascii(preset, &config.ascii_size)
        .map(Some)
        .ok_or_else(|| {
            format!(
                "Unknown mascot: {preset}. Available mascots: {}",
                ascii::names().collect::<Vec<_>>().join(", ")
            )
        })
}

pub fn terminal_width() -> usize {
    if let Ok(columns) = std::env::var("COLUMNS") {
        if let Ok(width) = columns.parse::<usize>() {
            if width > 0 {
                return width;
            }
        }
    }
    native_terminal_width().unwrap_or(80)
}

#[cfg(unix)]
fn native_terminal_width() -> Option<usize> {
    let mut size: libc::winsize = unsafe { std::mem::zeroed() };
    let result = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) };
    (result == 0 && size.ws_col > 0).then_some(size.ws_col as usize)
}

#[cfg(windows)]
fn native_terminal_width() -> Option<usize> {
    use windows_sys::Win32::System::Console::{
        GetConsoleScreenBufferInfo, GetStdHandle, CONSOLE_SCREEN_BUFFER_INFO, STD_OUTPUT_HANDLE,
    };
    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
    if unsafe { GetConsoleScreenBufferInfo(handle, &mut info) } == 0 {
        return None;
    }
    Some((info.srWindow.Right - info.srWindow.Left + 1) as usize)
}

fn render_adaptive(lines: &[String], theme: &Theme, art: Option<&str>, width: usize) -> String {
    let art_lines = colored_art(art, theme);
    let art_width = art_lines
        .iter()
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(0);
    let max_info = lines
        .iter()
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(0);
    let side_by_side = !art_lines.is_empty()
        && width >= 48
        && art_width.saturating_add(max_info).saturating_add(6) <= width;

    if side_by_side {
        let available = width.saturating_sub(art_width + 2).max(4);
        let box_lines = boxed_lines(lines, theme, available);
        stitch_columns(&art_lines, &box_lines, width)
    } else {
        let available = width.max(4);
        let box_lines = boxed_lines(lines, theme, available);
        if art_lines.is_empty() || width < 24 {
            box_lines.join("\n")
        } else {
            let mut output = art_lines
                .iter()
                .map(|line| truncate_ansi(line, width))
                .collect::<Vec<_>>();
            output.extend(box_lines);
            output.join("\n")
        }
    }
}

fn render_compact(lines: &[String], theme: &Theme, art: Option<&str>, width: usize) -> String {
    let art_lines = colored_art(art, theme);
    let art_width = art_lines
        .iter()
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(0);
    let side_by_side = !art_lines.is_empty() && width >= 46;
    let available = if side_by_side {
        width.saturating_sub(art_width + 2).max(1)
    } else {
        width
    };
    let info = lines
        .iter()
        .map(|line| truncate_ansi(line, available))
        .collect::<Vec<_>>();
    if side_by_side {
        stitch_columns(&art_lines, &info, width)
    } else {
        info.join("\n")
    }
}

fn colored_art(art: Option<&str>, theme: &Theme) -> Vec<String> {
    art.unwrap_or_default()
        .lines()
        .map(|line| theme.accent(line))
        .collect()
}

fn boxed_lines(lines: &[String], theme: &Theme, available: usize) -> Vec<String> {
    let inner_limit = available.saturating_sub(4).max(1);
    let max_len = lines
        .iter()
        .map(|line| visible_width(line).min(inner_limit))
        .max()
        .unwrap_or(0);
    let mut boxed = Vec::with_capacity(lines.len() + 2);
    boxed.push(theme.border(&format!("╭─{}─╮", "─".repeat(max_len))));
    for line in lines {
        let truncated = truncate_ansi(line, max_len);
        let padded = pad_to_width(&truncated, max_len);
        boxed.push(format!(
            "{} {} {}",
            theme.border("│"),
            padded,
            theme.border("│")
        ));
    }
    boxed.push(theme.border(&format!("╰─{}─╯", "─".repeat(max_len))));
    boxed
}

fn stitch_columns(left: &[String], right: &[String], width: usize) -> String {
    let left_width = left
        .iter()
        .map(|line| visible_width(line))
        .max()
        .unwrap_or(0);
    let rows = left.len().max(right.len());
    let mut lines = Vec::with_capacity(rows);
    for index in 0..rows {
        let left_line = left.get(index).map(String::as_str).unwrap_or("");
        let right_line = right.get(index).map(String::as_str).unwrap_or("");
        let left_padded = pad_to_width(left_line, left_width);
        let row = if right_line.is_empty() {
            left_padded
        } else {
            format!("{left_padded}  {right_line}")
        };
        lines.push(truncate_ansi(&row, width));
    }
    lines.join("\n")
}

pub fn visible_width(value: &str) -> usize {
    let bytes = value.as_bytes();
    let mut index = 0;
    let mut width = 0;
    while index < bytes.len() {
        if bytes[index] == 0x1b {
            index = skip_ansi(bytes, index);
            continue;
        }
        let ch = value[index..].chars().next().unwrap();
        width += char_width(ch);
        index += ch.len_utf8();
    }
    width
}

fn char_width(ch: char) -> usize {
    let code = ch as u32;
    if ch == '\0'
        || ch.is_control()
        || matches!(code, 0x0300..=0x036f | 0x1ab0..=0x1aff | 0x1dc0..=0x1dff | 0x20d0..=0x20ff | 0xfe00..=0xfe0f)
    {
        0
    } else if matches!(code, 0x1100..=0x115f | 0x2329..=0x232a | 0x2e80..=0xa4cf | 0xac00..=0xd7a3 | 0xf900..=0xfaff | 0xfe10..=0xfe19 | 0xfe30..=0xfe6f | 0xff00..=0xff60 | 0xffe0..=0xffe6 | 0x1f300..=0x1faff | 0x20000..=0x3fffd)
    {
        2
    } else {
        1
    }
}

fn skip_ansi(bytes: &[u8], start: usize) -> usize {
    if bytes.get(start + 1) == Some(&b'[') {
        let mut index = start + 2;
        while index < bytes.len() {
            let byte = bytes[index];
            index += 1;
            if (0x40..=0x7e).contains(&byte) {
                return index;
            }
        }
        bytes.len()
    } else {
        (start + 1).min(bytes.len())
    }
}

pub fn strip_ansi(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = String::with_capacity(value.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == 0x1b {
            index = skip_ansi(bytes, index);
            continue;
        }
        let ch = value[index..].chars().next().unwrap();
        output.push(ch);
        index += ch.len_utf8();
    }
    output
}

pub fn truncate_ansi(value: &str, width: usize) -> String {
    if visible_width(value) <= width {
        return value.to_string();
    }
    if width == 0 {
        return String::new();
    }
    let target = width.saturating_sub(1);
    let bytes = value.as_bytes();
    let mut output = String::with_capacity(value.len().min(width + 16));
    let mut index = 0;
    let mut used = 0;
    let mut had_ansi = false;
    while index < bytes.len() {
        if bytes[index] == 0x1b {
            let end = skip_ansi(bytes, index);
            output.push_str(&value[index..end]);
            had_ansi = true;
            index = end;
            continue;
        }
        let ch = value[index..].chars().next().unwrap();
        let ch_width = char_width(ch);
        if used + ch_width > target {
            break;
        }
        output.push(ch);
        used += ch_width;
        index += ch.len_utf8();
    }
    output.push('…');
    if had_ansi {
        output.push_str("\u{1b}[0m");
    }
    output
}

fn pad_to_width(value: &str, width: usize) -> String {
    let len = visible_width(value);
    if len >= width {
        return value.to_string();
    }
    let mut output = String::with_capacity(value.len() + width - len);
    output.push_str(value);
    output.push_str(&" ".repeat(width - len));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::themes;

    fn result(name: &str, key: &str, value: &str) -> ModuleResult {
        ModuleResult {
            name: name.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            took_ms: Some(0.125),
            error: None,
        }
    }

    #[test]
    fn measures_ansi_and_wide_unicode() {
        assert_eq!(visible_width("\u{1b}[31mhello\u{1b}[0m"), 5);
        assert_eq!(visible_width("猫a"), 3);
        assert_eq!(strip_ansi("a\u{1b}[2Kb"), "ab");
    }

    #[test]
    fn truncates_without_splitting_unicode_or_ansi() {
        let value = truncate_ansi("\u{1b}[31m猫abcdef\u{1b}[0m", 5);
        assert!(visible_width(&value) <= 5);
        assert!(strip_ansi(&value).ends_with('…'));
    }

    #[test]
    fn narrow_layout_never_exceeds_width() {
        let modules = [result("cpu", "CPU ✿", "A very long processor description")];
        let config = AppConfig {
            terminal_width: Some(20),
            ..AppConfig::default()
        };
        let theme = themes::get_theme("pastel", true).unwrap();
        let output = render_pretty(&modules, &config, &theme, Some(" /\\_/\\"));
        assert!(output.lines().all(|line| visible_width(line) <= 20));
        assert!(!output.contains("/\\_/\\"));
    }

    #[test]
    fn wide_layout_keeps_columns_side_by_side() {
        let modules = [result("os", "OS ✿", "Linux")];
        let config = AppConfig {
            terminal_width: Some(100),
            ..AppConfig::default()
        };
        let theme = themes::get_theme("pastel", true).unwrap();
        let output = render_pretty(&modules, &config, &theme, Some("wolf"));
        assert!(output.lines().next().unwrap().contains('╭'));
    }

    #[test]
    fn json_has_stable_version_and_parses() {
        let parsed: serde_json::Value =
            serde_json::from_str(&render_json(&[result("os", "OS ✿", "Linux")])).unwrap();
        assert_eq!(parsed["version"], 1);
        assert_eq!(parsed["modules"][0]["name"], "os");
    }
}
