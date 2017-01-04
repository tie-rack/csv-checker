use std::fs::File;
use std::io::Read;
use std::sync::mpsc::Sender;

const LF: u8 = 10;
const CR: u8 = 13;
const QUOTE: u8 = 34;
const COMMA: u8 = 44;

// Error messages
pub const UNEXPECTED_EOF: &'static str = "Unexpected end of field";
pub const UNEXPECTED_EOL: &'static str = "Unexpected end of line";
pub const UNEXPECTED_CHAR: &'static str = "Unexpected character after quote";
pub const EXPECTED_LF: &'static str = "Expected linefeed after carriage return";

enum CSVState {
    Start,
    NonQuotedValue,
    NonQuotedQuote,
    QuotedValue,
    QuoteQuote,
    ExpectLF,
    Error,
}

type CSVResult = Result<CSVState, &'static str>;

type ByteParser = fn(u8) -> CSVResult;

#[derive(Debug, PartialEq, Eq)]
pub struct CSVError {
    pub line: i32,
    pub col: i32,
    pub text: &'static str,
}

fn parse_start(byte: u8) -> CSVResult {
    match byte {
        QUOTE => Ok(CSVState::QuotedValue),
        COMMA => Ok(CSVState::Start),
        _ => Ok(CSVState::NonQuotedValue),
    }
}

fn parse_non_quoted(byte: u8) -> CSVResult {
    match byte {
        COMMA => Ok(CSVState::Start),
        CR => Ok(CSVState::ExpectLF),
        LF => Ok(CSVState::Start),
        QUOTE => Ok(CSVState::NonQuotedQuote),
        _ => Ok(CSVState::NonQuotedValue),
    }
}

fn parse_non_quoted_quote(byte: u8) -> CSVResult {
    match byte {
        QUOTE => Ok(CSVState::NonQuotedValue),
        COMMA => Err(UNEXPECTED_EOF),
        CR => Err(UNEXPECTED_EOL),
        LF => Err(UNEXPECTED_EOL),
        _ => Ok(CSVState::NonQuotedQuote),
    }
}

fn parse_quoted(byte: u8) -> CSVResult {
    match byte {
        QUOTE => Ok(CSVState::QuoteQuote),
        CR => Err(UNEXPECTED_EOL),
        LF => Err(UNEXPECTED_EOL),
        _ => Ok(CSVState::QuotedValue),
    }
}

fn parse_quote_quote(byte: u8) -> CSVResult {
    match byte {
        QUOTE => Ok(CSVState::QuotedValue),
        COMMA => Ok(CSVState::Start),
        CR => Ok(CSVState::ExpectLF),
        LF => Ok(CSVState::Start),
        _ => Err(UNEXPECTED_CHAR),
    }
}

fn parse_cr(byte: u8) -> CSVResult {
    match byte {
        LF => Ok(CSVState::Start),
        _ => Err(EXPECTED_LF),
    }
}

fn parse_err(byte: u8) -> CSVResult {
    match byte {
        LF => Ok(CSVState::Start),
        _ => Ok(CSVState::Error),
    }
}

fn next_state(state: CSVState, byte: u8) -> CSVResult {
    let parse_fn: ByteParser = match state {
        CSVState::Start => parse_start,
        CSVState::NonQuotedValue => parse_non_quoted,
        CSVState::NonQuotedQuote => parse_non_quoted_quote,
        CSVState::QuotedValue => parse_quoted,
        CSVState::QuoteQuote => parse_quote_quote,
        CSVState::ExpectLF => parse_cr,
        CSVState::Error => parse_err,
    };
    parse_fn(byte)
}

pub fn publish_errors_for_csv(file: File, sender: Sender<CSVError>) {
    let mut state = CSVState::Start;
    let mut line = 1;
    let mut col = 0;

    for b in file.bytes() {
        let byte = b.unwrap();

        state = match next_state(state, byte) {
            Ok(new_state) => new_state,
            Err(error) => {
                sender.send(CSVError {
                    line: line,
                    col: col,
                    text: error,
                }).unwrap();
                if error == UNEXPECTED_EOL {
                    CSVState::Start
                } else {
                    CSVState::Error
                }
            }
        };

        if byte == LF {
            line = line + 1;
            col = 0;
        } else {
            col = col + 1;
        }
    }
}
