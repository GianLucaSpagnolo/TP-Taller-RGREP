pub mod regex;

use std::error::Error;
use std::fs;

pub struct Arguments {
    pub regex: String,
    pub path: String,
}

impl Arguments {
    pub fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() != 3 {
            return Err("Incorrect amount of arguments");
        }

        let regex = args[1].clone();
        let path = args[2].clone();

        Ok(Arguments { regex, path })
    }
}

pub fn run(arguments: Arguments) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(arguments.path)?;

    println!("\nWith text:\n\n{contents}");

    Ok(())
}
