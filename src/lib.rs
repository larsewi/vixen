use avian3d::prelude::*;
use bevy::prelude::*;

mod block;
mod camera;
mod chunk;
mod light;
mod mesher;
mod player;
mod terrain;
mod world;

use camera::CameraPlugin;
use light::LightPlugin;
use player::PlayerPlugin;
use world::WorldPlugin;

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
        app.add_plugins((
            // Avian steps physics in `FixedPostUpdate` at a fixed timestep. Rendering
            // is smoothed by interpolation, opted into per-entity with a
            // `TranslationInterpolation` component (see `Player::spawn`).
            PhysicsPlugins::default(),
            PlayerPlugin,
            CameraPlugin,
            LightPlugin,
            WorldPlugin,
        ))
        .add_systems(Update, close_on_escape);
    }
}
