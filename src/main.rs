// WELCOME TO RGREP: RUSTIC GREP

use std::env;

// use rgrep::regex::Regex;
use rgrep::run;
use rgrep::Arguments;

fn main() {
    let args: Vec<String> = env::args_os()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    match Arguments::new(&args) {
        Ok(arguments) => {
            println!("Searching for {}", arguments.regex);
            println!("In file {}", arguments.path);

            if let Err(err) = run(arguments) {
                eprintln!("rgrep: {err}");
            }
        }
        Err(err) => {
            eprint!("rgrep: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use rgrep::regex::Regex;

    #[test]
    fn test_no_ascii() {
        let value = "abacdef";

        let regex = Regex::new("ab.*c").unwrap();

        let matches = regex.evaluate(value);
        //assert!(matches.is_err());
        //assert_eq!(matches, true);

        assert!(matches.is_ok());
        assert_eq!(matches.unwrap(), true);
    }

    #[test]
    fn test_match() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("ab.*e").unwrap();

        let matches = regex.evaluate(value)?;
        assert_eq!(matches, true);

        Ok(())
    }

    #[test]
    fn test_no_match() -> Result<(), &'static str> {
        let value = "abcdef";

        let regex = Regex::new("ab.*h").unwrap();

        let matches = regex.evaluate(value)?;
        assert_eq!(matches, false);

        Ok(())
    }

    #[test]
    fn test_match_2() -> Result<(), &'static str> {
        let value = "ab1234cdefg";

        let regex = Regex::new("ab.*c.*f").unwrap();

        let matches = regex.evaluate(value)?;
        assert_eq!(matches, true);

        Ok(())
    }

    #[test]
    fn test_no_match_2() -> Result<(), &'static str> {
        let value = "ab1234cdegh";

        let regex = Regex::new("ab.*c.*f").unwrap();

        let matches = regex.evaluate(value)?;
        assert_eq!(matches, false);

        Ok(())
    }
}
