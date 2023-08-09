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
    UnterminatedStringLiteral { pos: usize },
    ExpectCharAfter { pos: usize, expected: char, after: char, got: char },
    UnknownChar { pos: usize, got: char}
}