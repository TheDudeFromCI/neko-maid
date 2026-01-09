use bevy::prelude::*;
use neko_maid::components::{NekoUINode, NekoUITree};
use neko_maid::marker::{MarkerAppExt, NekoMarker};

#[derive(Component, NekoMarker)]
#[neko_marker("hovered")]
pub struct Hovered;

#[derive(Component, NekoMarker)]
#[neko_marker("pressed")]
pub struct Pressed;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(neko_maid::NekoMaidPlugin)
        .add_marker::<Pressed>()
        .add_marker::<Hovered>()
        .add_systems(Startup, setup)
        .add_observer(hover_start)
        .add_observer(hover_end)
        .add_observer(pressed)
        .add_observer(released)
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    let handle = asset_server.load("interactions.neko_ui");
    commands.spawn(NekoUITree::new(handle));
}

pub fn hover_start(event: On<Add, Hovered>, mut nodes: Query<&mut NekoUINode, With<Hovered>>) {
    let Ok(_) = nodes.get_mut(event.entity) else {
        return;
    };
    println!("node hovered");
}

pub fn hover_end(event: On<Remove, Hovered>, mut nodes: Query<&mut NekoUINode, With<Hovered>>) {
    let Ok(_) = nodes.get_mut(event.entity) else {
        return;
    };
    println!("node unhovered");
}

pub fn pressed(event: On<Add, Pressed>, mut nodes: Query<&mut NekoUINode, With<Pressed>>) {
    let Ok(_) = nodes.get_mut(event.entity) else {
        return;
    };
    println!("node pressed: ");
}

pub fn released(event: On<Remove, Pressed>, mut nodes: Query<&mut NekoUINode, With<Pressed>>) {
    let Ok(_) = nodes.get_mut(event.entity) else {
        return;
    };
    println!("node released");
}
