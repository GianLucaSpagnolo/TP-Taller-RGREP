use super::regex_class::RegexClass;

#[derive(Debug, Clone)]
pub enum RegexVal {
    Literal(char),
    Wildcard,
    Class(RegexClass),
    Bracket(Vec<char>),
    NotBracket(Vec<char>),
}

impl RegexVal {
    /// Given a string, returns the size of the amount of characters that match the RegexVal
    ///
    /// # Arguments
    ///
    /// * `value` - A string to be checked
    ///
    /// # Returns
    ///
    /// * usize - The size of the amount of characters that match the RegexVal
    ///
    /// # Examples
    ///
    /// ```
    /// use rgrep::regex::regex_val::*;
    ///
    /// let regex_val = RegexVal::Literal('a');
    ///
    /// let value = "abc";
    /// assert_eq!(regex_val.matches(value), 1);
    /// ```
    ///
    pub fn matches(&self, value: &str) -> usize {
        match self {
            RegexVal::Literal(l) => {
                if value.starts_with(*l) {
                    l.len_utf8()
                } else {
                    0
                }
            }
            RegexVal::Wildcard => {
                if let Some(c) = value.chars().next() {
                    c.len_utf8()
                } else {
                    0
                }
            }
            RegexVal::Class(class) => {
                if let Some(c) = value.chars().next() {
                    if class.matches(c) {
                        c.len_utf8()
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            RegexVal::Bracket(vec) => {
                for c in vec {
                    if value.starts_with(*c) {
                        return c.len_utf8();
                    }
                }
                0
            }
            RegexVal::NotBracket(vec) => {
                for c in vec {
                    if value.starts_with(*c) {
                        return 0;
                    }
                }

                let next_char = value.chars().next();
                if let Some(c) = next_char {
                    c.len_utf8()
                } else {
                    0
                }
            }
        }
    }
}
