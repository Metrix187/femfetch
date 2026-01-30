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

pub fn os_name_version() -> Option<String> {
    platform().os_name_version()
}

pub fn host() -> Option<String> {
    platform().host()
}

pub fn model() -> Option<String> {
    platform().model()
}

pub fn kernel() -> Option<String> {
    platform().kernel()
}

pub fn uptime_seconds() -> Option<u64> {
    platform().uptime_seconds()
}

pub fn packages() -> Option<String> {
    platform().packages()
}

pub fn shell() -> Option<String> {
    platform().shell()
}

pub fn resolutions() -> Option<String> {
    platform().resolutions()
}

pub fn de_wm() -> Option<String> {
    platform().de_wm()
}

pub fn terminal() -> Option<String> {
    platform().terminal()
}

pub fn cpu() -> Option<String> {
    platform().cpu()
}

pub fn gpu() -> Option<String> {
    platform().gpu()
}

pub fn memory() -> Option<(u64, u64)> {
    platform().memory()
}

pub fn disk(all_mounts: bool) -> Option<(u64, u64)> {
    platform().disk(all_mounts)
}

pub fn local_ip() -> Option<String> {
    platform().local_ip()
}

pub fn theme_info() -> ThemeInfo {
    platform().theme_info()
}

trait Platform {
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
}

fn platform() -> Box<dyn Platform> {
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxPlatform {})
    }
    #[cfg(windows)]
    {
        Box::new(windows::WindowsPlatform {})
    }
}
