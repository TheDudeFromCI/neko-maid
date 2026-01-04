//! A tokenizer for parsing source code into tokens.

use bevy::color::Srgba;
use lazy_static::lazy_static;
use regex::Regex;

use crate::parse::token::{Token, TokenPosition, TokenType, TokenValue};

#[rustfmt::skip]
lazy_static! {
    static ref TOKENS: Vec<(TokenType, Regex)> = vec![
        // symbols
        (TokenType::Plus,            Regex::new(r"^\s*(\+)").unwrap()),
        (TokenType::Exclamation,     Regex::new(r"^\s*(!)").unwrap()),
        (TokenType::Semicolon,       Regex::new(r"^\s*(;)").unwrap()),
        (TokenType::Colon,           Regex::new(r"^\s*(:)").unwrap()),
        (TokenType::OpenBrace,       Regex::new(r"^\s*(\{)").unwrap()),
        (TokenType::CloseBrace,      Regex::new(r"^\s*(\})").unwrap()),
        (TokenType::Equals,          Regex::new(r"^\s*(=)").unwrap()),

        // keywords
        (TokenType::ImportKeyword,   Regex::new(r"^\s*(import)\b").unwrap()),
        (TokenType::StyleKeyword,    Regex::new(r"^\s*(style)\b").unwrap()),
        (TokenType::VarKeyword,      Regex::new(r"^\s*(var)\b").unwrap()),
        (TokenType::LayoutKeyword,   Regex::new(r"^\s*(layout)\b").unwrap()),
        (TokenType::WithKeyword,     Regex::new(r"^\s*(with)\b").unwrap()),
        (TokenType::DefKeyword,      Regex::new(r"^\s*(def)\b").unwrap()),
        (TokenType::ClassKeyword,    Regex::new(r"^\s*(class)\b").unwrap()),
        (TokenType::OutputKeyword,   Regex::new(r"^\s*(output)\b").unwrap()),
        (TokenType::InKeyword,   Regex::new(r"^\s*(in)\b").unwrap()),

        // literals
        (TokenType::BooleanLiteral,  Regex::new(r"^\s*([Tt]rue|[Ff]alse)\b").unwrap()),
        (TokenType::ColorLiteral,    Regex::new(r"^\s*#([a-fA-F0-9]{8}|[a-fA-F0-9]{6}|[a-fA-F0-9]{4}|[a-fA-F0-9]{3})\b").unwrap()),
        (TokenType::PercentLiteral,  Regex::new(r"^\s*(-?\d+\.?\d*|-?\d*\.\d+)%").unwrap()),
        (TokenType::PixelsLiteral,   Regex::new(r"^\s*(-?\d+\.?\d*|-?\d*\.\d+)px\b").unwrap()),
        (TokenType::NumberLiteral,   Regex::new(r"^\s*(-?\d+\.?\d*|-?\d*\.\d+)\b").unwrap()),
        (TokenType::StringLiteral,   Regex::new(r#"^\s*"(.*)""#).unwrap()),
        (TokenType::StringLiteral,   Regex::new(r#"^\s*'(.*)'"#).unwrap()),
        (TokenType::StringLiteral,   Regex::new(r#"^\s*`(.*)`"#).unwrap()),

        // non-literals
        (TokenType::Variable,        Regex::new(r"^\s*\$([a-zA-Z_][a-zA-Z0-9_-]*)").unwrap()),
        (TokenType::Identifier,      Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_-]*)").unwrap()),

        // ignore
        (TokenType::Comment,         Regex::new(r"^\s*//(.*)(?:\n|$)").unwrap()),
        (TokenType::EndOfStream,     Regex::new(r"^(\s*)$").unwrap()),
    ];
}

/// A position within the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CodePos {
    /// The index within the source code.
    index: usize,

    /// The line number within the source code.
    line: usize,

    /// The column number within the source code.
    column: usize,
}

impl Default for CodePos {
    fn default() -> Self {
        Self {
            index: 0,
            line: 1,
            column: 1,
        }
    }
}

/// A tokenizer for parsing source code into tokens.
pub(crate) struct Tokenizer;
impl Tokenizer {
    /// Tokenizes the given source code into a vector of tokens.
    ///
    /// Tokens marked as "ignore" (e.g., whitespace, comments) are omitted from
    /// the resulting vector.
    pub(super) fn tokenize(code: &str) -> Result<Vec<Token>, TokenizeError> {
        let mut position = CodePos::default();
        let mut tokens = Vec::new();

        'outer: while position.index < code.len() {
            for (token_type, regex) in TOKENS.iter() {
                if let Some(t) = try_token(code, &mut position, regex, *token_type) {
                    if !t.token_type.is_ignore() {
                        tokens.push(t);
                    }
                    continue 'outer;
                }
            }

            return Err(TokenizeError::UnexpectedCharacter {
                character: code.chars().nth(position.index).unwrap(),
                position: TokenPosition {
                    line: position.line,
                    column: position.column,
                    length: 1,
                },
            });
        }

        Ok(tokens)
    }
}

/// Errors that can occur during tokenization.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum TokenizeError {
    /// An unexpected character was encountered during tokenization.
    #[error("Unexpected character '{character}' at {position}")]
    UnexpectedCharacter {
        /// The unexpected character.
        character: char,

        /// The position of the unexpected character.
        position: TokenPosition,
    },
}

fn try_token(
    code: &str,
    position: &mut CodePos,
    regex: &Regex,
    token_type: TokenType,
) -> Option<Token> {
    if let Some((start, end, full_end)) = try_regex(regex, code, position.index) {
        let mut token = Token {
            token_type,
            position: TokenPosition {
                line: position.line,
                column: position.column,
                length: end - start,
            },
            value: TokenValue::None,
        };
        update_position(code, position, full_end);

        if token_type.has_string() {
            let matched_str = &code[start .. end];
            token.value = TokenValue::String(matched_str.to_string());
        }

        if token_type.has_number() {
            let matched_str = &code[start .. end];
            token.value = TokenValue::Number(matched_str.parse::<f64>().unwrap());
        }

        if token_type.has_boolean() {
            let matched_str = &code[start .. end].to_lowercase();
            if matched_str == "true" {
                token.value = TokenValue::Boolean(true);
            } else if matched_str == "false" {
                token.value = TokenValue::Boolean(false);
            }
        }

        if token_type.has_color() {
            let matched_str = &code[start .. end];
            let color = Srgba::hex(matched_str)
                .expect("Hex code Validated by regex")
                .into();
            token.value = TokenValue::Color(color);
        }

        Some(token)
    } else {
        None
    }
}

/// Updates the current token position based on the new start index.
fn update_position(code: &str, position: &mut CodePos, new_start: usize) {
    for c in code[position.index .. new_start].chars() {
        if c == '\n' {
            position.line += 1;
            position.column = 1;
        } else {
            position.column += 1;
        }
    }
    position.index = new_start;
}

/// Attempts to match the given regex at the current position in the code.
///
/// Returns the start and end indices of the match if successful. Also returns
/// the end index of the full match (including any trailing characters that are
/// not part of the captured group).
fn try_regex(re: &Regex, code: &str, offset: usize) -> Option<(usize, usize, usize)> {
    if let Some(captures) = re.captures(&code[offset ..])
        && let Some(matched) = captures.get(1)
        && let Some(full_match) = captures.get(0)
    {
        let start = matched.start() + offset;
        let end = matched.end() + offset;
        let full_end = full_match.end() + offset;
        return Some((start, end, full_end));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_positions() {
        let code = r"
my code
  is here
    and here
but not here";

        let mut pos = CodePos::default();
        let regex = Regex::new(r"^\s*([a-z]+)").unwrap();

        let token = try_token(code, &mut pos, &regex, TokenType::Identifier).unwrap();
        assert_eq!(
            token.position,
            TokenPosition {
                line: 2,
                column: 0,
                length: 2,
            }
        );

        let token = try_token(code, &mut pos, &regex, TokenType::Identifier).unwrap();
        assert_eq!(
            token.position,
            TokenPosition {
                line: 2,
                column: 4,
                length: 4,
            }
        );

        let token = try_token(code, &mut pos, &regex, TokenType::Identifier).unwrap();
        assert_eq!(
            token.position,
            TokenPosition {
                line: 3,
                column: 2,
                length: 2,
            }
        );

        let token = try_token(code, &mut pos, &regex, TokenType::Identifier).unwrap();
        assert_eq!(
            token.position,
            TokenPosition {
                line: 3,
                column: 5,
                length: 4,
            }
        );
    }

    #[test]
    fn tokenize_boolean() {
        let code = "true false True False";
        let tokens = Tokenizer::tokenize(code).unwrap();

        assert_eq!(tokens.len(), 4);

        assert_eq!(tokens[0].token_type, TokenType::BooleanLiteral);
        assert_eq!(tokens[0].value, true.into());

        assert_eq!(tokens[1].token_type, TokenType::BooleanLiteral);
        assert_eq!(tokens[1].value, false.into());

        assert_eq!(tokens[2].token_type, TokenType::BooleanLiteral);
        assert_eq!(tokens[2].value, true.into());

        assert_eq!(tokens[3].token_type, TokenType::BooleanLiteral);
        assert_eq!(tokens[3].value, false.into());
    }

    #[test]
    fn tokenize_identifier() {
        let code = "my_var anotherVar _privateVar var123 red-blue";
        let tokens = Tokenizer::tokenize(code).unwrap();

        assert_eq!(tokens.len(), 5);

        assert_eq!(tokens[0].token_type, TokenType::Identifier);
        assert_eq!(tokens[0].value, "my_var".into());

        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].value, "anotherVar".into());

        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].value, "_privateVar".into());

        assert_eq!(tokens[3].token_type, TokenType::Identifier);
        assert_eq!(tokens[3].value, "var123".into());

        assert_eq!(tokens[4].token_type, TokenType::Identifier);
        assert_eq!(tokens[4].value, "red-blue".into());
    }

    #[test]
    fn tokenizer_numbers() {
        let code = "123 45.67 0.001 1000 .5 1. -3 -.2";
        let tokens = Tokenizer::tokenize(code).unwrap();

        assert_eq!(tokens.len(), 7);

        assert_eq!(tokens[0].token_type, TokenType::NumberLiteral);
        assert_eq!(tokens[0].value, 123.0.into());

        assert_eq!(tokens[1].token_type, TokenType::NumberLiteral);
        assert_eq!(tokens[1].value, 45.67.into());

        assert_eq!(tokens[2].token_type, TokenType::NumberLiteral);
        assert_eq!(tokens[2].value, 0.001.into());

        assert_eq!(tokens[3].token_type, TokenType::NumberLiteral);
        assert_eq!(tokens[3].value, 1000.0.into());

        assert_eq!(tokens[4].token_type, TokenType::NumberLiteral);
        assert_eq!(tokens[4].value, 0.5.into());

        assert_eq!(tokens[5].token_type, TokenType::NumberLiteral);
        assert_eq!(tokens[5].value, 1.0.into());

        assert_eq!(tokens[6].token_type, TokenType::NumberLiteral);
        assert_eq!(tokens[6].value, (-3.0).into());
    }

    #[test]
    fn tokenize_strings() {
        let code = r#""hello" 'world' `backtick`"#;
        let tokens = Tokenizer::tokenize(code).unwrap();

        assert_eq!(tokens.len(), 3);

        assert_eq!(tokens[0].token_type, TokenType::StringLiteral);
        assert_eq!(tokens[0].value, "hello".into());

        assert_eq!(tokens[1].token_type, TokenType::StringLiteral);
        assert_eq!(tokens[1].value, "world".into());

        assert_eq!(tokens[2].token_type, TokenType::StringLiteral);
        assert_eq!(tokens[2].value, "backtick".into());
    }
}
