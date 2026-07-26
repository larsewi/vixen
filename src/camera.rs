use bevy::{input::mouse::MouseMotion, prelude::*};

pub struct CameraPlugin;

#[derive(Component)]
#[require(Camera3d)]
pub struct Camera;

impl Camera {
    fn pitch(
        mut mouse: MessageReader<MouseMotion>,
        mut camera: Single<&mut Transform, With<Camera>>,
    ) {
        // TODO: Also look into picking
        // See https://taintedcoders.com/bevy/picking
        let sensitivity = 0.003;

        for motion in mouse.read() {
            // Add pitch which is looking up/down. Yaw is handled by the player.
            let delta_pitch = -motion.delta.y * sensitivity;
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
        app.add_systems(Update, Camera::pitch);
    }
}
