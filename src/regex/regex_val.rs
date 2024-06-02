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
            RegexVal::Class(_) => 0,
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
