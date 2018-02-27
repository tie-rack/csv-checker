/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io::Read;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;

const LF: u8 = b'\n';
const CR: u8 = b'\r';
const QUOTE: u8 = b'"';
const COMMA: u8 = b',';

// Error messages
pub const UNEXPECTED_EOL: &str = "Unexpected end of line";
pub const UNEXPECTED_CHAR: &str = "Unexpected character after quote";
pub const EXPECTED_LF: &str = "Expected linefeed after carriage return";

enum CSVState {
    Start,
    NonQuotedValue,
    QuotedValue,
    QuoteQuote,
    ExpectLF,
    Error,
}

type CSVResult = Result<CSVState, &'static str>;

trait ParseByte {
    fn parse_byte(self, byte: u8) -> CSVResult;
}

impl ParseByte for CSVState {
    fn parse_byte(self, byte: u8) -> CSVResult {
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

#[derive(Debug, PartialEq, Eq)]
pub struct CSVError {
    pub line: i32,
    pub col: i32,
    pub text: &'static str,
}

pub fn publish_errors_for_csv<T: Read + Send + 'static>(reader: T) -> Receiver<CSVError> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        let mut state = CSVState::Start;
        let mut line = 1;
        let mut col = 0;

        for b in reader.bytes() {
            let byte = b.unwrap();

            state = match state.parse_byte(byte) {
                Ok(new_state) => new_state,
                Err(error) => {
                    sender
                        .send(CSVError {
                            line: line,
                            col: col,
                            text: error,
                        })
                        .unwrap();
                    match error {
                        UNEXPECTED_EOL => CSVState::Start,
                        _ => CSVState::Error,
                    }
                }
            };

            if byte == LF {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
    });

    receiver
}
