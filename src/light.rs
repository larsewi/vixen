use bevy::prelude::*;

pub struct LightPlugin;

fn spawn_light(mut commands: Commands) {
    // A single sun over the whole terrain. A directional light casts parallel rays,
    // so the far chunks are lit as evenly as those under the origin -- unlike a point
    // light, whose falloff would leave the edges of the world dark.
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(-1.0, -2.0, -1.0), Vec3::Y),
    ));
}

impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_light);
    }
}
