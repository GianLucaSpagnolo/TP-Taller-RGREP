// WELCOME TO RGREP: RUSTIC GREP
// Made by: Gian Luca Spagnolo
use std::env;

use rgrep::Arguments;
use rgrep::*;

fn main() {
    let args = env::args_os().map(|arg| arg.to_string_lossy().into_owned());

    match Arguments::new(args) {
        Ok(arguments) => {
            let file_text = read_file(arguments.path);

            if let Err(err) = file_text {
                print_error(err.message());
            } else if let Ok(text) = file_text {
                let program_output = run_rgrep(arguments.regex, text);

                if let Ok(output) = program_output {
                    print_lines(output);
                } else if let Err(error) = program_output {
                    print_error(&error);
                }
            }
        }
        Err(err) => {
            print_error(err.message());
        }
    }
}
