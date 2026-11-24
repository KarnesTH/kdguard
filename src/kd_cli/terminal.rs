use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyType {
    Char,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Enter,
    Escape,
    Backspace,
    Delete,
    Tab,
    Space,
    Interrupt,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    pub kind: KeyType,
    pub text: String,
}

pub struct Terminal;

pub struct RawMode {
    #[cfg(unix)]
    state: Option<sys::Termios>,
    #[cfg(windows)]
    state: Option<u32>,
}

static COLOR_MODE: AtomicU8 = AtomicU8::new(0);
static CURSOR_HIDDEN: AtomicBool = AtomicBool::new(false);
static PREPARED: AtomicBool = AtomicBool::new(false);
const SHOW_CURSOR: &[u8] = b"\x1b[?25h";

impl Key {
    fn of(kind: KeyType) -> Self {
        Key { kind, text: String::new() }
    }
}

impl Terminal {
    /// Sets the color mode: `Auto` colors only a terminal without NO_COLOR, `Always` and `Never` force it.
    pub fn set_color_mode(mode: ColorMode) {
        COLOR_MODE.store(mode as u8, Ordering::Relaxed);
    }

    /// Parses `auto`, `always` or `never` (e.g. from `--color`) and sets the mode. Returns false for
    /// other values.
    pub fn parse_color_mode(mode: &str) -> bool {
        let mode = match mode {
            "auto" => ColorMode::Auto,
            "always" => ColorMode::Always,
            "never" => ColorMode::Never,
            _ => return false,
        };
        Self::set_color_mode(mode);
        true
    }

    /// True when output on stdout is colored.
    pub fn colors() -> bool {
        match COLOR_MODE.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => Self::is_terminal() && std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty()),
        }
    }

    /// True when stdout is a terminal.
    pub fn is_terminal() -> bool {
        sys::is_terminal(sys::STDOUT)
    }

    /// True when stdin and stdout are terminals.
    pub fn interactive() -> bool {
        sys::is_terminal(sys::STDIN) && sys::is_terminal(sys::STDOUT)
    }

    /// Columns of the terminal: from the console, then COLUMNS, otherwise 80.
    pub fn width() -> usize {
        sys::width()
            .or_else(|| std::env::var("COLUMNS").ok()?.parse().ok())
            .filter(|&width| width > 0)
            .unwrap_or(80)
    }

    /// Writes text to stdout and flushes it. On Windows the console is switched to UTF-8 and ANSI
    /// sequences on first use.
    pub fn write(text: &str) {
        Self::prepare();
        let mut stdout = io::stdout().lock();
        let _ = stdout.write_all(text.as_bytes());
        let _ = stdout.flush();
    }

    /// Writes text to stderr and flushes it.
    pub fn write_error(text: &str) {
        Self::prepare();
        let _ = io::stdout().flush();
        let mut stderr = io::stderr().lock();
        let _ = stderr.write_all(text.as_bytes());
        let _ = stderr.flush();
    }

    /// Hides the cursor and installs a handler that shows it again when the program is interrupted.
    pub fn hide_cursor() {
        if !Self::is_terminal() {
            return;
        }
        if !CURSOR_HIDDEN.swap(true, Ordering::SeqCst) {
            sys::install_interrupt_handler();
        }
        Self::write("\x1b[?25l");
    }

    /// Shows the cursor.
    pub fn show_cursor() {
        if CURSOR_HIDDEN.swap(false, Ordering::SeqCst) {
            Self::write("\x1b[?25h");
        }
    }

    /// Reads one key while a `RawMode` is active. Arrow, Home, End and Delete sequences become their
    /// key type, Ctrl+C becomes `Interrupt`, printable input is returned as text.
    pub fn read_key() -> Key {
        sys::read_key()
    }

    fn prepare() {
        if !PREPARED.swap(true, Ordering::SeqCst) {
            sys::prepare();
        }
    }
}

