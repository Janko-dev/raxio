use std::{fmt::Display, error::Error};

#[derive(Debug, PartialEq, Clone)]
pub enum LexError {
    UnterminatedStringLiteral { pos: usize },
    UnterminatedStringLiteralAtEnd,
    ExpectCharAfter { pos: usize, expected: char, after: char, got: char },
    UnknownChar { pos: usize, got: char}
}

impl Error for LexError {}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnterminatedStringLiteral { pos } => 
                writeln!(f, "Syntax error: unterminated string literal in path at position {}", pos),
            LexError::UnterminatedStringLiteralAtEnd => 
                writeln!(f, "Syntax error: unterminated string literal in path at the end of the line"),
            LexError::ExpectCharAfter { pos, expected, after, got } => 
                writeln!(f, "Syntax error: expected '{}' after '{}', but got '{}' at position {}", expected, after, got, pos),
            LexError::UnknownChar { pos, got } => 
                writeln!(f, "Syntax error: Unknown character found '{}' at position {}", got, pos)
        }
    }
}

#[derive(Debug)]
pub enum ParsingError {
    ExpectToken { expected: String, got: Option<String> },
    ExpectTokenAfter { expected: String, after: String, got: Option<String> },
    ExpectDepthValue,
    UnexpectedToken { got: Option<String> }
}

impl Error for ParsingError {}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::ExpectToken { expected, got } => 
                writeln!(f, "Parsing error: expected {}, but got {}", 
                    expected, 
                    got.clone().unwrap_or("nothing".to_string())),
            ParsingError::ExpectTokenAfter { expected, after, got } => 
                writeln!(f, "Parsing error: expected {} after {}, but got {}", 
                    expected, 
                    after, 
                    got.clone().unwrap_or("nothing".to_string())),
            ParsingError::ExpectDepthValue => 
                writeln!(f, "Parsing error: expected a depth value after in-line rule"),
            ParsingError::UnexpectedToken { got } => 
                writeln!(f, "Parsing error: unexpected token found, got {}",
                    got.clone().unwrap_or("nothing".to_string())),
        }
    }
}