/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::io::Read;

const LF: u8 = b'\n';
const CR: u8 = b'\r';
const QUOTE: u8 = b'"';
const COMMA: u8 = b',';

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    UnexpectedEOL,
    UnexpectedChar,
    ExpectedLF,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            ErrorType::UnexpectedEOL => "Unexpected EOL",
            ErrorType::UnexpectedChar => "Unexpected char",
            ErrorType::ExpectedLF => "Expected LF",
        };
        write!(f, "{}", text)
    }
}

enum CSVState {
    Start,
    NonQuotedValue,
    QuotedValue,
    QuoteQuote,
    ExpectLF,
    Error,
}

type CSVResult = Result<CSVState, ErrorType>;

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
            (CSVState::QuotedValue, CR) => Err(ErrorType::UnexpectedEOL),
            (CSVState::QuotedValue, LF) => Err(ErrorType::UnexpectedEOL),
            (CSVState::QuotedValue, _) => Ok(CSVState::QuotedValue),

            (CSVState::QuoteQuote, QUOTE) => Ok(CSVState::QuotedValue),
            (CSVState::QuoteQuote, COMMA) => Ok(CSVState::Start),
            (CSVState::QuoteQuote, LF) => Ok(CSVState::Start),
            (CSVState::QuoteQuote, CR) => Ok(CSVState::ExpectLF),
            (CSVState::QuoteQuote, _) => Err(ErrorType::UnexpectedChar),

            (CSVState::ExpectLF, LF) => Ok(CSVState::Start),
            (CSVState::ExpectLF, _) => Err(ErrorType::ExpectedLF),

            (CSVState::Error, LF) => Ok(CSVState::Start),
            (CSVState::Error, _) => Ok(CSVState::Error),
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct CSVError {
    pub line: u32,
    pub col: u32,
    pub error_type: ErrorType,
}

impl fmt::Display for CSVError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{} {}", self.line, self.col, self.error_type)
    }
}

pub fn csv_report(reader: impl Read) -> impl Iterator<Item = CSVError> {
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
            Err(error_type) => {
                let err = CSVError {line, col, error_type};
                if byte == LF {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                };
                state = match err.error_type {
                    ErrorType::UnexpectedEOL => CSVState::Start,
                    _ => CSVState::Error,
                };
                Some(err)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{csv_report, CSVError, ErrorType};

    #[test]
    fn finds_errors_in_csv() {
        use std::fs::File;

        let mut file = File::open("tests/test.csv").unwrap();

        let mut report = csv_report(&mut file);

        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 2,
                col: 13,
                error_type: ErrorType::UnexpectedChar,
            })
        );
        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 3,
                col: 28,
                error_type: ErrorType::UnexpectedEOL,
            })
        );
        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 5,
                col: 31,
                error_type: ErrorType::UnexpectedEOL,
            })
        );
        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 8,
                col: 14,
                error_type: ErrorType::UnexpectedChar,
            })
        );
        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 11,
                col: 39,
                error_type: ErrorType::UnexpectedEOL,
            })
        );
        assert_eq!(
            report.next(),
            Some(CSVError {
                line: 12,
                col: 28,
                error_type: ErrorType::UnexpectedEOL,
            })
        );
        assert_eq!(report.next(), None);
    }
}
