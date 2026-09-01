# femfetch

Cute, fast, native system information for Linux and Windows.

femfetch reads the operating system's own interfaces (`/proc`, `/sys`, `statvfs`, Win32 APIs, and the Windows registry), runs selected modules concurrently, and makes every expensive or privacy-sensitive collector opt-in. It performs no network requests and enables no telemetry.

## Build

```bash
cargo build --release
./target/release/femfetch
```

Windows:

```powershell
cargo build --release
.\target\release\femfetch.exe
```

## Everyday usage

```bash
femfetch
femfetch --compact
femfetch --plain --no-color
femfetch --modules os,cpu,memory,disk
femfetch --mascot wolf --theme mint
femfetch --json
femfetch --watch 2 --modules uptime,cpu-usage,memory,disk
femfetch --prompt
femfetch --export report.html
```

Run `femfetch --help`, `--list-modules`, `--list-themes`, or `--list-mascots` for discoverable options.

### Output modes

- Default: adaptive mascot plus bordered information. Wide terminals use columns, medium terminals stack the mascot above the box, and extremely narrow terminals hide art and truncate safely.
- `--compact`: borderless, width-aware output using the same module results.
- `--plain`: one module per line without art or borders.
- `--json`: stable machine-readable output with no ANSI or diagnostics on stdout.
- `--prompt`: intentionally small static set (`os`, `host`, `cpu`, `memory`) suitable for prompt scripts.
- `--watch [seconds]`: live in-place refresh; defaults to one second. Static modules are collected once and only dynamic modules refresh.
- `--export FILE`: writes a dependency-free, escaped, self-contained HTML report with no JavaScript or remote assets.

Color is disabled automatically when stdout is redirected, `NO_COLOR` is set, or `TERM=dumb`.

## Configuration

The optional configuration file is:

- Linux: `$XDG_CONFIG_HOME/femfetch/config.toml`, or `~/.config/femfetch/config.toml`
- Windows: `%APPDATA%\femfetch\config.toml`

Use `femfetch --config-path` to print the effective default path, `--config FILE` for another file, or `--no-config` to ignore configuration. Missing configuration is normal. Invalid syntax is reported to stderr with a line number and a non-zero exit; femfetch never panics. Explicit CLI values override configuration values field by field.

The parser intentionally supports a small, human-readable TOML subset without a runtime dependency:

```toml
[presentation]
theme = "mint"                 # pastel, mint, sunset
mascot = "wolf"                # bunny, cat, fox, wolf, whale
ascii_size = "medium"          # small, medium, large
compact = false
plain = false
no_color = false
mascot_state = false

[collection]
modules = ["os", "host", "uptime", "cpu", "memory", "disk"]
all_disks = false
local_ip = false               # explicit privacy opt-in
show_theme = false
show_icons = false
show_font = false
show_cursor = false
cache = false                  # false/none, or TTL in seconds

[watch]
interval = 1.0
```

Top-level forms such as `theme = "mint"` and comma-separated `modules = "os,cpu,memory"` are also accepted. Strings do not support escapes; use `--ascii-file` for arbitrary custom art.

## Modules

Default modules:

```text
os, host, kernel, uptime, packages, shell, resolution, dewm,
terminal, cpu, gpu, memory, disk
```

Optional hardware/system modules, collected only when selected:

```text
cpu-temp, cpu-usage, cpu-frequency, gpu-temp, gpu-usage, vram,
battery, battery-health, swap, motherboard, bios, storage-model,
filesystem, interface, wifi, network, monitor
```

Privacy/presentation-gated modules:

```text
local-ip       requires --local-ip
theme          requires --show-theme
icons          requires --show-icons
font           requires --show-font
cursor         requires --show-cursor
```

Unavailable values render as `N/A` and carry an `error` string in JSON. Linux modules use kernel files and libc APIs; Windows uses Win32 APIs and registry data. Sensor support depends on hardware drivers exposing standard interfaces. On Windows, unsupported sensor values degrade to `N/A` rather than invoking PowerShell/WMI.

