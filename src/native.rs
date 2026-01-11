//! A module that defines the native widgets.

use bevy::prelude::*;
use lazy_static::lazy_static;

use crate::parse::widget::NativeWidget;
use crate::render::spawn::{spawn_div, spawn_img, spawn_p, spawn_span};

lazy_static! {
    /// The list of native widgets available in NekoMaid UI.
    pub static ref NATIVE_WIDGETS: Vec<NativeWidget> = vec![
        NativeWidget {
            name: String::from("div"),
            spawn_func: spawn_div,
        },
        NativeWidget {
            name: String::from("img"),
            spawn_func: spawn_img,
        },
        NativeWidget {
            name: String::from("p"),
            spawn_func: spawn_p,
        },
        NativeWidget {
            name: String::from("span"),
            spawn_func: spawn_span,
        }
    ];
}
