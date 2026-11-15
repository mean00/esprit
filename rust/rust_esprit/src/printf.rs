//! Minimal embedded `vsnprintf_` / `snprintf_` / `sprintf_` / `printf_` in Rust.
//!
//! Replaces `esprit/cpp/embedded_printf/printf.c` when `LN_ENABLE_RUST` is
//! enabled on `arm_gd32fx`.

use core::ffi::{VaList, c_char, c_int};

struct BufWriter {
    buf: *mut u8,
    cap: usize,
    pos: usize,
}

impl BufWriter {
    fn new(buf: *mut u8, cap: usize) -> Self {
        Self { buf, cap, pos: 0 }
    }

    fn push_byte(&mut self, b: u8) {
        if self.cap > 0 && self.pos + 1 < self.cap && !self.buf.is_null() {
            // SAFETY: `self.pos + 1 < self.cap` guarantees space for byte + NUL.
            unsafe { *self.buf.add(self.pos) = b };
        }
        self.pos += 1;
    }

    fn push_pad(&mut self, ch: u8, count: usize) {
        for _ in 0..count {
            self.push_byte(ch);
        }
    }

    fn push_bytes(&mut self, bytes: &[u8], width: usize, left_align: bool, zero_pad: bool) {
        let pad = width.saturating_sub(bytes.len());
        if !left_align {
            self.push_pad(if zero_pad { b'0' } else { b' ' }, pad);
        }
        for &b in bytes {
            self.push_byte(b);
        }
        if left_align {
            self.push_pad(b' ', pad);
        }
    }

    fn finish(self) -> c_int {
        if self.cap > 0 && !self.buf.is_null() {
            let nul_idx = self.pos.min(self.cap - 1);
            // SAFETY: `nul_idx < self.cap`.
            unsafe { *self.buf.add(nul_idx) = 0 };
        }
        self.pos as c_int
    }
}

fn format_uint(mut val: u64, base: u64, upper: bool, neg: bool, buf: &mut [u8; 24]) -> &[u8] {
    let digits = if upper {
        b"0123456789ABCDEF"
    } else {
        b"0123456789abcdef"
    };
    let mut i = buf.len();
    if val == 0 {
        i -= 1;
        buf[i] = b'0';
    } else {
        while val > 0 {
            i -= 1;
            buf[i] = digits[(val % base) as usize];
            val /= base;
        }
    }
    if neg {
        i -= 1;
        buf[i] = b'-';
    }
    &buf[i..]
}

/// Core `vsnprintf_` implementation.
///
/// # Safety
/// `s` (if `n > 0`) and `format` must be valid pointers, and `ap` must match `format`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vsnprintf_(
    s: *mut c_char,
    n: usize,
    format: *const c_char,
    mut ap: VaList<'_>,
) -> c_int {
    if format.is_null() {
        return 0;
    }
    let mut w = BufWriter::new(s as *mut u8, n);
    let mut p = format as *const u8;

    loop {
        let ch = unsafe { *p };
        if ch == 0 {
            break;
        }
        p = unsafe { p.add(1) };
        if ch != b'%' {
            w.push_byte(ch);
            continue;
        }

        let mut left_align = false;
        let mut zero_pad = false;
        loop {
            match unsafe { *p } {
                b'-' => {
                    left_align = true;
                    p = unsafe { p.add(1) };
                }
                b'0' => {
                    zero_pad = true;
                    p = unsafe { p.add(1) };
                }
                b'+' | b' ' | b'#' => {
                    p = unsafe { p.add(1) };
                }
                _ => break,
            }
        }
        if left_align {
            zero_pad = false;
        }

        let mut width: usize = 0;
        while unsafe { *p }.is_ascii_digit() {
            width = width * 10 + (unsafe { *p } - b'0') as usize;
            p = unsafe { p.add(1) };
        }

        if unsafe { *p } == b'.' {
            p = unsafe { p.add(1) };
            while unsafe { *p }.is_ascii_digit() {
                p = unsafe { p.add(1) };
            }
        }

        let mut is_long_long = false;
        while matches!(unsafe { *p }, b'l' | b'h' | b'z' | b't') {
            if unsafe { *p } == b'l' && unsafe { *p.add(1) } == b'l' {
                is_long_long = true;
                p = unsafe { p.add(2) };
            } else {
                p = unsafe { p.add(1) };
            }
        }

        let spec = unsafe { *p };
        if spec == 0 {
            break;
        }
        p = unsafe { p.add(1) };

        let mut num_buf = [0u8; 24];
        match spec {
            b'%' => w.push_byte(b'%'),
            b'c' => {
                let c = unsafe { ap.next_arg::<c_int>() } as u8;
                w.push_bytes(&[c], width, left_align, false);
            }
            b's' => {
                let ptr = unsafe { ap.next_arg::<*const u8>() };
                let bytes = if ptr.is_null() {
                    b"(null)".as_slice()
                } else {
                    let mut len = 0;
                    while unsafe { *ptr.add(len) } != 0 {
                        len += 1;
                    }
                    unsafe { core::slice::from_raw_parts(ptr, len) }
                };
                w.push_bytes(bytes, width, left_align, false);
            }
            b'd' | b'i' => {
                let val: i64 = if is_long_long {
                    unsafe { ap.next_arg::<i64>() }
                } else {
                    unsafe { ap.next_arg::<i32>() as i64 }
                };
                let neg = val < 0;
                let mag = val.unsigned_abs();
                let slice = format_uint(mag, 10, false, neg, &mut num_buf);
                w.push_bytes(slice, width, left_align, zero_pad);
            }
            b'u' | b'x' | b'X' | b'o' | b'p' => {
                let val: u64 = if spec == b'p' {
                    unsafe { ap.next_arg::<usize>() as u64 }
                } else if is_long_long {
                    unsafe { ap.next_arg::<u64>() }
                } else {
                    unsafe { ap.next_arg::<u32>() as u64 }
                };
                let (base, upper) = match spec {
                    b'u' => (10, false),
                    b'o' => (8, false),
                    b'X' => (16, true),
                    _ => (16, false),
                };
                let slice = format_uint(val, base, upper, false, &mut num_buf);
                w.push_bytes(slice, width, left_align, zero_pad);
            }
            other => {
                w.push_byte(b'%');
                w.push_byte(other);
            }
        }
    }

    w.finish()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn snprintf_(
    s: *mut c_char,
    n: usize,
    format: *const c_char,
    args: ...
) -> c_int {
    unsafe { vsnprintf_(s, n, format, args) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sprintf_(s: *mut c_char, format: *const c_char, args: ...) -> c_int {
    unsafe { vsnprintf_(s, usize::MAX, format, args) }
}
