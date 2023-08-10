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

// #[derive(Debug)]
// pub enum RuntimeResult {
//     Ok,
//     Warn(Warning),
//     Err(RuntimeError)
// }


#[derive(Debug)]
pub enum Warning {
    ExprHasNoEffect,
    ApplyRuleNoEffect,
    InLineRuleNoEffect,
    EndStmtHasNoEffect,
    RuleDoesNotExist(String)
}

impl Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Warning::ExprHasNoEffect => {
                writeln!(f, "Warning: the provided expression has no effect inside a pattern matching context.")?;
                writeln!(f, "         Consider applying a rule statement, 'apply YOUR_RULE_NAME at SOME_DEPTH'; or,")?;
                writeln!(f, "         consider applying an in-line rule statement, 'LEFT_EXPR => RIGHT_EXPR at SOME_DEPTH'; or")?;
                Ok(())
            },
            Warning::ApplyRuleNoEffect => {
                writeln!(f, "Warning: cannot apply rule statement outside of pattern matching context.")?;
                writeln!(f, "         Thus this statement is ignored.")?;
                Ok(())
            },
            Warning::InLineRuleNoEffect => {
                writeln!(f, "Warning: cannot apply in-line rule statement outside of pattern matching context.")?;
                writeln!(f, "         Thus this statement is ignored.")?;
                Ok(())
            },
            Warning::EndStmtHasNoEffect => {
                writeln!(f, "Warning: End-statement takes no effect, as there is no pattern matching context to end. ")?;
                writeln!(f, "         Thus this statement is ignored.")?;
                Ok(())
            },
            Warning::RuleDoesNotExist(s) => {
                writeln!(f, "Warning: cannot find rule '{}'. First define the rule before applying it like", s)?;
                writeln!(f, "         'def YOUR_RULE_NAME as LEFT_EXPR => RIGHT_EXPR'. Thus this statement is ignored.")?;
                Ok(())
            }
        }
    }
}

// #[derive(Debug)]
// pub enum RuntimeError {
//     PathNotFound(String),
//     CannotWriteToPath(String),
// }

// impl Error for RuntimeError {}

// impl Display for RuntimeError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             RuntimeError::CannotWriteToPath(s) => writeln!(f, "Runtime error: cannot write to file specified by path '{}'", s),
//             RuntimeError::PathNotFound(s) => writeln!(f, "Runtime error: path to file not found, '{}'", s),
//         }
//     }
// }