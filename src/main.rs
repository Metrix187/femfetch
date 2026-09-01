use femfetch::{cache, config, export, modules, render, signals, themes, AppConfig, ModuleResult};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
struct Cli {
    plain: Option<bool>,
    compact: Option<bool>,
    json: bool,
    no_color: Option<bool>,
    theme: Option<String>,
    ascii: Option<String>,
    ascii_file: Option<String>,
    ascii_size: Option<AsciiSize>,
    mascot_state: bool,
    speed: bool,
    modules: Option<String>,
    cache: Option<u64>,
    all_disks: Option<bool>,
    local_ip: Option<bool>,
    show_theme: Option<bool>,
    show_icons: Option<bool>,
    show_font: Option<bool>,
    show_cursor: Option<bool>,
    watch: Option<f64>,
    benchmark: Option<usize>,
    prompt: bool,
    export: Option<PathBuf>,
    config: Option<PathBuf>,
    no_config: bool,
    config_path: bool,
    list_modules: bool,
    list_themes: bool,
    list_mascots: bool,
    shell_init: bool,
}

#[derive(Debug, Clone, Copy)]
enum AsciiSize {
    Small,
    Medium,
    Large,
}

impl Cli {
    fn parse() -> Result<Self, String> {
        Self::parse_from(std::env::args().skip(1))
    }

