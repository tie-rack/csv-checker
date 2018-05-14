/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

extern crate csv_checker;

fn report_errors_for_file(file: File) -> i32 {
    let mut reader = BufReader::new(file);

    let mut report = csv_checker::csv_report(&mut reader).peekable();

    let exit = match report.peek() {
        None => 0,
        _ => 1,
    };

    for error in report {
        println!("{}", error);
    }

    exit
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
