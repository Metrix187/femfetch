use crate::platform::{BatteryInfo, Platform, ThemeInfo};
use crate::util::first_non_empty;
use libc::{freeifaddrs, getifaddrs, sockaddr, sockaddr_in, statvfs, AF_INET, IFF_LOOPBACK};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

pub struct LinuxPlatform {}

impl Platform for LinuxPlatform {
    fn os_name_version(&self) -> Option<String> {
        let contents = fs::read_to_string("/etc/os-release").ok()?;
        let mut pretty = None;
        let mut name = None;
        let mut version = None;
        for line in contents.lines() {
            if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                pretty = Some(trim_os_release(value));
            } else if let Some(value) = line.strip_prefix("NAME=") {
                name = Some(trim_os_release(value));
            } else if let Some(value) = line.strip_prefix("VERSION=") {
                version = Some(trim_os_release(value));
            }
        }
        first_non_empty(&[
            pretty,
            match (name, version) {
                (Some(n), Some(v)) => Some(format!("{n} {v}")),
                (Some(n), None) => Some(n),
                _ => None,
            },
        ])
    }

    fn host(&self) -> Option<String> {
        std::env::var("HOSTNAME")
            .ok()
            .or_else(|| fs::read_to_string("/etc/hostname").ok())
            .map(|s| s.trim().to_string())
    }

    fn model(&self) -> Option<String> {
        let product = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .ok()
            .map(|s| s.trim().to_string());
        let vendor = fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor")
            .ok()
            .map(|s| s.trim().to_string());
        match (vendor, product) {
            (Some(v), Some(p)) => Some(format!("{v} {p}")),
            (Some(v), None) => Some(v),
            (None, Some(p)) => Some(p),
            _ => None,
        }
    }

    fn kernel(&self) -> Option<String> {
        fs::read_to_string("/proc/sys/kernel/osrelease")
            .ok()
            .map(|s| s.trim().to_string())
    }

    fn uptime_seconds(&self) -> Option<u64> {
        let mut file = fs::File::open("/proc/uptime").ok()?;
        let mut buf = String::new();
        file.read_to_string(&mut buf).ok()?;
        let uptime = buf.split_whitespace().next()?;
        uptime.parse::<f64>().ok().map(|v| v.floor() as u64)
    }

    fn packages(&self) -> Option<String> {
        if let Some(count) = count_dpkg_packages() {
            return Some(format!("{count} (dpkg)"));
        }
        if let Some(count) = count_pacman_packages() {
            return Some(format!("{count} (pacman)"));
        }
        if let Some(count) = count_rpm_packages() {
            return Some(format!("{count} (rpm)"));
        }
        None
    }

    fn shell(&self) -> Option<String> {
        std::env::var("SHELL").ok()
    }

    fn resolutions(&self) -> Option<String> {
        let mut modes = Vec::new();
        let drm = fs::read_dir("/sys/class/drm").ok()?;
        for entry in drm.flatten() {
            let path = entry.path();
            if !path.file_name()?.to_string_lossy().starts_with("card") {
                continue;
            }
            let status_path = path.join("status");
            let status = fs::read_to_string(status_path).unwrap_or_default();
            if status.trim() != "connected" {
                continue;
            }
            let mode_path = path.join("modes");
            if let Ok(contents) = fs::read_to_string(mode_path) {
                if let Some(first) = contents.lines().next() {
                    modes.push(first.trim().to_string());
                }
            }
        }
        if modes.is_empty() {
            None
        } else {
            Some(modes.join(", "))
        }
    }

    fn de_wm(&self) -> Option<String> {
        let desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();
        let session = std::env::var("DESKTOP_SESSION").ok();
        let session_type = std::env::var("XDG_SESSION_TYPE").ok();
        let wayland = std::env::var("WAYLAND_DISPLAY").ok();
        let mut parts = Vec::new();
        if let Some(d) = desktop {
            parts.push(d);
        } else if let Some(s) = session {
            parts.push(s);
        }
        if let Some(t) = session_type {
            parts.push(t);
        } else if wayland.is_some() {
            parts.push("wayland".to_string());
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" / "))
        }
    }

    fn terminal(&self) -> Option<String> {
        first_non_empty(&[
            std::env::var("TERM_PROGRAM").ok(),
            std::env::var("TERM").ok(),
        ])
    }

    fn cpu(&self) -> Option<String> {
        let contents = fs::read_to_string("/proc/cpuinfo").ok()?;
        let mut model = None;
        let mut cores = 0;
        for line in contents.lines() {
            if let Some(value) = line.strip_prefix("model name") {
                if model.is_none() {
                    model = Some(value.split(':').nth(1).unwrap_or("").trim().to_string());
                }
            } else if line.starts_with("processor") {
                cores += 1;
            }
        }
        match (model, cores) {
            (Some(m), c) if c > 0 => Some(format!("{m} ({c} cores)")),
            (Some(m), _) => Some(m),
            _ => None,
        }
    }

    fn gpu(&self) -> Option<String> {
        let drm = fs::read_dir("/sys/class/drm").ok()?;
        for entry in drm.flatten() {
            let path = entry.path();
            if !path.file_name()?.to_string_lossy().starts_with("card") {
                continue;
            }
            let uevent = path.join("device/uevent");
            if let Ok(contents) = fs::read_to_string(uevent) {
                let mut driver = None;
                let mut pci = None;
                for line in contents.lines() {
                    if let Some(v) = line.strip_prefix("DRIVER=") {
                        driver = Some(v.to_string());
                    } else if let Some(v) = line.strip_prefix("PCI_ID=") {
                        pci = Some(v.to_string());
                    }
                }
                if driver.is_some() || pci.is_some() {
                    return Some(match (driver, pci) {
                        (Some(d), Some(p)) => format!("{d} ({p})"),
                        (Some(d), None) => d,
                        (None, Some(p)) => p,
                        _ => "N/A".to_string(),
                    });
                }
            }
        }
        None
    }

    fn memory(&self) -> Option<(u64, u64)> {
        let contents = fs::read_to_string("/proc/meminfo").ok()?;
        let mut total = None;
        let mut available = None;
        for line in contents.lines() {
            if line.starts_with("MemTotal:") {
                total = parse_kib(line);
            } else if line.starts_with("MemAvailable:") {
                available = parse_kib(line);
            }
        }
        let total = total?;
        let available = available.unwrap_or(0);
        let used = total.saturating_sub(available);
        Some((total * 1024, used * 1024))
    }

    fn disk(&self, all_mounts: bool) -> Option<(u64, u64)> {
        if all_mounts {
            disk_all_mounts()
        } else {
            disk_for_path(Path::new("/"))
        }
    }

    fn local_ip(&self) -> Option<String> {
        unsafe {
            let mut addrs: *mut libc::ifaddrs = std::ptr::null_mut();
            if getifaddrs(&mut addrs) != 0 {
                return None;
            }
            let mut cursor = addrs;
            let mut found = None;
            while !cursor.is_null() {
                let iface = &*cursor;
                if iface.ifa_addr.is_null() {
                    cursor = iface.ifa_next;
                    continue;
                }
                let addr = &*iface.ifa_addr;
                if addr.sa_family as i32 == AF_INET {
                    let ip = sockaddr_in_from(addr);
                    let flags = iface.ifa_flags as i32;
                    if flags & IFF_LOOPBACK == 0 {
                        found = Some(ip.to_string());
                        break;
                    }
                }
                cursor = iface.ifa_next;
            }
            freeifaddrs(addrs);
            found
        }
    }

    fn theme_info(&self) -> ThemeInfo {
        ThemeInfo {
            theme: read_gtk_setting("gtk-theme-name"),
            icons: read_gtk_setting("gtk-icon-theme-name"),
            font: read_gtk_setting("gtk-font-name"),
            cursor: read_gtk_setting("gtk-cursor-theme-name"),
        }
    }

    fn cpu_temperature(&self) -> Option<f64> {
        read_temperature(&["x86_pkg_temp", "cpu_thermal", "k10temp", "coretemp"])
            .or_else(|| read_hwmon_temperature("temp1_input"))
    }

    fn cpu_utilization(&self) -> Option<f64> {
        let first = read_cpu_times()?;
        thread::sleep(Duration::from_millis(100));
        let second = read_cpu_times()?;
        cpu_usage_between(first, second)
    }

    fn cpu_frequency(&self) -> Option<f64> {
        let khz = read_u64("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")?;
        Some(khz as f64 / 1_000.0)
    }

    fn gpu_temperature(&self) -> Option<f64> {
        read_temperature(&["amdgpu", "nouveau", "i915"])
    }

    fn gpu_utilization(&self) -> Option<f64> {
        first_matching_card_value("gpu_busy_percent").map(|value| value as f64)
    }

    fn vram(&self) -> Option<(u64, u64)> {
        let total = first_matching_card_value("mem_info_vram_total")?;
        let used = first_matching_card_value("mem_info_vram_used")?;
        Some((total, used))
    }

    fn battery(&self) -> Option<BatteryInfo> {
        let path = first_power_supply("Battery")?;
        let capacity = read_u64(path.join("capacity"))?.min(100) as u8;
        let status = fs::read_to_string(path.join("status"))
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        Some(BatteryInfo { capacity, status })
    }

    fn battery_health(&self) -> Option<f64> {
        let path = first_power_supply("Battery")?;
        let full =
            read_u64(path.join("energy_full")).or_else(|| read_u64(path.join("charge_full")))?;
        let design = read_u64(path.join("energy_full_design"))
            .or_else(|| read_u64(path.join("charge_full_design")))?;
        (design > 0).then_some((full as f64 / design as f64 * 100.0).min(100.0))
    }

    fn swap(&self) -> Option<(u64, u64)> {
        let contents = fs::read_to_string("/proc/meminfo").ok()?;
        parse_swap(&contents)
    }

    fn motherboard(&self) -> Option<String> {
        combine_files(
            "/sys/devices/virtual/dmi/id/board_vendor",
            "/sys/devices/virtual/dmi/id/board_name",
        )
    }

    fn bios(&self) -> Option<String> {
        combine_files(
            "/sys/devices/virtual/dmi/id/bios_vendor",
            "/sys/devices/virtual/dmi/id/bios_version",
        )
    }

    fn storage_model(&self) -> Option<String> {
        storage_models()
    }

    fn filesystem(&self) -> Option<String> {
        filesystem_for_root()
    }

    fn network_interface(&self) -> Option<String> {
        default_network_interface()
    }

    fn wifi(&self) -> Option<String> {
        let interface = default_network_interface()?;
        Path::new("/sys/class/net")
            .join(&interface)
            .join("wireless")
            .exists()
            .then_some(interface)
    }

    fn network_connected(&self) -> bool {
        default_network_interface().is_some()
    }

    fn monitor(&self) -> Option<String> {
        monitor_info()
    }
}

