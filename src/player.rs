use avian3d::prelude::*;
use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::block::BLOCK_SIZE;
use crate::camera::Camera;
use crate::character::{
    CharacterController, CharacterMovementSettings, GroundDetection, MovementAction,
};
use crate::chunk::WORLD_HEIGHT;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

impl Player {
    pub fn spawn(mut commands: Commands) {
        // Local height of the camera above the body's center. The body is ~1.8 tall,
        // so an offset of 0.8 puts the eyes near the top of the body.
        const EYE_OFFSET: f32 = 0.8;

        commands
            .spawn((
                Player,
                // Spawn high above the world center so the player drops and settles on
                // top of the generated terrain, facing horizontally. Only yaw lives on
                // the body; pitch is applied to the camera.
                Transform::from_xyz(0.0, WORLD_HEIGHT as f32 * BLOCK_SIZE + 2.0, 0.0)
                    .looking_to(Vec3::NEG_Z, Vec3::Y),
                // The player has no mesh of its own, but the camera child inherits
                // visibility, so the parent needs the visibility components too;
                // otherwise Bevy warns about an inconsistent hierarchy (B0004).
                Visibility::default(),
                // Moved by the kinematic character controller in `character.rs` rather
                // than by the physics solver, so the player never gets shoved around by
                // contacts and stays exactly where we put it.
                CharacterController,
                // A cylinder (radius 0.3, height 1.8) rather than a capsule: its flat
                // bottom rests flush on a block's top face, so the player doesn't slide
                // off the rounded edge of a capsule when standing on stepped terrain.
                Collider::cylinder(0.3, 1.8),
                // Ground detection casts a slightly slimmer copy of the body downwards,
                // so hugging a wall doesn't register as standing on it.
                GroundDetection::new(Collider::cylinder(0.29, 1.8)),
                CharacterMovementSettings {
                    // Tops out at acceleration / damping, so a little over 3 m/s.
                    acceleration: 40.0,
                    damping: 12.0,
                    ..default()
                },
                // Smooth the rendered position between fixed physics steps. Only
                // translation - yaw is driven manually and should stay instant.
                TranslationInterpolation,
            ))
            // Mount the camera on the player's head. It inherits the body's yaw;
            // pitch is applied locally by the camera.
            .with_child((Camera, Transform::from_xyz(0.0, EYE_OFFSET, 0.0)));
    }

    fn yaw(mut mouse: MessageReader<MouseMotion>, player: Single<&mut Rotation, With<Player>>) {
        let sensitivity = 0.001;
        let mut rotation = player.into_inner();

        for motion in mouse.read() {
            // Add yaw which is turning left/right. Pitch is handled by the camera.
            // We write the physics `Rotation` (its source of truth) rather than the
            // `Transform`, so the physics writeback doesn't overwrite it.
            let delta_yaw = -motion.delta.x * sensitivity;
            rotation.0 = Quat::from_rotation_y(delta_yaw) * rotation.0;
        }
    }

    /// Turns keyboard input into [`MovementAction`]s for the character controller.
    fn input(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut actions: MessageWriter<MovementAction>,
        rotation: Single<&Rotation, With<Player>>,
    ) {
        let forward = keyboard.pressed(KeyCode::KeyW) as i8 - keyboard.pressed(KeyCode::KeyS) as i8;
        let right = keyboard.pressed(KeyCode::KeyD) as i8 - keyboard.pressed(KeyCode::KeyA) as i8;

        // Input direction in local space, where -Z is forward.
        let local = Vec3::new(right as f32, 0.0, -forward as f32);

        if local != Vec3::ZERO {
            // Rotate by the body's yaw so movement is relative to where the player
            // faces. The body only ever yaws, so this stays horizontal.
            let world = rotation.0 * local.normalize();
            actions.write(MovementAction::Move(Vec2::new(world.x, world.z)));
        }

        if keyboard.just_pressed(KeyCode::Space) {
            actions.write(MovementAction::Jump);
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Player::spawn);
        // Collect input in `PreUpdate` so it is queued before the fixed timestep runs
        // and the character controller consumes it.
        app.add_systems(PreUpdate, Player::input);
        app.add_systems(Update, Player::yaw);
    }
}
