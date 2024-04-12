// WELCOME TO RGREP: RUSTIC GREP
// Gian Luca Spagnolo - 108072
use std::env;

use rgrep::print_error;
use rgrep::read_file;
use rgrep::run_rgrep;
use rgrep::Arguments;

fn main() {
    let args = env::args_os().map(|arg| arg.to_string_lossy().into_owned());

    match Arguments::new(args) {
        Ok(arguments) => {
            let file_text = read_file(arguments.path);
            if let Err(err) = file_text {
                print_error(err.message());
            } else if let Ok(text) = file_text {
                if let Err(err) = run_rgrep(arguments.regex, text) {
                    print_error(&err);
                }
            }
        }
        Err(err) => {
            print_error(err.message());
        }
    }
}
