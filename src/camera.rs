use bevy::{input::mouse::MouseMotion, prelude::*};

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
        keyboard: Res<ButtonInput<KeyCode>>,
        mut camera: Single<&mut Transform, With<Camera>>,
    ) {
        // TODO: Consider bevy_enhanced_input if we run into frame issues
        // See https://taintedcoders.com/bevy/input
        let delta_time = time.delta_secs();
        let move_speed = 10.0;
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            direction += *camera.forward();
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction += *camera.left();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction += *camera.back();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += *camera.right();
        }

        if direction != Vec3::ZERO {
            let direction = direction.normalize();
            camera.translation += direction * move_speed * delta_time;
        }
    }

    fn rotate(
        time: Res<Time>,
        mut mouse: MessageReader<MouseMotion>,
        mut camera: Single<&mut Transform, With<Camera>>,
    ) {
        let delta_time = time.delta_secs();
        let sensitivity = Vec2::new(0.08, 0.08);

        for motion in mouse.read() {
            // Add yaw which is turning left/right
            let delta_yaw = -motion.delta.x * delta_time * sensitivity.x;
            camera.rotate_y(delta_yaw);

            // Add pitch which is looking up/down
            let delta_pitch = -motion.delta.y * delta_time * sensitivity.y;
            const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
            let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

            // Apply the rotation
            let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
            camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Camera::spawn);
        app.add_systems(Update, Camera::walk);
        app.add_systems(Update, Camera::rotate);
    }
}
