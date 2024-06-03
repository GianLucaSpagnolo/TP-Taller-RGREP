#[derive(Debug)]
pub enum RegexError {
    InvalidRange,
    InvalidBackslash,
    NoAsciiCharacter,
    InvalidBracket,
    InvalidClass,
}

impl RegexError {
    pub fn message(&self) -> &str {
        match self {
            RegexError::InvalidRange => "Invalid regex: invalid range",
            RegexError::InvalidBackslash => "Invalid regex: invalid backslash",
            RegexError::NoAsciiCharacter => "Non-ascii characters in input",
            RegexError::InvalidBracket => "Invalid bracket in regex",
            RegexError::InvalidClass => "Invalid character class in regex",
        }
    }
}
