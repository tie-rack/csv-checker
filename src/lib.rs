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

type CSVResult = Result<CSVState, ()>;

#[derive(Debug, PartialEq, Eq)]
pub struct CSVError {
    pub line: i32,
    pub col: i32,
}

fn parse_start(byte: u8) -> CSVResult {
    match byte {
        QUOTE => Ok(CSVState::QuotedValue),
        COMMA => Ok(CSVState::Start),
        _ => Ok(CSVState::NonQuotedValue)
    }
}

fn parse_non_quoted(byte: u8) -> CSVResult {
    match byte {
        COMMA => Ok(CSVState::Start),
        CR    => Ok(CSVState::ExpectLF),
        LF    => Ok(CSVState::Start),
        QUOTE => Ok(CSVState::NonQuotedQuote),
        _     => Ok(CSVState::NonQuotedValue),
    }
}

fn parse_non_quoted_quote(byte: u8) -> CSVResult {
    match byte {
        QUOTE => Ok(CSVState::NonQuotedValue),
        COMMA => Err(()),
        CR    => Err(()),
        LF    => Err(()),
        _     => Ok(CSVState::NonQuotedQuote),
    }
}

fn parse_quoted(byte: u8) -> CSVResult {
    match byte {
        QUOTE => Ok(CSVState::QuoteQuote),
        CR    => Err(()),
        LF    => Err(()),
        _     => Ok(CSVState::QuotedValue),
    }
}

fn parse_quote_quote(byte: u8) -> CSVResult {
    match byte {
        QUOTE => Ok(CSVState::QuotedValue),
        COMMA => Ok(CSVState::Start),
        CR    => Ok(CSVState::ExpectLF),
        LF    => Ok(CSVState::Start),
        _     => Err(()),
    }
}

fn parse_cr(byte: u8) -> CSVResult {
    match byte {
        LF => Ok(CSVState::Start),
        _  => Err(()),
    }
}

fn parse_err(byte: u8) -> CSVResult {
    match byte {
        LF => Ok(CSVState::Start),
        _  => Ok(CSVState::Error),
    }
}

pub fn errors_for_csv(file: File) -> Vec<CSVError> {
    let mut state = CSVState::Start;
    let mut line = 1;
    let mut col = 0;

    let mut errors: Vec<CSVError> = Vec::new();

    for byte in file.bytes() {
        let b = byte.unwrap();

        let next_state = match state {
            CSVState::Start          => parse_start(b),
            CSVState::NonQuotedValue => parse_non_quoted(b),
            CSVState::NonQuotedQuote => parse_non_quoted_quote(b),
            CSVState::QuotedValue    => parse_quoted(b),
            CSVState::QuoteQuote     => parse_quote_quote(b),
            CSVState::ExpectLF       => parse_cr(b),
            CSVState::Error          => parse_err(b),
        };

        state = match next_state {
            Ok(new_state) => new_state,
            Err(_)        => {
                errors.push(CSVError { line: line, col: col });
                CSVState::Error
            },
        };

        if b == LF {
            line = line + 1;
            col = 0;
        } else {
            col = col + 1;
        }
    }

    errors
}
