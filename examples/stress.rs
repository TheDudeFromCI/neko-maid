use bevy::prelude::*;
use neko_maid::components::NekoUITree;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(neko_maid::NekoMaidPlugin)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, update_tree)
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    let handle = asset_server.load("stress.neko_ui");
    commands.spawn(NekoUITree::new(handle));
}

pub fn update_tree(q: Query<&mut NekoUITree>) {
    for mut root in q {
        root.mark_dirty();
    }
}
