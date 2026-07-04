// femfetch for FemmeOS: no_std, no alloc, no libc -- just the femapi table
// the kernel hands us. output is plain ascii because the femwm terminal
// draws an 8x8 bitmap font and has never heard of ansi escapes.
#![no_std]
#![no_main]

mod femapi;
use femapi::{cstr, FemApi, FEMAPI_VERSION};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}

struct Out<'a>(&'a FemApi);

impl<'a> Out<'a> {
    fn s(&self, t: &str) {
        (self.0.print)(t.as_ptr(), t.len() as u32);
    }

    fn c(&self, b: u8) {
        (self.0.print)(&b as *const u8, 1);
    }

    fn num(&self, mut v: u64) {
        let mut buf = [0u8; 20];
        let mut n = 0;
        loop {
            buf[n] = b'0' + (v % 10) as u8;
            v /= 10;
            n += 1;
            if v == 0 {
                break;
            }
        }
        while n > 0 {
            n -= 1;
            self.c(buf[n]);
        }
    }

    fn num2(&self, v: u64) {
        self.c(b'0' + (v / 10 % 10) as u8);
        self.c(b'0' + (v % 10) as u8);
    }
}

// ascii cousin of the femfetch bunny preset (the real one has utf-8 eyes
// the femwm font can't draw)
const ART: [&str; 6] = [
    "  /\\_/\\   ",
    " ( o.o )  ",
    " (> u <)  ",
    "  /   \\   ",
    " (_____)  ",
    "          ",
];

fn art_row(o: &Out, i: usize) {
    o.s(ART[if i < ART.len() { i } else { ART.len() - 1 }]);
    o.s(" ");
}

#[no_mangle]
pub extern "C" fn _start(api: *const FemApi) -> i32 {
    let api = unsafe { &*api };
    if api.version != FEMAPI_VERSION {
        return 2; // kernel speaks a different femapi; bail before we misread it
    }
    let o = Out(api);

    let mut brand = [0u8; 64];
    (api.cpu_brand)(brand.as_mut_ptr(), brand.len() as u32);
    let brand_len = brand.iter().position(|&b| b == 0).unwrap_or(0);
    let brand_str = core::str::from_utf8(&brand[..brand_len]).unwrap_or("?");

    let (mut w, mut h, mut bpp) = (0u32, 0u32, 0u32);
    (api.fb_info)(&mut w, &mut h, &mut bpp);

    let (mut yr, mut mo, mut dy) = (0u16, 0u8, 0u8);
    let (mut hh, mut mi, mut ss) = (0u8, 0u8, 0u8);
    (api.rtc_now)(&mut yr, &mut mo, &mut dy, &mut hh, &mut mi, &mut ss);

    let up_s = (api.uptime_ms)() / 1000;
    let mem_kb = (api.mem_total_kb)();

    o.s("\n");

    art_row(&o, 0);
    o.s("femme @ ");
    o.s(cstr((api.os_name)()));
    o.s("\n");

    art_row(&o, 1);
    o.s("-----------------\n");

    art_row(&o, 2);
    o.s("os       ");
    o.s(cstr((api.os_name)()));
    o.s(" ");
    o.s(cstr((api.os_version)()));
    o.s(" (femwm)\n");

    art_row(&o, 3);
    o.s("arch     ");
    o.s(cstr((api.arch)()));
    o.s("\n");

    art_row(&o, 4);
    o.s("cpu      ");
    o.s(brand_str);
    o.s("\n");

    art_row(&o, 5);
    o.s("memory   ");
    if mem_kb == 0 {
        o.s("unknown");
    } else {
        o.num(mem_kb / 1024);
        o.s(" MiB");
    }
    o.s("\n");

    art_row(&o, 6);
    o.s("display  ");
    o.num(w as u64);
    o.c(b'x');
    o.num(h as u64);
    o.s(" @ ");
    o.num(bpp as u64);
    o.s("bpp\n");

    art_row(&o, 7);
    o.s("uptime   ");
    o.num(up_s / 3600);
    o.c(b':');
    o.num2(up_s / 60 % 60);
    o.c(b':');
    o.num2(up_s % 60);
    o.s("\n");

    art_row(&o, 8);
    o.s("theme    ");
    o.s(cstr((api.theme_name)()));
    o.s("\n");

    art_row(&o, 9);
    o.s("date     ");
    o.num(yr as u64);
    o.c(b'-');
    o.num2(mo as u64);
    o.c(b'-');
    o.num2(dy as u64);
    o.s(" ");
    o.num2(hh as u64);
    o.c(b':');
    o.num2(mi as u64);
    o.c(b':');
    o.num2(ss as u64);
    o.s("\n\n");

    0
}
