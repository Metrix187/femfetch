#[cfg(target_os = "linux")]
mod linux;
#[cfg(windows)]
mod windows;

#[derive(Debug, Clone)]
pub struct ThemeInfo {
    pub theme: Option<String>,
    pub icons: Option<String>,
    pub font: Option<String>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub capacity: u8,
    pub status: Option<String>,
}

macro_rules! platform_getter {
    ($name:ident, $return:ty) => {
        pub fn $name() -> $return {
            platform().$name()
        }
    };
}

platform_getter!(os_name_version, Option<String>);
platform_getter!(host, Option<String>);
platform_getter!(model, Option<String>);
platform_getter!(kernel, Option<String>);
platform_getter!(uptime_seconds, Option<u64>);
platform_getter!(packages, Option<String>);
platform_getter!(shell, Option<String>);
platform_getter!(resolutions, Option<String>);
platform_getter!(de_wm, Option<String>);
platform_getter!(terminal, Option<String>);
platform_getter!(cpu, Option<String>);
platform_getter!(gpu, Option<String>);
platform_getter!(memory, Option<(u64, u64)>);
platform_getter!(local_ip, Option<String>);
platform_getter!(theme_info, ThemeInfo);
platform_getter!(cpu_temperature, Option<f64>);
platform_getter!(cpu_utilization, Option<f64>);
platform_getter!(cpu_frequency, Option<f64>);
platform_getter!(gpu_temperature, Option<f64>);
platform_getter!(gpu_utilization, Option<f64>);
platform_getter!(vram, Option<(u64, u64)>);
platform_getter!(battery, Option<BatteryInfo>);
platform_getter!(battery_health, Option<f64>);
platform_getter!(swap, Option<(u64, u64)>);
platform_getter!(motherboard, Option<String>);
platform_getter!(bios, Option<String>);
platform_getter!(storage_model, Option<String>);
platform_getter!(filesystem, Option<String>);
platform_getter!(network_interface, Option<String>);
platform_getter!(wifi, Option<String>);
platform_getter!(monitor, Option<String>);

pub fn disk(all_mounts: bool) -> Option<(u64, u64)> {
    platform().disk(all_mounts)
}

pub fn network_connected() -> bool {
    platform().network_connected()
}

trait Platform: Sync {
    fn os_name_version(&self) -> Option<String>;
    fn host(&self) -> Option<String>;
    fn model(&self) -> Option<String>;
    fn kernel(&self) -> Option<String>;
    fn uptime_seconds(&self) -> Option<u64>;
    fn packages(&self) -> Option<String>;
    fn shell(&self) -> Option<String>;
    fn resolutions(&self) -> Option<String>;
    fn de_wm(&self) -> Option<String>;
    fn terminal(&self) -> Option<String>;
    fn cpu(&self) -> Option<String>;
    fn gpu(&self) -> Option<String>;
    fn memory(&self) -> Option<(u64, u64)>;
    fn disk(&self, all_mounts: bool) -> Option<(u64, u64)>;
    fn local_ip(&self) -> Option<String>;
    fn theme_info(&self) -> ThemeInfo;
    fn cpu_temperature(&self) -> Option<f64> {
        None
    }
    fn cpu_utilization(&self) -> Option<f64> {
        None
    }
    fn cpu_frequency(&self) -> Option<f64> {
        None
    }
    fn gpu_temperature(&self) -> Option<f64> {
        None
    }
    fn gpu_utilization(&self) -> Option<f64> {
        None
    }
    fn vram(&self) -> Option<(u64, u64)> {
        None
    }
    fn battery(&self) -> Option<BatteryInfo> {
        None
    }
    fn battery_health(&self) -> Option<f64> {
        None
    }
    fn swap(&self) -> Option<(u64, u64)> {
        None
    }
    fn motherboard(&self) -> Option<String> {
        None
    }
    fn bios(&self) -> Option<String> {
        None
    }
    fn storage_model(&self) -> Option<String> {
        None
    }
    fn filesystem(&self) -> Option<String> {
        None
    }
    fn network_interface(&self) -> Option<String> {
        None
    }
    fn wifi(&self) -> Option<String> {
        None
    }
    fn network_connected(&self) -> bool {
        self.local_ip().is_some()
    }
    fn monitor(&self) -> Option<String> {
        self.resolutions()
    }
}

fn platform() -> &'static dyn Platform {
    #[cfg(target_os = "linux")]
    {
        static PLATFORM: linux::LinuxPlatform = linux::LinuxPlatform {};
        &PLATFORM
    }
    #[cfg(windows)]
    {
        static PLATFORM: windows::WindowsPlatform = windows::WindowsPlatform {};
        &PLATFORM
    }
}
