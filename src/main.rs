// WELCOME TO RGREP: RUSTIC GREP
// Gian Luca Spagnolo - 108072
use std::env;

use rgrep::print_error;
use rgrep::run_rgrep;
use rgrep::Arguments;

fn main() {
    let args = env::args_os().map(|arg| arg.to_string_lossy().into_owned());

    match Arguments::new(args) {
        Ok(arguments) => {
            if let Err(err) = run_rgrep(arguments) {
                print_error(err.to_string());
            }
        }
        Err(err) => {
            print_error(err.message().to_string());
        }
    }
}