impl RawMode {
    /// Switches stdin to unbuffered input without echo and signals. Restores it on drop.
    pub fn new() -> Self {
        RawMode { state: sys::enable_raw() }
    }
}

impl Default for RawMode {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            sys::restore(state);
        }
    }
}

#[cfg(unix)]
mod sys {
    use super::{CURSOR_HIDDEN, Key, KeyType, SHOW_CURSOR};
    use std::sync::atomic::Ordering;

    pub const STDIN: i32 = 0;
    pub const STDOUT: i32 = 1;
    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;
    const POLLIN: i16 = 1;

    #[cfg(target_os = "linux")]
    mod layout {
        pub type Flag = u32;
        pub type Fds = u64;
        pub const NCCS: usize = 32;
        pub const ICANON: Flag = 0o2;
        pub const ECHO: Flag = 0o10;
        pub const ISIG: Flag = 0o1;
        pub const IEXTEN: Flag = 0o100000;
        pub const IXON: Flag = 0o2000;
        pub const ICRNL: Flag = 0o400;
        pub const VMIN: usize = 6;
        pub const VTIME: usize = 5;
        pub const TIOCGWINSZ: u64 = 0x5413;

        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct Termios {
            pub iflag: Flag,
            pub oflag: Flag,
            pub cflag: Flag,
            pub lflag: Flag,
            pub line: u8,
            pub cc: [u8; NCCS],
            pub ispeed: u32,
            pub ospeed: u32,
        }
    }

    #[cfg(target_os = "macos")]
    mod layout {
        pub type Flag = u64;
        pub type Fds = u32;
        pub const NCCS: usize = 20;
        pub const ICANON: Flag = 0x100;
        pub const ECHO: Flag = 0x8;
        pub const ISIG: Flag = 0x80;
        pub const IEXTEN: Flag = 0x400;
        pub const IXON: Flag = 0x200;
        pub const ICRNL: Flag = 0x100;
        pub const VMIN: usize = 16;
        pub const VTIME: usize = 17;
        pub const TIOCGWINSZ: u64 = 0x40087468;

        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct Termios {
            pub iflag: Flag,
            pub oflag: Flag,
            pub cflag: Flag,
            pub lflag: Flag,
            pub cc: [u8; NCCS],
            pub ispeed: u64,
            pub ospeed: u64,
        }
    }

    pub use layout::Termios;
    use layout::*;

    #[repr(C)]
    struct PollFd {
        fd: i32,
        events: i16,
        revents: i16,
    }

    #[repr(C)]
    #[derive(Default)]
    struct WinSize {
        rows: u16,
        cols: u16,
        x: u16,
        y: u16,
    }

    unsafe extern "C" {
        fn isatty(fd: i32) -> i32;
        fn tcgetattr(fd: i32, termios: *mut Termios) -> i32;
        fn tcsetattr(fd: i32, action: i32, termios: *const Termios) -> i32;
        fn poll(fds: *mut PollFd, count: Fds, timeout: i32) -> i32;
        fn read(fd: i32, buffer: *mut u8, count: usize) -> isize;
        fn write(fd: i32, buffer: *const u8, count: usize) -> isize;
        fn ioctl(fd: i32, request: u64, ...) -> i32;
        fn signal(signal: i32, handler: usize) -> usize;
        fn raise(signal: i32) -> i32;
    }

    pub fn is_terminal(fd: i32) -> bool {
        unsafe { isatty(fd) == 1 }
    }

    pub fn width() -> Option<usize> {
        let mut size = WinSize::default();
        let ok = unsafe { ioctl(STDOUT, TIOCGWINSZ, &mut size as *mut WinSize) } == 0;
        (ok && size.cols > 0).then_some(size.cols as usize)
    }

    pub fn prepare() {}

    extern "C" fn restore_cursor(number: i32) {
        if CURSOR_HIDDEN.load(Ordering::SeqCst) {
            unsafe { write(STDOUT, SHOW_CURSOR.as_ptr(), SHOW_CURSOR.len()) };
        }
        unsafe {
            signal(number, 0);
            raise(number);
        }
    }

