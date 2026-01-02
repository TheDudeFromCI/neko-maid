//! A module for parsing NekoMaid UI widget definitions.

use std::sync::Arc;

use bevy::asset::AssetServer;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Res};
use bevy::platform::collections::{HashMap, HashSet};

use crate::parse::NekoMaidParseError;
use crate::parse::class::parse_class;
use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::element::NekoElement;
use crate::parse::property::{UnresolvedPropertyValue, parse_unresolved_property, parse_variable};
use crate::parse::token::TokenType;
use crate::parse::value::PropertyValue;

/// A NekoMaid UI widget definition.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Widget {
    /// A custom widget defined in NekoMaid UI.
    Custom(CustomWidget),

    /// A native widget provided by the NekoMaid UI system.
    Native(NativeWidget),
}

impl Widget {
    /// Gets the name of the widget.
    pub fn name(&self) -> &str {
        match self {
            Widget::Custom(custom) => &custom.name,
            Widget::Native(native) => &native.name,
        }
    }
}

/// A custom widget definition.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct CustomWidget {
    /// The name of the widget.
    pub name: String,

    /// The default properties of the widget.
    pub default_properties: HashMap<String, UnresolvedPropertyValue>,

    /// The layout of the widget.
    pub layout: WidgetLayout,
}

/// A native widget definition.
#[derive(Debug, Clone)]
pub struct NativeWidget {
    /// The name of the widget.
    pub name: String,

    /// The default properties of the widget.
    pub default_properties: Arc<HashMap<String, PropertyValue>>,

    /// The function used to spawn the widget.
    ///
    /// This function takes a mutable reference to `Commands` and the parent
    /// entity, and returns the spawned widget entity.
    pub spawn_func: fn(&Res<AssetServer>, &mut Commands, &NekoElement, Entity) -> Entity,
}

impl PartialEq<NativeWidget> for NativeWidget {
    fn eq(&self, other: &NativeWidget) -> bool {
        self.name == other.name
    }
}

/// Represents a layout for a widget definition in the UI.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct WidgetLayout {
    /// The widget type.
    pub widget: String,

    /// The properties of the layout.
    pub properties: HashMap<String, UnresolvedPropertyValue>,

    /// The child layouts.
    pub children: Vec<WidgetLayout>,

    /// The classes applied to this layout.
    pub classes: HashSet<String>,

    /// Whether this layout is an output slot.
    pub is_output: bool,
}

/// Parses a widget from the input and returns a [`Widget`].
pub(super) fn parse_widget(ctx: &mut ParseContext) -> NekoResult<Widget> {
    ctx.expect(TokenType::DefKeyword)?;

    let widget_position = ctx.next_position().unwrap_or_default();
    let name = ctx.expect_as_string(TokenType::Identifier)?;

    ctx.expect(TokenType::OpenBrace)?;

    let mut properties = HashMap::new();
    let mut layout = None;

    while let Some(next) = ctx.peek() {
        match next.token_type {
            TokenType::VarKeyword => {
                let property = parse_variable(ctx)?;
                properties.insert(property.name, property.value);
            }
            TokenType::LayoutKeyword => {
                if layout.is_some() {
                    return Err(NekoMaidParseError::MultipleLayoutsDefined {
                        position: next.position,
                    });
                }

                let parsed_layout = parse_widget_layout(ctx)?;
                let outputs = count_layout_outputs(&parsed_layout);

                if outputs == 0 {
                    return Err(NekoMaidParseError::LayoutHasNoOutput {
                        widget: name,
                        position: widget_position,
                    });
                } else if outputs > 1 {
                    return Err(NekoMaidParseError::MultipleWidgetOutputsDefined {
                        widget: name,
                        position: widget_position,
                    });
                }

                layout = Some(parsed_layout);
            }
            TokenType::CloseBrace => break,
            _ => {
                return Err(NekoMaidParseError::UnexpectedToken {
                    expected: vec![
                        TokenType::VarKeyword.type_name().to_string(),
                        TokenType::LayoutKeyword.type_name().to_string(),
                        TokenType::CloseBrace.type_name().to_string(),
                    ],
                    found: next.token_type.type_name().to_string(),
                    position: next.position,
                });
            }
        }
    }

    ctx.expect(TokenType::CloseBrace)?;

    let Some(layout) = layout else {
        return Err(NekoMaidParseError::IncompleteWidgetDefinition {
            widget: name,
            position: widget_position,
        });
    };

    Ok(Widget::Custom(CustomWidget {
        name,
        default_properties: properties,
        layout,
    }))
}

/// Parses a layout from the input and returns a [`Layout`].
pub(super) fn parse_widget_layout(ctx: &mut ParseContext) -> NekoResult<WidgetLayout> {
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

    let mut layout = WidgetLayout {
        widget: widget.clone(),
        properties: HashMap::new(),
        children: Vec::new(),
        classes: HashSet::new(),
        is_output: false,
    };

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
                let child_layout = parse_widget_layout(ctx)?;
                layout.children.push(child_layout);
            }
            TokenType::OutputKeyword => {
                ctx.expect(TokenType::OutputKeyword)?;
                ctx.expect(TokenType::Semicolon)?;
                layout.is_output = true;
            }
            TokenType::CloseBrace => break,
            _ => {
                return Err(NekoMaidParseError::UnexpectedToken {
                    expected: vec![
                        TokenType::Identifier.type_name().to_string(),
                        TokenType::ClassKeyword.type_name().to_string(),
                        TokenType::WithKeyword.type_name().to_string(),
                        TokenType::OutputKeyword.type_name().to_string(),
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

/// Counts the number of output slots in the given layout.
///
/// A layout *should* have only one output slot.
fn count_layout_outputs(layout: &WidgetLayout) -> u32 {
    let mut count = 0;

    if layout.is_output {
        count += 1;
    }

    for child in &layout.children {
        count += count_layout_outputs(child);
    }

    count
}
