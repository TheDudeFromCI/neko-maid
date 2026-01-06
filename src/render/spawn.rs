//! A module for defining the spawning functions for each native widget
//! and the default property values.

use bevy::prelude::*;
use bevy::text::{FontSmoothing, LineHeight};

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
            default_node(),
            default_background_color(),
            default_border_color(),
            default_border_radius(),
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
            default_node(),
            default_background_color(),
            default_border_color(),
            default_border_radius(),
            default_image(),
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
            default_node(),
            default_background_color(),
            default_border_color(),
            default_border_radius(),
            default_text(),
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
            default_node(),
            default_background_color(),
            default_border_color(),
            default_border_radius(),
            default_text_span(),
        ))
        .id()
}

fn default_node() -> Node {
    Node {
        // layout
        display: Display::Flex,
        box_sizing: BoxSizing::BorderBox,
        position_type: PositionType::Relative,
        // overflow
        overflow: Overflow {
            x: OverflowAxis::Visible,
            y: OverflowAxis::Visible,
        },
        scrollbar_width: 0.0,
        overflow_clip_margin: OverflowClipMargin {
            visual_box: OverflowClipBox::PaddingBox,
            margin: 0.0,
        },
        // positioning
        left: Val::Auto,
        top: Val::Auto,
        right: Val::Auto,
        bottom: Val::Auto,
        // sizing
        width: Val::Auto,
        height: Val::Auto,
        min_width: Val::Auto,
        min_height: Val::Auto,
        max_width: Val::Auto,
        max_height: Val::Auto,
        aspect_ratio: None,
        // alignment
        align_items: AlignItems::Default,
        justify_items: JustifyItems::Default,
        align_self: AlignSelf::Auto,
        justify_self: JustifySelf::Auto,
        align_content: AlignContent::Default,
        justify_content: JustifyContent::Default,
        // spacing
        margin: UiRect::all(Val::Px(0.0)),
        padding: UiRect::all(Val::Px(0.0)),
        border: UiRect::all(Val::Px(0.0)),
        // flex
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        flex_basis: Val::Auto,
        // gaps
        row_gap: Val::Px(0.0),
        column_gap: Val::Px(0.0),
        // grid
        grid_auto_flow: GridAutoFlow::Row,
        ..Default::default()
    }
}

fn default_background_color() -> BackgroundColor {
    BackgroundColor(Color::NONE)
}

fn default_border_color() -> BorderColor {
    BorderColor {
        top: Color::NONE,
        left: Color::NONE,
        right: Color::NONE,
        bottom: Color::NONE,
    }
}

fn default_border_radius() -> BorderRadius {
    BorderRadius {
        top_left: Val::Px(0.0),
        top_right: Val::Px(0.0),
        bottom_left: Val::Px(0.0),
        bottom_right: Val::Px(0.0),
    }
}

fn default_image() -> ImageNode {
    ImageNode {
        image: Handle::default(),
        color: Color::WHITE,
        flip_x: false,
        flip_y: false,
        image_mode: NodeImageMode::Auto,
        ..Default::default()
    }
}

fn default_text() -> (Text, TextFont, TextLayout, TextColor) {
    (
        Text(String::new()),
        TextFont {
            font: Handle::<Font>::default(), // "auto"
            font_size: 16.0,
            line_height: LineHeight::RelativeToFont(1.2),
            font_smoothing: FontSmoothing::AntiAliased,
        },
        TextLayout {
            justify: Justify::Left,
            linebreak: LineBreak::WordBoundary,
        },
        TextColor(Color::WHITE),
    )
}

fn default_text_span() -> (TextSpan, TextFont, TextColor) {
    (
        TextSpan(String::new()),
        TextFont {
            font: Handle::<Font>::default(), // "auto"
            font_size: 16.0,
            line_height: LineHeight::Px(120.0),
            font_smoothing: FontSmoothing::AntiAliased,
        },
        TextColor(Color::WHITE),
    )
}
