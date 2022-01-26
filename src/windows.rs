use std::{
    io::{self, Write},
    mem::MaybeUninit,
    os::windows::io::{AsRawHandle, RawHandle},
};

pub fn wait(msg: &str) -> io::Result<()> {
    let stdin = io::stdin();
    // Lock the handle until we finish waiting.
    let handle = stdin.lock();
    let handle = handle.as_raw_handle();

    // `io::Stdin` doesn't guarantee that the handle is valid
    // before it is read so we need to check it ourselves.
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    } else if handle.is_null() || !is_console(handle) {
        // Prompt and wait only if running in a console.
        return Ok(());
    }

    prompt(msg)?;
    read_key(handle)
}

fn prompt(msg: &str) -> io::Result<()> {
    if msg.is_empty() {
        return Ok(());
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(msg.as_bytes())?;
    handle.flush()
}

// See: https://docs.microsoft.com/en-us/windows/console/input-record-str
#[repr(C)]
union InputRecord {
    event_type: u16,
    // `INPUT_RECORD` has a size of 20 and an alignment of 4.
    uninit: MaybeUninit<[u32; 5]>,
}

const FILE_TYPE_CHAR: u16 = 0x0002;
const INVALID_HANDLE_VALUE: RawHandle = !0 as RawHandle;
const KEY_EVENT: u16 = 0x0001;

#[link(name = "kernel32")]
extern "system" {
    fn FlushConsoleInputBuffer(handle: RawHandle) -> i32;
    fn GetFileType(handle: RawHandle) -> u16;
    fn ReadConsoleInputW(handle: RawHandle, buf: *mut InputRecord, len: u32, read: *mut u32)
        -> i32;
}

fn is_console(handle: RawHandle) -> bool {
    unsafe { GetFileType(handle) == FILE_TYPE_CHAR }
}

fn flush_input_buffer(handle: RawHandle) -> io::Result<()> {
    let res = unsafe { FlushConsoleInputBuffer(handle) };
    if res == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn read_key(handle: RawHandle) -> io::Result<()> {
    flush_input_buffer(handle)?;

    let mut rec = InputRecord {
        uninit: MaybeUninit::uninit(),
    };
    let mut read = 0;

    loop {
        let res = unsafe { ReadConsoleInputW(handle, &mut rec, 1, &mut read) };
        if res == 0 {
            return Err(io::Error::last_os_error());
        }
        assert!(read == 1);
        if unsafe { rec.event_type } == KEY_EVENT {
            return Ok(());
        }
    }
}
