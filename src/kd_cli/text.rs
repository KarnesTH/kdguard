use super::texts::Texts;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

pub struct Text;

impl Text {
    /// Counts the visible columns: one per character, ANSI escape sequences skipped.
    pub fn width(text: &str) -> usize {
        let mut columns = 0;
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                Self::skip_escape(&mut chars);
            } else {
                columns += 1;
            }
        }
        columns
    }

    /// Cuts text to `columns` visible columns ending with `…`. Escape sequences are kept, and a
    /// reset is appended when the text held any.
    pub fn truncate(text: &str, columns: usize) -> String {
        if Self::width(text) <= columns {
            return text.to_string();
        }
        let keep = columns.saturating_sub(1);
        let mut out = String::new();
        let mut visible = 0;
        let mut escaped = false;
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                out.push(c);
                out.push_str(&Self::skip_escape(&mut chars));
                escaped = true;
                continue;
            }
            if visible == keep {
                escaped |= text.contains('\x1b');
                break;
            }
            out.push(c);
            visible += 1;
        }
        if columns > 0 {
            out.push('…');
        }
        if escaped {
            out.push_str("\x1b[0m");
        }
        out
    }

    /// Fills text with spaces to `columns` visible columns on the right (left aligned) or on the
    /// left (right aligned). Longer text is truncated.
    pub fn pad(text: &str, columns: usize, align: Align) -> String {
        let cut = Self::truncate(text, columns);
        let fill = " ".repeat(columns.saturating_sub(Self::width(&cut)));
        match align {
            Align::Left => cut + &fill,
            Align::Right => fill + &cut,
        }
    }

    /// Formats a number with one decimal, using a comma in German.
    pub fn decimal(value: f64) -> String {
        let out = format!("{value:.1}");
        if Texts::get().german { out.replace('.', ",") } else { out }
    }

    /// Formats a byte count as `B`, `KB`, `MB`, `GB` or `TB` with one decimal from KB on.
    pub fn bytes(value: u64) -> String {
        if value < 1024 {
            return format!("{value} B");
        }
        let mut scaled = value as f64;
        let mut unit = "B";
        for next in ["KB", "MB", "GB", "TB"] {
            scaled /= 1024.0;
            unit = next;
            if scaled < 1024.0 {
                break;
            }
        }
        format!("{} {unit}", Self::decimal(scaled))
    }

    /// Formats seconds as `m:ss` or `h:mm:ss`.
    pub fn duration(seconds: u64) -> String {
        let (hours, minutes, rest) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
        if hours > 0 { format!("{hours}:{minutes:02}:{rest:02}") } else { format!("{minutes}:{rest:02}") }
    }

    /// Groups the digits in blocks of three with a space (German) or comma (English).
    pub fn grouped(value: u64) -> String {
        let digits = value.to_string();
        let separator = if Texts::get().german { ' ' } else { ',' };
        let mut out = String::new();
        for (index, digit) in digits.chars().enumerate() {
            if index > 0 && (digits.len() - index).is_multiple_of(3) {
                out.push(separator);
            }
            out.push(digit);
        }
        out
    }

    fn skip_escape(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut sequence = String::new();
        if chars.peek() != Some(&'[') {
            return sequence;
        }
        sequence.push(chars.next().unwrap_or('['));
        for c in chars.by_ref() {
            sequence.push(c);
            if ('\x40'..='\x7e').contains(&c) {
                break;
            }
        }
        sequence
    }
}