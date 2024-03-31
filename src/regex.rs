use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RegexStep {
    val: RegexVal,
    rep: RegexRep,
}

#[derive(Debug, Clone)]
pub enum RegexVal {
    Literal(char),
    Wildcard,
    Class(RegexClass),
}

#[derive(Debug, Clone)]
pub enum RegexRep {
    Any,
    Exact(usize),
    Range {
        min: Option<usize>,
        max: Option<usize>,
    },
}

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

#[derive(Debug, Clone)]
pub struct EvaluatedStep {
    step: RegexStep,
    match_size: usize,
    backtrackable: bool,
}

#[derive(Debug, Clone)]
pub struct Regex {
    steps: Vec<RegexStep>,
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

impl TryFrom<&str> for Regex {
    type Error = &'static str;

    fn try_from(expression: &str) -> Result<Self, Self::Error> {
        let mut steps: Vec<RegexStep> = vec![]; // Vec::new()

        // expression.chars.for_each(f) no sirve porque a veces puedo moverme mas de una posicion

        let mut chars_iter = expression.chars();
        while let Some(c) = chars_iter.next() {
            let step = match c {
                // pattern matching
                '.' => Some(RegexStep {
                    rep: RegexRep::Exact(1),
                    val: RegexVal::Wildcard,
                }),
                'a'..='z' | 'A'..='Z' => Some(RegexStep {
                    rep: RegexRep::Exact(1),
                    val: RegexVal::Literal(c),
                }),
                '*' => {
                    if let Some(last) = steps.last_mut() {
                        last.rep = RegexRep::Any;
                    } else {
                        return Err("Invalid regex: cant start with *");
                    }
                    None
                }
                '?' => {
                    if let Some(last) = steps.last_mut() {
                        last.rep = RegexRep::Range {
                            min: Some(0),
                            max: Some(1),
                        };
                    } else {
                        return Err("Invalid regex: unexpected ? character");
                    }
                    None
                }
                '\\' => match chars_iter.next() {
                    Some(literal) => Some(RegexStep {
                        rep: RegexRep::Exact(1),
                        val: RegexVal::Literal(literal),
                    }),
                    None => return Err("Invalid regex: invalid backslash"),
                },
                _ => return Err("Invalid character in regex"),
            };

            if let Some(s) = step {
                steps.push(s);
            }
        }

        Ok(Regex { steps })
    }
}

impl Regex {
    pub fn new(expression: &str) -> Result<Self, &str> {
        Regex::try_from(expression)
    }

    pub fn evaluate(self, value: &str) -> Result<bool, &str> {
        if !value.is_ascii() {
            return Err("Non-ascii characters in input");
        }

        let mut queue = VecDeque::from(self.steps);
        let mut stack: Vec<EvaluatedStep> = Vec::new();
        let mut index = 0;

        'steps: while let Some(step) = queue.pop_front() {
            match step.rep {
                RegexRep::Exact(n) => {
                    let mut match_size = 0;
                    for _ in [1..n] {
                        let size = step.val.matches(&value[index..]);

                        if size == 0 {
                            match backtrack(step, &mut stack, &mut queue) {
                                Some(size) => {
                                    index -= size;
                                    continue 'steps;
                                    // no queremos que continue para no registrar al step como evaluado
                                }
                                None => return Ok(false),
                            }
                        } else {
                            match_size += size;
                            index += size;
                        }
                    }
                    stack.push(EvaluatedStep {
                        step,
                        match_size,
                        backtrackable: false,
                    })
                }
                RegexRep::Any => {
                    let mut keep_matching = true;
                    while keep_matching {
                        let match_size = step.val.matches(&value[index..]);

                        if match_size != 0 {
                            index += match_size;
                            stack.push(EvaluatedStep {
                                // estoy clonando este step porque realmente necesito duplicarlo
                                step: step.clone(),
                                match_size,
                                backtrackable: true,
                            });
                        } else {
                            keep_matching = false;
                        }
                    }
                }
                //RegexRep::Range { min, max } => todo!(),
                _ => return Ok(false),
            }
        }

        Ok(true)
    }
}

fn backtrack(
    current: RegexStep,
    evaluated: &mut Vec<EvaluatedStep>,
    next: &mut VecDeque<RegexStep>,
) -> Option<usize> {
    let mut back_size = 0;
    next.push_front(current);
    while let Some(e) = evaluated.pop() {
        back_size += e.match_size;
        if e.backtrackable {
            println!("Backtrack {}", back_size);
            return Some(back_size);
        } else {
            next.push_front(e.step);
        }
    }
    None
}
