use crate::regex::regex_error::RegexError;

#[derive(Debug, Clone)]
pub enum RegexClass {
    Alnum,
    Alpha,
    Digit,
    Lower,
    Upper,
    Space,
    Punct,
}

impl RegexClass {
    pub fn matches(&self, c: char) -> bool {
        match self {
            RegexClass::Alnum => c.is_alphanumeric(),
            RegexClass::Alpha => c.is_alphabetic(),
            RegexClass::Digit => c.is_ascii_digit(),
            RegexClass::Lower => c.is_lowercase(),
            RegexClass::Upper => c.is_uppercase(),
            RegexClass::Space => c.is_whitespace(),
            RegexClass::Punct => c.is_ascii_punctuation(),
        }
    }
}

pub fn determinate_regex_class(chars_class: Vec<char>) -> Result<RegexClass, RegexError> {
    let class: String = chars_class.iter().collect();

    match class.as_str() {
        "alnum" => Ok(RegexClass::Alnum),
        "alpha" => Ok(RegexClass::Alpha),
        "digit" => Ok(RegexClass::Digit),
        "lower" => Ok(RegexClass::Lower),
        "upper" => Ok(RegexClass::Upper),
        "space" => Ok(RegexClass::Space),
        "punct" => Ok(RegexClass::Punct),
        _ => Err(RegexError::InvalidClass),
    }
}