fn trim_os_release(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn parse_kib(line: &str) -> Option<u64> {
    line.split_whitespace().nth(1)?.parse::<u64>().ok()
}

fn disk_for_path(path: &Path) -> Option<(u64, u64)> {
    let mut stats: statvfs = unsafe { std::mem::zeroed() };
    let c_path = std::ffi::CString::new(path.to_string_lossy().as_bytes()).ok()?;
    let res = unsafe { statvfs(c_path.as_ptr(), &mut stats) };
    if res != 0 {
        return None;
    }
    let total = stats.f_blocks as u64 * stats.f_frsize as u64;
    let free = stats.f_bfree as u64 * stats.f_frsize as u64;
    let used = total.saturating_sub(free);
    Some((total, used))
}

fn disk_all_mounts() -> Option<(u64, u64)> {
    let mounts = fs::read_to_string("/proc/mounts").ok()?;
    let mut total = 0;
    let mut used = 0;
    let mut devices = HashSet::new();
    for line in mounts.lines() {
        let mut parts = line.split_whitespace();
        let device = parts.next().unwrap_or("");
        let mount = parts.next().unwrap_or("");
        let fstype = parts.next().unwrap_or("");
        if !devices.insert(device)
            || mount.starts_with("/proc")
            || mount.starts_with("/sys")
            || mount.starts_with("/run")
            || mount.starts_with("/dev")
            || matches!(
                fstype,
                "tmpfs" | "sysfs" | "proc" | "devtmpfs" | "squashfs" | "overlay"
            )
            || fstype.starts_with("cgroup")
        {
            continue;
        }
        if let Some((disk_total, disk_used)) = disk_for_path(Path::new(mount)) {
            total += disk_total;
            used += disk_used;
        }
    }
    (total > 0).then_some((total, used))
}

fn count_dpkg_packages() -> Option<u64> {
    let contents = fs::read_to_string("/var/lib/dpkg/status").ok()?;
    Some(
        contents
            .split("\n\n")
            .filter(|entry| {
                entry
                    .lines()
                    .any(|line| line == "Status: install ok installed")
            })
            .count() as u64,
    )
}

fn count_pacman_packages() -> Option<u64> {
    let mut count = 0u64;
    let entries = fs::read_dir("/var/lib/pacman/local").ok()?;
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            count += 1;
        }
    }
    Some(count)
}

