use bevy::prelude::*;

pub struct CameraPlugin;

#[derive(Component)]
#[require(Camera3d)]
pub struct Camera;

impl Camera {
    fn spawn(mut commands: Commands) {
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Camera::spawn);
    }
}
