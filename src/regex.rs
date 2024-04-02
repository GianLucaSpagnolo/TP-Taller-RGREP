use std::collections::VecDeque;

pub mod regex_class;
pub mod regex_rep;
pub mod regex_val;

//use regex_class::RegexClass;
use regex_rep::RegexRep;
use regex_val::RegexVal;

#[derive(Debug, Clone)]
pub struct RegexStep {
    val: RegexVal,
    rep: RegexRep,
}

#[derive(Debug, Clone)]
pub struct EvaluatedStep {
    step: RegexStep,
    match_size: usize,
    backtrackable: bool,
}

#[derive(Debug, Clone)]
pub struct Regex {
    pub steps: Vec<RegexStep>,
}

#[derive(Debug, Clone)]
pub struct LineEvaluated {
    pub result: bool,
    pub line: String,
}

impl TryFrom<&str> for Regex {
    type Error = &'static str;

    fn try_from(expression: &str) -> Result<Self, Self::Error> {
        let mut steps: Vec<RegexStep> = vec![];

        let mut chars_iter = expression.chars();
        while let Some(c) = chars_iter.next() {
            let step = match c {
                '.' => Some(RegexStep {
                    rep: RegexRep::Exact(1),
                    val: RegexVal::Wildcard,
                }),
                'a'..='z' | 'A'..='Z' | '0'..='9' => Some(RegexStep {
                    rep: RegexRep::Exact(1),
                    val: RegexVal::Literal(c),
                }),
                '*' => {
                    if let Some(last) = steps.last_mut() {
                        last.rep = RegexRep::Any;
                    } else {
                        Some(RegexStep {
                            rep: RegexRep::Any,
                            val: RegexVal::Wildcard,
                        });
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
                '+' => {
                    if let Some(last) = steps.last_mut() {
                        last.rep = RegexRep::Range {
                            min: Some(1),
                            max: None,
                        };
                    } else {
                        return Err("Invalid regex: unexpected + character");
                    }
                    None
                }
                '{' => {
                    if let Some(last) = steps.last_mut() {
                        let mut min = None;
                        let mut max = None;
                        let mut count = 0;
                        let mut is_comma = false;
                        let mut is_end = false;
                        let mut is_invalid = false;

                        while let Some(c) = chars_iter.next() {
                            match c {
                                '0'..='9' => {
                                    count = count * 10 + c.to_digit(10).unwrap() as usize;
                                }
                                ',' => {
                                    if is_comma {
                                        is_invalid = true;
                                        break;
                                    }
                                    is_comma = true;

                                    if count > 0 {
                                        min = Some(count);
                                        count = 0;
                                    }
                                }
                                '}' => {
                                    is_end = true;
                                    break;
                                }
                                _ => {
                                    is_invalid = true;
                                    break;
                                }
                            }
                        }

                        if is_invalid || !is_end {
                            return Err("Invalid regex: invalid range");
                        }

                        if count > 0 {
                            max = Some(count);
                        }

                        if !is_comma {
                            last.rep = RegexRep::Exact(count);
                        } else {
                            last.rep = RegexRep::Range { min, max };
                        }
                    } else {
                        return Err("Invalid regex: unexpected { character");
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

    pub fn evaluate(self, value: &str) -> Result<LineEvaluated, &str> {
        if !value.is_ascii() {
            return Err("Non-ascii characters in input");
        }

        let mut queue = VecDeque::from(self.steps);
        let mut stack: Vec<EvaluatedStep> = Vec::new();
        let mut index = 0;

        'steps: while let Some(step) = queue.pop_front() {
            //println!("{:?}", step.rep);
            match step.rep {
                RegexRep::Exact(n) => {
                    let mut match_size = 0;
                    for _ in 0..n {
                        let size = step.val.matches(&value[index..]);

                        if size == 0 {
                            match backtrack(step, &mut stack, &mut queue) {
                                Some(size) => {
                                    index -= size;
                                    continue 'steps;
                                }
                                None => {
                                    return Ok(LineEvaluated {
                                        result: false,
                                        line: value.to_string(),
                                    });
                                }
                            }
                        } else {
                            match_size += size;
                            index += size;
                        }
                    }
                    stack.push(EvaluatedStep {
                        step: step,
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
                                step: step.clone(),
                                match_size,
                                backtrackable: true,
                            });
                        } else {
                            keep_matching = false;
                        }
                    }
                }
                RegexRep::Range { min, max } => {
                    let mut match_size = 0;
                    let mut count = 0;
                    let mut keep_matching = true;
                    while keep_matching {
                        let size = step.val.matches(&value[index..]);

                        if size == 0 {
                            if let Some(min) = min {
                                if count < min {
                                    match backtrack(step, &mut stack, &mut queue) {
                                        Some(size) => {
                                            index -= size;
                                            continue 'steps;
                                        }
                                        None => {
                                            return Ok(LineEvaluated {
                                                result: false,
                                                line: value.to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                            keep_matching = false;
                        } else {
                            match_size += size;
                            index += size;
                            count += 1;
                        }

                        if let Some(max) = max {
                            if count >= max {
                                keep_matching = false;
                            }
                        }
                    }
                    stack.push(EvaluatedStep {
                        step: step,
                        match_size,
                        backtrackable: false,
                    });
                }
            }
        }

        Ok(LineEvaluated {
            result: true,
            line: value.to_string(),
        })
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
            return Some(back_size);
        } else {
            next.push_front(e.step);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii() {
        let value = "abacdef";

        let regex = Regex::new("ab.*c").unwrap();

        let matches = regex.evaluate(value);
        assert!(matches.is_ok());
        let line = matches.unwrap();
        assert_eq!(line.result, true);
    }

    #[test]
    fn test_no_ascii() {
        let value = "abacdதிf";

        let regex = Regex::new("ab.*c").unwrap();

        let matches = regex.evaluate(value);
        assert!(matches.is_err());
        assert_eq!(
            matches.unwrap_err().to_string(),
            "Non-ascii characters in input"
        );
    }

    #[test]
    fn test_match_point() -> Result<(), &'static str> {
        let value = "abcdefg";

        let regex = Regex::new(".").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_match_multiple_points() -> Result<(), &'static str> {
        let value = "abcdefg";

        let regex = Regex::new("...").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_match_more_points_than_letters() -> Result<(), &'static str> {
        let value = "abc";

        let regex = Regex::new("....").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, false);

        Ok(())
    }

    #[test]
    fn test_match_literal() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("a").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_match_multiple_literal() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("abc").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    /*
    #[test]
    fn test_match_middle_literals() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("cde").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_middle_literals() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("ce").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, false);

        Ok(())
    }
    */

    #[test]
    fn test_match_literal_and_point() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("a.c").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_literal_and_point() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("a.d").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, false);

        Ok(())
    }

    #[test]
    fn test_match_multiple_literal_and_point() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("a.c..f").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_match_point_and_asterisk() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("ab.*e").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_point_and_asterisk() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("ab.*h").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, false);

        Ok(())
    }

    #[test]
    fn test_match_2_point_and_asterisk() -> Result<(), &'static str> {
        let value = "ab1234cdefg";

        let regex = Regex::new("ab.*c.*f").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_2_point_and_asterisk() -> Result<(), &'static str> {
        let value = "ab1234cdegh";

        let regex = Regex::new("ab.*c.*f").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, false);

        Ok(())
    }

    #[test]
    fn test_match_literal_and_asterisk() -> Result<(), &'static str> {
        let value = "ab111cde";

        let regex = Regex::new("ab1*").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_literal_and_asterisk() -> Result<(), &'static str> {
        let value = "ab111cde";

        let regex = Regex::new("ab2*").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_multiple_literal_and_asterisk() -> Result<(), &'static str> {
        let value = "ab111cde";

        let regex = Regex::new("ab2*g*3*").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_match_single_asterisk() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("*").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_match_point_and_asterisk_at_start() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new(".*").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_match_single_asterisk_and_literal() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new(".*fgh").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_match_question_mark() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("abcd?").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_question_mark() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("abcr?").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_match_question_mark_and_literal() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("abc?de.g.*").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_incorrect_question_mark() -> Result<(), &'static str> {
        let regex = Regex::new("?cd").unwrap_err();
        assert_eq!(regex, "Invalid regex: unexpected ? character");

        Ok(())
    }

    #[test]
    fn test_match_single_plus() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("abcd+").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_plus() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("abce+").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, false);

        Ok(())
    }

