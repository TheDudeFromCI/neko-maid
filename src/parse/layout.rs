//! Defines the layout structure and parsing logic for NekoMaid UI files.

use bevy::platform::collections::{HashMap, HashSet};

use crate::parse::NekoMaidParseError;
use crate::parse::class::parse_class;
use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::property::parse_property;
use crate::parse::token::TokenType;
use crate::parse::value::PropertyValue;

/// Represents a layout in the UI.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Layout {
    /// The widget type.
    pub(super) widget: String,

    /// The properties of the layout.
    pub(super) properties: HashMap<String, PropertyValue>,

    /// The child layouts.
    pub(super) children: Vec<Layout>,

    /// The classes applied to this layout.
    pub(super) classes: HashSet<String>,
}

/// Parses a layout from the input and returns a [`Layout`].
pub(super) fn parse_layout(ctx: &mut ParseContext) -> NekoResult<Layout> {
    ctx.maybe_consume(TokenType::LayoutKeyword);
    ctx.maybe_consume(TokenType::WithKeyword);

    let widget_position = ctx.next_position().unwrap_or_default();
    let widget = ctx.expect_as_string(TokenType::Identifier)?;

    if ctx.get_widget(&widget).is_none() {
        return Err(NekoMaidParseError::UnknownWidget {
            widget,
            position: widget_position,
        });
    };

    let mut layout = Layout {
        widget: widget.clone(),
        properties: HashMap::new(),
        children: Vec::new(),
        classes: HashSet::new(),
    };

    ctx.expect(TokenType::OpenBrace)?;

    while let Some(next) = ctx.peek() {
        match next.token_type {
            TokenType::Identifier => {
                let property = parse_property(ctx)?;
                layout.properties.insert(property.name, property.value);
            }
            TokenType::ClassKeyword => {
                let class = parse_class(ctx)?;
                layout.classes.insert(class);
            }
            TokenType::WithKeyword => {
                let child_layout = parse_layout(ctx)?;
                layout.children.push(child_layout);
            }
            TokenType::CloseBrace => break,
            _ => {
                return Err(NekoMaidParseError::UnexpectedToken {
                    expected: vec![
                        TokenType::Identifier.type_name().to_string(),
                        TokenType::ClassKeyword.type_name().to_string(),
                        TokenType::WithKeyword.type_name().to_string(),
                        TokenType::CloseBrace.type_name().to_string(),
                    ],
                    found: next.token_type.type_name().to_string(),
                    position: next.position,
                });
            }
        }
    }

    ctx.expect(TokenType::CloseBrace)?;
    Ok(layout)
}
