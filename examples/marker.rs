use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use neko_derive::NekoMarker;
use neko_maid::components::NekoUITree;
use neko_maid::marker::{MarkerAppExt, NekoMarker};

#[derive(Component, NekoMarker)]
#[neko_marker("test")]
pub struct Test;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(neko_maid::NekoMaidPlugin)
        .add_marker::<Test>()
        .add_systems(Startup, setup)
        .add_observer(spawned_test)
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    let handle = asset_server.load("marker.neko_ui");
    commands.spawn(NekoUITree::new(handle));
}

pub fn spawned_test(event: On<Add, Test>, mut cmds: Commands) {
    // Could add any arbitrary logic here. We're gonna just build some UI manually.

    println!("Spawned test {}", event.entity);
    cmds.entity(event.entity).with_children(|parent| {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                right: Val::Px(0.0),
                width: Val::Auto,
                height: Val::Auto,
                border: UiRect::all(Val::Px(2.0)),
                ..Default::default()
            },
            BorderColor::all(RED),
            BackgroundColor(Color::NONE),
        ));
    });
}
