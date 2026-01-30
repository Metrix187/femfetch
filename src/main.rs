// Rust choice: single static binary, strong cross-platform stdlib, and fast IO for system reads.
use clap::{Parser, ValueEnum};
use femfetch::cache;
use femfetch::modules;
use femfetch::render;
use femfetch::themes;
use femfetch::AppConfig;

#[derive(Debug, Parser)]
#[command(name = "femfetch", version, about = "Cute, fast, neofetch-like system info.")]
struct Cli {
    #[arg(long)]
    plain: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    no_color: bool,
    #[arg(long, default_value = "pastel")]
    theme: String,
    #[arg(long)]
    ascii: Option<String>,
    #[arg(long)]
    ascii_file: Option<String>,
    #[arg(long, default_value = "medium")]
    ascii_size: AsciiSize,
    #[arg(long)]
    speed: bool,
    #[arg(long)]
    modules: Option<String>,
    #[arg(long, num_args = 0..=1, value_name = "SECONDS", default_missing_value = "10")]
    cache: Option<u64>,
    #[arg(long)]
    all_disks: bool,
    #[arg(long)]
    local_ip: bool,
    #[arg(long)]
    show_theme: bool,
    #[arg(long)]
    show_icons: bool,
    #[arg(long)]
    show_font: bool,
    #[arg(long)]
    show_cursor: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum AsciiSize {
    Small,
    Medium,
    Large,
}

fn main() {
    let cli = Cli::parse();
    let config = AppConfig {
        no_color: cli.no_color || cli.json,
        theme: cli.theme.clone(),
        ascii_preset: cli.ascii.clone(),
        ascii_file: cli.ascii_file.clone(),
        ascii_size: match cli.ascii_size {
            AsciiSize::Small => "small".to_string(),
            AsciiSize::Medium => "medium".to_string(),
            AsciiSize::Large => "large".to_string(),
        },
        plain: cli.plain,
        json: cli.json,
        speed: cli.speed,
        modules: cli
            .modules
            .as_ref()
            .map(|list| list.split(',').map(|s| s.trim().to_string()).collect()),
        cache_ttl_seconds: cli.cache,
        all_disks: cli.all_disks,
        local_ip: cli.local_ip,
        show_theme: cli.show_theme,
        show_icons: cli.show_icons,
        show_font: cli.show_font,
        show_cursor: cli.show_cursor,
    };

    let mut selected = match modules::resolve_modules(config.modules.as_deref()) {
        Ok(mods) => mods,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    if config.modules.is_none() {
        selected.retain(|module| !matches!(module.name, "local-ip" | "theme" | "icons" | "font" | "cursor"));
        if config.local_ip {
            if let Ok(mut extra) = modules::resolve_modules(Some(&vec!["local-ip".to_string()])) {
                selected.append(&mut extra);
            }
        }
        if config.show_theme {
            if let Ok(mut extra) = modules::resolve_modules(Some(&vec!["theme".to_string()])) {
                selected.append(&mut extra);
            }
        }
        if config.show_icons {
            if let Ok(mut extra) = modules::resolve_modules(Some(&vec!["icons".to_string()])) {
                selected.append(&mut extra);
            }
        }
        if config.show_font {
            if let Ok(mut extra) = modules::resolve_modules(Some(&vec!["font".to_string()])) {
                selected.append(&mut extra);
            }
        }
        if config.show_cursor {
            if let Ok(mut extra) = modules::resolve_modules(Some(&vec!["cursor".to_string()])) {
                selected.append(&mut extra);
            }
        }
    } else {
        if selected.iter().any(|m| m.name == "local-ip") && !config.local_ip {
            eprintln!("local-ip module requires --local-ip for privacy.");
            std::process::exit(1);
        }
        if selected.iter().any(|m| m.name == "theme") && !config.show_theme {
            eprintln!("theme module requires --show-theme.");
            std::process::exit(1);
        }
        if selected.iter().any(|m| m.name == "icons") && !config.show_icons {
            eprintln!("icons module requires --show-icons.");
            std::process::exit(1);
        }
        if selected.iter().any(|m| m.name == "font") && !config.show_font {
            eprintln!("font module requires --show-font.");
            std::process::exit(1);
        }
        if selected.iter().any(|m| m.name == "cursor") && !config.show_cursor {
            eprintln!("cursor module requires --show-cursor.");
            std::process::exit(1);
        }
    }

    let cache_key = build_cache_key(&config, &selected);
    let modules = if let Some(ttl) = config.cache_ttl_seconds {
        cache::load_cache(&cache_key, ttl)
            .unwrap_or_else(|| {
                let results = modules::run_modules(selected.clone(), config.clone());
                let _ = cache::write_cache(&cache_key, &results);
                results
            })
    } else {
        modules::run_modules(selected, config.clone())
    };

    if config.json {
        println!("{}", render::render_json(&modules));
        return;
    }

    let theme = themes::get_theme(&config.theme, config.no_color);
    let ascii_art = render::resolve_ascii(&config);
    let output = render::render_pretty(&modules, &config, &theme, ascii_art);
    println!("{output}");
}

fn build_cache_key(config: &AppConfig, modules: &[modules::Module]) -> String {
    let module_names = modules
        .iter()
        .map(|m| m.name)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "theme={}|ascii={}|plain={}|json={}|speed={}|modules={}|all_disks={}|local_ip={}|theme_info={}|icons={}|font={}|cursor={}",
        config.theme,
        config.ascii_preset.clone().unwrap_or_default(),
        config.plain,
        config.json,
        config.speed,
        module_names,
        config.all_disks,
        config.local_ip,
        config.show_theme,
        config.show_icons,
        config.show_font,
        config.show_cursor
    )
}
