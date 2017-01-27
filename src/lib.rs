use std::io::Read;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;

const LF: u8 = b'\n';
const CR: u8 = b'\r';
const QUOTE: u8 = b'"';
const COMMA: u8 = b',';

// Error messages
pub const UNEXPECTED_EOL: &'static str = "Unexpected end of line";
pub const UNEXPECTED_CHAR: &'static str = "Unexpected character after quote";
pub const EXPECTED_LF: &'static str = "Expected linefeed after carriage return";

enum CSVState {
    Start,
    NonQuotedValue,
    QuotedValue,
    QuoteQuote,
    ExpectLF,
    Error,
}

type CSVResult = Result<CSVState, &'static str>;

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
        COMMA | LF => Ok(CSVState::Start),
        CR => Ok(CSVState::ExpectLF),
        _ => Ok(CSVState::NonQuotedValue),
    }
}

fn parse_quoted(byte: u8) -> CSVResult {
    match byte {
        QUOTE => Ok(CSVState::QuoteQuote),
        CR | LF => Err(UNEXPECTED_EOL),
        _ => Ok(CSVState::QuotedValue),
    }
}

fn parse_quote_quote(byte: u8) -> CSVResult {
    match byte {
        QUOTE => Ok(CSVState::QuotedValue),
        COMMA | LF => Ok(CSVState::Start),
        CR => Ok(CSVState::ExpectLF),
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
    match state {
        CSVState::Start => parse_start(byte),
        CSVState::NonQuotedValue => parse_non_quoted(byte),
        CSVState::QuotedValue => parse_quoted(byte),
        CSVState::QuoteQuote => parse_quote_quote(byte),
        CSVState::ExpectLF => parse_cr(byte),
        CSVState::Error => parse_err(byte),
    }
}

pub fn publish_errors_for_csv<T: Read + Send + 'static>(reader: T) -> Receiver<CSVError> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        let mut state = CSVState::Start;
        let mut line = 1;
        let mut col = 0;

        for b in reader.bytes() {
            let byte = b.unwrap();

            state = match next_state(state, byte) {
                Ok(new_state) => new_state,
                Err(error) => {
                    sender.send(CSVError {
                        line: line,
                        col: col,
                        text: error,
                    }).unwrap();
                    match error {
                        UNEXPECTED_EOL => CSVState::Start,
                        _ => CSVState::Error
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
