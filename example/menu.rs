use bevy::prelude::*;
use neko_maid::components::NekoUITree;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(neko_maid::NekoMaidPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    let handle = asset_server.load("example.neko_ui");
    commands.spawn(NekoUITree::new(handle));
}
