use crate::platform::{BatteryInfo, Platform, ThemeInfo};
use crate::util::first_non_empty;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, MAX_PATH};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplaySettingsW, DEVMODEW, DISPLAY_DEVICEW, ENUM_CURRENT_SETTINGS,
};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH,
};
use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
use windows_sys::Win32::System::SystemInformation::{
    ComputerNameDnsHostname, GetComputerNameExW, GetSystemInfo, GetTickCount64,
    GlobalMemoryStatusEx, SYSTEM_INFO,
};

pub struct WindowsPlatform {}

impl Platform for WindowsPlatform {
    fn os_name_version(&self) -> Option<String> {
        let name = read_registry_string(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "ProductName",
        );
        let display_version = read_registry_string(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "DisplayVersion",
        );
        let build = read_registry_string(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "CurrentBuildNumber",
        );
        match (name, display_version, build) {
            (Some(n), Some(d), Some(b)) => Some(format!("{n} {d} (Build {b})")),
            (Some(n), None, Some(b)) => Some(format!("{n} (Build {b})")),
            (Some(n), Some(d), None) => Some(format!("{n} {d}")),
            (Some(n), None, None) => Some(n),
            _ => None,
        }
    }

    fn host(&self) -> Option<String> {
        get_computer_name(ComputerNameDnsHostname)
    }

    fn model(&self) -> Option<String> {
        let vendor =
            read_registry_string(r"HARDWARE\DESCRIPTION\System\BIOS", "SystemManufacturer");
        let product =
            read_registry_string(r"HARDWARE\DESCRIPTION\System\BIOS", "SystemProductName");
        match (vendor, product) {
            (Some(v), Some(p)) => Some(format!("{v} {p}")),
            (Some(v), None) => Some(v),
            (None, Some(p)) => Some(p),
            _ => None,
        }
    }

    fn kernel(&self) -> Option<String> {
        read_registry_string(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "CurrentVersion",
        )
        .zip(read_registry_string(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "CurrentBuildNumber",
        ))
        .map(|(version, build)| format!("NT {version}.{build}"))
    }

    fn uptime_seconds(&self) -> Option<u64> {
        Some(unsafe { GetTickCount64() / 1000 })
    }

    fn packages(&self) -> Option<String> {
        None
    }

    fn shell(&self) -> Option<String> {
        first_non_empty(&[std::env::var("COMSPEC").ok(), std::env::var("SHELL").ok()])
    }

    fn resolutions(&self) -> Option<String> {
        let mut modes = Vec::new();
        let mut device_index = 0;
        loop {
            let mut device: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
            device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
            let ok = unsafe { EnumDisplayDevicesW(std::ptr::null(), device_index, &mut device, 0) };
            if ok == 0 {
                break;
            }
            let mut devmode: DEVMODEW = unsafe { std::mem::zeroed() };
            devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
            let ok = unsafe {
                EnumDisplaySettingsW(
                    device.DeviceName.as_ptr(),
                    ENUM_CURRENT_SETTINGS,
                    &mut devmode,
                )
            };
            if ok != 0 {
                modes.push(format!("{}x{}", devmode.dmPelsWidth, devmode.dmPelsHeight));
            }
            device_index += 1;
        }
        if modes.is_empty() {
            None
        } else {
            modes.sort();
            modes.dedup();
            Some(modes.join(", "))
        }
    }

    fn de_wm(&self) -> Option<String> {
        Some("Explorer".to_string())
    }

    fn terminal(&self) -> Option<String> {
        if std::env::var("WT_SESSION").is_ok() {
            return Some("Windows Terminal".to_string());
        }
        first_non_empty(&[
            std::env::var("TERM_PROGRAM").ok(),
            std::env::var("TERM").ok(),
            std::env::var("ConEmuPID")
                .ok()
                .map(|_| "ConEmu".to_string()),
        ])
    }

    fn cpu(&self) -> Option<String> {
        let name = read_registry_string(
            r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
            "ProcessorNameString",
        );
        let mut info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
        unsafe { GetSystemInfo(&mut info) };
        let cores = info.dwNumberOfProcessors;
        match name {
            Some(n) if cores > 0 => Some(format!("{n} ({cores} cores)")),
            Some(n) => Some(n),
            None => None,
        }
    }

    fn gpu(&self) -> Option<String> {
        let mut device: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
        device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
        let ok = unsafe { EnumDisplayDevicesW(std::ptr::null(), 0, &mut device, 0) };
        if ok == 0 {
            return None;
        }
        let name = widestring_to_string(&device.DeviceString);
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    }

    fn memory(&self) -> Option<(u64, u64)> {
        let mut mem: windows_sys::Win32::System::SystemInformation::MEMORYSTATUSEX =
            unsafe { std::mem::zeroed() };
        mem.dwLength = std::mem::size_of::<
            windows_sys::Win32::System::SystemInformation::MEMORYSTATUSEX,
        >() as u32;
        let ok = unsafe { GlobalMemoryStatusEx(&mut mem) };
        if ok == 0 {
            return None;
        }
        let total = mem.ullTotalPhys;
        let avail = mem.ullAvailPhys;
        let used = total.saturating_sub(avail);
        Some((total, used))
    }

    fn disk(&self, all_mounts: bool) -> Option<(u64, u64)> {
        if all_mounts {
            disk_all_drives()
        } else {
            let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
            disk_for_drive(&format!("{drive}\\"))
        }
    }

    fn local_ip(&self) -> Option<String> {
        active_adapter().map(|(_, ip)| ip.to_string())
    }

    fn theme_info(&self) -> ThemeInfo {
        ThemeInfo {
            theme: None,
            icons: None,
            font: None,
            cursor: None,
        }
    }

