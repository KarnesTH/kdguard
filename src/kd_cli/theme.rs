#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

pub struct Theme;

impl Theme {
    pub const ACCENT: Rgb = Rgb(0xb2, 0x10, 0xf2);
    pub const TEXT: Rgb = Rgb(0xf0, 0xec, 0xf5);
    pub const SECONDARY: Rgb = Rgb(0xc5, 0xbf, 0xd4);
    pub const MUTED: Rgb = Rgb(0x8c, 0x86, 0x9c);
    pub const FAINT: Rgb = Rgb(0x5e, 0x59, 0x6e);
    pub const SUCCESS: Rgb = Rgb(0x6f, 0xd6, 0xa1);
    pub const WARNING: Rgb = Rgb(0xf0, 0xb1, 0x50);
    pub const ERROR: Rgb = Rgb(0xff, 0x7a, 0x8a);
    pub const INFO: Rgb = Rgb(0x3d, 0xae, 0xe9);
}