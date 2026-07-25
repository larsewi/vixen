use bevy::prelude::*;

pub struct LightPlugin;

fn spawn_light(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_light);
    }
}
