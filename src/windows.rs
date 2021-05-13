use std::{ffi::c_void, io::{self, Write}};
use std::os::windows::io::AsRawHandle;

pub fn wait(txt: &str) {
    if !txt.is_empty() {
        println!("{}", txt);
    }

    let stdin = io::stdin();
    let handle = stdin.as_raw_handle();
    get_char(handle).unwrap();
}

#[link(name="kernel32")]
extern "C" {
    fn FlushConsoleInputBuffer(handle: *mut c_void) -> i32;
    fn ReadConsoleInputW(handle: *mut c_void, buffer: *mut u8, len: u32, evt_read: *mut u32) -> i32;
}

fn flush_inputs(handle: *mut c_void) -> io::Result<()> {
    let res = unsafe { FlushConsoleInputBuffer(handle) };
    if res == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn get_char(handle: *mut c_void) -> io::Result<()> {
    flush_inputs(handle)?;
    let buff = &mut [0u8];
    let mut evt_read = 032;
    let res = unsafe { ReadConsoleInputW(handle, buff.as_mut_ptr(), 1, &mut evt_read)};

    if res == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}