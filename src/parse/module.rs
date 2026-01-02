//! Module parsing functionality.

use bevy::platform::collections::HashMap;

use crate::parse::NekoMaidParseError;
use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::element::NekoElementBuilder;
use crate::parse::import::parse_import;
use crate::parse::layout::parse_layout;
use crate::parse::property::{UnresolvedPropertyValue, parse_variable};
use crate::parse::style::{Selector, Style, parse_style};
use crate::parse::token::TokenType;
use crate::parse::widget::{Widget, parse_widget};

/// A NekoMaid UI module.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// A map of defined variables and their values.
    pub(crate) variables: HashMap<String, UnresolvedPropertyValue>,

    /// A list of defined styles.
    ///
    /// Styles later in the list have higher precedence.
    pub(crate) styles: Vec<Style>,

    /// A map of available widgets. (Both native and user-defined)
    pub(super) widgets: HashMap<String, Widget>,

    /// A list of elements defined in this module, ready to be instantiated.
    pub(crate) elements: Vec<NekoElementBuilder>,
}

/// Parses a module from the given parse context.
pub(super) fn parse_module(mut ctx: ParseContext) -> NekoResult<Module> {
    while let Some(next) = ctx.peek() {
        match next.token_type {
            TokenType::ImportKeyword => parse_import(&mut ctx)?,
            TokenType::VarKeyword => {
                let variable = parse_variable(&mut ctx)?;
                ctx.set_variable(variable.name, variable.value);
            }
            TokenType::DefKeyword => {
                let widget = parse_widget(&mut ctx)?;
                ctx.add_widget(widget);
            }
            TokenType::StyleKeyword => {
                parse_style(&mut ctx, Selector::default())?;
            }
            TokenType::LayoutKeyword => {
                let layout = parse_layout(&mut ctx)?;
                ctx.add_layout(layout);
            }
            _ => {
                return Err(NekoMaidParseError::UnexpectedToken {
                    expected: vec![
                        TokenType::ImportKeyword.type_name().to_string(),
                        TokenType::VarKeyword.type_name().to_string(),
                        TokenType::DefKeyword.type_name().to_string(),
                        TokenType::StyleKeyword.type_name().to_string(),
                        TokenType::LayoutKeyword.type_name().to_string(),
                    ],
                    found: next.token_type.type_name().to_string(),
                    position: next.position,
                });
            }
        }
    }

    ctx.into_module()
}
