use std::sync::atomic::{AtomicBool, Ordering};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn install() {
    INTERRUPTED.store(false, Ordering::Relaxed);
    #[cfg(unix)]
    unsafe {
        libc::signal(
            libc::SIGINT,
            handle_signal as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            handle_signal as *const () as libc::sighandler_t,
        );
    }
    #[cfg(windows)]
    unsafe {
        windows_sys::Win32::System::Console::SetConsoleCtrlHandler(Some(handle_console), 1);
    }
}

pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::Relaxed)
}

#[cfg(unix)]
extern "C" fn handle_signal(_: libc::c_int) {
    INTERRUPTED.store(true, Ordering::Relaxed);
}

#[cfg(windows)]
unsafe extern "system" fn handle_console(kind: u32) -> i32 {
    const CTRL_C_EVENT: u32 = 0;
    const CTRL_BREAK_EVENT: u32 = 1;
    if matches!(kind, CTRL_C_EVENT | CTRL_BREAK_EVENT) {
        INTERRUPTED.store(true, Ordering::Relaxed);
        1
    } else {
        0
    }
}