fn count_rpm_packages() -> Option<u64> {
    if Path::new("/var/lib/rpm/Packages").exists()
        || Path::new("/var/lib/rpm/rpmdb.sqlite").exists()
    {
        if let Some(count) = count_rpm_with_command() {
            return Some(count);
        }
    }
    None
}

fn count_rpm_with_command() -> Option<u64> {
    let rpm = which("rpm")?;
    let output = std::process::Command::new(rpm).arg("-qa").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.lines().count() as u64)
}

fn which(cmd: &str) -> Option<String> {
    let paths = std::env::var("PATH").ok()?;
    for path in paths.split(':') {
        let full = Path::new(path).join(cmd);
        if full.exists() {
            return Some(full.to_string_lossy().to_string());
        }
    }
    None
}

fn read_gtk_setting(key: &str) -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let paths = [
        format!("{home}/.config/gtk-3.0/settings.ini"),
        format!("{home}/.gtkrc-2.0"),
    ];
    for path in paths {
        if let Ok(contents) = fs::read_to_string(&path) {
            for line in contents.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.starts_with(';') {
                    continue;
                }
                if let Some(value) = trimmed.strip_prefix(&format!("{key}=")) {
                    return Some(trim_gtk_value(value));
                }
                if let Some(value) = trimmed.strip_prefix(&format!("{key} =")) {
                    return Some(trim_gtk_value(value));
                }
            }
        }
    }
    None
}

