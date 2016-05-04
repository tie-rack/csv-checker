use std::env;
use std::error::Error;
use std::fs::File;
use std::path::Path;

extern crate csv_checker;

fn errors_for_args() -> i32 {
    let mut exit_code: i32 = 0;

    // Skip the first arg since it's the program itself
    let paths = env::args().skip(1);

    if paths.len() == 0 {
        panic!("Provide some paths!");
    }

    for arg in paths {
        println!("= {} =", arg);

        let path = Path::new(&arg);

        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open: {}", Error::description(&why)),
            Ok(file) => file,
        };

        let errors: Vec<csv_checker::CSVError> = csv_checker::errors_for_csv(file);

        if errors.len() > 0 {
            exit_code = 1;
        }

        for error in errors {
            println!("error at line {}, col {}", error.line, error.col);
        }
    }

    exit_code
}

fn main() {
    let exit_code = errors_for_args();
    std::process::exit(exit_code);
}