    fn battery(&self) -> Option<BatteryInfo> {
        let mut status: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
        if unsafe { GetSystemPowerStatus(&mut status) } == 0 || status.BatteryFlag == 128 {
            return None;
        }
        let state = match status.ACLineStatus {
            1 if status.BatteryFlag & 8 != 0 => "Full",
            1 => "Charging",
            0 => "Discharging",
            _ => "Unknown",
        };
        Some(BatteryInfo {
            capacity: status.BatteryLifePercent.min(100),
            status: Some(state.to_string()),
        })
    }

    fn motherboard(&self) -> Option<String> {
        let vendor =
            read_registry_string(r"HARDWARE\DESCRIPTION\System\BIOS", "BaseBoardManufacturer");
        let product = read_registry_string(r"HARDWARE\DESCRIPTION\System\BIOS", "BaseBoardProduct");
        combine(vendor, product)
    }

    fn bios(&self) -> Option<String> {
        let vendor = read_registry_string(r"HARDWARE\DESCRIPTION\System\BIOS", "BIOSVendor");
        let version = read_registry_string(r"HARDWARE\DESCRIPTION\System\BIOS", "BIOSVersion");
        combine(vendor, version)
    }

    fn network_interface(&self) -> Option<String> {
        active_adapter().map(|(name, _)| name)
    }

    fn network_connected(&self) -> bool {
        active_adapter().is_some()
    }
}

fn read_registry_string(path: &str, value: &str) -> Option<String> {
    let subkey = to_wide(path);
    let value = to_wide(value);
    let mut buffer = vec![0u16; 512];
    let mut len = (buffer.len() * 2) as u32;
    let res = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            buffer.as_mut_ptr() as *mut _,
            &mut len,
        )
    };
    if res != 0 {
        return None;
    }
    let mut slice = &buffer[..(len as usize / 2)];
    if let Some(pos) = slice.iter().position(|c| *c == 0) {
        slice = &slice[..pos];
    }
    Some(String::from_utf16_lossy(slice).trim().to_string())
}

fn get_computer_name(format: i32) -> Option<String> {
    let mut buffer = vec![0u16; MAX_PATH as usize];
    let mut size = buffer.len() as u32;
    let ok = unsafe { GetComputerNameExW(format, buffer.as_mut_ptr(), &mut size) };
    if ok == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&buffer[..size as usize]))
}

fn disk_for_drive(drive: &str) -> Option<(u64, u64)> {
    let wide = to_wide(drive);
    let mut free = 0u64;
    let mut total = 0u64;
    let mut avail = 0u64;
    let ok = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut avail,
            &mut total,
            &mut free,
        )
    };
    if ok == 0 {
        return None;
    }
    let used = total.saturating_sub(free);
    Some((total, used))
}

fn disk_all_drives() -> Option<(u64, u64)> {
    let mut buffer = vec![0u16; 512];
    let len = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetLogicalDriveStringsW(
            buffer.len() as u32,
            buffer.as_mut_ptr(),
        )
    };
    if len == 0 {
        return None;
    }
    let mut total = 0u64;
    let mut used = 0u64;
    let mut start = 0usize;
    for i in 0..len as usize {
        if buffer[i] == 0 {
            if i > start {
                let drive = String::from_utf16_lossy(&buffer[start..i]);
                if let Some((t, u)) = disk_for_drive(&drive) {
                    total += t;
                    used += u;
                }
            }
            start = i + 1;
        }
    }
    if total == 0 {
        None
    } else {
        Some((total, used))
    }
}

fn to_wide(value: &str) -> Vec<u16> {
    OsString::from(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn widestring_to_string(buffer: &[u16]) -> String {
    let len = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len]).trim().to_string()
}

fn combine(first: Option<String>, second: Option<String>) -> Option<String> {
    match (first, second) {
        (Some(first), Some(second)) if first != second => Some(format!("{first} {second}")),
        (Some(first), _) => Some(first),
        (_, Some(second)) => Some(second),
        _ => None,
    }
}

fn active_adapter() -> Option<(String, std::net::Ipv4Addr)> {
    unsafe {
        let mut buffer_len = 0;
        if GetAdaptersAddresses(0, 0, null_mut(), null_mut(), &mut buffer_len)
            != ERROR_BUFFER_OVERFLOW
        {
            return None;
        }
        let mut buffer = vec![0u8; buffer_len as usize];
        let addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;
        if GetAdaptersAddresses(0, 0, null_mut(), addresses, &mut buffer_len) != 0 {
            return None;
        }
        let mut current = addresses;
        while !current.is_null() {
            let unicast = (*current).FirstUnicastAddress;
            if !unicast.is_null() {
                let socket = (*unicast).Address.lpSockaddr;
                if !socket.is_null()
                    && (*socket).sa_family == windows_sys::Win32::Networking::WinSock::AF_INET
                {
                    let socket =
                        socket as *const windows_sys::Win32::Networking::WinSock::SOCKADDR_IN;
                    let ip = std::net::Ipv4Addr::from(u32::from_be((*socket).sin_addr.S_un.S_addr));
                    if !ip.is_loopback() {
                        let name = if (*current).FriendlyName.is_null() {
                            "Network".to_string()
                        } else {
                            let mut len = 0;
                            while *(*current).FriendlyName.add(len) != 0 {
                                len += 1;
                            }
                            String::from_utf16_lossy(std::slice::from_raw_parts(
                                (*current).FriendlyName,
                                len,
                            ))
                        };
                        return Some((name, ip));
                    }
                }
            }
            current = (*current).Next;
        }
        None
    }
}
