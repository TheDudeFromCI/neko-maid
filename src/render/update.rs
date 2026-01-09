//! A module that defines the node update logic.

use bevy::prelude::*;

use crate::parse::element::NekoElementView;
use crate::parse::value::PropertyValue;

macro_rules! d {
    ($expr:expr) => {
        match $expr {
            Some(v) => v,
            None => Default::default(),
        }
    };
}

/// Partially updates the given components based on the current computed
/// properties.
pub fn update_node<'a>(
    asset_server: &Res<AssetServer>,
    mut element: NekoElementView<'a>,
    updated_properties: impl Iterator<Item = &'a String>,
    // node
    node: &mut Node,
    border_color: &mut BorderColor,
    border_radius: &mut BorderRadius,
    background_color: &mut BackgroundColor,
    // img
    image: &mut Option<&mut ImageNode>,
    // text
    text: &mut Option<&mut Text>,
    span: &mut Option<&mut TextSpan>,
    font: &mut Option<&mut TextFont>,
    color: &mut Option<&mut TextColor>,
    layout: &mut Option<&mut TextLayout>,
) {
    for property in updated_properties {
        // println!("Updating {property}");
        match property.as_str() {
            // --- node ---

            // basic layout
            "display" => node.display = d!(element.get_as("display")),
            "box-sizing" => node.box_sizing = d!(element.get_as("box-sizing")),
            "position-type" => node.position_type = d!(element.get_as("position-type")),
            // overflow
            "overflow-x" => node.overflow.x = d!(element.get_as("overflow-x")),
            "overflow-y" => node.overflow.y = d!(element.get_as("overflow-y")),
            "scrollbar-width" => node.scrollbar_width = d!(element.get_as("scrollbar-width")),
            "overflow-clip-margin-box" => {
                node.overflow_clip_margin.visual_box =
                    d!(element.get_as("overflow-clip-margin-box"))
            }
            "overflow-clip-margin" => {
                node.overflow_clip_margin.margin = d!(element.get_as("overflow-clip-margin"))
            }
            // positioning
            "left" => node.left = d!(element.get_as("left")),
            "top" => node.top = d!(element.get_as("top")),
            "right" => node.right = d!(element.get_as("right")),
            "bottom" => node.bottom = d!(element.get_as("bottom")),
            // sizing
            "width" => node.width = d!(element.get_as("width")),
            "height" => node.height = d!(element.get_as("height")),
            "min-width" => node.min_width = d!(element.get_as("min-width")),
            "min-height" => node.min_height = d!(element.get_as("min-height")),
            "max-width" => node.max_width = d!(element.get_as("max-width")),
            "max-height" => node.max_height = d!(element.get_as("max-height")),
            "aspect-ratio" => node.aspect_ratio = d!(element.get_as("aspect-ratio")),
            // alignment
            "align-items" => node.align_items = d!(element.get_as("align-items")),
            "justify-items" => node.justify_items = d!(element.get_as("justify-items")),
            "align-self" => node.align_self = d!(element.get_as("align-self")),
            "justify-self" => node.justify_self = d!(element.get_as("justify-self")),
            "align-content" => node.align_content = d!(element.get_as("align-content")),
            "justify-content" => node.justify_content = d!(element.get_as("justify-content")),
            // margin
            "margin-top" | "margin-left" | "margin-right" | "margin-bottom" | "margin" => {
                let margin = element.get_as("margin").unwrap_or(Val::Px(0.0));
                node.margin.top = element.get_as_or("margin-top", margin);
                node.margin.left = element.get_as_or("margin-left", margin);
                node.margin.right = element.get_as_or("margin-right", margin);
                node.margin.bottom = element.get_as_or("margin-bottom", margin);
            }
            // padding
            "padding-top" | "padding-left" | "padding-right" | "padding-bottom" | "padding" => {
                let padding = element.get_as("padding").unwrap_or(Val::Px(0.0));
                node.padding.top = element.get_as_or("padding-top", padding);
                node.padding.left = element.get_as_or("padding-left", padding);
                node.padding.right = element.get_as_or("padding-right", padding);
                node.padding.bottom = element.get_as_or("padding-bottom", padding);
            }
            // border
            "border-thickness-top"
            | "border-thickness-left"
            | "border-thickness-right"
            | "border-thickness-bottom"
            | "border-thickness" => {
                let border = element.get_as("border-thickness").unwrap_or(Val::Px(0.0));
                node.border.top = element.get_as_or("border-thickness-top", border);
                node.border.left = element.get_as_or("border-thickness-left", border);
                node.border.right = element.get_as_or("border-thickness-right", border);
                node.border.bottom = element.get_as_or("border-thickness-bottom", border);
            }
            // flex
            "flex-direction" => node.flex_direction = d!(element.get_as("flex-direction")),
            "flex-wrap" => node.flex_wrap = d!(element.get_as("flex-wrap")),
            "flex-grow" => node.flex_grow = d!(element.get_as("flex-grow")),
            "flex-shrink" => node.flex_shrink = d!(element.get_as("flex-shrink")),
            "flex-basis" => node.flex_basis = d!(element.get_as("flex-basis")),
            // gaps
            "row-gap" => node.row_gap = d!(element.get_as("row-gap")),
            "column-gap" => node.column_gap = d!(element.get_as("column-gap")),
            // grid
            "grid-auto-flow" => node.grid_auto_flow = d!(element.get_as("grid-auto-flow")),

            // --- border color ---
            "border-color-top"
            | "border-color-left"
            | "border-color-right"
            | "border-color-bottom"
            | "border-color" => {
                let color = element.get_as("border-color").unwrap_or(Color::NONE);
                border_color.top = element.get_as_or("border-color-top", color);
                border_color.left = element.get_as_or("border-color-left", color);
                border_color.right = element.get_as_or("border-color-right", color);
                border_color.bottom = element.get_as_or("border-color-bottom", color);
            }

            // --- border radius ---
            "border-radius-top-left"
            | "border-radius-top-right"
            | "border-radius-bottom-left"
            | "border-radius-bottom-right"
            | "border-radius" => {
                let radius = element.get_as("border-radius").unwrap_or(Val::Px(0.0));
                border_radius.top_left = element.get_as_or("border-radius-top-left", radius);
                border_radius.top_right = element.get_as_or("border-radius-top-right", radius);
                border_radius.bottom_left = element.get_as_or("border-radius-bottom-left", radius);
                border_radius.bottom_right = element.get_as_or("border-radius-bottom-right", radius)
            }
            // --- background color ---
            "background-color" => background_color.0 = element.get_as("background-color").unwrap_or(Color::NONE),
            "tint" => {
                if let Some(image) = image {
                    image.color = d!(element.get_as("tint"))
                }
            }

            // --- image ---
            "src" => {
                if let Some(image) = image {
                    image.image = asset_server.load(d!(element.get_as::<String>("src")));
                }
            }
            "flip-x" => {
                if let Some(image) = image {
                    image.flip_x = d!(element.get_as("flip-x"))
                }
            }
            "flip-y" => {
                if let Some(image) = image {
                    image.flip_y = d!(element.get_as("flip-y"))
                }
            }
            "mode"
            | "slice-size"
            | "slice-size-top"
            | "slice-size-left"
            | "slice-size-right"
            | "slice-size-bottom"
            | "center-scale-mode"
            | "center-scale-stretch"
            | "sides-scale-mode"
            | "sides-scale-stretch"
            | "max-corner-scale"
            | "tile-x"
            | "tile-y"
            | "stretch-value" => {
                if let Some(image) = image {
                    image.image_mode = match element.get_property("mode") {
                        Some(PropertyValue::String(s)) if s == "auto" => NodeImageMode::Auto,
                        Some(PropertyValue::String(s)) if s == "stretch" => NodeImageMode::Stretch,
                        Some(PropertyValue::String(s)) if s == "sliced" => {
                            let slice_size = element.get_as("slice-size").unwrap_or(0.0);

                            NodeImageMode::Sliced(TextureSlicer {
                                border: BorderRect {
                                    top: element.get_as_or("slice-size-top", slice_size),
                                    left: element.get_as_or("slice-size-left", slice_size),
                                    right: element.get_as_or("slice-size-right", slice_size),
                                    bottom: element.get_as_or("slice-size-bottom", slice_size),
                                },
                                center_scale_mode: match element.get_property("center-scale-mode") {
                                    Some(PropertyValue::String(s)) if s == "stretch" => {
                                        SliceScaleMode::Stretch
                                    }
                                    Some(PropertyValue::String(s)) if s == "tile" => {
                                        SliceScaleMode::Tile {
                                            stretch_value: element
                                                .get_as("center-scale-stretch")
                                                .unwrap_or(1.0),
                                        }
                                    }
                                    Some(property) => {
                                        warn!(
                                            "Failed to convert PropertyValue {} to SliceScaleMode",
                                            property
                                        );
                                        SliceScaleMode::default()
                                    }
                                    None => SliceScaleMode::default(),
                                },

                                sides_scale_mode: match element.get_property("sides-scale-mode") {
                                    Some(PropertyValue::String(s)) if s == "stretch" => {
                                        SliceScaleMode::Stretch
                                    }
                                    Some(PropertyValue::String(s)) if s == "tile" => {
                                        SliceScaleMode::Tile {
                                            stretch_value: element
                                                .get_as("sides-scale-stretch")
                                                .unwrap_or(1.0),
                                        }
                                    }
                                    Some(property) => {
                                        warn!(
                                            "Failed to convert PropertyValue {} to SliceScaleMode",
                                            property
                                        );
                                        SliceScaleMode::default()
                                    }
                                    None => SliceScaleMode::default(),
                                },

                                max_corner_scale: element.get_as("max-corner-scale").unwrap_or(1.0),
                            })
                        }
                        Some(PropertyValue::String(s)) if s == "tiled" => NodeImageMode::Tiled {
                            tile_x: element.get_as("tile-x").unwrap_or(true),
                            tile_y: element.get_as("tile-y").unwrap_or(true),
                            stretch_value: element.get_as("stretch-value").unwrap_or(1.0),
                        },
                        Some(property) => {
                            warn!(
                                "Failed to convert PropertyValue {} to NodeImageMode",
                                property
                            );
                            NodeImageMode::default()
                        }
                        None => NodeImageMode::default(),
                    };
                }
            }

            // --- text ---

            // text content
            "text" => {
                if let Some(text) = text {
                    text.0 = d!(element.get_as("text"));
                } else if let Some(span) = span {
                    span.0 = d!(element.get_as("text"));
                }
            }
            // font
            "font" => {
                if let Some(font) = font {
                    let font_path: String = d!(element.get_as("font"));
                    font.font = match font_path.as_str() {
                        "auto" => Handle::<Font>::default(),
                        _ => asset_server.load(font_path),
                    };
                }
            }
            "font-size" => {
                if let Some(font) = font {
                    font.font_size = d!(element.get_as("font-size"))
                }
            }
            "line-height" => {
                if let Some(font) = font {
                    font.line_height = d!(element.get_as("line-height"))
                }
            }
            "font-smoothing" => {
                if let Some(font) = font {
                    font.font_smoothing = d!(element.get_as("font-smoothing"))
                }
            }
            // layout (Text only
            "justify" | "line-break" => {
                if let Some(layout) = layout {
                    match property.as_str() {
                        "justify" => layout.justify = d!(element.get_as("justify")),
                        "line-break" => layout.linebreak = d!(element.get_as("line-break")),
                        _ => {}
                    }
                }
            }
            // color
            "color" => {
                if let Some(color) = color {
                    color.0 = d!(element.get_as("color"))
                }
            }

            _ => {}
        }
    }
}
