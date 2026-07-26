use bevy::prelude::*;

mod camera;
mod cube;
mod light;

use camera::CameraPlugin;
use cube::Cube;
use light::LightPlugin;

pub struct GamePlugin;

pub fn close_on_escape(
    mut commands: Commands,
    windows: Query<(Entity, &Window)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in windows.iter() {
        if !focus.focused {
            continue;
        }

        if keyboard.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CameraPlugin, LightPlugin))
            .add_systems(Startup, Cube::spawn)
            .add_systems(Update, close_on_escape);
    }
}
