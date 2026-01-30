use crate::platform::{Platform, ThemeInfo};
use crate::util::first_non_empty;
use libc::{getifaddrs, statvfs, freeifaddrs, sockaddr, sockaddr_in, AF_INET, IFF_LOOPBACK};
use std::fs;
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::Path;

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
    for line in mounts.lines() {
        let mut parts = line.split_whitespace();
        let _device = parts.next();
        let mount = parts.next().unwrap_or("");
        let fstype = parts.next().unwrap_or("");
        if mount.starts_with("/proc")
            || mount.starts_with("/sys")
            || mount.starts_with("/run")
            || mount.starts_with("/dev")
            || fstype == "tmpfs"
            || fstype == "sysfs"
            || fstype == "proc"
            || fstype == "devtmpfs"
            || fstype.starts_with("cgroup")
        {
            continue;
        }
        if let Some((t, u)) = disk_for_path(Path::new(mount)) {
            total += t;
            used += u;
        }
    }
    if total == 0 {
        None
    } else {
        Some((total, used))
    }
}

fn count_dpkg_packages() -> Option<u64> {
    let contents = fs::read_to_string("/var/lib/dpkg/status").ok()?;
    Some(contents.lines().filter(|l| l.starts_with("Package: ")).count() as u64)
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
    if Path::new("/var/lib/rpm/Packages").exists() || Path::new("/var/lib/rpm/rpmdb.sqlite").exists() {
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
