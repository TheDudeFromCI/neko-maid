//! Handles parsing for import statements and predicts the imports required by a
//! set of tokens.

use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::token::{Token, TokenType, TokenValue};

/// Predicts the imports required by the given tokens.
///
/// This function is not guaranteed to be accurate if the tokens are malformed.
pub fn predict_imports(tokens: &[Token]) -> Vec<String> {
    let mut imports = Vec::new();

    for i in 0 .. tokens.len() - 1 {
        if tokens[i].token_type != TokenType::ImportKeyword {
            continue;
        }

        if tokens[i + 1].token_type != TokenType::StringLiteral {
            continue;
        }

        let TokenValue::String(name) = &tokens[i + 1].value else {
            continue;
        };

        imports.push(name.clone());
    }

    imports
}

/// Parses an import statement from the token stream an attempts to import it.
pub fn parse_import(ctx: &mut ParseContext) -> NekoResult<()> {
    ctx.expect(TokenType::ImportKeyword)?;
    let path_pos = ctx.next_position().unwrap_or_default();
    let path = ctx.expect_as_string(TokenType::StringLiteral)?;
    ctx.expect(TokenType::Semicolon)?;

    ctx.import_module(&path, path_pos)?;
    Ok(())
}