    #[test]
    fn test_incorrect_plus() -> Result<(), &'static str> {
        let regex = Regex::new("+cd").unwrap_err();
        assert_eq!(regex, "Invalid regex: unexpected + character");

        Ok(())
    }

    #[test]
    fn test_match_multiple_literal_plus() -> Result<(), &'static str> {
        let value = "abcddddddddef";

        let regex = Regex::new("abcd+").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_match_question_mark_literal_and_plus() -> Result<(), &'static str> {
        let value = "abcdefghijklllllllm";

        let regex = Regex::new("abc?de.g.*l+").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    /*#[test]
    fn test_match_middle_repetition() -> Result<(), &'static str> {
        let value = "abcccccdeeeeeefghij";

        let regex = Regex::new("c*de+fg.i?").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }*/

    #[test]
    fn test_match_range_combination_with_start_and_end() -> Result<(), &'static str> {
        let value = "abccccc";

        let regex = Regex::new("abc{2,10}").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_range_combination_with_start_and_end() -> Result<(), &'static str> {
        let value = "abc";

        let regex = Regex::new("abc{2,10}").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, false);

        Ok(())
    }

    #[test]
    fn test_match_range_combination_exact() -> Result<(), &'static str> {
        let value = "abccccc33";

        let regex = Regex::new("abc{5}").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_range_combination_exact() -> Result<(), &'static str> {
        let value = "abcc33";

        let regex = Regex::new("abc{5}").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, false);

        Ok(())
    }

    #[test]
    fn test_match_range_combination_only_start() -> Result<(), &'static str> {
        let value = "abccccc";

        let regex = Regex::new("abc{2,}").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_range_combination_only_start() -> Result<(), &'static str> {
        let value = "abc";

        let regex = Regex::new("abc{2,}").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, false);

        Ok(())
    }

    #[test]
    fn test_match_range_combination_only_end() -> Result<(), &'static str> {
        let value = "abcccd";

        let regex = Regex::new("abc{,5}").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, true);

        Ok(())
    }

    #[test]
    fn test_no_match_range_combination_only_end() -> Result<(), &'static str> {
        let value = "abccccccd";

        let regex = Regex::new("abc{,5}d").unwrap();

        let line = regex.evaluate(value)?;
        assert_eq!(line.result, false);

        Ok(())
    }
}
