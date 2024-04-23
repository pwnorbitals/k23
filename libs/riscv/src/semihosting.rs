use core::arch::asm;
use core::fmt::{Error, Write};
use core::{fmt, slice};

const OPEN: usize = 0x01;
const WRITE: usize = 0x05;
const OPEN_W_TRUNC: usize = 4;

pub struct HostStream(usize);

impl HostStream {
    pub fn new_stdout() -> Self {
        Self(open(":tt\0", OPEN_W_TRUNC).unwrap())
    }
}

impl Write for HostStream {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut buf = s.as_bytes();
        while !buf.is_empty() {
            match unsafe { syscall(WRITE, &[self.0, buf.as_ptr() as usize, buf.len()]) } {
                // Done
                0 => return Ok(()),
                // `n` bytes were not written
                n if n <= buf.len() => {
                    let offset = (buf.len() - n) as isize;
                    buf = unsafe { slice::from_raw_parts(buf.as_ptr().offset(offset), n) }
                }
                // Error
                _ => return Err(Error::default()),
            }
        }
        Ok(())
    }
}

fn open(name: &str, mode: usize) -> Result<usize, ()> {
    let name = name.as_bytes();
    match unsafe { syscall(OPEN, &[name.as_ptr() as usize, mode, name.len() - 1]) } as isize {
        -1 => Err(()),
        fd => Ok(fd as usize),
    }
}

unsafe fn syscall(_nr: usize, _arg: &[usize]) -> usize {
    cfg_if::cfg_if! {
        if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
            let mut nr = _nr;
            let arg = _arg;
            // The instructions below must always be uncompressed, otherwise
            // it will be treated as a regular break, hence the norvc option.
            //
            // See https://github.com/riscv/riscv-semihosting-spec for more details.
            asm!("
                .balign 16
                .option push
                .option norvc
                slli x0, x0, 0x1f
                ebreak
                srai x0, x0, 0x7
                .option pop
                ",
                inout("a0") nr,
                inout("a1") arg.as_ptr() => _,
                options(nostack, preserves_flags),
            );
            nr
        } else {
            unimplemented!();
        }
    }
}