    pub fn install_interrupt_handler() {
        let handler = restore_cursor as extern "C" fn(i32) as usize;
        unsafe {
            signal(SIGINT, handler);
            signal(SIGTERM, handler);
        }
    }

    pub fn enable_raw() -> Option<Termios> {
        let mut original = std::mem::MaybeUninit::<Termios>::uninit();
        if unsafe { tcgetattr(STDIN, original.as_mut_ptr()) } != 0 {
            return None;
        }
        let original = unsafe { original.assume_init() };
        let mut raw = original;
        raw.lflag &= !(ICANON | ECHO | ISIG | IEXTEN);
        raw.iflag &= !(IXON | ICRNL);
        raw.cc[VMIN] = 1;
        raw.cc[VTIME] = 0;
        (unsafe { tcsetattr(STDIN, 0, &raw) } == 0).then_some(original)
    }

    pub fn restore(state: Termios) {
        unsafe { tcsetattr(STDIN, 0, &state) };
    }

    fn byte(timeout: Option<i32>) -> Option<u8> {
        if let Some(timeout) = timeout {
            let mut fd = PollFd { fd: STDIN, events: POLLIN, revents: 0 };
            if unsafe { poll(&mut fd, 1, timeout) } <= 0 {
                return None;
            }
        }
        let mut value = 0u8;
        (unsafe { read(STDIN, &mut value, 1) } == 1).then_some(value)
    }

    fn escape() -> Key {
        let Some(prefix) = byte(Some(30)) else {
            return Key::of(KeyType::Escape);
        };
        if prefix != b'[' && prefix != b'O' {
            return Key::of(KeyType::Unknown);
        }
        let kind = match byte(Some(30)) {
            Some(b'A') => KeyType::Up,
            Some(b'B') => KeyType::Down,
            Some(b'C') => KeyType::Right,
            Some(b'D') => KeyType::Left,
            Some(b'H') => KeyType::Home,
            Some(b'F') => KeyType::End,
            Some(code @ b'0'..=b'9') => {
                while let Some(next) = byte(Some(30)) {
                    if next == b'~' {
                        break;
                    }
                }
                match code {
                    b'1' | b'7' => KeyType::Home,
                    b'4' | b'8' => KeyType::End,
                    b'3' => KeyType::Delete,
                    _ => KeyType::Unknown,
                }
            }
            _ => KeyType::Unknown,
        };
        Key::of(kind)
    }

    pub fn read_key() -> Key {
        let Some(first) = byte(None) else {
            return Key::of(KeyType::Escape);
        };
        match first {
            3 => return Key::of(KeyType::Interrupt),
            9 => return Key::of(KeyType::Tab),
            10 | 13 => return Key::of(KeyType::Enter),
            27 => return escape(),
            32 => return Key { kind: KeyType::Space, text: " ".into() },
            8 | 127 => return Key::of(KeyType::Backspace),
            0..32 => return Key::of(KeyType::Unknown),
            _ => {}
        }
        let continuation = match first {
            0xF0.. => 3,
            0xE0.. => 2,
            0xC0.. => 1,
            _ => 0,
        };
        let mut bytes = vec![first];
        for _ in 0..continuation {
            match byte(Some(30)) {
                Some(next) => bytes.push(next),
                None => break,
            }
        }
        Key { kind: KeyType::Char, text: String::from_utf8_lossy(&bytes).into_owned() }
    }
}

#[cfg(windows)]
mod sys {
    use super::{CURSOR_HIDDEN, Key, KeyType, SHOW_CURSOR};
    use std::ffi::c_void;
    use std::sync::atomic::Ordering;

    pub const STDIN: i32 = -10;
    pub const STDOUT: i32 = -11;
    const STDERR: i32 = -12;
    const ENABLE_PROCESSED_INPUT: u32 = 0x1;
    const ENABLE_LINE_INPUT: u32 = 0x2;
    const ENABLE_ECHO_INPUT: u32 = 0x4;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x4;
    const CP_UTF8: u32 = 65001;

