use std::fmt::Display;

pub const RUSTC_DEPENDENCIES: [&str; 19] = [
    "std",
    "core",
    "compiler_builtins",
    "rustc_std_workspace_core",
    "alloc",
    "libc",
    "unwind",
    "cfg_if",
    "miniz_oxide",
    "adler",
    "hashbrown",
    "rustc_std_workspace_alloc",
    "std_detect",
    "rustc_demangle",
    "addr2line",
    "gimli",
    "object",
    "memchr",
    "panic_unwind",
];

pub enum TextMod {
    Reset,
    // Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    // Cyan,
    // White,
    // Bold,
    // Underline,
    // Reversed,
}

impl TextMod {
    pub fn apply(&self, text: &str) -> String {
        format!("{}{}{}", self, text, TextMod::Reset)
    }
}

impl Display for TextMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextMod::Reset => write!(f, "\x1b[0m"),
            // TextMod::Red => write!(f, "\x1b[31m"),
            TextMod::Green => write!(f, "\x1b[32m"),
            TextMod::Yellow => write!(f, "\x1b[33m"),
            TextMod::Blue => write!(f, "\x1b[34m"),
            TextMod::Magenta => write!(f, "\x1b[35m"),
            // TextMod::Cyan => write!(f, "\x1b[36m"),
            // TextMod::White => write!(f, "\x1b[37m"),
            // TextMod::Bold => write!(f, "\x1b[1m"),
            // TextMod::Underline => write!(f, "\x1b[4m"),
            // TextMod::Reversed => write!(f, "\x1b[7m"),
        }
    }
}
