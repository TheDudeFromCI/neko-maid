//! Converts a NekoMaid UI file into a stream of tokens for parsing.

use std::fmt;

use bevy::color::Color;

use crate::parse::NekoMaidParseError;
use crate::parse::value::PropertyValue;

/// A token with its type and position.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Token {
    /// The type of the token.
    pub token_type: TokenType,

    /// The position of the token in the input.
    pub position: TokenPosition,

    /// The raw value of the token.
    pub value: TokenValue,
}

impl Token {
    /// Converts the token value to a string, if possible. Otherwise, returns an
    /// error.
    pub(crate) fn into_string_property(
        self,
        position: TokenPosition,
    ) -> Result<PropertyValue, NekoMaidParseError> {
        match self.value {
            TokenValue::String(s) => Ok(s.into()),
            v => Err(NekoMaidParseError::InvalidTokenValue {
                expected: "string".to_string(),
                found: format!("{:?}", v),
                position,
            }),
        }
    }

    /// Converts the token value to a number, if possible. Otherwise, returns an
    /// error.
    pub(crate) fn into_number_property(
        self,
        position: TokenPosition,
    ) -> Result<PropertyValue, NekoMaidParseError> {
        match self.value {
            TokenValue::Number(n) => Ok(n.into()),
            v => Err(NekoMaidParseError::InvalidTokenValue {
                expected: "number".to_string(),
                found: format!("{:?}", v),
                position,
            }),
        }
    }

    /// Converts the token value to a color, if possible. Otherwise, returns an
    /// error.
    pub(crate) fn into_color_property(
        self,
        position: TokenPosition,
    ) -> Result<PropertyValue, NekoMaidParseError> {
        match self.value {
            TokenValue::Color(c) => Ok(c.into()),
            v => Err(NekoMaidParseError::InvalidTokenValue {
                expected: "color".to_string(),
                found: format!("{:?}", v),
                position,
            }),
        }
    }

    /// Converts the token value to a boolean, if possible. Otherwise, returns
    /// an error.
    pub(crate) fn into_boolean_property(
        self,
        position: TokenPosition,
    ) -> Result<PropertyValue, NekoMaidParseError> {
        match self.value {
            TokenValue::Boolean(b) => Ok(b.into()),
            v => Err(NekoMaidParseError::InvalidTokenValue {
                expected: "boolean".to_string(),
                found: format!("{:?}", v),
                position,
            }),
        }
    }

    /// Converts the token value to a pixel number, if possible. Otherwise,
    /// returns an error.
    pub(crate) fn into_pixels_property(
        self,
        position: TokenPosition,
    ) -> Result<PropertyValue, NekoMaidParseError> {
        match self.value {
            TokenValue::Number(n) => Ok(PropertyValue::Pixels(n)),
            v => Err(NekoMaidParseError::InvalidTokenValue {
                expected: "number".to_string(),
                found: format!("{:?}", v),
                position,
            }),
        }
    }

    /// Converts the token value to a percentage number, if possible. Otherwise,
    /// returns an error.
    pub(crate) fn into_percent_property(
        self,
        position: TokenPosition,
    ) -> Result<PropertyValue, NekoMaidParseError> {
        match self.value {
            TokenValue::Number(n) => Ok(PropertyValue::Percent(n)),
            v => Err(NekoMaidParseError::InvalidTokenValue {
                expected: "number".to_string(),
                found: format!("{:?}", v),
                position,
            }),
        }
    }

    /// Converts the token value to a variable name string, if possible.
    /// Otherwise, returns an error.
    pub(crate) fn into_variable_name(
        self,
        position: TokenPosition,
    ) -> Result<String, NekoMaidParseError> {
        match self.value {
            TokenValue::String(s) => Ok(s),
            v => Err(NekoMaidParseError::InvalidTokenValue {
                expected: "string".to_string(),
                found: format!("{:?}", v),
                position,
            }),
        }
    }
}

/// The value stored within a token.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenValue {
    /// Used for tokens that do not carry a specific value, such as keywords or
    /// symbols.
    None,

    /// A string literal. (Also used for identifiers)
    String(String),

    /// A numeric literal.
    Number(f64),

    /// A color literal.
    Color(Color),

    /// A boolean literal.
    Boolean(bool),
}

impl From<&str> for TokenValue {
    fn from(s: &str) -> Self {
        TokenValue::String(s.into())
    }
}

impl From<String> for TokenValue {
    fn from(s: String) -> Self {
        TokenValue::String(s)
    }
}

impl From<f64> for TokenValue {
    fn from(n: f64) -> Self {
        TokenValue::Number(n)
    }
}

impl From<f32> for TokenValue {
    fn from(n: f32) -> Self {
        TokenValue::Number(n as f64)
    }
}

impl From<bool> for TokenValue {
    fn from(b: bool) -> Self {
        TokenValue::Boolean(b)
    }
}

