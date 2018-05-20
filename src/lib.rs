/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::io::Read;

const LF: u8 = b'\n';
const CR: u8 = b'\r';
const QUOTE: u8 = b'"';
const COMMA: u8 = b',';

// Error messages
const UNEXPECTED_EOL: &str = "Unexpected end of line";
const UNEXPECTED_CHAR: &str = "Unexpected character after quote";
const EXPECTED_LF: &str = "Expected linefeed after carriage return";

enum CSVState {
    Start,
    NonQuotedValue,
    QuotedValue,
    QuoteQuote,
    ExpectLF,
    Error,
}

type CSVResult = Result<CSVState, &'static str>;

impl CSVState {
    fn parse_byte(&self, byte: u8) -> CSVResult {
        match (self, byte) {
            (CSVState::Start, QUOTE) => Ok(CSVState::QuotedValue),
            (CSVState::Start, COMMA) => Ok(CSVState::Start),
            (CSVState::Start, _) => Ok(CSVState::NonQuotedValue),

            (CSVState::NonQuotedValue, COMMA) => Ok(CSVState::Start),
            (CSVState::NonQuotedValue, LF) => Ok(CSVState::Start),
            (CSVState::NonQuotedValue, CR) => Ok(CSVState::ExpectLF),
            (CSVState::NonQuotedValue, _) => Ok(CSVState::NonQuotedValue),

            (CSVState::QuotedValue, QUOTE) => Ok(CSVState::QuoteQuote),
            (CSVState::QuotedValue, CR) => Err(UNEXPECTED_EOL),
            (CSVState::QuotedValue, LF) => Err(UNEXPECTED_EOL),
            (CSVState::QuotedValue, _) => Ok(CSVState::QuotedValue),

            (CSVState::QuoteQuote, QUOTE) => Ok(CSVState::QuotedValue),
            (CSVState::QuoteQuote, COMMA) => Ok(CSVState::Start),
            (CSVState::QuoteQuote, LF) => Ok(CSVState::Start),
            (CSVState::QuoteQuote, CR) => Ok(CSVState::ExpectLF),
            (CSVState::QuoteQuote, _) => Err(UNEXPECTED_CHAR),

            (CSVState::ExpectLF, LF) => Ok(CSVState::Start),
            (CSVState::ExpectLF, _) => Err(EXPECTED_LF),

            (CSVState::Error, LF) => Ok(CSVState::Start),
            (CSVState::Error, _) => Ok(CSVState::Error),
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct CSVError {
    pub line: u32,
    pub col: u32,
    pub text: &'static str,
}

impl fmt::Display for CSVError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{} {}", self.line, self.col, self.text)
    }
}

pub fn csv_report<'a>(reader: &'a mut Read) -> impl Iterator<Item = CSVError> + 'a {
    let mut line = 1;
    let mut col = 0;
    let mut state = CSVState::Start;

    reader.bytes().filter_map(move |b| {
        let byte = b.unwrap();

        match state.parse_byte(byte) {
            Ok(new_state) => {
                state = new_state;
                if byte == LF {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                }
                None
            }
            Err(error) => {
                let err = CSVError {
                    line: line,
                    col: col,
                    text: error,
                };
                if byte == LF {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                };
                state = match error {
                    UNEXPECTED_EOL => CSVState::Start,
                    _ => CSVState::Error,
                };
                Some(err)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{csv_report, CSVError, UNEXPECTED_CHAR, UNEXPECTED_EOL};

    #[test]
    fn finds_errors_in_csv() {
        use std::error::Error;
        use std::fs::File;

        let mut file = match File::open("tests/test.csv") {
            Err(why) => panic!("couldn't open: {}", Error::description(&why)),
            Ok(file) => file,
        };

        let mut report = csv_report(&mut file);

        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 2,
                col: 13,
                text: UNEXPECTED_CHAR,
            })
        );
        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 3,
                col: 28,
                text: UNEXPECTED_EOL,
            })
        );
        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 5,
                col: 31,
                text: UNEXPECTED_EOL,
            })
        );
        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 8,
                col: 14,
                text: UNEXPECTED_CHAR,
            })
        );
        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 11,
                col: 39,
                text: UNEXPECTED_EOL,
            })
        );
        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 12,
                col: 28,
                text: UNEXPECTED_EOL,
            })
        );
        assert_eq!(report.next(), None);
    }
}
