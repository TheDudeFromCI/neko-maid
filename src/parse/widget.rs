//! A module for parsing NekoMaid UI widget definitions.

use std::sync::Arc;

use bevy::asset::AssetServer;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Res};
use bevy::platform::collections::{HashMap, HashSet};

use crate::parse::NekoMaidParseError;
use crate::parse::context::{NekoResult, ParseContext};
use crate::parse::element::NekoElement;
use crate::parse::layout::{Layout, parse_layout};
use crate::parse::property::{UnresolvedPropertyValue, parse_variable};
use crate::parse::token::{TokenPosition, TokenType};
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
    pub layout: Layout,
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

/// Parses a widget from the input and returns a [`Widget`].
pub(super) fn parse_widget(ctx: &mut ParseContext) -> NekoResult<Widget> {
    ctx.expect(TokenType::DefKeyword)?;

    let widget_position = ctx.next_position().unwrap_or_default();
    let name = ctx.expect_as_string(TokenType::Identifier)?;
    ctx.set_current_widget(Some(name.clone()));

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

                let parsed_layout = parse_layout(ctx)?;
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

    validate_layout_slots(&layout, &name, &widget_position)?;

    ctx.set_current_widget(None);

    Ok(Widget::Custom(CustomWidget {
        name,
        default_properties: properties,
        layout,
    }))
}

/// Validates if layout does not contain duplicated slots and
/// contains at least one slot.
pub(super) fn validate_layout_slots(
    layout: &Layout,
    widget: &String,
    position: &TokenPosition,
) -> NekoResult<()> {
    fn f(
        l: &Layout,
        slots: &mut HashSet<String>,
        widget: &String,
        position: &TokenPosition,
    ) -> NekoResult<()> {
        for s in &l.slots {
            if slots.contains(&s.name) {
                return Err(NekoMaidParseError::LayoutWithDuplicatedOutputs {
                    widget: widget.clone(),
                    name: s.name.clone(),
                    position: position.clone(),
                });
            }
            slots.insert(s.name.clone());
        }

        for children in l.children_slots.values() {
            for c in children {
                f(c, slots, widget, position)?;
            }
        }

        Ok(())
    }

    let mut slots = HashSet::new();
    f(layout, &mut slots, widget, position)?;

    if slots.is_empty() {
        return Err(NekoMaidParseError::LayoutHasNoOutput {
            widget: widget.clone(),
            position: position.clone(),
        });
    }

    Ok(())
}
