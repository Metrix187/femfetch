// mirror of femos os/femmeos/kernel/include/femapi.h -- keep in lockstep,
// the version field is the tripwire if they drift.

pub const FEMAPI_VERSION: u32 = 1;

#[repr(C)]
pub struct FemApi {
    pub version: u32,
    pub reserved: u32,

    pub print: extern "C" fn(s: *const u8, len: u32),
    pub next_key: extern "C" fn() -> i32,

    pub uptime_ms: extern "C" fn() -> u64,
    pub rtc_now: extern "C" fn(*mut u16, *mut u8, *mut u8, *mut u8, *mut u8, *mut u8),

    pub os_name: extern "C" fn() -> *const u8,
    pub os_version: extern "C" fn() -> *const u8,
    pub arch: extern "C" fn() -> *const u8,
    pub fb_info: extern "C" fn(*mut u32, *mut u32, *mut u32),
    pub mem_total_kb: extern "C" fn() -> u64,
    pub cpu_brand: extern "C" fn(buf: *mut u8, cap: u32),
    pub theme_accent: extern "C" fn() -> u32,
    pub theme_name: extern "C" fn() -> *const u8,
}

pub fn cstr(p: *const u8) -> &'static str {
    unsafe {
        let mut n = 0;
        while *p.add(n) != 0 {
            n += 1;
        }
        core::str::from_utf8(core::slice::from_raw_parts(p, n)).unwrap_or("?")
    }
}

// the i686-unknown-linux-gnu target assumes libc brings these; there is no
// libc down here, just vibes. (x86_64-unknown-none ships its own, hence cfg.)
#[cfg(target_arch = "x86")]
mod mem {
    #[no_mangle]
    pub unsafe extern "C" fn memcpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8 {
        for i in 0..n {
            *d.add(i) = *s.add(i);
        }
        d
    }

    #[no_mangle]
    pub unsafe extern "C" fn memmove(d: *mut u8, s: *const u8, n: usize) -> *mut u8 {
        if (d as usize) < (s as usize) {
            for i in 0..n {
                *d.add(i) = *s.add(i);
            }
        } else {
            let mut i = n;
            while i > 0 {
                i -= 1;
                *d.add(i) = *s.add(i);
            }
        }
        d
    }

    #[no_mangle]
    pub unsafe extern "C" fn memset(d: *mut u8, c: i32, n: usize) -> *mut u8 {
        for i in 0..n {
            *d.add(i) = c as u8;
        }
        d
    }

    #[no_mangle]
    pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
        for i in 0..n {
            let (x, y) = (*a.add(i), *b.add(i));
            if x != y {
                return x as i32 - y as i32;
            }
        }
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn bcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
        memcmp(a, b, n)
    }

    // llvm recognizes a hand-rolled nul scan and helpfully calls strlen. sure.
    #[no_mangle]
    pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
        let mut n = 0;
        while *s.add(n) != 0 {
            n += 1;
        }
        n
    }
}
