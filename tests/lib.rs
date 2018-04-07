/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::error::Error;

extern crate csv_checker;

#[test]
fn finds_errors_in_csv() {
    let mut file = match File::open("tests/test.csv") {
        Err(why) => panic!("couldn't open: {}", Error::description(&why)),
        Ok(file) => file,
    };

    let mut eport = csv_checker::csv_report(&mut file);

    assert_eq!(report.next(),
               Some(csv_checker::CSVError {
                   line: 2,
                   col: 13,
                   text: csv_checker::UNEXPECTED_CHAR,
               }));
    assert_eq!(report.next(),
               Some(csv_checker::CSVError {
                   line: 3,
                   col: 28,
                   text: csv_checker::UNEXPECTED_EOL,
               }));
    assert_eq!(report.next(),
               Some(csv_checker::CSVError {
                   line: 5,
                   col: 31,
                   text: csv_checker::UNEXPECTED_EOL,
               }));
    assert_eq!(report.next(),
               Some(csv_checker::CSVError {
                   line: 8,
                   col: 14,
                   text: csv_checker::UNEXPECTED_CHAR,
               }));
    assert_eq!(report.next(),
               Some(csv_checker::CSVError {
                   line: 11,
                   col: 39,
                   text: csv_checker::UNEXPECTED_EOL,
               }));
    assert_eq!(report.next(),
               Some(csv_checker::CSVError {
                   line: 12,
                   col: 28,
                   text: csv_checker::UNEXPECTED_EOL,
               }));
    assert_eq!(report.next(), None);

}
