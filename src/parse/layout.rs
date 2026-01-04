//! Defines the layout structure and parsing logic for NekoMaid UI files.

use bevy::platform::collections::{HashMap, HashSet};
use lazy_static::lazy_static;

use crate::parse::NekoMaidParseError;
use crate::parse::class::parse_class;
use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::property::{UnresolvedPropertyValue, parse_unresolved_property};
use crate::parse::token::{TokenType, TokenValue};

/// A slot in a layout.
#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    /// The name of this slot.
    pub name: String,
    /// The name of the input slot this slot is contained by.
    pub location: String,
    /// The index in `location` this slot is positioned.
    pub index: usize,
}

lazy_static! {
    static ref EMPTY_CHILDREN: Vec<Layout> = Vec::new();
}

/// Represents a layout in the UI.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Layout {
    /// The widget type.
    pub(super) widget: String,

    /// The properties of the layout.
    pub(super) properties: HashMap<String, UnresolvedPropertyValue>,

    /// The children by input slot. Each key should be a
    /// valid slot in the widget's layout.
    pub(super) children_slots: HashMap<String, Vec<Layout>>,

    /// The classes applied to this layout.
    pub classes: HashSet<String>,

    /// The slots of this layout.
    pub slots: Vec<Slot>,
}

impl Layout {
    /// Create a new layout.
    pub fn new(widget: String) -> Self {
        Self {
            widget,
            properties: HashMap::new(),
            children_slots: HashMap::new(),
            classes: HashSet::new(),
            slots: vec![],
        }
    }

    /// Mutably gets or creates an input slot with the given name.
    pub fn get_slot_mut(&mut self, name: String) -> &mut Vec<Layout> {
        self.children_slots.entry(name).or_default()
    }

    /// Gets the input slot with the given name. If the slot does not exist,
    /// a default empty vector is returned.
    pub fn get_slot(&self, name: &str) -> &Vec<Layout> {
        self.children_slots.get(name).unwrap_or(&EMPTY_CHILDREN)
    }
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

    let mut layout = Layout::new(widget.clone());

    ctx.expect(TokenType::OpenBrace)?;

    while let Some(next) = ctx.peek() {
        match next.token_type {
            TokenType::Identifier => {
                let property = parse_unresolved_property(ctx)?;
                layout.properties.insert(property.name, property.value);
            }
            TokenType::ClassKeyword => {
                let class = parse_class(ctx)?;
                layout.classes.insert(class);
            }
            TokenType::WithKeyword => {
                let child_layout = parse_layout(ctx)?;
                let children = layout.get_slot_mut("default".to_string());
                children.push(child_layout);
            }
            TokenType::OutputKeyword => {
                let name = parse_slot(ctx)?;
                layout.slots.push(Slot {
                    name,
                    location: "default".to_string(),
                    index: layout.get_slot("default").len(),
                });
            }
            TokenType::InKeyword => {
                // FIX this does not to ignore whitespace
                let in_position = ctx.next_position().unwrap_or_default();
                let InStatement {
                    slot_name,
                    children,
                    slots,
                } = parse_in(ctx)?;

                if layout.children_slots.contains_key(&slot_name) {
                    // error, cannot define slot twice
                    return Err(NekoMaidParseError::InputSlotProvidedTwice {
                        slot: slot_name,
                        position: in_position,
                    });
                }
                layout.children_slots.insert(slot_name, children);
                layout.slots.extend(slots);
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

/// Parses a slot statement.
pub(super) fn parse_slot(ctx: &mut ParseContext) -> NekoResult<String> {
    ctx.expect(TokenType::OutputKeyword)?;

    let name = ctx
        .maybe_consume(TokenType::Identifier)
        .and_then(|t| match t { TokenValue::String(s) => Some(s), _ => None })
        .unwrap_or("default".to_string());

    ctx.expect(TokenType::Semicolon)?;

    Ok(name)
}

/// A parsed in statement.
pub(super) struct InStatement {
    /// The input slot this statement refers to.
    pub slot_name: String,
    /// The children nodes contained by the slot.
    pub children: Vec<Layout>,
    /// The output slots.
    pub slots: Vec<Slot>,
}

/// Parses an `in` statement.
pub(super) fn parse_in(ctx: &mut ParseContext) -> NekoResult<InStatement> {
    ctx.expect(TokenType::InKeyword)?;

    let slot_name = ctx.expect_as_string(TokenType::Identifier)?;

    ctx.expect(TokenType::OpenBrace)?;

    let mut children = vec![];
    let mut slots = vec![];

    while let Some(next) = ctx.peek() {
        match next.token_type {
            TokenType::WithKeyword => {
                let child_layout = parse_layout(ctx)?;
                children.push(child_layout);
            }
            TokenType::OutputKeyword => {
                let name = parse_slot(ctx)?;
                slots.push(Slot {
                    name,
                    location: slot_name.clone(),
                    index: children.len(),
                });
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

    Ok(InStatement {
        slot_name,
        children,
        slots,
    })
}