    #[repr(C)]
    #[derive(Default)]
    struct BufferInfo {
        size: [i16; 2],
        cursor: [i16; 2],
        attributes: u16,
        window: [i16; 4],
        maximum: [i16; 2],
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(handle: i32) -> *mut c_void;
        fn GetConsoleMode(handle: *mut c_void, mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: *mut c_void, mode: u32) -> i32;
        fn SetConsoleOutputCP(page: u32) -> i32;
        fn GetConsoleScreenBufferInfo(handle: *mut c_void, info: *mut BufferInfo) -> i32;
        fn SetConsoleCtrlHandler(handler: Option<unsafe extern "system" fn(u32) -> i32>, add: i32) -> i32;
        fn WriteConsoleA(handle: *mut c_void, buffer: *const u8, count: u32, written: *mut u32, reserved: *mut c_void) -> i32;
    }

    unsafe extern "C" {
        fn _getwch() -> u16;
    }

    fn mode(handle: i32) -> Option<u32> {
        let mut value = 0u32;
        (unsafe { GetConsoleMode(GetStdHandle(handle), &mut value) } != 0).then_some(value)
    }

    pub fn is_terminal(handle: i32) -> bool {
        mode(handle).is_some()
    }

    pub fn width() -> Option<usize> {
        let mut info = BufferInfo::default();
        let ok = unsafe { GetConsoleScreenBufferInfo(GetStdHandle(STDOUT), &mut info) } != 0;
        ok.then(|| (info.window[2] - info.window[0] + 1) as usize)
    }

    pub fn prepare() {
        unsafe { SetConsoleOutputCP(CP_UTF8) };
        for handle in [STDOUT, STDERR] {
            if let Some(value) = mode(handle) {
                unsafe { SetConsoleMode(GetStdHandle(handle), value | ENABLE_VIRTUAL_TERMINAL_PROCESSING) };
            }
        }
    }

    unsafe extern "system" fn restore_cursor(_event: u32) -> i32 {
        if CURSOR_HIDDEN.load(Ordering::SeqCst) {
            let mut written = 0u32;
            unsafe {
                WriteConsoleA(GetStdHandle(STDOUT), SHOW_CURSOR.as_ptr(), SHOW_CURSOR.len() as u32, &mut written, std::ptr::null_mut())
            };
        }
        0
    }

    pub fn install_interrupt_handler() {
        unsafe { SetConsoleCtrlHandler(Some(restore_cursor), 1) };
    }

    pub fn enable_raw() -> Option<u32> {
        let original = mode(STDIN)?;
        let raw = original & !(ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
        (unsafe { SetConsoleMode(GetStdHandle(STDIN), raw) } != 0).then_some(original)
    }

    pub fn restore(state: u32) {
        unsafe { SetConsoleMode(GetStdHandle(STDIN), state) };
    }

    pub fn read_key() -> Key {
        let first = unsafe { _getwch() };
        if first == 0 || first == 0xE0 {
            let kind = match unsafe { _getwch() } {
                72 => KeyType::Up,
                80 => KeyType::Down,
                75 => KeyType::Left,
                77 => KeyType::Right,
                71 => KeyType::Home,
                79 => KeyType::End,
                83 => KeyType::Delete,
                _ => KeyType::Unknown,
            };
            return Key::of(kind);
        }
        match first {
            3 => Key::of(KeyType::Interrupt),
            8 => Key::of(KeyType::Backspace),
            9 => Key::of(KeyType::Tab),
            13 => Key::of(KeyType::Enter),
            27 => Key::of(KeyType::Escape),
            32 => Key { kind: KeyType::Space, text: " ".into() },
            0..32 => Key::of(KeyType::Unknown),
            _ => Key { kind: KeyType::Char, text: String::from_utf16_lossy(&[first]) },
        }
    }
}