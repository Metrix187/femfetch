# femfetch

Cute, fast, neofetch-like system info — written from scratch in Rust with a focus on low overhead and native system reads.

## Why Rust

Rust compiles to a single static-ish binary, has great cross-platform support, and makes it easy to write fast, safe parsers for `/proc` and native APIs without a runtime.

## Benchmarks

### WSL (25 runs)

| Tool | Total Time | Average per Run |
| :--- | :--- | :--- |
| **neofetch** | 68.20s | ~2.73s |
| **fastfetch** | 0.79s | ~0.03s (32ms) |
| **femfetch** | **0.22s** | **~0.009s (9ms)** |

> `femfetch` is approximately **310x faster** than `neofetch` and **3.5x faster** than `fastfetch`.

### Ubuntu Server (AWS EC2 t3.small, 25 runs)

| Tool | Total Time | Average per Run |
| :--- | :--- | :--- |
| **fastfetch** | 0.145s | ~0.006s (5.8ms) |
| **femfetch** | **0.107s** | **~0.004s (4.3ms)** |

> `femfetch` is approximately **1.35x faster** than `fastfetch` on cloud hardware.

## Features

- Pastel themes, rounded box layout, cute labels.
- Linux first (Ubuntu/Debian/Arch/Fedora), Windows 10/11 parity.
- Native reads from `/etc`, `/proc`, `/sys`, syscalls, and Windows APIs.
- No network calls, no telemetry, no disk writes unless `--cache` is enabled.
- `--plain` and `--json` output modes.
- Built-in mascots with small/medium/large sizes.
- Modular architecture with per-module timing.

## Install / Build

### Linux

```bash
cargo build --release
./target/release/femfetch
```

### Windows

```powershell
cargo build --release
.\target\release\femfetch.exe
```

### Cross-compile notes

- Windows from Linux:
  - `rustup target add x86_64-pc-windows-gnu`
  - `cargo build --release --target x86_64-pc-windows-gnu`
- Linux from Windows (WSL):
  - Run `cargo build --release` inside WSL.

## Usage

```bash
femfetch [flags]
```

### Required flags (supported)

- `--no-color` disable ANSI colors
- `--theme <name>` theme name (`pastel`, `mint`, `sunset`)
- `--ascii <preset>` built-in art (`bunny`, `cat`, `whale`)
- `--ascii-file <path>` custom ASCII art file
- `--speed` show per-module timings
- `--modules <list>` select/reorder modules (comma-separated)
- `--cache [seconds]` enable cache with TTL (default 10s)

### Additional flags

- `--plain` no ASCII/box decorations
- `--json` machine-readable output (no ANSI)
- `--ascii-size <small|medium|large>`
- `--all-disks` show total across mounts
- `--local-ip` include local IP (privacy-sensitive)
- `--show-theme` include GTK theme info (Linux only)
- `--show-icons` include GTK icon theme (Linux only)
- `--show-font` include GTK font (Linux only)
- `--show-cursor` include GTK cursor theme (Linux only)

### Modules

Default modules (in order):

```
os, host, kernel, uptime, packages, shell, resolution, dewm, terminal, cpu, gpu, memory, disk
```

Optional modules:

```
local-ip, theme, icons, font, cursor
```

Example reorder:

```bash
femfetch --modules cpu,gpu,memory,disk,os
```

## Themes

Themes are simple palettes defined in `src/themes/mod.rs`. Add a new theme by inserting a new palette in `get_theme()`.

## Adding Modules

1. Create a module file in `src/modules/`.
2. Return a `Module` with `{ name, key, run }`.
3. Register it in `src/modules/mod.rs`.
4. Add any platform-specific reads in `src/platform/linux` or `src/platform/windows`.

## Examples

Default:

```
  /\_/\
 ( •.• )
 /づっ♡
╭────────────────────────────────────────────╮
│ OS ✿     Ubuntu 24.04 LTS                  │
│ Host ◍   mybox (Lenovo ThinkPad)           │
│ Kernel ❀ 6.8.0-12-generic                  │
│ Uptime ♡ 3h 14m                            │
│ Packages ♡ 2101 (dpkg)                     │
│ Shell ✿  /bin/zsh                          │
│ Resolution ♡ 2560x1440                     │
│ DE / WM ✿ GNOME / wayland                  │
│ Terminal ❀ kitty                           │
│ CPU ✿   AMD Ryzen 7 7840U (16 cores)       │
│ GPU ✿   amdgpu (1002:164e)                 │
│ Memory ♡ 6.3 GiB / 31.2 GiB                │
│ Disk ❀  82.1 GiB / 476.9 GiB               │
╰────────────────────────────────────────────╯
```

JSON:

```json
{
  "modules": [
    {
      "key": "OS ✿",
      "value": "Ubuntu 24.04 LTS",
      "took_ms": 1,
      "error": null
    }
  ]
}
```

## Notes

- RPM package counting falls back to `rpm -qa` if the RPM database is present (no other reliable on-disk count without a heavier dependency).
- GPU detection is best-effort and depends on DRM/Display APIs.
- No disk writes happen unless `--cache` is used.
