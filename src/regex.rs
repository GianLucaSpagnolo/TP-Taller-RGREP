use std::collections::VecDeque;

pub mod regex_class;
pub mod regex_error;
pub mod regex_rep;
pub mod regex_val;

//use regex_class::RegexClass;
use regex_error::RegexError;
use regex_rep::RegexRep;
use regex_val::RegexVal;

#[derive(Debug, Clone)]
pub struct RegexStep {
    pub val: RegexVal,
    pub rep: RegexRep,
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
                    if steps.is_empty() {
                        Some(RegexStep {
                            rep: RegexRep::Any,
                            val: RegexVal::Wildcard,
                        })
                    } else {
                        if let Some(last) = steps.last_mut() {
                            last.rep = RegexRep::Any;
                        }
                        None
                    }
                }
                '?' => {
                    if steps.is_empty() {
                        Some(RegexStep {
                            rep: RegexRep::Range {
                                min: Some(0),
                                max: Some(1),
                            },
                            val: RegexVal::Wildcard,
                        })
                    } else {
                        if let Some(last) = steps.last_mut() {
                            last.rep = RegexRep::Range {
                                min: Some(0),
                                max: Some(1),
                            };
                        }
                        None
                    }
                }
                '+' => {
                    if steps.is_empty() {
                        Some(RegexStep {
                            rep: RegexRep::Range {
                                min: Some(1),
                                max: None,
                            },
                            val: RegexVal::Wildcard,
                        })
                    } else {
                        if let Some(last) = steps.last_mut() {
                            last.rep = RegexRep::Range {
                                min: Some(1),
                                max: None,
                            };
                        }
                        None
                    }
                }
                '{' => {
                    if let Some(last) = steps.last_mut() {
                        let mut min = None;
                        let mut max = None;
                        let mut count = 0;
                        let mut is_comma = false;
                        let mut is_end = false;
                        let mut is_invalid = false;

                        for c in chars_iter.by_ref() {
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
                            return Err(RegexError::InvalidRange.message());
                        }

                        if count > 0 {
                            max = Some(count);
                        }

                        if !is_comma {
                            last.rep = RegexRep::Exact(count);
                        } else {
                            last.rep = RegexRep::Range { min, max };
                        }
                    }
                    None
                }
                '\\' => match chars_iter.next() {
                    Some(literal) => Some(RegexStep {
                        rep: RegexRep::Exact(1),
                        val: RegexVal::Literal(literal),
                    }),
                    None => return Err(RegexError::InvalidBackslash.message()),
                },
                _ => return Err(RegexError::InvalidCharacter.message()),
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
            return Err(RegexError::NoAsciiCharacter.message());
        }

        let mut queue = VecDeque::from(self.steps);
        let queue_size = queue.len();
        let mut state = false;

        if queue_size == 1 && value.is_empty() {
            if let Some(step) = queue.pop_front() {
                match step.val {
                    RegexVal::Wildcard => {
                        state = true;
                    }
                    _ => {
                        queue.push_front(step);
                    }
                }
            }
        }

        for char_index in 0..value.len() {
            let mut stack: Vec<EvaluatedStep> = Vec::new();
            let mut index = char_index;

            'steps: while let Some(step) = queue.pop_front() {
                match step.rep {
                    RegexRep::Exact(n) => {
                        let mut match_size = 0;
                        for i in 0..n {
                            let size = step.val.matches(&value[index..]);

                            if size == 0 {
                                match backtrack(step, &mut stack, &mut queue) {
                                    Some(size) => {
                                        index -= size;
                                        continue 'steps;
                                    }
                                    None => {
                                        break 'steps;
                                    }
                                }
                            } else {
                                if queue.is_empty() && i == n - 1 {
                                    state = true;
                                    break 'steps;
                                }
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
                        let mut is_match = false;
                        let mut keep_matching = true;
                        while keep_matching {
                            let match_size = step.val.matches(&value[index..]);

                            if match_size != 0 {
                                is_match = true;
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

                        if queue.is_empty() {
                            state = true;
                            break 'steps;
                        }
                        if !is_match {
                            continue 'steps;
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
                                                break 'steps;
                                            }
                                        }
                                    } else if min == 0 {
                                        state = true;
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
                        if queue.is_empty() {
                            state = true;
                            break 'steps;
                        }

                        stack.push(EvaluatedStep {
                            step,
                            match_size,
                            backtrackable: false,
                        });
                    }
                }
            }

            if !queue.is_empty() {
                queue.rotate_left(queue_size - queue.len());
            } else {
                break;
            }
        }

        Ok(LineEvaluated {
            result: state,
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
        assert!(line.result);
    }

    #[test]
    fn test_no_ascii() {
        let value = "abacdதிf";

        let regex = Regex::new("ab.*c").unwrap();

        let matches = regex.evaluate(value);
        assert!(matches.is_err());
        assert_eq!(
            matches.unwrap_err().to_string(),
            RegexError::NoAsciiCharacter.message()
        );
    }

    #[test]
    fn test_match_point() -> Result<(), &'static str> {
        let value = "abcdefg";

        let regex = Regex::new(".").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_multiple_points() -> Result<(), &'static str> {
        let value = "abcdefg";

        let regex = Regex::new("...").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_more_points_than_letters() -> Result<(), &'static str> {
        let value = "abc";

        let regex = Regex::new("....").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    #[test]
    fn test_match_literal() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("a").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_multiple_literal() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("abc").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_middle_literals() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("cde").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_middle_literals() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("ce").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    #[test]
    fn test_match_literal_and_point() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("a.c").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_literal_and_point() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("a.d").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    #[test]
    fn test_match_multiple_literal_and_point() -> Result<(), &'static str> {
        let value = "abcdefghijk";

        let regex = Regex::new("c..f..i").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_point_and_asterisk() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("ab.*e").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_point_and_asterisk() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("ab.*h").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    #[test]
    fn test_match_2_point_and_asterisk() -> Result<(), &'static str> {
        let value = "ab1234cdefg";

        let regex = Regex::new("ab.*c.*f").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_2_point_and_asterisk() -> Result<(), &'static str> {
        let value = "ab1234cdegh";

        let regex = Regex::new("ab.*c.*f").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    #[test]
    fn test_match_literal_and_asterisk() -> Result<(), &'static str> {
        let value = "ab111cde";

        let regex = Regex::new("ab1*").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_literal_and_asterisk() -> Result<(), &'static str> {
        let value = "ab111cde";

        let regex = Regex::new("ab2*").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_multiple_literal_and_asterisk() -> Result<(), &'static str> {
        let value = "ab111cde";

        let regex = Regex::new("ab2*g*3*").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_single_asterisk() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("*").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_single_point_and_asterisk() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new(".*").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_point_and_asterisk_at_start() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new(".*abcd").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_single_asterisk_and_literal() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new(".*fgh").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_question_mark() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("abcd?").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_question_mark() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("abcr?").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_question_mark_and_literal() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("abc?de.g.*").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_single_plus() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("abcd+").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_plus() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("abce+").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    #[test]
    fn test_match_multiple_literal_plus() -> Result<(), &'static str> {
        let value = "abcddddddddef";

        let regex = Regex::new("abcd+").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_question_mark_literal_and_plus() -> Result<(), &'static str> {
        let value = "abcdefghijklllllllm";

        let regex = Regex::new("abc?de.g.*l+").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_middle_repetition() -> Result<(), &'static str> {
        let value = "abcccccdeeeeeefghij";

        let regex = Regex::new("c*de+fg.i?").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_only_plus() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("+").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_only_point_and_plus() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new(".+").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_only_question_mark() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new("?").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_only_point_and_question_mark() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex = Regex::new(".?").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_match_empty_line() -> Result<(), &'static str> {
        let value = "";

        let regex1 = Regex::new("*").unwrap();
        let regex2 = Regex::new("+").unwrap();
        let regex3 = Regex::new("?").unwrap();

        let line1 = regex1.evaluate(value)?;
        let line2 = regex2.evaluate(value)?;
        let line3 = regex3.evaluate(value)?;

        assert!(line1.result);
        assert!(line2.result);
        assert!(line3.result);

        Ok(())
    }

    #[test]
    fn test_match_start_with_repetition() -> Result<(), &'static str> {
        let value = "testeo";

        let regex1 = Regex::new("*esteo").unwrap();
        let regex2 = Regex::new("+esteo").unwrap();
        let regex3 = Regex::new("?esteo").unwrap();

        let line1 = regex1.evaluate(value)?;
        let line2 = regex2.evaluate(value)?;
        let line3 = regex3.evaluate(value)?;

        assert!(line1.result);
        assert!(!line2.result);
        assert!(line3.result);

        Ok(())
    }

    #[test]
    fn test_match_range_combination_with_start_and_end() -> Result<(), &'static str> {
        let value = "abccccc";

        let regex = Regex::new("abc{2,10}").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_range_combination_with_start_and_end() -> Result<(), &'static str> {
        let value = "abc";

        let regex = Regex::new("abc{2,10}").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    #[test]
    fn test_match_range_combination_exact() -> Result<(), &'static str> {
        let value1 = "abccccc33";
        let value2 = "aaa";

        let regex1 = Regex::new("abc{5}").unwrap();
        let regex2 = Regex::new("a{3}").unwrap();

        let line = regex1.evaluate(value1)?;
        assert!(line.result);
        let line = regex2.evaluate(value2)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_range_combination_exact() -> Result<(), &'static str> {
        let value = "abcc33";

        let regex = Regex::new("abc{5}").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_range_combination_exact_2() -> Result<(), &'static str> {
        let value = "abcccccc33";

        let regex = Regex::new("abc{5}3").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    #[test]
    fn test_match_range_combination_only_start() -> Result<(), &'static str> {
        let value = "abccccc";

        let regex = Regex::new("abc{2,}").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_range_combination_only_start() -> Result<(), &'static str> {
        let value = "abc";

        let regex = Regex::new("abc{2,}").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    #[test]
    fn test_match_range_combination_only_end() -> Result<(), &'static str> {
        let value = "abcccd";

        let regex = Regex::new("abc{,5}").unwrap();

        let line = regex.evaluate(value)?;
        assert!(line.result);

        Ok(())
    }

    #[test]
    fn test_no_match_range_combination_only_end() -> Result<(), &'static str> {
        let value = "abccccccd";

        let regex = Regex::new("abc{,5}d").unwrap();

        let line = regex.evaluate(value)?;
        assert!(!line.result);

        Ok(())
    }

    /*
    TESTS PARA OTRO MOMENTO
    #[test]
    fn test_match_only_ranged_expression() -> Result<(), &'static str> {
        let value = "abcdefghij";

        let regex1 = Regex::new("{3}").unwrap();
        let regex2 = Regex::new("{,3}").unwrap();
        let regex3 = Regex::new("{3,}").unwrap();
        let regex4 = Regex::new("{3,5}").unwrap();

        let line1 = regex1.evaluate(value)?;
        let line2 = regex2.evaluate(value)?;
        let line3 = regex3.evaluate(value)?;
        let line4 = regex4.evaluate(value)?;

        assert!(line1.result);
        assert!(line2.result);
        assert!(line3.result);
        assert!(line4.result);

        Ok(())
    }

    #[test]
    fn test_match_empty_line_with_ranged_expression() -> Result<(), &'static str> {
        let value = "";

        let regex1 = Regex::new("{3}").unwrap();
        let regex2 = Regex::new("{3,}").unwrap();
        let regex3 = Regex::new("{,3}").unwrap();
        let regex4 = Regex::new("{3,5}").unwrap();

        let line1 = regex1.evaluate(value)?;
        let line2 = regex2.evaluate(value)?;
        let line3 = regex3.evaluate(value)?;
        let line4 = regex4.evaluate(value)?;

        assert!(line1.result);
        assert!(line2.result);
        assert!(line3.result);
        assert!(line4.result);

        Ok(())
    }

    #[test]
    fn test_match_start_with_ranged_expression() -> Result<(), &'static str> {
        let value = "testeo";

        let regex1 = Regex::new("{5}esteo").unwrap();
        let regex2 = Regex::new("{2,}esteo").unwrap();
        let regex3 = Regex::new("{,2}esteo").unwrap();
        let regex4 = Regex::new("{2,5}esteo").unwrap();

        let line1 = regex1.evaluate(value)?;
        let line2 = regex2.evaluate(value)?;
        let line3 = regex3.evaluate(value)?;
        let line4 = regex4.evaluate(value)?;

        assert!(line1.result);
        assert!(line2.result);
        assert!(line3.result);
        assert!(line4.result);

        Ok(())
    }*/
}
