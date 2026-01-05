use bevy::prelude::*;
use neko_maid::components::NekoUITree;
use neko_maid::parse::value::PropertyValue;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(neko_maid::NekoMaidPlugin)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, update_color)
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    let handle = asset_server.load("example.neko_ui");
    commands.spawn(NekoUITree::new(handle));
}

pub fn update_color(time: Res<Time>, mut q: Query<&mut NekoUITree>) {
    for mut root in &mut q {
        let h = (time.elapsed_secs_f64() % 4.0) / 4.0 * 360.0;
        let color = Color::hsl(h as f32, 0.5, 0.3);
        root.set_variable("color", PropertyValue::Color(color));

        let width = 400.0 + f64::sin(time.elapsed_secs_f64()) * 100.0;
        root.set_variable("width", PropertyValue::Number(width));
    }
}
