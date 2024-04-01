#[derive(Debug, Clone)]
pub enum RegexRep {
    Any,
    Exact(usize),
    Range {
        min: Option<usize>,
        max: Option<usize>,
    },
}
