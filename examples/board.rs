use bevy::prelude::*;
use neko_maid::components::{NekoUINode, NekoUITree};
use neko_maid::marker::{MarkerAppExt, NekoMarker};

#[derive(Component, NekoMarker)]
#[neko_marker("pressed")]
pub struct Pressed;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(neko_maid::NekoMaidPlugin)
        .add_systems(Startup, setup)
        .add_marker::<Pressed>()
        .add_observer(toggle_cell)
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    let handle = asset_server.load("board.neko_ui");
    commands.spawn(NekoUITree::new(handle));
}

pub fn toggle_cell(event: On<Remove, Pressed>, mut nodes: Query<&mut NekoUINode, With<Pressed>>) {
    let Ok(mut node) = nodes.get_mut(event.entity) else {
        return;
    };
    node.toggle_class("active");
}
