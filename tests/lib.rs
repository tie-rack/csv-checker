use std::fs::File;
use std::error::Error;
use std::sync::mpsc::channel;

extern crate csv_checker;

#[test]
fn finds_errors_in_csv() {
    let file = match File::open("tests/test.csv") {
        Err(why) => panic!("couldn't open: {}", Error::description(&why)),
        Ok(file) => file,
    };

    let (tx, rx) = channel();
    csv_checker::publish_errors_for_csv(file, tx);

    let mut error_iterator = rx.iter();

    assert_eq!(error_iterator.next(),
               Some(csv_checker::CSVError {
                   line: 2,
                   col: 13,
                   text: csv_checker::UNEXPECTED_CHAR,
               }));
    assert_eq!(error_iterator.next(),
               Some(csv_checker::CSVError {
                   line: 3,
                   col: 28,
                   text: csv_checker::UNEXPECTED_EOL,
               }));
    assert_eq!(error_iterator.next(),
               Some(csv_checker::CSVError {
                   line: 5,
                   col: 31,
                   text: csv_checker::UNEXPECTED_EOL,
               }));
    assert_eq!(error_iterator.next(),
               Some(csv_checker::CSVError {
                   line: 8,
                   col: 14,
                   text: csv_checker::UNEXPECTED_CHAR,
               }));
    assert_eq!(error_iterator.next(),
               Some(csv_checker::CSVError {
                   line: 11,
                   col: 39,
                   text: csv_checker::UNEXPECTED_EOL,
               }));
    assert_eq!(error_iterator.next(),
               Some(csv_checker::CSVError {
                   line: 12,
                   col: 28,
                   text: csv_checker::UNEXPECTED_EOL,
               }));
    assert_eq!(error_iterator.next(), None);

}
