//! Tests

use std::sync::Arc;

use bevy::asset::AssetServer;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Res};
use bevy::platform::collections::{HashMap, HashSet};
use pretty_assertions::assert_eq;

use crate::parse::NekoMaidParser;
use crate::parse::element::NekoElement;
use crate::parse::style::{Selector, SelectorPart, Style};
use crate::parse::widget::NativeWidget;

fn spawn_func(_: &Res<AssetServer>, _: &mut Commands, _: &NekoElement, _: Entity) -> Entity {
    Entity::PLACEHOLDER
}

fn native<S: Into<String>>(name: S) -> NativeWidget {
    NativeWidget {
        name: name.into(),
        default_properties: Arc::new(HashMap::new()),
        spawn_func,
    }
}

#[test]
fn style_unrolling() {
    const SOURCE: &str = r#"
    def scrollview {
        layout div {
            class scrollview;

            with div {
                class content-pane;

                scrollbar-width: 4px;
                output;
            }
        }
    }

    style div {
        with scrollview {
            with p {
                class h1;
                test: "Hello";
            }
        }
    }
    "#;

    let mut parse = NekoMaidParser::tokenize(SOURCE).unwrap();
    parse.register_native_widget(native("div"));
    parse.register_native_widget(native("p"));
    let module = parse.finish().unwrap();

    assert_eq!(
        module.styles,
        vec![Style {
            selector: Selector {
                hierarchy: vec![
                    SelectorPart {
                        widget: "div".into(),
                        whitelist: HashSet::new(),
                        blacklist: HashSet::new(),
                    },
                    SelectorPart {
                        widget: "div".into(),
                        whitelist: HashSet::from(["scrollview".into()]),
                        blacklist: HashSet::new(),
                    },
                    SelectorPart {
                        widget: "div".into(),
                        whitelist: HashSet::from(["scrollpane".into()]),
                        blacklist: HashSet::new(),
                    },
                    SelectorPart {
                        widget: "p".into(),
                        whitelist: HashSet::from(["h1".into()]),
                        blacklist: HashSet::new(),
                    },
                ]
            },
            properties: {
                let mut m = HashMap::new();
                m.insert("test".into(), "Hello".into());
                m
            }
        }]
    );
}
