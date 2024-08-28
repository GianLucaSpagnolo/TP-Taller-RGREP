use std::{collections::VecDeque, str::Chars};

pub mod regex_class;
pub mod regex_error;
pub mod regex_rep;
pub mod regex_val;

use regex_class::determinate_regex_class;
use regex_error::RegexError;
use regex_rep::RegexRep;
use regex_val::RegexVal;

#[derive(Debug, Clone)]
pub struct RegexStep {
    pub val: RegexVal,
    pub rep: RegexRep,
    pub anchoring_start: bool,
    pub anchoring_end: bool,
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

/// Point character for a regex
/// "." - Matches any character
///
fn point_char() -> Option<RegexStep> {
    Some(RegexStep {
        rep: RegexRep::Exact(1),
        val: RegexVal::Wildcard,
        anchoring_start: false,
        anchoring_end: false,
    })
}

/// Wildcard character for a regex
/// "*" - Matches zero or more of the preceding element
///
fn wildcard_char(steps: &mut [RegexStep]) -> Option<RegexStep> {
    if steps.is_empty() {
        Some(RegexStep {
            rep: RegexRep::Any,
            val: RegexVal::Wildcard,
            anchoring_start: false,
            anchoring_end: false,
        })
    } else {
        if let Some(last) = steps.last_mut() {
            last.rep = RegexRep::Any;
        }
        None
    }
}

/// Option character for a regex
/// "?" - Matches zero or one of the preceding element
///
fn option_char(steps: &mut [RegexStep]) -> Option<RegexStep> {
    if steps.is_empty() {
        Some(RegexStep {
            rep: RegexRep::Range {
                min: Some(0),
                max: Some(1),
            },
            val: RegexVal::Wildcard,
            anchoring_start: false,
            anchoring_end: false,
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

/// Option one or more character for a regex
/// "+" - Matches one or more of the preceding element
///
fn option_one_or_more_char(steps: &mut [RegexStep]) -> Option<RegexStep> {
    if steps.is_empty() {
        Some(RegexStep {
            rep: RegexRep::Range {
                min: Some(1),
                max: None,
            },
            val: RegexVal::Wildcard,
            anchoring_start: false,
            anchoring_end: false,
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

/// Repetition character for a regex
/// "{" - Matches the preceding element a specified number of times
/// "}" - End of the specified number of times
///
fn repetition_char(
    steps: &mut [RegexStep],
    chars_iter: &mut Chars<'_>,
) -> Result<Option<RegexStep>, &'static str> {
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
    Ok(None)
}

/// Anchor character for a regex
/// "^" - Anchors the regex at the start of the line
///
fn anchor_start_char(anchoring_start: &mut bool) -> Option<RegexStep> {
    *anchoring_start = true;
    None
}

/// Anchor character for a regex
/// "$" - Anchors the regex at the end of the line
///
fn anchor_end_char(steps: &mut Vec<RegexStep>) -> Option<RegexStep> {
    let end_regex = RegexStep {
        rep: RegexRep::Any,
        val: RegexVal::Wildcard,
        anchoring_start: false,
        anchoring_end: false,
    };
    steps.insert(0, end_regex);
    Some(RegexStep {
        rep: RegexRep::Any,
        val: RegexVal::Wildcard,
        anchoring_start: false,
        anchoring_end: true,
    })
}

/// Bracket character for a regex
/// "[" - Matches any character in the brackets
/// "]" - End of the bracket
///
fn bracket_char(chars_iter: &mut Chars<'_>) -> Result<Option<RegexStep>, &'static str> {
    let mut negated = false;
    let mut vec = Vec::new();
    let mut is_regex_class = false;

    if let Some(c) = chars_iter.next() {
        if c == '^' {
            negated = true;
        } else if c == '[' {
            is_regex_class = true;
        } else {
            vec.push(c);
        }
    } else {
        return Err(RegexError::InvalidBracket.message());
    }

    let mut end_bracket = false;
    let mut regex_class = None;
    if is_regex_class && chars_iter.next() == Some(':') {
        let mut class_vec = Vec::new();
        let mut end_class = false;
        while let Some(c) = chars_iter.next() {
            if c == ':' && chars_iter.next() == Some(']') {
                end_class = true;
                break;
            }
            class_vec.push(c);
        }

        if !end_class {
            return Err(RegexError::InvalidClass.message());
        }

        let class: String = class_vec.iter().collect();
        let character_class = determinate_regex_class(class);
        match character_class {
            Ok(class) => {
                regex_class = Some(class);
            }
            Err(_) => return Err(RegexError::InvalidClass.message()),
        }
    }

    while let Some(c) = chars_iter.next() {
        match c {
            ']' => {
                end_bracket = true;
                break;
            }
            '\\' => {
                if let Some(literal) = chars_iter.next() {
                    vec.push(literal);
                } else {
                    return Err(RegexError::InvalidBackslash.message());
                }
            }
            _ => vec.push(c),
        }
    }

    if !end_bracket {
        return Err(RegexError::InvalidBracket.message());
    }

    let val;
    if let Some(class) = regex_class {
        val = RegexVal::Class(class);
    } else if negated {
        val = RegexVal::NotBracket(vec);
    } else {
        val = RegexVal::Bracket(vec);
    }

    Ok(Some(RegexStep {
        rep: RegexRep::Exact(1),
        val,
        anchoring_start: false,
        anchoring_end: false,
    }))
}

/// Escape character for a regex
/// "\\" - Escapes the following character
///
fn escape_char(chars_iter: &mut Chars<'_>) -> Result<Option<RegexStep>, &'static str> {
    match chars_iter.next() {
        Some(literal) => Ok(Some(RegexStep {
            rep: RegexRep::Exact(1),
            val: RegexVal::Literal(literal),
            anchoring_start: false,
            anchoring_end: false,
        })),
        None => return Err(RegexError::InvalidBackslash.message()),
    }
}

/// Regular character for a regex
///
fn regular_char(c: char) -> Option<RegexStep> {
    Some(RegexStep {
        rep: RegexRep::Exact(1),
        val: RegexVal::Literal(c),
        anchoring_start: false,
        anchoring_end: false,
    })
}

impl TryFrom<&str> for Regex {
    type Error = &'static str;

    /// Given a string, returns a new Regex if the string is a valid regex.
    /// Characters are iterated and converted into RegexSteps.
    ///
    /// List of supported characters:
    ///
    /// * '.' - Matches any character
    /// * '*' - Matches zero or more of the preceding element
    /// * '?' - Matches zero or one of the preceding element
    /// * '+' - Matches one or more of the preceding element
    /// * '{' - Matches the preceding element a specified number of times
    /// * '}' - End of the specified number of times
    /// * '[' - Matches any character in the brackets
    /// * ']' - End of the bracket
    /// * '^' - Anchors the regex at the start of the line
    /// * '$' - Anchors the regex at the end of the line
    /// * '\\' - Escapes the following character
    ///
    /// # Arguments
    ///
    /// * `expression` - A string to be checked
    ///
    /// # Returns
    ///
    /// * Regex - The corresponding Regex if the string is a valid regex
    /// * Error - The corresponding error if the string is not a valid regex
    ///
    fn try_from(expression: &str) -> Result<Self, Self::Error> {
        let mut steps: Vec<RegexStep> = vec![];
        let mut anchoring_start = false;

        let mut chars_iter = expression.chars();
        while let Some(c) = chars_iter.next() {
            let step = match c {
                '.' => point_char(),
                '*' => wildcard_char(&mut steps),
                '?' => option_char(&mut steps),
                '+' => option_one_or_more_char(&mut steps),
                '{' => repetition_char(&mut steps, &mut chars_iter)?,
                '^' => anchor_start_char(&mut anchoring_start),
                '$' => anchor_end_char(&mut steps),
                '[' => bracket_char(&mut chars_iter)?,
                '\\' => escape_char(&mut chars_iter)?,
                _ => regular_char(c),
            };

            if let Some(s) = step {
                steps.push(s);
            }
        }

        if anchoring_start {
            let start_regex = RegexStep {
                rep: RegexRep::Any,
                val: RegexVal::Wildcard,
                anchoring_start: true,
                anchoring_end: false,
            };
            steps.push(start_regex);
        }

        Ok(Regex { steps })
    }
}

/// Given a queue of RegexSteps, a string and a state, returns a LineEvaluated if the string matches the regex
/// The function iterates over the string and the queue of RegexSteps to evaluate the match
/// The function returns a LineEvaluated with the result of the evaluation
/// The function is recursive and uses a stack to backtrack when needed
/// The function is used by the evaluate method of the Regex struct
///
fn evaluate_step(
    queue: &mut VecDeque<RegexStep>,
    value: &str,
    mut state: bool,
    queue_size: usize,
) -> Result<LineEvaluated, &'static str> {
    let regex_len = queue.len();
    for char_index in 0..value.len() {
        let mut stack: Vec<EvaluatedStep> = Vec::new();
        let mut index = char_index;

        'steps: while let Some(step) = queue.pop_front() {
            if step.anchoring_start {
                if index == regex_len - 1 {
                    return Ok(LineEvaluated {
                        result: true,
                        line: value.to_string(),
                    });
                } else {
                    break 'steps;
                }
            }

            if step.anchoring_end {
                if index == value.len() {
                    return Ok(LineEvaluated {
                        result: true,
                        line: value.to_string(),
                    });
                } else {
                    break 'steps;
                }
            }

            match step.rep {
                RegexRep::Exact(n) => {
                    let mut match_size = 0;
                    for i in 0..n {
                        let size = step.val.matches(&value[index..]);

                        if size == 0 {
                            match backtrack(step, &mut stack, queue) {
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
                                    match backtrack(step, &mut stack, queue) {
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

                    let mut backtrack_status = false;
                    if let Some(0) = min {
                        backtrack_status = true;
                    } else if let Some(max) = max {
                        if count < max {
                            backtrack_status = true;
                        }
                    }

                    stack.push(EvaluatedStep {
                        step,
                        match_size,
                        backtrackable: backtrack_status,
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

impl Regex {
    /// Given a string, returns a new Regex if the string is a valid regex
    ///
    /// # Arguments
    ///
    /// * `expression` - A string to be checked
    ///
    /// # Returns
    ///
    /// * Regex - The corresponding Regex if the string is a valid regex
    /// * &str - The corresponding error if the string is not a valid regex
    ///
    /// # Examples
    ///
    /// ```
    /// use rgrep::regex::Regex;
    ///
    /// let regex = Regex::new("abc.*").unwrap();
    /// ```
    ///
    pub fn new(expression: &str) -> Result<Self, &str> {
        Regex::try_from(expression)
    }

    /// Given a string, returns a LineEvaluated if the string matches the regex
    ///
    /// # Arguments
    ///
    /// * `value` - A string to be checked
    ///
    /// # Returns
    ///
    /// * LineEvaluated - The result of the evaluation
    /// * &str - The corresponding error if the string contains non-ascii characters
    ///
    /// # Examples
    ///
    /// ```
    /// use rgrep::regex::Regex;
    ///
    /// let regex = Regex::new("abc.*").unwrap();
    /// let line = regex.evaluate("abcdefg").unwrap();
    ///
    /// assert_eq!(line.result, true);
    /// ```
    ///
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

        evaluate_step(&mut queue, value, state, queue_size)
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
    fn test_match_question_mark_and_point() -> Result<(), &'static str> {
        let value = "abd";

        let regex = Regex::new("ab.?d").unwrap();
        println!("{:?}", regex);

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

    #[test]
    fn test_backslash_basic() -> Result<(), &'static str> {
        let value1 = "bca.bc";
        let regex1 = Regex::new("a\\.").unwrap();
        let line1 = regex1.evaluate(value1)?;
        assert!(line1.result);

        let value2 = "bcabc";
        let regex2 = Regex::new("a\\.").unwrap();
        let line2 = regex2.evaluate(value2)?;
        assert!(!line2.result);

        let value3 = "{abc";
        let regex3 = Regex::new("\\{abc").unwrap();
        let line3 = regex3.evaluate(value3)?;
        assert!(line3.result);

        let value4 = "abc";
        let regex4 = Regex::new("\\abc").unwrap();
        let line4 = regex4.evaluate(value4)?;
        assert!(line4.result);

        let value5 = ".e+e?e";
        let regex5 = Regex::new("\\.e\\+e\\?e").unwrap();
        let line5 = regex5.evaluate(value5)?;
        assert!(line5.result);

        let regex6 = Regex::new("abc\\").unwrap_err();
        assert_eq!(regex6, "Invalid regex: invalid backslash");

        Ok(())
    }

    #[test]
    fn test_backslash_backslash() -> Result<(), &'static str> {
        let value1 = "bca\\bc";
        let regex1 = Regex::new("a\\\\b").unwrap();
        let line1 = regex1.evaluate(value1)?;
        assert!(line1.result);

        let value2 = "bcabc";
        let regex2 = Regex::new("a\\\\b").unwrap();
        let line2 = regex2.evaluate(value2)?;
        assert!(!line2.result);

        Ok(())
    }

    #[test]
    fn test_anchoring_start() -> Result<(), &'static str> {
        let value1 = "start middle end";
        let value2 = "start with start";
        let value3 = "end with end";
        let value4 = "only this line";

        let regex = Regex::new("^start").unwrap();

        let line1 = regex.clone().evaluate(value1)?;
        let line2 = regex.clone().evaluate(value2)?;
        let line3 = regex.clone().evaluate(value3)?;
        let line4 = regex.evaluate(value4)?;

        assert!(line1.result);
        assert!(line2.result);
        assert!(!line3.result);
        assert!(!line4.result);

        Ok(())
    }

    #[test]
    fn test_anchoring_end() -> Result<(), &'static str> {
        let value1 = "start middle end";
        let value2 = "start with start";
        let value3 = "end with end";
        let value4 = "only this line";

        let regex = Regex::new("end$").unwrap();

        let line1 = regex.clone().evaluate(value1)?;
        let line2 = regex.clone().evaluate(value2)?;
        let line3 = regex.clone().evaluate(value3)?;
        let line4 = regex.evaluate(value4)?;

        assert!(line1.result);
        assert!(!line2.result);
        assert!(line3.result);
        assert!(!line4.result);

        Ok(())
    }

    #[test]
    fn test_anchoring_fails() -> Result<(), &'static str> {
        let value1 = "start middle end";
        let regex1 = Regex::new("^middle").unwrap();
        let line1 = regex1.evaluate(value1)?;
        assert!(!line1.result);

        let value2 = "start middle end";
        let regex2 = Regex::new("middle$").unwrap();
        let line2 = regex2.evaluate(value2)?;
        assert!(!line2.result);

        Ok(())
    }

    #[test]
    fn test_bracket_expressions() -> Result<(), &'static str> {
        let value1 = "abc";
        let value2 = "acc";
        let value3 = "azc";
        let value4 = "a9c";
        let value5 = "a3";
        let value6 = "ae";
        let value7 = "aec";
        let value8 = "aaaaaaeccccccc";
        let value9 = "aabcdefc";

        let regex = Regex::new("a[abcdef]c").unwrap();

        let line1 = regex.clone().evaluate(value1)?;
        let line2 = regex.clone().evaluate(value2)?;
        let line3 = regex.clone().evaluate(value3)?;
        let line4 = regex.clone().evaluate(value4)?;
        let line5 = regex.clone().evaluate(value5)?;
        let line6 = regex.clone().evaluate(value6)?;
        let line7 = regex.clone().evaluate(value7)?;
        let line8 = regex.clone().evaluate(value8)?;
        let line9 = regex.evaluate(value9)?;

        assert!(line1.result);
        assert!(line2.result);
        assert!(!line3.result);
        assert!(!line4.result);
        assert!(!line5.result);
        assert!(!line6.result);
        assert!(line7.result);
        assert!(line8.result);
        assert!(line9.result);

        Ok(())
    }

    #[test]
    fn test_negated_bracket_expressions() -> Result<(), &'static str> {
        let value1 = "abc";
        let value2 = "acc";
        let value3 = "azc";
        let value4 = "a9c";
        let value5 = "a3";
        let value6 = "ae";
        let value7 = "aec";
        let value8 = "aaaaaaeccccccc";
        let value9 = "ahcalcazcakc";

        let regex = Regex::new("a[^ghijkl]c").unwrap();

        let line1 = regex.clone().evaluate(value1)?;
        let line2 = regex.clone().evaluate(value2)?;
        let line3 = regex.clone().evaluate(value3)?;
        let line4 = regex.clone().evaluate(value4)?;
        let line5 = regex.clone().evaluate(value5)?;
        let line6 = regex.clone().evaluate(value6)?;
        let line7 = regex.clone().evaluate(value7)?;
        let line8 = regex.clone().evaluate(value8)?;
        let line9 = regex.evaluate(value9)?;

        assert!(line1.result);
        assert!(line2.result);
        assert!(line3.result);
        assert!(line4.result);
        assert!(!line5.result);
        assert!(!line6.result);
        assert!(line7.result);
        assert!(line8.result);
        assert!(line9.result);

        Ok(())
    }

    const VALUE1: &str = "abc";
    const VALUE2: &str = "a1c";
    const VALUE3: &str = "a%c";
    const VALUE4: &str = "aBc";
    const VALUE5: &str = "a c";
    const VALUE6: &str = "a-c";

    #[test]
    fn test_regex_alnum_class() -> Result<(), &'static str> {
        // Alphanumeric
        let alnum_regex = Regex::new("a[[:alnum:]]c").unwrap();

        let alnum_line1 = alnum_regex.clone().evaluate(VALUE1)?;
        let alnum_line2 = alnum_regex.clone().evaluate(VALUE2)?;
        let alnum_line3 = alnum_regex.clone().evaluate(VALUE3)?;
        let alnum_line4 = alnum_regex.clone().evaluate(VALUE4)?;
        let alnum_line5 = alnum_regex.clone().evaluate(VALUE5)?;
        let alnum_line6 = alnum_regex.evaluate(VALUE6)?;

        assert!(alnum_line1.result);
        assert!(alnum_line2.result);
        assert!(!alnum_line3.result);
        assert!(alnum_line4.result);
        assert!(!alnum_line5.result);
        assert!(!alnum_line6.result);

        Ok(())
    }

    #[test]
    fn test_regex_alpha_class() -> Result<(), &'static str> {
        // Alphabetic
        let alpha_regex = Regex::new("a[[:alpha:]]c").unwrap();

        let alpha_line1 = alpha_regex.clone().evaluate(VALUE1)?;
        let alpha_line2 = alpha_regex.clone().evaluate(VALUE2)?;
        let alpha_line3 = alpha_regex.clone().evaluate(VALUE3)?;
        let alpha_line4 = alpha_regex.clone().evaluate(VALUE4)?;
        let alpha_line5 = alpha_regex.clone().evaluate(VALUE5)?;
        let alpha_line6 = alpha_regex.evaluate(VALUE6)?;

        assert!(alpha_line1.result);
        assert!(!alpha_line2.result);
        assert!(!alpha_line3.result);
        assert!(alpha_line4.result);
        assert!(!alpha_line5.result);
        assert!(!alpha_line6.result);

        Ok(())
    }

    #[test]
    fn test_regex_digit_class() -> Result<(), &'static str> {
        // Digit - Numeric
        let digit_regex = Regex::new("a[[:digit:]]c").unwrap();

        let digit_line1 = digit_regex.clone().evaluate(VALUE1)?;
        let digit_line2 = digit_regex.clone().evaluate(VALUE2)?;
        let digit_line3 = digit_regex.clone().evaluate(VALUE3)?;
        let digit_line4 = digit_regex.clone().evaluate(VALUE4)?;
        let digit_line5 = digit_regex.clone().evaluate(VALUE5)?;
        let digit_line6 = digit_regex.evaluate(VALUE6)?;

        assert!(!digit_line1.result);
        assert!(digit_line2.result);
        assert!(!digit_line3.result);
        assert!(!digit_line4.result);
        assert!(!digit_line5.result);
        assert!(!digit_line6.result);

        Ok(())
    }

    #[test]
    fn test_regex_lower_class() -> Result<(), &'static str> {
        // Lowercase letters
        let lower_regex = Regex::new("a[[:lower:]]c").unwrap();

        let lower_line1 = lower_regex.clone().evaluate(VALUE1)?;
        let lower_line2 = lower_regex.clone().evaluate(VALUE2)?;
        let lower_line3 = lower_regex.clone().evaluate(VALUE3)?;
        let lower_line4 = lower_regex.clone().evaluate(VALUE4)?;
        let lower_line5 = lower_regex.clone().evaluate(VALUE5)?;
        let lower_line6 = lower_regex.evaluate(VALUE6)?;

        assert!(lower_line1.result);
        assert!(!lower_line2.result);
        assert!(!lower_line3.result);
        assert!(!lower_line4.result);
        assert!(!lower_line5.result);
        assert!(!lower_line6.result);

        Ok(())
    }

    #[test]
    fn test_regex_upper_class() -> Result<(), &'static str> {
        // Uppercase letters
        let upper_regex = Regex::new("a[[:upper:]]c").unwrap();

        let upper_line1 = upper_regex.clone().evaluate(VALUE1)?;
        let upper_line2 = upper_regex.clone().evaluate(VALUE2)?;
        let upper_line3 = upper_regex.clone().evaluate(VALUE3)?;
        let upper_line4 = upper_regex.clone().evaluate(VALUE4)?;
        let upper_line5 = upper_regex.clone().evaluate(VALUE5)?;
        let upper_line6 = upper_regex.evaluate(VALUE6)?;

        assert!(!upper_line1.result);
        assert!(!upper_line2.result);
        assert!(!upper_line3.result);
        assert!(upper_line4.result);
        assert!(!upper_line5.result);
        assert!(!upper_line6.result);

        Ok(())
    }

    #[test]
    fn test_regex_space_class() -> Result<(), &'static str> {
        // Space character
        let space_regex = Regex::new("a[[:space:]]c").unwrap();

        let space_line1 = space_regex.clone().evaluate(VALUE1)?;
        let space_line2 = space_regex.clone().evaluate(VALUE2)?;
        let space_line3 = space_regex.clone().evaluate(VALUE3)?;
        let space_line4 = space_regex.clone().evaluate(VALUE4)?;
        let space_line5 = space_regex.clone().evaluate(VALUE5)?;
        let space_line6 = space_regex.evaluate(VALUE6)?;

        assert!(!space_line1.result);
        assert!(!space_line2.result);
        assert!(!space_line3.result);
        assert!(!space_line4.result);
        assert!(space_line5.result);
        assert!(!space_line6.result);

        Ok(())
    }

    #[test]
    fn test_regex_punct_class() -> Result<(), &'static str> {
        // Punctuation character
        let punct_regex = Regex::new("a[[:punct:]]c").unwrap();

        let punct_line1 = punct_regex.clone().evaluate(VALUE1)?;
        let punct_line2 = punct_regex.clone().evaluate(VALUE2)?;
        let punct_line3 = punct_regex.clone().evaluate(VALUE3)?;
        let punct_line4 = punct_regex.clone().evaluate(VALUE4)?;
        let punct_line5 = punct_regex.clone().evaluate(VALUE5)?;
        let punct_line6 = punct_regex.evaluate(VALUE6)?;

        assert!(!punct_line1.result);
        assert!(!punct_line2.result);
        assert!(punct_line3.result);
        assert!(!punct_line4.result);
        assert!(!punct_line5.result);
        assert!(punct_line6.result);

        Ok(())
    }
}
