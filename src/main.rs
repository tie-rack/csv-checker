use std::env;
use std::fs::File;
use std::path::Path;

extern crate csv_checker;

fn report_errors_for_file(file: File) -> i32 {
    let errors = csv_checker::errors_for_csv(file);

    for error in errors.iter() {
        println!("error at line {}, col {}", error.line, error.col);
    }

    match errors.len() {
        0 => 0,
        _ => 1,
    }
}

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

        exit_code += match File::open(&path) {
            Err(_) => {
                println!("!!! Not found");
                1
            }
            Ok(file) => report_errors_for_file(file),
        };
    }

    exit_code
}

fn main() {
    let exit_code = errors_for_args();
    std::process::exit(exit_code);
}