`cpu-usage` samples `/proc/stat` over 100 ms on Linux, so it is deliberately excluded from the default invocation. RPM package counting is the only Linux fallback that may execute a process (`rpm -qa`), and only when an RPM database is detected.

## Mascots

Built-ins are static, local, and dependency-free: `bunny`, `cat`, `fox`, `wolf`, and `whale`, with small/medium/large sizes. `--ascii-file FILE` preserves custom art support.

`--mascot-state` adds only the minimal hidden state modules needed and maps state to static art:

- CPU temperature at least 80 °C → hot wolf
- battery at most 15% → sleepy wolf
- memory at least 90% or disconnected network → alert wolf
- otherwise → selected/default mascot

Users who do not enable this option pay no state-sampling cost.

## Stable JSON schema

`--json` emits valid UTF-8 JSON and never mixes diagnostics into stdout:

```json
{
  "version": 1,
  "modules": [
    {
      "name": "cpu",
      "key": "CPU ✿",
      "value": "AMD Ryzen 7 7840U (16 cores)",
      "took_ms": 0.143,
      "error": null
    }
  ]
}
```

- `version`: integer schema version; currently `1`.
- `modules`: selected modules in deterministic request order.
- `name`: stable programmatic module identifier.
- `key`: decorated display label retained for compatibility.
- `value`: display value; unavailable values are consistently `"N/A"`.
- `took_ms`: floating-point collection duration with sub-millisecond precision, or omitted only if a worker failed before reporting.
- `error`: `null` on success or a diagnostic string on unavailable data.

JSON mode does not select or collect extra modules.

## Watch mode

```bash
femfetch --watch
femfetch --watch 0.5 --compact
femfetch --watch 2 --modules uptime,cpu-usage,memory,disk,battery
```

Watch mode requires an interactive terminal, refreshes in place, recomputes terminal width each frame, and restores the cursor after Ctrl+C. Static rows are cached for the process lifetime. Dynamic rows (`uptime`, memory, disk, sensors, battery, and network state) are recollected without spawning a persistent service or daemon.

## Benchmarking

Internal benchmark mode measures the existing module and rendering paths with caching disabled:

```bash
femfetch --benchmark           # 25 runs
femfetch --benchmark 100
femfetch --benchmark 50 --modules os,cpu,memory,disk
```

It reports collection/rendering median and p95 plus mean per-module time. It does not claim process startup latency for each iteration because collection happens in-process; use the regression script below for process-level measurements.

For repeatable development comparisons:

```bash
./scripts/perf-regression.sh ./target/release/femfetch 100
```

The script warms the binary, measures default/plain/JSON/representative invocations using a monotonic clock, and reports median, p95, mean, binary bytes, and peak RSS when `/usr/bin/time` is available. Run it on an otherwise idle machine against binaries built with the same Rust version and profile. Results are observational, not hard CI gates.

## Prompt integration

`femfetch --prompt` prints a single line and does not modify shell files. `femfetch --shell-init` prints example Bash and Zsh snippets for manual review. Prompt command substitution runs a process on every prompt, so users with extremely frequent prompts may prefer caching at the shell level.

## Development checks

```bash
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
rustup target add x86_64-pc-windows-gnu
cargo check --locked --all-targets --target x86_64-pc-windows-gnu
cargo build --release --locked
```

The test suite parses actual JSON, exercises CLI/config precedence and malformed files, verifies HTML safety, covers Linux metric parsers, and checks ANSI/Unicode width behavior across narrow and wide layouts.

## Design principles

- Protect the default hot path: optional features perform no collection or filesystem work unless enabled.
- Prefer native interfaces and existing dependencies over subprocesses or frameworks.
- Keep deterministic module order even though collection is concurrent.
- Fail gracefully per module; one unavailable metric does not kill the fetch.
- Keep machine output valid and diagnostics on stderr.
- Never fetch assets, transmit system information, or modify shell configuration.
