use crate::AppConfig;
use std::path::{Path, PathBuf};

pub fn default_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|base| base.join("femfetch").join("config.toml"))
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .map(|base| base.join("femfetch").join("config.toml"))
    }
}

pub fn load(path: &Path) -> Result<Option<AppConfig>, String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => parse(&contents).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("could not read {}: {error}", path.display())),
    }
}

pub fn parse(contents: &str) -> Result<AppConfig, String> {
    let mut config = AppConfig::default();
    let mut section = String::new();

    for (index, raw_line) in contents.lines().enumerate() {
        let line_number = index + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_ascii_lowercase();
            if !matches!(section.as_str(), "presentation" | "collection" | "watch") {
                return Err(format!("line {line_number}: unknown section [{section}]"));
            }
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {line_number}: expected key = value"))?;
        let key = key.trim().replace('_', "-").to_ascii_lowercase();
        let full_key = if section.is_empty() {
            key
        } else {
            format!("{section}.{key}")
        };
        let value = value.trim();

        match full_key.as_str() {
            "theme" | "presentation.theme" => config.theme = parse_string(value, line_number)?,
            "mascot" | "ascii" | "presentation.mascot" | "presentation.ascii" => {
                config.ascii_preset = optional_string(value, line_number)?
            }
            "ascii-file" | "presentation.ascii-file" => {
                config.ascii_file = optional_string(value, line_number)?
            }
            "ascii-size" | "presentation.ascii-size" => {
                let size = parse_string(value, line_number)?;
                if !matches!(size.as_str(), "small" | "medium" | "large") {
                    return Err(format!(
                        "line {line_number}: ascii-size must be small, medium, or large"
                    ));
                }
                config.ascii_size = size;
            }
            "plain" | "presentation.plain" => config.plain = parse_bool(value, line_number)?,
            "compact" | "presentation.compact" => config.compact = parse_bool(value, line_number)?,
            "no-color" | "presentation.no-color" => {
                config.no_color = parse_bool(value, line_number)?
            }
            "mascot-state" | "presentation.mascot-state" => {
                config.mascot_state = parse_bool(value, line_number)?
            }
            "modules" | "collection.modules" => {
                config.modules = Some(parse_list(value, line_number)?)
            }
            "all-disks" | "collection.all-disks" => {
                config.all_disks = parse_bool(value, line_number)?
            }
            "local-ip" | "collection.local-ip" => config.local_ip = parse_bool(value, line_number)?,
            "show-theme" | "collection.show-theme" => {
                config.show_theme = parse_bool(value, line_number)?
            }
            "show-icons" | "collection.show-icons" => {
                config.show_icons = parse_bool(value, line_number)?
            }
            "show-font" | "collection.show-font" => {
                config.show_font = parse_bool(value, line_number)?
            }
            "show-cursor" | "collection.show-cursor" => {
                config.show_cursor = parse_bool(value, line_number)?
            }
            "cache" | "collection.cache" => {
                config.cache_ttl_seconds = optional_u64(value, line_number)?
            }
            "interval" | "watch.interval" => {
                config.watch_interval_seconds = parse_positive_f64(value, line_number)?
            }
            _ => return Err(format!("line {line_number}: unknown option {full_key}")),
        }
    }

    Ok(config)
}

fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    let mut escaped = false;
    for (index, byte) in line.bytes().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' if quoted => escaped = true,
            b'"' => quoted = !quoted,
            b'#' if !quoted => return &line[..index],
            _ => {}
        }
    }
    line
}

fn parse_string(value: &str, line: usize) -> Result<String, String> {
    let value = value.trim();
    if value.starts_with('"') {
        if value.len() < 2 || !value.ends_with('"') {
            return Err(format!("line {line}: unterminated string"));
        }
        let inner = &value[1..value.len() - 1];
        if inner.contains('"') || inner.contains('\\') {
            return Err(format!("line {line}: escapes are not supported in strings"));
        }
        return Ok(inner.to_string());
    }
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err(format!("line {line}: expected a quoted string"));
    }
    Ok(value.to_string())
}

fn optional_string(value: &str, line: usize) -> Result<Option<String>, String> {
    if value.eq_ignore_ascii_case("none") {
        Ok(None)
    } else {
        parse_string(value, line).map(Some)
    }
}

fn parse_bool(value: &str, line: usize) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("line {line}: expected true or false")),
    }
}

fn optional_u64(value: &str, line: usize) -> Result<Option<u64>, String> {
    if matches!(value, "false" | "none") {
        return Ok(None);
    }
    value
        .parse::<u64>()
        .map(Some)
        .map_err(|_| format!("line {line}: expected a non-negative integer, false, or none"))
}

fn parse_positive_f64(value: &str, line: usize) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| format!("line {line}: expected a positive number"))?;
    if parsed.is_finite() && parsed > 0.0 {
        Ok(parsed)
    } else {
        Err(format!("line {line}: expected a positive number"))
    }
}

fn parse_list(value: &str, line: usize) -> Result<Vec<String>, String> {
    let value = value.trim();
    if value.starts_with('[') {
        if !value.ends_with(']') {
            return Err(format!("line {line}: unterminated list"));
        }
        let inner = &value[1..value.len() - 1];
        if inner.trim().is_empty() {
            return Ok(Vec::new());
        }
        return inner
            .split(',')
            .map(|item| parse_string(item.trim(), line))
            .collect();
    }
    Ok(parse_string(value, line)?
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sections_and_lists() {
        let config = parse(
            r#"
            [presentation]
            theme = "mint"
            compact = true
            mascot = "fox"
            [collection]
            modules = ["os", "cpu", "memory"]
            local_ip = true
            [watch]
            interval = 0.5
            "#,
        )
        .unwrap();

        assert_eq!(config.theme, "mint");
        assert!(config.compact);
        assert_eq!(config.ascii_preset.as_deref(), Some("fox"));
        assert_eq!(config.modules.unwrap(), ["os", "cpu", "memory"]);
        assert!(config.local_ip);
        assert_eq!(config.watch_interval_seconds, 0.5);
    }

    #[test]
    fn rejects_unknown_and_malformed_values() {
        assert!(parse("wat = true").unwrap_err().contains("unknown option"));
        assert!(parse("compact = maybe")
            .unwrap_err()
            .contains("true or false"));
        assert!(parse("[wat]\nfoo = true")
            .unwrap_err()
            .contains("unknown section"));
    }

    #[test]
    fn strips_comments_only_outside_quotes() {
        let config = parse("theme = \"mint#night\" # note").unwrap();
        assert_eq!(config.theme, "mint#night");
    }
}
