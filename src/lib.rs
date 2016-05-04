use std::fs::File;
use std::io::Read;

const LF: u8 = 10;
const CR: u8 = 13;
const QUOTE: u8 = 34;
const COMMA: u8 = 44;

enum CSVState {
    Start,
    NonQuotedValue,
    NonQuotedQuote,
    QuotedValue,
    QuoteQuote,
    ExpectLF,
    Error,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CSVError {
    pub line: i32,
    pub col: i32,
}

pub fn errors_for_csv(file: File) -> Vec<CSVError> {
    let mut state = CSVState::Start;
    let mut line = 1;
    let mut col = 0;

    let mut errors: Vec<CSVError> = Vec::new();

    for byte in file.bytes() {
        let b = byte.unwrap();

        match (&state, b) {
            // At the start of a field, are we a quoted field or not?
            (&CSVState::Start, QUOTE) => {
                state = CSVState::QuotedValue;
            },
            (&CSVState::Start, _) => {
                state = CSVState::NonQuotedValue;
            },

            // In a non-quoted field, watch for quotes
            (&CSVState::NonQuotedValue, COMMA) => {
                state = CSVState::Start;
            }
            (&CSVState::NonQuotedValue, CR) => {
                state = CSVState::ExpectLF;
            }
            (&CSVState::NonQuotedValue, LF) => {
                state = CSVState::Start;
            }
            (&CSVState::NonQuotedValue, QUOTE) => {
                state = CSVState::NonQuotedQuote;
            }

            // A quote in a non-quoted field needs a matching quote to
            // close it out.
            (&CSVState::NonQuotedQuote, QUOTE) => {
                state = CSVState::NonQuotedValue;
            }
            (&CSVState::NonQuotedQuote, COMMA) => {
                errors.push(CSVError { line: line, col: col });
                state = CSVState::Error;
            }
            (&CSVState::NonQuotedQuote, CR) => {
                errors.push(CSVError { line: line, col: col });
                state = CSVState::Start;
            }
            (&CSVState::NonQuotedQuote, LF) => {
                errors.push(CSVError { line: line, col: col });
                state = CSVState::Start;
            }

            // In a quoted field, watch for quotes or newlines
            (&CSVState::QuotedValue, QUOTE) => {
                state = CSVState::QuoteQuote;
            },
            (&CSVState::QuotedValue, CR) => {
                errors.push(CSVError { line: line, col: col });
                state = CSVState::Start;
            },
            (&CSVState::QuotedValue, LF) => {
                errors.push(CSVError { line: line, col: col });
                state = CSVState::Start;
            },

            // A quote in a quote needs a quote immediately following
            // to be an escaped quote. If there's a comma or newline
            // following, it means that the field is done. Otherwise,
            // it's an error.
            (&CSVState::QuoteQuote, QUOTE) => {
                state = CSVState::QuotedValue;
            },
            (&CSVState::QuoteQuote, COMMA) => {
                state = CSVState::Start;
            },
            (&CSVState::QuoteQuote, CR) => {
                state = CSVState::ExpectLF;
            },
            (&CSVState::QuoteQuote, LF) => {
                state = CSVState::Start;
            },
            (&CSVState::QuoteQuote, _) => {
                errors.push(CSVError { line: line, col: col });
                state = CSVState::Error;
            },

            // If we're looking at CR, we need a LF to follow
            (&CSVState::ExpectLF, LF) => {
                state = CSVState::Start;
            },
            (&CSVState::ExpectLF, _) => {
                errors.push(CSVError { line: line, col: col });
                state = CSVState::Error;
            },

            // If we're in an error state, once we reach a newline, we
            // start over.
            (&CSVState::Error, LF) => {
                state = CSVState::Start;
            },
            _ => ()
        };

        if b == 10 {
            line = line + 1;
            col = 0;
        } else {
            col = col + 1;
        }
    }

    errors
}
