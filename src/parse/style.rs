//! A parser for NekoMaid UI style definitions.

use bevy::platform::collections::{HashMap, HashSet};

use crate::parse::NekoMaidParseError;
use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::property::parse_property;
use crate::parse::token::TokenType;
use crate::parse::value::PropertyValue;

/// A NekoMaid UI style definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    /// The selector for the style.
    pub(crate) selector: Selector,

    /// The properties defined in the style.
    pub(crate) properties: HashMap<String, PropertyValue>,
}

impl Style {
    /// Creates a new Style with the given selector and properties.
    pub fn new(selector: Selector, properties: HashMap<String, PropertyValue>) -> Self {
        Self {
            selector,
            properties,
        }
    }

    /// Returns a reference to the selector of this style.
    pub fn get_property(&self, name: &str) -> Option<&PropertyValue> {
        self.properties.get(name)
    }

    /// Sets a property in this style.
    pub fn set_property(&mut self, name: String, value: PropertyValue) {
        self.properties.insert(name, value);
    }

    /// Returns a reference to the selector of this style.
    pub fn selector(&self) -> &Selector {
        &self.selector
    }

    /// Returns a reference to the properties of this style.
    pub fn properties(&self) -> &HashMap<String, PropertyValue> {
        &self.properties
    }

    /// Merges another style into this one, overriding existing properties, and
    /// adding new ones.
    pub fn merge(&mut self, other: Style) {
        for (key, value) in other.properties {
            self.properties.insert(key, value);
        }
    }
}

/// A selector for targeting widgets in styles.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Selector {
    /// A hierarchy of selector parts, to target multi-level widget structures.
    pub hierarchy: Vec<SelectorPart>,
}

/// A part of a style selector, targeting a specific widget and classes.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectorPart {
    /// The widget the selector part applies to.
    pub widget: String,

    /// The classes the selector part requires.
    pub whitelist: HashSet<String>,

    /// The classes the selector part excludes.
    pub blacklist: HashSet<String>,
}

/// Parses a style from the given parse context.
pub(super) fn parse_style(ctx: &mut ParseContext, mut selector: Selector) -> NekoResult<()> {
    ctx.maybe_consume(TokenType::StyleKeyword);
    ctx.maybe_consume(TokenType::WithKeyword);

    let selector_part = parse_style_selector(ctx)?;
    selector.hierarchy.push(selector_part);

    ctx.expect(TokenType::OpenBrace)?;

    let mut properties = HashMap::new();

    while let Some(next) = ctx.peek() {
        match next.token_type {
            TokenType::Identifier => {
                let property = parse_property(ctx)?;
                properties.insert(property.name, property.value);
            }
            TokenType::WithKeyword => {
                parse_style(ctx, selector.clone())?;
            }
            TokenType::CloseBrace => break,
            _ => {
                return Err(NekoMaidParseError::UnexpectedToken {
                    expected: vec![
                        TokenType::Identifier.type_name().to_string(),
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

    ctx.add_style(Style {
        selector,
        properties,
    });

    Ok(())
}

/// Parses a style selector part from the input and returns a [`SelectorPart`].
pub(super) fn parse_style_selector(ctx: &mut ParseContext) -> NekoResult<SelectorPart> {
    let widget_position = ctx.next_position().unwrap_or_default();
    let widget = ctx.expect_as_string(TokenType::Identifier)?;

    if ctx.get_widget(&widget).is_none() {
        return Err(NekoMaidParseError::UnknownWidget {
            widget,
            position: widget_position,
        });
    }

    let mut whitelist = HashSet::new();
    let mut blacklist = HashSet::new();

    while let Some(next) = ctx.peek() {
        match next.token_type {
            TokenType::Plus => {
                ctx.expect(TokenType::Plus)?;

                let class_name = ctx.expect_as_string(TokenType::Identifier)?;
                whitelist.insert(class_name);
            }
            TokenType::Exclamation => {
                ctx.expect(TokenType::Exclamation)?;

                let class_name = ctx.expect_as_string(TokenType::Identifier)?;
                blacklist.insert(class_name);
            }
            TokenType::OpenBrace => break,
            _ => {
                return Err(NekoMaidParseError::UnexpectedToken {
                    expected: vec![
                        TokenType::Plus.type_name().to_string(),
                        TokenType::Exclamation.type_name().to_string(),
                        TokenType::OpenBrace.type_name().to_string(),
                    ],
                    found: next.token_type.type_name().to_string(),
                    position: next.position,
                });
            }
        }
    }

    Ok(SelectorPart {
        widget,
        whitelist,
        blacklist,
    })
}
