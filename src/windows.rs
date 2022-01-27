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

#[repr(C)]
struct KeyEventRecord {
    key_down: i32,
    repeat_count: u16,
    virtual_key_code: u16,
    virtual_scan_code: u16,
    // A union between WCHAR and CHAR. A `u16` is fine here because
    // we only use the Unicode variant of `ReadConsoleInput`.
    char: u16,
    control_key_state: u32,
}

// `INPUT_RECORD` structure has a size of 20 and an alignment of 4.
// See: https://docs.microsoft.com/en-us/windows/console/input-record-str
#[repr(C)]
struct InputRecord {
    event_type: u16,
    // A union between several types of record.
    // We only care about key events so a `MaybeUninit` is enough.
    // Also one `KeyEventRecord` exactly makes the right size and alignment of this struct.
    key_event: MaybeUninit<KeyEventRecord>,
}

const INVALID_HANDLE_VALUE: RawHandle = !0 as RawHandle;

const FILE_TYPE_CHAR: u16 = 0x0002;

const EVENT_TYPE_KEY: u16 = 0x0001;

const KEY_SHIFT: u16 = 0x10;
const KEY_CTRL: u16 = 0x11;
const KEY_ALT: u16 = 0x12;
const KEY_CAPS_LOCK: u16 = 0x14;
const KEY_PRINT_SCREEN: u16 = 0x2c;
const KEY_LEFT_WINDOWS: u16 = 0x5b;
const KEY_RIGHT_WINDOWS: u16 = 0x5c;
const KEY_APPLICATIONS: u16 = 0x5d;
const KEY_NUM_LOCK: u16 = 0x90;
const KEY_SCROLL_LOCK: u16 = 0x91;

#[rustfmt::skip]
#[link(name = "kernel32")]
extern "system" {
    fn FlushConsoleInputBuffer(handle: RawHandle) -> i32;
    fn GetFileType(handle: RawHandle) -> u16;
    fn ReadConsoleInputW(handle: RawHandle, buf: *mut InputRecord, len: u32, read: *mut u32) -> i32;
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

fn is_mod_key(key: u16) -> bool {
    matches!(
        key,
        KEY_SHIFT
            | KEY_CTRL
            | KEY_ALT
            | KEY_CAPS_LOCK
            | KEY_PRINT_SCREEN
            | KEY_LEFT_WINDOWS
            | KEY_RIGHT_WINDOWS
            | KEY_APPLICATIONS
            | KEY_NUM_LOCK
            | KEY_SCROLL_LOCK
    )
}

fn read_key(handle: RawHandle) -> io::Result<()> {
    flush_input_buffer(handle)?;

    let mut rec = InputRecord {
        event_type: 0,
        key_event: MaybeUninit::uninit(),
    };
    let mut read = 0;

    loop {
        let res = unsafe { ReadConsoleInputW(handle, &mut rec, 1, &mut read) };
        if res == 0 {
            return Err(io::Error::last_os_error());
        }
        // MS Docs: The function does not return until at least one input record has been read.
        // Still, we check this just in case.
        assert!(read == 1);

        if rec.event_type == EVENT_TYPE_KEY {
            let evt = unsafe { rec.key_event.assume_init_ref() };
            // Ignore key-up events and mod keys.
            if evt.key_down != 0 && !is_mod_key(evt.virtual_key_code) {
                return Ok(());
            }
        }
    }
}