fn trim_gtk_value(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn sockaddr_in_from(addr: &sockaddr) -> Ipv4Addr {
    let addr_in: &sockaddr_in = unsafe { &*(addr as *const sockaddr as *const sockaddr_in) };
    Ipv4Addr::from(u32::from_be(addr_in.sin_addr.s_addr))
}

fn read_u64(path: impl AsRef<Path>) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn read_temperature(preferred_types: &[&str]) -> Option<f64> {
    let thermal = fs::read_dir("/sys/class/thermal").ok()?;
    for entry in thermal.flatten() {
        let path = entry.path();
        let kind = fs::read_to_string(path.join("type")).unwrap_or_default();
        if preferred_types
            .iter()
            .any(|name| kind.trim().contains(name))
        {
            if let Some(value) = temperature_value(path.join("temp")) {
                return Some(value);
            }
        }
    }
    None
}

fn read_hwmon_temperature(file: &str) -> Option<f64> {
    for entry in fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        if let Some(value) = temperature_value(entry.path().join(file)) {
            return Some(value);
        }
    }
    None
}

fn temperature_value(path: impl AsRef<Path>) -> Option<f64> {
    let raw = fs::read_to_string(path).ok()?.trim().parse::<f64>().ok()?;
    let value = if raw.abs() > 1_000.0 {
        raw / 1_000.0
    } else {
        raw
    };
    (-50.0..=200.0).contains(&value).then_some(value)
}

fn read_cpu_times() -> Option<(u64, u64)> {
    let contents = fs::read_to_string("/proc/stat").ok()?;
    parse_cpu_times(contents.lines().next()?)
}

fn parse_cpu_times(line: &str) -> Option<(u64, u64)> {
    let mut values = line.split_whitespace();
    if values.next()? != "cpu" {
        return None;
    }
    let fields: Vec<u64> = values.filter_map(|value| value.parse().ok()).collect();
    if fields.len() < 4 {
        return None;
    }
    let total = fields.iter().copied().sum();
    let idle = fields[3] + fields.get(4).copied().unwrap_or(0);
    Some((total, idle))
}

