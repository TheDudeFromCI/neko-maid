//! Defines the native widgets available in NekoMaid UI.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lazy_static::lazy_static;

use crate::parse::element::NekoElement;
use crate::parse::property::PropertyValue;
use crate::parse::widget::NativeWidget;

lazy_static! {
    /// The list of native widgets available in NekoMaid UI.
    pub static ref NATIVE_WIDGETS: Vec<NativeWidget> = vec![NativeWidget {
        name: String::from("div"),
        properties: {
            let mut m = HashMap::new();
            m.insert("width".into(), "auto".into());
            m.insert("height".into(), "auto".into());
            m.insert("background-color".into(), Color::NONE.into());

            m.insert("border-color".into(), Color::NONE.into());
            m.insert("border-color-top".into(), Color::NONE.into());
            m.insert("border-color-left".into(), Color::NONE.into());
            m.insert("border-color-right".into(), Color::NONE.into());
            m.insert("border-color-bottom".into(), Color::NONE.into());

            m.insert("border-thickness".into(), Color::NONE.into());
            m.insert("border-thickness-top".into(), Color::NONE.into());
            m.insert("border-thickness-left".into(), Color::NONE.into());
            m.insert("border-thickness-right".into(), Color::NONE.into());
            m.insert("border-thickness-bottom".into(), Color::NONE.into());
            m
        },
        spawn_func: spawn_div,
    }];
}

/// Spawns a `div` native widget.
fn spawn_div(commands: &mut Commands, parent: Entity, element: &NekoElement) -> Entity {
    commands
        .spawn((
            ChildOf(parent),
            node_bundle(element),
            background_color_bundle(element),
            border_color_bundle(element),
        ))
        .id()
}

/// Build [`Node`] bundle
fn node_bundle(element: &NekoElement) -> Node {
    let width = element.get_property("width").map_or(Val::Auto, as_val);
    let height = element.get_property("height").map_or(Val::Auto, as_val);

    let border_thickness = element
        .get_property("border-thickness")
        .map_or(Val::Auto, as_val);
    let border_thickness_top = element
        .get_property("border-thickness-top")
        .map_or(border_thickness, as_val);
    let border_thickness_left = element
        .get_property("border-thickness-left")
        .map_or(border_thickness, as_val);
    let border_thickness_right = element
        .get_property("border-thickness-right")
        .map_or(border_thickness, as_val);
    let border_thickness_bottom = element
        .get_property("border-thickness-bottom")
        .map_or(border_thickness, as_val);

    Node {
        width,
        height,
        border: UiRect {
            top: border_thickness_top,
            left: border_thickness_left,
            right: border_thickness_right,
            bottom: border_thickness_bottom,
        },
        ..default()
    }
}

/// Build [`BorderColor`] bundle
fn border_color_bundle(element: &NekoElement) -> BorderColor {
    let border_color = element
        .get_property("border-color")
        .map_or(Color::NONE, as_color);
    let border_color_top = element
        .get_property("border-color-top")
        .map_or(border_color, as_color);
    let border_color_left = element
        .get_property("border-color-left")
        .map_or(border_color, as_color);
    let border_color_right = element
        .get_property("border-color-right")
        .map_or(border_color, as_color);
    let border_color_bottom = element
        .get_property("border-color-bottom")
        .map_or(border_color, as_color);

    BorderColor {
        top: border_color_top,
        left: border_color_left,
        right: border_color_right,
        bottom: border_color_bottom,
    }
}

/// Build [`BackgroundColor`] bundle
fn background_color_bundle(element: &NekoElement) -> BackgroundColor {
    let bg_color = element
        .get_property("background-color")
        .map_or(Color::NONE, as_color);

    BackgroundColor(bg_color)
}

/// Converts a [`PropertyValue`] to a Bevy UI [`Val`].
fn as_val(property: &PropertyValue) -> Val {
    match property {
        PropertyValue::String(s) if s == "auto" => Val::Auto,
        PropertyValue::Pixels(n) => Val::Px(*n as f32),
        PropertyValue::Percent(n) => Val::Percent(*n as f32),
        _ => {
            warn_once!("Failed to convert PropertyValue {:?} to Val", property);
            Val::Auto
        }
    }
}

/// Converts a [`PropertyValue`] to a Bevy UI [`Color`].
fn as_color(property: &PropertyValue) -> Color {
    match property {
        PropertyValue::Color(c) => *c,
        _ => {
            warn_once!("Failed to convert PropertyValue {:?} to Color", property);
            Color::NONE
        }
    }
}
