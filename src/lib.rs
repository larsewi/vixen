use bevy::prelude::*;

mod camera;
mod cube;
mod light;

use camera::CameraPlugin;
use cube::Cube;
use light::LightPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CameraPlugin, LightPlugin))
            .add_systems(Startup, Cube::spawn);
    }
}
