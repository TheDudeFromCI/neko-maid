//! Tests

use bevy::asset::AssetServer;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Res};
use bevy::platform::collections::HashSet;
use pretty_assertions::assert_eq;

use crate::parse::NekoMaidParser;
use crate::parse::element::NekoElement;
use crate::parse::style::{Selector, SelectorPart};
use crate::parse::widget::NativeWidget;

fn spawn_func(_: &Res<AssetServer>, _: &mut Commands, _: &NekoElement, _: Entity) -> Entity {
    Entity::PLACEHOLDER
}

fn native<S: Into<String>>(name: S) -> NativeWidget {
    NativeWidget {
        name: name.into(),
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
    with scrollview +active {
        with p +h1 {
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
        module.styles[0].selector,
        Selector {
            hierarchy: vec![
                SelectorPart {
                    widget: "div".into(),
                    whitelist: HashSet::new(),
                    blacklist: HashSet::new(),
                },
                SelectorPart {
                    widget: "div".into(),
                    whitelist: HashSet::from(["scrollview".into(), "active".into()]),
                    blacklist: HashSet::new(),
                },
                SelectorPart {
                    widget: "div".into(),
                    whitelist: HashSet::from(["content-pane".into()]),
                    blacklist: HashSet::new(),
                },
                SelectorPart {
                    widget: "p".into(),
                    whitelist: HashSet::from(["h1".into()]),
                    blacklist: HashSet::new(),
                },
            ]
        },
    );
}

#[test]
fn style_unrolling_slots() {
    const SOURCE: &str = r#"
def card {
    layout div {
        class card;

        with div {
            class card-header;
            output head;
        }

        with div {
            class card-body;
            output body;
        }
    }
}

style card {
    in body {
        with p +h3 {
            test: "Awesome Card";
        }
    }
}
    "#;

    let mut parse = NekoMaidParser::tokenize(SOURCE).unwrap();
    parse.register_native_widget(native("div"));
    parse.register_native_widget(native("p"));
    let module = parse.finish().unwrap();

    assert_eq!(
        module.styles[0].selector,
        Selector {
            hierarchy: vec![
                SelectorPart {
                    widget: "div".into(),
                    whitelist: HashSet::from(["card".into()]),
                    blacklist: HashSet::new(),
                },
                SelectorPart {
                    widget: "div".into(),
                    whitelist: HashSet::from(["card-body".into()]),
                    blacklist: HashSet::new(),
                },
                SelectorPart {
                    widget: "p".into(),
                    whitelist: HashSet::from(["h3".into()]),
                    blacklist: HashSet::new(),
                },
            ]
        },
    );
}
