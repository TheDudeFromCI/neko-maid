use std::collections::VecDeque;

use bevy::color::palettes::css::WHITE;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use neko_maid::components::NekoUITree;
use neko_maid::parse::value::PropertyValue;



#[derive(Resource, Clone)]
pub struct FpsSettings {
    pub startup_visibility: Visibility,
}
impl Default for FpsSettings {
    fn default() -> Self {
        Self {
            startup_visibility: Visibility::Hidden,
        }
    }
}

/// Marker to find the container entity so we can show/hide the FPS counter
#[derive(Component)]
struct FpsRoot;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct FpsText;

#[derive(Resource, Default)]
pub struct FpsHistory {
    last_update: f64,
    values: VecDeque<f64>,
    sum: f64,
}
impl FpsHistory {
    pub fn push(&mut self, value: f64) {
        self.sum += value;
        self.values.push_back(value);

        while self.values.len() > 10 {
            let Some(v) = self.values.pop_front() else { break };
            self.sum -= v;
        }
    }

    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        Some(self.sum / self.values.len() as f64)
    }
}

fn setup_fps_counter(settings: Res<FpsSettings>, mut commands: Commands) {
    let font = TextFont {
        font_size: 20.0,
        ..Default::default()
    };
    let color = TextColor(WHITE.into());

    commands.spawn((
        FpsRoot,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(1.),
            top: Val::Percent(1.),
            bottom: Val::Auto,
            left: Val::Auto,
            padding: UiRect::all(Val::Px(4.0)),
            ..Default::default()
        },
        settings.startup_visibility,
        ZIndex(i32::MAX),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
        children![
            (Text("FPS:".to_string()), font.clone(), color.clone()),
            (FpsText, Text("N/A".to_string()), font, color),
        ],
    ));
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut history: ResMut<FpsHistory>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    let t = time.elapsed_secs_f64();
    if (t - history.last_update) < 0.1 { return }
    history.last_update = t;

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed());

    if let Some(value) = fps {
        history.push(value);
    }

    let mean_fps = history.mean();

    for mut text in &mut query {
        if let Some(mean) = mean_fps {
            text.0 = format!("{mean:>4.0}");
        } else {
            text.0 = "N/A".into();
        }
    }
}

fn fps_counter_showhide(
    mut q: Query<&mut Visibility, With<FpsRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F3) {
        let Ok(mut vis) = q.single_mut() else { return };
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}

#[derive(Default)]
pub struct FpsCounter {
    settings: FpsSettings,
}
impl FpsCounter {
    pub fn set_visibility(mut self, visibility: Visibility) -> Self {
        self.settings.startup_visibility = visibility;
        self
    }
}
impl Plugin for FpsCounter {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .insert_resource(self.settings.clone())
            .init_resource::<FpsHistory>()
            .add_systems(Startup, setup_fps_counter)
            .add_systems(Update, (fps_text_update_system, fps_counter_showhide));
    }
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(neko_maid::NekoMaidPlugin)
        .add_plugins(FpsCounter::default().set_visibility(Visibility::Visible))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, update_animation)
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    let handle = asset_server.load("animated.neko_ui");
    commands.spawn(NekoUITree::new(handle));
}

pub fn update_animation(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut q: Query<&mut NekoUITree>,
) {
    if mouse.pressed(MouseButton::Left) { return }

    let h = (time.elapsed_secs_f64() % 4.0) / 4.0 * 360.0;
    let color = Color::hsl(h as f32, 0.5, 0.3);
    
    let width = 20.0 + f64::sin(time.elapsed_secs_f64() * 5.0) * 5.0;

    for mut root in &mut q {
        root.set_variable("random-color", PropertyValue::Color(color));
        root.set_variable("random-num", PropertyValue::Number(width));
    }
}