impl From<Color> for TokenValue {
    fn from(c: Color) -> Self {
        TokenValue::Color(c)
    }
}

/// A token representing a lexical unit in the NekoMaid UI file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TokenType {
    // === Symbols ===
    /// The plus symbol.
    Plus,

    /// The exclamation symbol.
    Exclamation,

    /// The semicolon symbol.
    Semicolon,

    /// The colon symbol.
    Colon,

    /// The open brace symbol.
    OpenBrace,

    /// The close brace symbol.
    CloseBrace,

    /// The equals symbol.
    Equals,

    // === Keywords ===
    /// The `import` keyword.
    ImportKeyword,

    /// The `style` keyword,
    StyleKeyword,

    /// The `var` keyword.
    VarKeyword,

    /// The `layout` keyword.
    LayoutKeyword,

    /// The `with` keyword.
    WithKeyword,

    /// The `def` keyword.
    DefKeyword,

    /// The `class` keyword.
    ClassKeyword,

    /// The `output` keyword.
    OutputKeyword,

    /// The `in` keyword.
    InKeyword,

    // === Literals ===
    /// A boolean literal.
    BooleanLiteral,

    /// A color literal.
    ColorLiteral,

    /// A numeric literal.
    NumberLiteral,

    /// A percentage literal.
    PercentLiteral,

    /// A pixels literal.
    PixelsLiteral,

    /// A string literal.
    StringLiteral,

    // === Non-literals ===
    /// The Variable token.
    Variable,

    /// An identifier token.
    Identifier,

    // === Ignore ===
    /// A comment token.
    Comment,

    /// No more tokens (end of stream)
    EndOfStream,
}

impl TokenType {
    /// Returns the name of the token type.
    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            TokenType::Plus => "+",
            TokenType::Exclamation => "!",
            TokenType::Semicolon => ";",
            TokenType::Colon => ":",
            TokenType::OpenBrace => "{",
            TokenType::CloseBrace => "}",
            TokenType::Equals => "=",
            TokenType::ImportKeyword => "import",
            TokenType::StyleKeyword => "style",
            TokenType::VarKeyword => "var",
            TokenType::LayoutKeyword => "layout",
            TokenType::WithKeyword => "with",
            TokenType::DefKeyword => "def",
            TokenType::ClassKeyword => "class",
            TokenType::OutputKeyword => "output",
            TokenType::InKeyword => "in",
            TokenType::BooleanLiteral => "boolean",
            TokenType::ColorLiteral => "color",
            TokenType::NumberLiteral => "number",
            TokenType::PercentLiteral => "percent",
            TokenType::PixelsLiteral => "pixels",
            TokenType::StringLiteral => "string",
            TokenType::Variable => "variable",
            TokenType::Identifier => "identifier",
            TokenType::Comment => "comment",
            TokenType::EndOfStream => "EOS",
        }
    }

    /// Returns true if the token type represents a string value.
    pub(crate) fn has_string(&self) -> bool {
        matches!(
            self,
            TokenType::Identifier | TokenType::StringLiteral | TokenType::Variable
        )
    }

    /// Returns true if the token type represents a numeric value.
    pub(crate) fn has_number(&self) -> bool {
        matches!(
            self,
            TokenType::NumberLiteral | TokenType::PercentLiteral | TokenType::PixelsLiteral
        )
    }

    /// Returns true if the token type represents a boolean value.
    pub(crate) fn has_boolean(&self) -> bool {
        matches!(self, TokenType::BooleanLiteral)
    }

    /// Returns true if the token type represents a color value.
    pub(crate) fn has_color(&self) -> bool {
        matches!(self, TokenType::ColorLiteral)
    }

    /// Returns true if the token type should be ignored by the tokenizer.
    pub(crate) fn is_ignore(&self) -> bool {
        matches!(self, TokenType::Comment | TokenType::EndOfStream)
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.type_name())
    }
}

/// Represents the position of a token within the input string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenPosition {
    /// The line number of the token.
    pub line: usize,

    /// The column number of the token.
    pub column: usize,

    /// The length of the token span.
    pub length: usize,
}

impl TokenPosition {
    /// A constant representing an unknown token position.
    pub const UNKNOWN: TokenPosition = TokenPosition {
        line: 0,
        column: 0,
        length: 0,
    };

    /// Creates a new [`TokenPosition`].
    pub fn new(line: usize, column: usize, length: usize) -> Self {
        TokenPosition {
            line,
            column,
            length,
        }
    }
}

impl Default for TokenPosition {
    fn default() -> Self {
        TokenPosition {
            line: 1,
            column: 1,
            length: 0,
        }
    }
}

impl fmt::Display for TokenPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 && self.column == 0 {
            write!(f, "unknown position")
        } else {
            write!(
                f,
                "line {}, col {}-{}",
                self.line,
                self.column,
                self.column + self.length - 1
            )
        }
    }
}
