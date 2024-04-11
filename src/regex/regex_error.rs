#[derive (Debug)]
pub enum RegexError {
    InvalidRange,
    InvalidBackslash,
    InvalidCharacter,
}

impl RegexError {
    pub fn message(&self) -> &str {
        match self {
            RegexError::InvalidRange => "Invalid regex: invalid range",
            RegexError::InvalidBackslash => "Invalid regex: invalid backslash",
            RegexError::InvalidCharacter => "Invalid character in regex",
        }
    }
}