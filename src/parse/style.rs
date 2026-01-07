//! A parser for NekoMaid UI style definitions.

use bevy::platform::collections::HashSet;

use crate::parse::NekoMaidParseError;
use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::layout::Layout;
use crate::parse::property::parse_unresolved_property;
use crate::parse::scope::ScopeId;
use crate::parse::token::TokenType;
use crate::parse::widget::Widget;

/// A NekoMaid UI style definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    /// The selector for the style.
    pub(crate) selector: Selector,

    /// The id of the scope containing the properties of this style.
    pub(crate) scope_id: ScopeId
}

impl Style {
    /// Creates a new Style with the given selector and properties.
    pub(crate) fn new(
        selector: Selector, scope_id: ScopeId
    ) -> Self {
        Self {
            selector,
            scope_id,
        }
    }

    /// Returns a reference to the selector of this style.
    pub fn selector(&self) -> &Selector {
        &self.selector
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

    let widget_position = ctx.next_position().unwrap_or_default();
    let widget = ctx.expect_as_string(TokenType::Identifier)?;

    let (whitelist, blacklist) = parse_style_selector(ctx)?;

    let Some(w) = ctx.get_widget(&widget) else {
        return Err(NekoMaidParseError::UnknownWidget {
            widget,
            position: widget_position,
        });
    };

    if let Widget::Custom(custom_widget) = w {
        let selector_index = selector.hierarchy.len();
        unroll_widget(&custom_widget.layout, "default", &mut selector);

        selector.hierarchy[selector_index]
            .whitelist
            .extend(whitelist);
        selector.hierarchy[selector_index]
            .blacklist
            .extend(blacklist);
    } else {
        selector.hierarchy.push(SelectorPart {
            widget,
            whitelist,
            blacklist,
        });
    }
    
    ctx.expect(TokenType::OpenBrace)?;

    let mut properties = vec![];

    while let Some(next) = ctx.peek() {
        match next.token_type {
            TokenType::Identifier => {
                let property = parse_unresolved_property(ctx)?;
                properties.push((property.name, property.value));
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

    
    if !properties.is_empty() {
        let scope = ctx.create_scope(ScopeId(0));
        scope.add_properties(properties.iter().map(|(k, v)| (k, v)));
        let scope_id = scope.id();
        ctx.add_style(Style::new(selector, scope_id));
    }

    Ok(())
}

/// Parses a style selector part from the input and returns a [`SelectorPart`].
pub(super) fn parse_style_selector(
    ctx: &mut ParseContext,
) -> NekoResult<(HashSet<String>, HashSet<String>)> {
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

    Ok((whitelist, blacklist))
}

/// Unrolls a custom widget's layout into selector parts.
fn unroll_widget(layout: &Layout, slot: &str, selector: &mut Selector) {
    selector.hierarchy.push(SelectorPart {
        widget: layout.widget.clone(),
        whitelist: layout.classes.clone(),
        blacklist: HashSet::new(),
    });

    for child in layout.get_slot(slot) {
        unroll_widget(child, "default", selector);
    }
}
