pub mod program_error;
pub mod regex;

use program_error::ProgramError;
use regex::Regex;

use std::error::Error;
use std::fs;
use std::io::Write;

#[derive(Debug)]
pub struct Arguments {
    pub regex: String,
    pub path: String,
}

impl Arguments {
    pub fn new(mut args: impl Iterator<Item = String>) -> Result<Arguments, ProgramError> {
        args.next();

        let regex = match args.next() {
            Some(arg) => arg,
            None => return Err(ProgramError::ArgumentMissing),
        };

        let path = match args.next() {
            Some(arg) => arg,
            None => return Err(ProgramError::PathMissing),
        };

        if args.next().is_some() {
            return Err(ProgramError::InvalidAmountOfArguments);
        }

        Ok(Arguments { regex, path })
    }
}

pub fn run_rgrep(regex: String, text: String) -> Result<Vec<String>, String> {
    let iter = text.lines();
    let mut correct_lines: Vec<String> = Vec::new();
    let regex = Regex::new(&regex)?;

    for line in iter {
        let evaluation = regex.clone().evaluate(line)?;
        if evaluation.result {
            correct_lines.push(evaluation.line);
        }
    }

    Ok(correct_lines)
}

pub fn print_lines(lines: Vec<String>) {
    for line in lines {
        println!("{}", line);
    }
}

pub fn read_file(path: String) -> Result<String, ProgramError> {
    let text = fs::read_to_string(path);
    match text {
        Ok(text) => Ok(text),
        Err(err) => Err(process_error(Box::new(err))),
    }
}

fn process_error(err: Box<dyn Error>) -> ProgramError {
    match err {
        err if err.to_string().contains("No such file or directory") => {
            ProgramError::InvalidFilePath
        }
        err if err
            .to_string()
            .contains("stream did not contain valid UTF-8") =>
        {
            ProgramError::InvalidFileFormat
        }
        _ => ProgramError::ErrorWhileReadingFile,
    }
}

pub fn print_error(err: &str) {
    writeln!(&mut std::io::stderr(), "rgrep: {}", err).unwrap_or_else(|_| ());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_correct_arguments() {
        let binding = { vec!["rgrep", "regex", "path"] };
        let args = binding.iter().map(|s| s.to_string());

        let arguments = Arguments::new(args).unwrap();
        assert_eq!(arguments.regex, "regex".to_string());
        assert_eq!(arguments.path, "path".to_string());
    }

    #[test]
    fn verify_incorrect_arguments() {
        let binding1 = { vec!["rgrep", "regex"] };
        let args1 = binding1.iter().map(|s| s.to_string());
        let return1 = Arguments::new(args1).unwrap_err();
        assert_eq!(return1.message(), ProgramError::PathMissing.message());

        let binding2 = { vec!["rgrep", "regex", "path", "extra"] };
        let args2 = binding2.iter().map(|s| s.to_string());
        let return2 = Arguments::new(args2).unwrap_err();
        assert_eq!(
            return2.message(),
            ProgramError::InvalidAmountOfArguments.message()
        );

        let binding3 = { vec!["rgrep"] };
        let args3 = binding3.iter().map(|s| s.to_string());
        let return3 = Arguments::new(args3).unwrap_err();
        assert_eq!(return3.message(), ProgramError::ArgumentMissing.message());
    }

    #[test]
    fn try_invalid_file() {
        let binding1 = { vec!["rgrep", "regex", "res/test-1.txt"] };
        let args1 = binding1.iter().map(|s| s.to_string());
        let arguments1 = Arguments::new(args1).unwrap();
        let text_read1 = read_file(arguments1.path).unwrap_err();
        assert_eq!(
            text_read1.message(),
            ProgramError::InvalidFilePath.message()
        );

        let binding2 = { vec!["rgrep", "regex", "res/invalid_format.txt"] };
        let args2 = binding2.iter().map(|s| s.to_string());
        let arguments2 = Arguments::new(args2).unwrap();
        let text_read2 = read_file(arguments2.path).unwrap_err();
        assert_eq!(
            text_read2.message(),
            ProgramError::InvalidFileFormat.message()
        );
    }

    #[test]
    fn try_valid_file_relative_path() {
        let binding = { vec!["rgrep", "regex", "res/test0.txt"] };
        let args = binding.iter().map(|s| s.to_string());
        let arguments = Arguments::new(args).unwrap();
        let text_read = read_file(arguments.path).unwrap();
        let result = run_rgrep(arguments.regex, text_read).is_ok();
        assert!(result);
    }
}
