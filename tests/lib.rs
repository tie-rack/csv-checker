use std::fs::File;
use std::error::Error;

extern crate csv_checker;

#[test]
fn finds_errors_in_csv() {
    let file = match File::open("tests/test.csv") {
        Err(why) => panic!("couldn't open: {}", Error::description(&why)),
        Ok(file) => file,
    };

    let errors = csv_checker::errors_for_csv(file);

    assert_eq!(errors.len(), 4);

    assert_eq!(errors[0],
               csv_checker::CSVError {
                   line: 2,
                   col: 13,
                   text: csv_checker::UNEXPECTED_CHAR,
               });
    assert_eq!(errors[1],
               csv_checker::CSVError {
                   line: 3,
                   col: 28,
                   text: csv_checker::UNEXPECTED_EOL,
               });
    assert_eq!(errors[2],
               csv_checker::CSVError {
                   line: 5,
                   col: 16,
                   text: csv_checker::UNEXPECTED_EOF,
               });
    assert_eq!(errors[3],
               csv_checker::CSVError {
                   line: 8,
                   col: 14,
                   text: csv_checker::UNEXPECTED_CHAR,
               });
}
