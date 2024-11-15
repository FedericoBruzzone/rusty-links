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
        format!("{}{}{}", self.to_string(), text, TextMod::Reset.to_string())
    }
}

impl ToString for TextMod {
    fn to_string(&self) -> String {
        match self {
            TextMod::Reset => "\x1b[0m".to_string(),
            // TextMod::Red => "\x1b[31m".to_string(),
            TextMod::Green => "\x1b[32m".to_string(),
            TextMod::Yellow => "\x1b[33m".to_string(),
            TextMod::Blue => "\x1b[34m".to_string(),
            TextMod::Magenta => "\x1b[35m".to_string(),
            // TextMod::Cyan => "\x1b[36m".to_string(),
            // TextMod::White => "\x1b[37m".to_string(),
            // TextMod::Bold => "\x1b[1m".to_string(),
            // TextMod::Underline => "\x1b[4m".to_string(),
            // TextMod::Reversed => "\x1b[7m".to_string(),
        }
    }
}
