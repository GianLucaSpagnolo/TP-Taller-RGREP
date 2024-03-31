pub mod regex;

use std::io::Write;
use std::error::Error;
use std::fs;

#[derive(Debug)]
pub struct Arguments {
    pub regex: String,
    pub path: String,
}

impl Arguments {
    pub fn new(mut args: impl Iterator<Item = String>,
    ) -> Result<Arguments, &'static str> {
        args.next();

        let regex = match args.next() {
            Some(arg) => arg,
            None => return Err("Invalid arguments: regex and path missing"),
        };

        let path = match args.next() {
            Some(arg) => arg,
            None => return Err("Invalid arguments: path missing"),
        };

        if args.next().is_some() {
            return Err("Invalid amount of arguments");
        }

        Ok(Arguments { regex, path })
    }
}

pub fn run(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(arguments.path)?;

    println!("With text:\n\n{contents}");

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
        let binding = {
            vec![
                "rgrep",
                "regex",
                "path",
            ]
        };
        let args = binding.iter().map(|s| s.to_string());

        let arguments = Arguments::new(args).unwrap();
        assert_eq!(arguments.regex, "regex".to_string());
        assert_eq!(arguments.path, "path".to_string());
    }



    #[test]
    fn verify_incorrect_arguments() {
        let binding1 = {
            vec![
                "rgrep",
                "regex",
            ]
        };
        let args1 = binding1.iter().map(|s| s.to_string());
        let arguments1 = Arguments::new(args1).unwrap_err();
        assert_eq!(arguments1, "Invalid arguments: path missing");

        let binding2 = {
            vec![
                "rgrep",
                "regex",
                "path",
                "extra",
            ]
        };
        let args2 = binding2.iter().map(|s| s.to_string());
        let arguments2 = Arguments::new(args2).unwrap_err();
        assert_eq!(arguments2, "Invalid amount of arguments");

        let binding3 = {
            vec![
                "rgrep",
            ]
        };
        let args3 = binding3.iter().map(|s| s.to_string());
        let arguments3 = Arguments::new(args3).unwrap_err();
        assert_eq!(arguments3, "Invalid arguments: regex and path missing");
    }
}