    fn parse_from(args: impl IntoIterator<Item = impl Into<String>>) -> Result<Self, String> {
        let mut cli = Self::default();
        let mut args = args.into_iter().map(Into::into).peekable();
        while let Some(argument) = args.next() {
            let (flag, inline) = argument
                .split_once('=')
                .map_or((argument.as_str(), None), |(flag, value)| {
                    (flag, Some(value))
                });
            match flag {
                "-h" | "--help" => {
                    print!("{HELP}");
                    std::process::exit(0);
                }
                "-V" | "--version" => {
                    println!("femfetch {}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                "--plain" => cli.plain = Some(true),
                "--compact" => cli.compact = Some(true),
                "--json" => cli.json = true,
                "--no-color" => cli.no_color = Some(true),
                "--mascot-state" => cli.mascot_state = true,
                "--speed" => cli.speed = true,
                "--all-disks" => cli.all_disks = Some(true),
                "--local-ip" => cli.local_ip = Some(true),
                "--show-theme" => cli.show_theme = Some(true),
                "--show-icons" => cli.show_icons = Some(true),
                "--show-font" => cli.show_font = Some(true),
                "--show-cursor" => cli.show_cursor = Some(true),
                "--prompt" => cli.prompt = true,
                "--no-config" => cli.no_config = true,
                "--config-path" => cli.config_path = true,
                "--list-modules" => cli.list_modules = true,
                "--list-themes" => cli.list_themes = true,
                "--list-mascots" => cli.list_mascots = true,
                "--shell-init" => cli.shell_init = true,
                "--theme" => cli.theme = Some(required_value(flag, inline, &mut args)?),
                "--ascii" | "--mascot" => {
                    cli.ascii = Some(required_value(flag, inline, &mut args)?)
                }
                "--ascii-file" => cli.ascii_file = Some(required_value(flag, inline, &mut args)?),
                "--modules" => cli.modules = Some(required_value(flag, inline, &mut args)?),
                "--export" => cli.export = Some(required_value(flag, inline, &mut args)?.into()),
                "--config" => cli.config = Some(required_value(flag, inline, &mut args)?.into()),
                "--ascii-size" => {
                    let value = required_value(flag, inline, &mut args)?;
                    cli.ascii_size = Some(match value.as_str() {
                        "small" => AsciiSize::Small,
                        "medium" => AsciiSize::Medium,
                        "large" => AsciiSize::Large,
                        _ => return Err(format!("invalid value '{value}' for --ascii-size; expected small, medium, or large")),
                    });
                }
                "--cache" => {
                    cli.cache = Some(
                        optional_value(inline, &mut args)
                            .map_or(Ok(10), |value| parse_cli_number(flag, value))?,
                    )
                }
                "--watch" => {
                    cli.watch = Some(
                        optional_value(inline, &mut args)
                            .map_or(Ok(1.0), |value| parse_cli_number(flag, value))?,
                    )
                }
                "--benchmark" => {
                    cli.benchmark = Some(
                        optional_value(inline, &mut args)
                            .map_or(Ok(25), |value| parse_cli_number(flag, value))?,
                    )
                }
                _ if flag.starts_with('-') => {
                    return Err(format!("unknown option: {flag}; try --help"))
                }
                _ => return Err(format!("unexpected argument: {argument}; try --help")),
            }
        }
        Ok(cli)
    }
}

fn required_value<I>(
    flag: &str,
    inline: Option<&str>,
    args: &mut std::iter::Peekable<I>,
) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    inline
        .map(str::to_string)
        .or_else(|| args.next().filter(|value| !value.starts_with('-')))
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn optional_value<I>(inline: Option<&str>, args: &mut std::iter::Peekable<I>) -> Option<String>
where
    I: Iterator<Item = String>,
{
    if let Some(value) = inline {
        return Some(value.to_string());
    }
    if args.peek().is_some_and(|value| !value.starts_with('-')) {
        args.next()
    } else {
        None
    }
}

fn parse_cli_number<T>(flag: &str, value: String) -> Result<T, String>
where
    T: std::str::FromStr,
{
    value
        .parse()
        .map_err(|_| format!("invalid value '{value}' for {flag}"))
}

const HELP: &str = r#"Cute, fast, native system info.

Usage: femfetch [OPTIONS]

Output:
      --plain                    Print undecorated module lines
      --compact                  Use the compact adaptive layout
      --json                     Emit stable JSON schema version 1
      --prompt                   Print a prompt-friendly one-line summary
      --watch [SECONDS]          Refresh dynamic modules in place [default: 1]
      --export <FILE>            Write a self-contained HTML report

Presentation:
      --no-color                 Disable ANSI colors
      --theme <THEME>            Select pastel, mint, or sunset
      --ascii, --mascot <NAME>   Select a built-in mascot
      --ascii-file <FILE>        Read custom ASCII art
      --ascii-size <SIZE>        Select small, medium, or large
      --mascot-state             Select static art from system state

Collection:
      --modules <LIST>           Select and order comma-separated modules
      --cache [SECONDS]          Cache module results [default: 10]
      --all-disks                Aggregate local filesystems
      --local-ip                 Permit the local-ip module
      --show-theme               Permit the theme module
      --show-icons               Permit the icons module
      --show-font                Permit the font module
      --show-cursor              Permit the cursor module

Development and discovery:
      --speed                    Show sub-millisecond module timings
      --benchmark [RUNS]         Benchmark collection and rendering [default: 25]
      --config <FILE>            Use an alternate configuration file
      --no-config                Ignore persistent configuration
      --config-path              Print the default configuration path
      --list-modules             List available module names
      --list-themes              List built-in themes
      --list-mascots             List built-in mascots
      --shell-init               Print shell integration examples
  -h, --help                     Print help
  -V, --version                  Print version
"#;

fn main() {
    if let Err(error) = run() {
        eprintln!("femfetch: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let startup = Instant::now();
    let cli = Cli::parse()?;

    if cli.config_path {
        let path = cli
            .config
            .clone()
            .or_else(config::default_path)
            .ok_or_else(|| "configuration directory unavailable".to_string())?;
        println!("{}", path.display());
        return Ok(());
    }
    if cli.list_modules {
        println!("{}", modules::module_names().join("\n"));
        return Ok(());
    }
    if cli.list_themes {
        println!("{}", themes::names().join("\n"));
        return Ok(());
    }
    if cli.list_mascots {
        println!("{}", render::ascii::names().collect::<Vec<_>>().join("\n"));
        return Ok(());
    }
    if cli.shell_init {
        println!("# Add this where your shell builds its prompt; femfetch never edits shell files.\n# bash: PS1=\"$(femfetch --prompt) $PS1\"\n# zsh:  PROMPT='$(femfetch --prompt) %~ %# '");
        return Ok(());
    }

    let mut app = load_config(&cli)?;
    apply_cli(&mut app, &cli);
    validate_options(&app, &cli)?;

    if cli.prompt {
        println!("{}", render_prompt(&app)?);
        return Ok(());
    }

    let mut selected = modules::resolve_modules(app.modules.as_deref())?;
    if app.modules.is_none() {
        modules::append_enabled_optional(&mut selected, &app);
    }
    modules::validate_gates(&selected, &app)?;
    let visible_module_count = selected.len();
    append_state_modules(&mut selected, &app)?;

    if let Some(runs) = cli.benchmark {
        return run_benchmark(&selected, &app, runs, startup);
    }
    if let Some(interval) = cli.watch {
        app.watch_interval_seconds = interval;
        return run_watch(&selected, &app, visible_module_count);
    }

    let collection_start = Instant::now();
    let results = collect(&selected, &app);
    let collection = collection_start.elapsed();
    let visible_results = &results[..visible_module_count];

    if let Some(path) = &cli.export {
        export::write_html(path, visible_results)?;
        println!("Wrote {}", path.display());
        return Ok(());
    }

    let render_start = Instant::now();
    let output = render_output(visible_results, &app, Some(&results))?;
    let rendering = render_start.elapsed();
    println!("{output}");
    if app.speed && !app.json {
        eprintln!(
            "femfetch: collect {:.3} ms · render {:.3} ms · total {:.3} ms",
            collection.as_secs_f64() * 1_000.0,
            rendering.as_secs_f64() * 1_000.0,
            startup.elapsed().as_secs_f64() * 1_000.0
        );
    }
    Ok(())
}

fn load_config(cli: &Cli) -> Result<AppConfig, String> {
    if cli.no_config {
        return Ok(AppConfig::default());
    }
    let path = cli.config.clone().or_else(config::default_path);
    match path {
        Some(path) => config::load(&path)
            .map_err(|error| format!("config {}: {error}", path.display()))
            .map(|loaded| loaded.unwrap_or_default()),
        None => Ok(AppConfig::default()),
    }
}

fn apply_cli(config: &mut AppConfig, cli: &Cli) {
    if let Some(value) = cli.plain {
        config.plain = value;
    }
    if let Some(value) = cli.compact {
        config.compact = value;
    }
    if let Some(value) = cli.no_color {
        config.no_color = value;
    }
    if let Some(value) = &cli.theme {
        config.theme.clone_from(value);
    }
    if let Some(value) = &cli.ascii {
        config.ascii_preset = Some(value.clone());
    }
    if let Some(value) = &cli.ascii_file {
        config.ascii_file = Some(value.clone());
    }
    if let Some(value) = cli.ascii_size {
        config.ascii_size = match value {
            AsciiSize::Small => "small",
            AsciiSize::Medium => "medium",
            AsciiSize::Large => "large",
        }
        .to_string();
    }
    if let Some(value) = &cli.modules {
        config.modules = Some(
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
        );
    }
    if let Some(value) = cli.cache {
        config.cache_ttl_seconds = Some(value);
    }
    if let Some(value) = cli.all_disks {
        config.all_disks = value;
    }
    if let Some(value) = cli.local_ip {
        config.local_ip = value;
    }
    if let Some(value) = cli.show_theme {
        config.show_theme = value;
    }
    if let Some(value) = cli.show_icons {
        config.show_icons = value;
    }
    if let Some(value) = cli.show_font {
        config.show_font = value;
    }
    if let Some(value) = cli.show_cursor {
        config.show_cursor = value;
    }
    config.json = cli.json;
    config.speed = cli.speed;
    config.mascot_state |= cli.mascot_state;
    config.no_color |=
        cli.json || !io::stdout().is_terminal() || std::env::var_os("NO_COLOR").is_some();
    if std::env::var("TERM").is_ok_and(|term| term == "dumb") {
        config.no_color = true;
    }
}

fn validate_options(config: &AppConfig, cli: &Cli) -> Result<(), String> {
    themes::get_theme(&config.theme, true)?;
    if let Some(name) = &config.ascii_preset {
        if !render::ascii::contains(name) {
            return Err(format!(
                "Unknown mascot: {name}. Available mascots: {}",
                render::ascii::names().collect::<Vec<_>>().join(", ")
            ));
        }
    }
    if config.plain && config.compact {
        return Err("--plain and --compact cannot be used together".to_string());
    }
    if cli.json && (cli.watch.is_some() || cli.export.is_some()) {
        return Err("--json cannot be combined with --watch or --export".to_string());
    }
    if let Some(interval) = cli.watch {
        if !interval.is_finite() || interval <= 0.0 {
            return Err("watch interval must be positive".to_string());
        }
    }
    if cli.benchmark == Some(0) {
        return Err("benchmark runs must be at least 1".to_string());
    }
    Ok(())
}

fn collect(selected: &[modules::Module], config: &AppConfig) -> Vec<ModuleResult> {
    let key = build_cache_key(config, selected);
    if let Some(ttl) = config.cache_ttl_seconds {
        if let Some(results) = cache::load_cache(&key, ttl) {
            return results;
        }
        let results = modules::run_modules(selected, config);
        let _ = cache::write_cache(&key, &results);
        results
    } else {
        modules::run_modules(selected, config)
    }
}

fn render_output(
    results: &[ModuleResult],
    config: &AppConfig,
    state_results: Option<&[ModuleResult]>,
) -> Result<String, String> {
    if config.json {
        return Ok(render::render_json(results));
    }
    let theme = themes::get_theme(&config.theme, config.no_color)?;
    let fixed_art = resolve_fixed_ascii(config)?;
    render_prepared(results, config, state_results, &theme, fixed_art.as_deref())
}

fn resolve_fixed_ascii(config: &AppConfig) -> Result<Option<String>, String> {
    if config.mascot_state && config.ascii_file.is_none() && !config.plain && !config.json {
        Ok(None)
    } else {
        render::resolve_ascii(config, None)
    }
}

fn render_prepared(
    results: &[ModuleResult],
    config: &AppConfig,
    state_results: Option<&[ModuleResult]>,
    theme: &themes::Theme,
    fixed_art: Option<&str>,
) -> Result<String, String> {
    if config.json {
        return Ok(render::render_json(results));
    }
    let state_art;
    let art = if config.mascot_state && config.ascii_file.is_none() && !config.plain {
        let mascot = mascot_for_state(state_results.unwrap_or(results));
        state_art = render::resolve_ascii(config, mascot)?;
        state_art.as_deref()
    } else {
        fixed_art
    };
    Ok(render::render_pretty(results, config, theme, art))
}

fn append_state_modules(
    selected: &mut Vec<modules::Module>,
    config: &AppConfig,
) -> Result<(), String> {
    if !config.mascot_state {
        return Ok(());
    }
    let existing: Vec<&str> = selected.iter().map(|module| module.name).collect();
    let state_names = ["cpu-temp", "battery", "network"];
    let missing: Vec<String> = state_names
        .iter()
        .filter(|name| !existing.contains(name))
        .map(|name| (*name).to_string())
        .collect();
    selected.extend(modules::resolve_modules(Some(&missing))?);
    Ok(())
}

fn mascot_for_state(results: &[ModuleResult]) -> Option<&'static str> {
    let value = |name: &str| {
        results
            .iter()
            .find(|result| result.name == name)
            .map(|result| result.value.as_str())
    };
    if value("cpu-temp")
        .and_then(parse_number)
        .is_some_and(|temp| temp >= 80.0)
    {
        return Some("hot-wolf");
    }
    if value("battery")
        .and_then(parse_number)
        .is_some_and(|capacity| capacity <= 15.0)
    {
        return Some("sleepy-wolf");
    }
    if value("memory")
        .and_then(memory_percent)
        .is_some_and(|percent| percent >= 90.0)
    {
        return Some("alert-wolf");
    }
    if results
        .iter()
        .any(|result| result.name == "network" && result.error.is_some())
    {
        return Some("alert-wolf");
    }
    None
}

fn parse_number(value: &str) -> Option<f64> {
    value
        .split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .find(|part| !part.is_empty())?
        .parse()
        .ok()
}

fn memory_percent(value: &str) -> Option<f64> {
    let mut parts = value.split('/');
    let used = parse_size(parts.next()?.trim())?;
    let total = parse_size(parts.next()?.trim())?;
    (total > 0.0).then_some(used / total * 100.0)
}

fn parse_size(value: &str) -> Option<f64> {
    let mut fields = value.split_whitespace();
    let amount: f64 = fields.next()?.parse().ok()?;
    let multiplier = match fields.next()? {
        "TiB" => 1024.0,
        "GiB" => 1.0,
        "MiB" => 1.0 / 1024.0,
        "KiB" => 1.0 / 1_048_576.0,
        _ => return None,
    };
    Some(amount * multiplier)
}

fn run_watch(
    selected: &[modules::Module],
    config: &AppConfig,
    visible_module_count: usize,
) -> Result<(), String> {
    if !io::stdout().is_terminal() {
        return Err("--watch requires an interactive terminal".to_string());
    }
    let dynamic: Vec<_> = selected
        .iter()
        .copied()
        .filter(|module| module.dynamic)
        .collect();
    let mut results = modules::run_modules(selected, config);
    let delay = Duration::from_secs_f64(config.watch_interval_seconds);
    let theme = themes::get_theme(&config.theme, config.no_color)?;
    let fixed_art = resolve_fixed_ascii(config)?;
    signals::install();
    let mut stdout = io::stdout().lock();
    write!(stdout, "\u{1b}[?25l").map_err(|error| error.to_string())?;
    let outcome = (|| {
        while !signals::interrupted() {
            let frame = render_prepared(
                &results[..visible_module_count],
                config,
                Some(&results),
                &theme,
                fixed_art.as_deref(),
            )?;
            writeln!(stdout, "\u{1b}[H\u{1b}[2J{frame}").map_err(|error| error.to_string())?;
            stdout.flush().map_err(|error| error.to_string())?;
            if sleep_until_interrupted(delay) {
                break;
            }
            if !dynamic.is_empty() {
                modules::merge_dynamic(&mut results, modules::run_modules(&dynamic, config));
            }
        }
        Ok(())
    })();
    let restored = write!(stdout, "\u{1b}[?25h")
        .and_then(|()| stdout.flush())
        .map_err(|error| error.to_string());
    outcome.and(restored)
}

fn sleep_until_interrupted(delay: Duration) -> bool {
    let deadline = Instant::now() + delay;
    while !signals::interrupted() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return false;
        }
        std::thread::sleep(remaining.min(Duration::from_millis(100)));
    }
    true
}

fn run_benchmark(
    selected: &[modules::Module],
    config: &AppConfig,
    runs: usize,
    startup: Instant,
) -> Result<(), String> {
    let mut config = config.clone();
    config.cache_ttl_seconds = None;
    let theme = themes::get_theme(&config.theme, config.no_color)?;
    let fixed_art = resolve_fixed_ascii(&config)?;
    let mut collect_times = Vec::with_capacity(runs);
    let mut render_times = Vec::with_capacity(runs);
    let mut module_totals = vec![0.0; selected.len()];
    for _ in 0..runs {
        let started = Instant::now();
        let results = modules::run_modules(selected, &config);
        collect_times.push(started.elapsed().as_secs_f64() * 1_000.0);
        for (index, result) in results.iter().enumerate() {
            module_totals[index] += result.took_ms.unwrap_or(0.0);
        }
        let started = Instant::now();
        let _ = render_prepared(&results, &config, None, &theme, fixed_art.as_deref())?;
        render_times.push(started.elapsed().as_secs_f64() * 1_000.0);
    }
    collect_times.sort_by(f64::total_cmp);
    render_times.sort_by(f64::total_cmp);
    println!("femfetch benchmark ({runs} runs, cache disabled)");
    println!(
        "collection median {:.3} ms · p95 {:.3} ms",
        median(&collect_times),
        percentile(&collect_times, 0.95)
    );
    println!(
        "rendering  median {:.3} ms · p95 {:.3} ms",
        median(&render_times),
        percentile(&render_times, 0.95)
    );
    println!(
        "startup to benchmark completion {:.3} ms",
        startup.elapsed().as_secs_f64() * 1_000.0
    );
    println!("module means:");
    for (module, total) in selected.iter().zip(module_totals) {
        println!("  {:<18} {:.3} ms", module.name, total / runs as f64);
    }
    Ok(())
}

fn median(values: &[f64]) -> f64 {
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}

fn percentile(values: &[f64], percentile: f64) -> f64 {
    values[((values.len() as f64 * percentile).ceil() as usize)
        .saturating_sub(1)
        .min(values.len() - 1)]
}

fn render_prompt(config: &AppConfig) -> Result<String, String> {
    let requested = ["os", "host", "cpu", "memory"].map(str::to_string);
    let selected = modules::resolve_modules(Some(&requested))?;
    let results = modules::run_modules(&selected, config);
    let value = |name: &str| {
        results
            .iter()
            .find(|result| result.name == name && result.error.is_none())
            .map(|result| result.value.as_str())
    };
    let host = value("host")
        .and_then(|value| value.split_whitespace().next())
        .unwrap_or("unknown");
    let os = value("os").unwrap_or("N/A");
    let cpu = value("cpu")
        .map(|value| value.split(" (").next().unwrap_or(value))
        .unwrap_or("N/A");
    let memory = value("memory")
        .and_then(|value| value.split('/').nth(1))
        .map(str::trim)
        .unwrap_or("N/A");
    Ok(format!("♡ {host} · {os} · {cpu} · {memory}"))
}

fn build_cache_key(config: &AppConfig, selected: &[modules::Module]) -> String {
    let names = selected
        .iter()
        .map(|module| module.name)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "v2|modules={names}|all-disks={}|local-ip={}",
        config.all_disks, config.local_ip
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_overrides_config_fields() {
        let cli = Cli::parse_from(["--theme", "mint", "--compact", "--modules", "os,cpu"]).unwrap();
        let mut config = AppConfig {
            theme: "sunset".to_string(),
            ..AppConfig::default()
        };
        apply_cli(&mut config, &cli);
        assert_eq!(config.theme, "mint");
        assert!(config.compact);
        assert_eq!(config.modules.unwrap(), ["os", "cpu"]);
    }

    #[test]
    fn parses_memory_percentage() {
        assert_eq!(memory_percent("9.0 GiB / 10.0 GiB"), Some(90.0));
    }

    #[test]
    fn statistics_are_deterministic() {
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), 2.5);
        assert_eq!(percentile(&[1.0, 2.0, 3.0, 4.0], 0.95), 4.0);
    }
}