fn cpu_usage_between(first: (u64, u64), second: (u64, u64)) -> Option<f64> {
    let total = second.0.saturating_sub(first.0);
    let idle = second.1.saturating_sub(first.1);
    (total > 0).then_some((total.saturating_sub(idle)) as f64 / total as f64 * 100.0)
}

fn first_matching_card_value(name: &str) -> Option<u64> {
    for index in 0..16 {
        let path = format!("/sys/class/drm/card{index}/device/{name}");
        if let Some(value) = read_u64(path) {
            return Some(value);
        }
    }
    None
}

fn first_power_supply(kind: &str) -> Option<PathBuf> {
    for entry in fs::read_dir("/sys/class/power_supply").ok()?.flatten() {
        let path = entry.path();
        let actual = fs::read_to_string(path.join("type")).unwrap_or_default();
        if actual.trim() == kind {
            return Some(path);
        }
    }
    None
}

fn parse_swap(contents: &str) -> Option<(u64, u64)> {
    let mut total = None;
    let mut free = None;
    for line in contents.lines() {
        if line.starts_with("SwapTotal:") {
            total = parse_kib(line);
        } else if line.starts_with("SwapFree:") {
            free = parse_kib(line);
        }
    }
    let total = total? * 1024;
    let free = free.unwrap_or(0) * 1024;
    Some((total, total.saturating_sub(free)))
}

fn combine_files(first: &str, second: &str) -> Option<String> {
    let first = fs::read_to_string(first)
        .ok()
        .map(|value| value.trim().to_string());
    let second = fs::read_to_string(second)
        .ok()
        .map(|value| value.trim().to_string());
    match (
        first.filter(|value| !value.is_empty()),
        second.filter(|value| !value.is_empty()),
    ) {
        (Some(first), Some(second)) if first != second => Some(format!("{first} {second}")),
        (Some(first), _) => Some(first),
        (_, Some(second)) => Some(second),
        _ => None,
    }
}

fn storage_models() -> Option<String> {
    let mut models = Vec::new();
    for entry in fs::read_dir("/sys/block").ok()?.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("zram") {
            continue;
        }
        let model = fs::read_to_string(entry.path().join("device/model"))
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| name.into_owned());
        models.push(model);
    }
    models.sort();
    models.dedup();
    (!models.is_empty()).then(|| models.join(", "))
}

fn filesystem_for_root() -> Option<String> {
    let mounts = fs::read_to_string("/proc/mounts").ok()?;
    mounts.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let device = fields.next()?;
        let mount = fields.next()?;
        let filesystem = fields.next()?;
        (mount == "/").then(|| format!("{filesystem} ({device})"))
    })
}

fn default_network_interface() -> Option<String> {
    let routes = fs::read_to_string("/proc/net/route").ok()?;
    for line in routes.lines().skip(1) {
        let mut fields = line.split_whitespace();
        let interface = fields.next()?;
        let destination = fields.next()?;
        let gateway = fields.next()?;
        let flags = u16::from_str_radix(fields.next()?, 16).ok()?;
        if destination == "00000000" && gateway != "00000000" && flags & 1 != 0 {
            return Some(interface.to_string());
        }
    }
    None
}

fn monitor_info() -> Option<String> {
    let mut monitors = Vec::new();
    for entry in fs::read_dir("/sys/class/drm").ok()?.flatten() {
        let path = entry.path();
        if fs::read_to_string(path.join("status")).ok()?.trim() != "connected" {
            continue;
        }
        let mode = fs::read_to_string(path.join("modes"))
            .ok()
            .and_then(|contents| contents.lines().next().map(str::to_string));
        let connector = entry.file_name().to_string_lossy().into_owned();
        monitors.push(match mode {
            Some(mode) => format!("{connector} {mode}"),
            None => connector,
        });
    }
    (!monitors.is_empty()).then(|| monitors.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cpu_times_and_usage() {
        assert_eq!(
            parse_cpu_times("cpu  100 2 30 400 10 0 0 0"),
            Some((542, 410))
        );
        let usage = cpu_usage_between((100, 80), (200, 130)).unwrap();
        assert!((usage - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parses_swap_usage() {
        let swap = parse_swap("MemTotal: 100 kB\nSwapTotal: 1024 kB\nSwapFree: 256 kB\n").unwrap();
        assert_eq!(swap, (1_048_576, 786_432));
    }
}
