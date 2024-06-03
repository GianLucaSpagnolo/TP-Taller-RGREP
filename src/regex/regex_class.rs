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
    /// Given a char, returns if it matches the RegexClass
    ///
    /// # Arguments
    ///
    /// * `c` - A char to be checked
    ///
    /// # Returns
    ///
    /// * bool - If the char matches the RegexClass
    ///
    /// # Examples
    ///
    /// ```
    /// use rgrep::regex::regex_class::*;
    ///
    /// let regex_class = RegexClass::Alnum;
    ///
    /// let a = 'a';
    /// assert_eq!(regex_class.matches(a), true);
    /// ```
    ///
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

/// Given a vector of chars, returns the corresponding RegexClass
///
/// # Arguments
///
/// * `class` - A string that represents a RegexClass
///
/// # Returns
///
/// * RegexClass - The corresponding RegexClass if it is valid
/// * RegexError - If the class is invalid
///
/// # Examples
///
/// ```
/// use rgrep::regex::regex_class::*;
///
/// let class = "alnum".to_string();
/// let regex_class = determinate_regex_class(class).unwrap();
///
/// let a = 'a';
/// assert_eq!(regex_class.matches(a), true);
/// ```
///
pub fn determinate_regex_class(class: String) -> Result<RegexClass, RegexError> {
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
