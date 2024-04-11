pub mod regex;
pub mod program_error;

use regex::Regex;
use program_error::ProgramError;

use std::fs;
use std::io::Write;
use std::error::Error;

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

pub fn run_rgrep(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let text = fs::read_to_string(arguments.path)?;
    let iter = text.lines();
    let mut correct_lines: Vec<String> = Vec::new();

    let regex = Regex::new(&arguments.regex)?;

    for line in iter {
        let evaluation = regex.clone().evaluate(line)?;
        if evaluation.result {
            correct_lines.push(evaluation.line);
        }
    }

    //println!("\x1b[1;33mHOLA A TODOS!\x1b[0m BIENVENIDOS AL RGREP!");
    for line in correct_lines {
        println!("{}", line);
    }

    Ok(())
}

pub fn print_error(err: String) {
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
        assert_eq!(return2.message(), ProgramError::InvalidAmountOfArguments.message());

        let binding3 = { vec!["rgrep"] };
        let args3 = binding3.iter().map(|s| s.to_string());
        let return3 = Arguments::new(args3).unwrap_err();
        assert_eq!(return3.message(), ProgramError::ArgumentMissing.message());
    }

    #[test]
    fn try_invalid_file() {
        let binding = { vec!["rgrep", "regex", "res/test-1.txt"] };
        let args = binding.iter().map(|s| s.to_string());
        let arguments = Arguments::new(args).unwrap();
        let result = run_rgrep(arguments).unwrap_err();
        assert_eq!(result.to_string(), "No such file or directory (os error 2)");
    }

    #[test]
    fn try_valid_file_relative_path() {
        let binding = { vec!["rgrep", "regex", "res/test0.txt"] };
        let args = binding.iter().map(|s| s.to_string());
        let arguments = Arguments::new(args).unwrap();
        let result = run_rgrep(arguments).is_ok();
        assert_eq!(result, true);
    }
}
