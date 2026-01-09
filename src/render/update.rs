//! A module that defines the node update logic.

use bevy::image::TRANSPARENT_IMAGE_HANDLE;
use bevy::prelude::*;

use crate::parse::element::NekoElementView;
use crate::parse::value::PropertyValue;

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
            "display" => node.display = element.get_as("display").unwrap_or_default(),
            "box-sizing" => node.box_sizing = element.get_as("box-sizing").unwrap_or_default(),
            "position-type" => {
                node.position_type = element.get_as("position-type").unwrap_or_default()
            }
            // overflow
            "overflow-x" => node.overflow.x = element.get_as("overflow-x").unwrap_or_default(),
            "overflow-y" => node.overflow.y = element.get_as("overflow-y").unwrap_or_default(),
            "scrollbar-width" => {
                node.scrollbar_width = element.get_as("scrollbar-width").unwrap_or_default()
            }
            "overflow-clip-margin-box" => {
                node.overflow_clip_margin.visual_box = element
                    .get_as("overflow-clip-margin-box")
                    .unwrap_or_default()
            }
            "overflow-clip-margin" => {
                node.overflow_clip_margin.margin =
                    element.get_as("overflow-clip-margin").unwrap_or_default()
            }
            // positioning
            "left" => node.left = element.get_as("left").unwrap_or_default(),
            "top" => node.top = element.get_as("top").unwrap_or_default(),
            "right" => node.right = element.get_as("right").unwrap_or_default(),
            "bottom" => node.bottom = element.get_as("bottom").unwrap_or_default(),
            // sizing
            "width" => node.width = element.get_as("width").unwrap_or_default(),
            "height" => node.height = element.get_as("height").unwrap_or_default(),
            "min-width" => node.min_width = element.get_as("min-width").unwrap_or_default(),
            "min-height" => node.min_height = element.get_as("min-height").unwrap_or_default(),
            "max-width" => node.max_width = element.get_as("max-width").unwrap_or_default(),
            "max-height" => node.max_height = element.get_as("max-height").unwrap_or_default(),
            "aspect-ratio" => {
                node.aspect_ratio = element.get_as("aspect-ratio").unwrap_or_default()
            }
            // alignment
            "align-items" => node.align_items = element.get_as("align-items").unwrap_or_default(),
            "justify-items" => {
                node.justify_items = element.get_as("justify-items").unwrap_or_default()
            }
            "align-self" => node.align_self = element.get_as("align-self").unwrap_or_default(),
            "justify-self" => {
                node.justify_self = element.get_as("justify-self").unwrap_or_default()
            }
            "align-content" => {
                node.align_content = element.get_as("align-content").unwrap_or_default()
            }
            "justify-content" => {
                node.justify_content = element.get_as("justify-content").unwrap_or_default()
            }
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
            "flex-direction" => {
                node.flex_direction = element.get_as("flex-direction").unwrap_or_default()
            }
            "flex-wrap" => node.flex_wrap = element.get_as("flex-wrap").unwrap_or_default(),
            "flex-grow" => node.flex_grow = element.get_as("flex-grow").unwrap_or_default(),
            "flex-shrink" => node.flex_shrink = element.get_as("flex-shrink").unwrap_or(1.0),
            "flex-basis" => node.flex_basis = element.get_as("flex-basis").unwrap_or_default(),
            // gaps
            "row-gap" => node.row_gap = element.get_as("row-gap").unwrap_or_default(),
            "column-gap" => node.column_gap = element.get_as("column-gap").unwrap_or_default(),
            // grid
            "grid-auto-flow" => {
                node.grid_auto_flow = element.get_as("grid-auto-flow").unwrap_or_default()
            }

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
            "background-color" => {
                background_color.0 = element.get_as("background-color").unwrap_or(Color::NONE)
            }
            "tint" => {
                if let Some(image) = image {
                    image.color = element.get_as("tint").unwrap_or(Color::WHITE)
                }
            }

            // --- image ---
            "src" => {
                if let Some(image) = image {
                    image.image = if let Some(src) = element.get_as::<String>("src") {
                        asset_server.load(src)
                    } else {
                        TRANSPARENT_IMAGE_HANDLE
                    }
                }
            }
            "flip-x" => {
                if let Some(image) = image {
                    image.flip_x = element.get_as("flip-x").unwrap_or_default()
                }
            }
            "flip-y" => {
                if let Some(image) = image {
                    image.flip_y = element.get_as("flip-y").unwrap_or_default()
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
                    text.0 = element.get_as("text").unwrap_or_default();
                } else if let Some(span) = span {
                    span.0 = element.get_as("text").unwrap_or_default();
                }
            }
            // font
            "font" => {
                if let Some(font) = font {
                    let font_path: String = element.get_as("font").unwrap_or_default();
                    font.font = match font_path.as_str() {
                        "auto" => Handle::<Font>::default(),
                        _ => asset_server.load(font_path),
                    };
                }
            }
            "font-size" => {
                if let Some(font) = font {
                    font.font_size = element.get_as("font-size").unwrap_or(20.0)
                }
            }
            "line-height" => {
                if let Some(font) = font {
                    font.line_height = element.get_as("line-height").unwrap_or_default()
                }
            }
            "font-smoothing" => {
                if let Some(font) = font {
                    font.font_smoothing = element.get_as("font-smoothing").unwrap_or_default()
                }
            }
            // layout (Text only
            "justify" | "line-break" => {
                if let Some(layout) = layout {
                    match property.as_str() {
                        "justify" => layout.justify = element.get_as("justify").unwrap_or_default(),
                        "line-break" => {
                            layout.linebreak = element.get_as("line-break").unwrap_or_default()
                        }
                        _ => {}
                    }
                }
            }
            // color
            "color" => {
                if let Some(color) = color {
                    color.0 = element.get_as("color").unwrap_or(Color::WHITE)
                }
            }

            _ => {}
        }
    }
}
