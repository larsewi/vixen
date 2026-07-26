use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::camera::Camera;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

impl Player {
    pub fn spawn(mut commands: Commands) {
        const HEAD_HEIGHT: f32 = 1.8;

        commands
            .spawn((
                Player,
                // On the ground, facing roughly toward the cube at the origin.
                // Only yaw lives on the body; pitch is applied to the camera.
                Transform::from_xyz(-2.5, 0.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ))
            // Mount the camera on the player's head. It inherits the body's
            // yaw; pitch is applied locally by the camera.
            .with_child((Camera, Transform::from_xyz(0.0, HEAD_HEIGHT, 0.0)));
    }

    fn walk(
        time: Res<Time>,
        keyboard: Res<ButtonInput<KeyCode>>,
        mut player: Single<&mut Transform, With<Player>>,
    ) {
        // TODO: Consider bevy_enhanced_input if we run into frame issues
        // See https://taintedcoders.com/bevy/input
        let delta_time = time.delta_secs();
        let move_speed = 10.0;
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            direction += *player.forward();
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction += *player.left();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction += *player.back();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += *player.right();
        }

        if direction != Vec3::ZERO {
            let direction = direction.normalize();
            player.translation += direction * move_speed * delta_time;
        }
    }

    fn yaw(
        mut mouse: MessageReader<MouseMotion>,
        mut player: Single<&mut Transform, With<Player>>,
    ) {
        let sensitivity = 0.005;

        for motion in mouse.read() {
            // Add yaw which is turning left/right. Pitch is handled by the camera.
            let delta_yaw = -motion.delta.x * sensitivity;
            player.rotate_y(delta_yaw);
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Player::spawn);
        app.add_systems(Update, Player::walk);
        app.add_systems(Update, Player::yaw);
    }
}
