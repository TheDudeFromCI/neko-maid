//! A module for defining the spawning functions for each native widget
//! and the default property values.

use bevy::prelude::*;

use crate::parse::element::NekoElement;

/// Spawns a `div` native widget.
pub(crate) fn spawn_div(
    _: &Res<AssetServer>,
    commands: &mut Commands,
    _: &NekoElement,
    parent: Entity,
) -> Entity {
    commands
        .spawn((
            ChildOf(parent),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
        ))
        .id()
}

/// Spawns an `img` native widget.
pub(crate) fn spawn_img(
    _: &Res<AssetServer>,
    commands: &mut Commands,
    _: &NekoElement,
    parent: Entity,
) -> Entity {
    commands
        .spawn((
            ChildOf(parent),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            ImageNode::default(),
        ))
        .id()
}

/// Spawns an `p` native widget.
pub(crate) fn spawn_p(
    _: &Res<AssetServer>,
    commands: &mut Commands,
    _: &NekoElement,
    parent: Entity,
) -> Entity {
    commands
        .spawn((
            ChildOf(parent),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            Text::default(),
            TextFont::default(),
            TextLayout::default(),
            TextColor::default(),
        ))
        .id()
}

/// Spawns an `span` native widget.
pub(crate) fn spawn_span(
    _: &Res<AssetServer>,
    commands: &mut Commands,
    _: &NekoElement,
    parent: Entity,
) -> Entity {
    commands
        .spawn((
            ChildOf(parent),
            Node::default(),
            BackgroundColor::default(),
            BorderColor::default(),
            BorderRadius::default(),
            TextSpan::default(),
            TextFont::default(),
            TextColor::default(),
        ))
        .id()
}
