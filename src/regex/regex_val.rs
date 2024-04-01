use super::regex_class::RegexClass;

#[derive(Debug, Clone)]
pub enum RegexVal {
    Literal(char),
    Wildcard,
    Class(RegexClass),
}

impl RegexVal {
    pub fn matches(&self, value: &str) -> usize {
        match self {
            RegexVal::Literal(l) => {
                if value.chars().next() == Some(*l) {
                    println!("Matcheo literal {} size: {}", l, l.len_utf8());
                    l.len_utf8() // cantidad consumida en el input (1 porque es caracter ascii)
                } else {
                    println!("No matcheo literal {}", l);
                    0
                }
            }
            RegexVal::Wildcard => {
                if let Some(c) = value.chars().next() {
                    println!("Matcheo wildcard size: {}", c.len_utf8());
                    c.len_utf8() // cantidad consumida en el input (1 porque es caracter ascii)
                } else {
                    println!("No matcheo wildcard");
                    0
                }
            }
            RegexVal::Class(_) => 0,
        }
    }
}
