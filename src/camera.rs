use bevy::prelude::*;

pub struct CameraPlugin;

#[derive(Component)]
#[require(Camera3d)]
pub struct Camera;

impl Camera {
    fn spawn(mut commands: Commands) {
        commands.spawn((
            Camera,
            Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    }

    fn walk(
        time: Res<Time>,
        input: Res<ButtonInput<KeyCode>>,
        mut camera: Single<&mut Transform, With<Camera>>,
    ) {
        let delta_time = time.delta_secs();
        let move_speed = 10.0;
        let mut direction = Vec3::ZERO;

        if input.pressed(KeyCode::KeyW) {
            direction += *camera.forward();
        }
        if input.pressed(KeyCode::KeyA) {
            direction += *camera.left();
        }
        if input.pressed(KeyCode::KeyS) {
            direction += *camera.back();
        }
        if input.pressed(KeyCode::KeyD) {
            direction += *camera.right();
        }

        if direction != Vec3::ZERO {
            let direction = direction.normalize();
            camera.translation += direction * move_speed * delta_time;
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Camera::spawn);
        app.add_systems(Update, Camera::walk);
    }
}
