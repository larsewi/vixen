use avian3d::prelude::*;
use bevy::{ecs::query::Has, input::mouse::MouseMotion, prelude::*};

use crate::camera::Camera;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

/// Marker present while the player is standing on the ground (the cube).
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// Target horizontal speed while moving on the ground, in m/s.
#[derive(Component)]
pub struct MovementSpeed(f32);

/// Acceleration available for steering while airborne, in m/s². Lower than instant
/// ground control, so a jump keeps most of its momentum but can still be nudged.
#[derive(Component)]
pub struct AirAcceleration(f32);

/// Upward velocity applied on jump.
#[derive(Component)]
pub struct JumpImpulse(f32);

/// Steepest ground the player can stand on and jump from, in radians.
#[derive(Component)]
pub struct MaxSlopeAngle(f32);

impl Player {
    pub fn spawn(mut commands: Commands) {
        // Local height of the camera above the capsule's center. The capsule rests
        // with its center ~0.9 above the cube's top face, so this puts the eyes near
        // the top of the body (~1.7 above the ground the player stands on).
        const EYE_OFFSET: f32 = 0.8;

        // Slim capsule (radius 0.3, cylinder length 1.2 => ~1.8 total) so it fits the
        // 1×1 top of the cube.
        let collider = Collider::capsule(0.3, 1.2);

        // Ground detection: a slightly smaller copy of the collider cast straight down.
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vec3::ONE * 0.99, 10);

        commands
            .spawn((
                Player,
                // Spawn above the cube so the player drops and settles on top of it,
                // facing horizontally. Only yaw lives on the body; pitch is applied to
                // the camera.
                Transform::from_xyz(0.0, 2.5, 0.0).looking_to(Vec3::NEG_Z, Vec3::Y),
                // The player has no mesh of its own, but the camera child inherits
                // visibility, so the parent needs the visibility components too —
                // otherwise Bevy warns about an inconsistent hierarchy (B0004).
                Visibility::default(),
                RigidBody::Dynamic,
                collider,
                // Keep the capsule upright but leave yaw free so we can turn it.
                LockedAxes::new().lock_rotation_x().lock_rotation_z(),
                ShapeCaster::new(caster_shape, Vec3::ZERO, Quat::default(), Dir3::NEG_Y)
                    .with_max_distance(0.2),
                MovementSpeed(3.0),
                AirAcceleration(10.0),
                JumpImpulse(5.0),
                MaxSlopeAngle(std::f32::consts::PI * 0.45),
                // Don't stick to or bounce off surfaces we brush against.
                Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
                Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
                // Smooth the rendered position between fixed physics steps. Only
                // translation — yaw is driven manually and should stay instant.
                TranslationInterpolation,
            ))
            // Mount the camera on the player's head. It inherits the body's yaw;
            // pitch is applied locally by the camera.
            .with_child((Camera, Transform::from_xyz(0.0, EYE_OFFSET, 0.0)));
    }

    fn yaw(
        mut mouse: MessageReader<MouseMotion>,
        player: Single<(&mut Rotation, &mut AngularVelocity), With<Player>>,
    ) {
        let sensitivity = 0.003;
        let (mut rotation, mut angular_velocity) = player.into_inner();

        for motion in mouse.read() {
            // Add yaw which is turning left/right. Pitch is handled by the camera.
            // We write the physics `Rotation` (its source of truth) rather than the
            // `Transform`, so the solver doesn't overwrite it.
            let delta_yaw = -motion.delta.x * sensitivity;
            rotation.0 = Quat::from_rotation_y(delta_yaw) * rotation.0;
        }

        // Yaw is driven manually, so cancel any spin the solver picked up from
        // brushing against geometry.
        angular_velocity.0 = Vec3::ZERO;
    }

    /// Toggles the [`Grounded`] marker based on the downward ground caster.
    fn update_grounded(
        mut commands: Commands,
        player: Single<(Entity, &ShapeHits, &Rotation, &MaxSlopeAngle), With<Player>>,
    ) {
        let (entity, hits, rotation, max_slope_angle) = player.into_inner();

        // Grounded if the caster hit a surface whose normal isn't too steep.
        let is_grounded = hits
            .iter()
            .any(|hit| (rotation * -hit.normal2).angle_between(Vec3::Y).abs() <= max_slope_angle.0);

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }

    fn movement(
        time: Res<Time>,
        keyboard: Res<ButtonInput<KeyCode>>,
        player: Single<
            (
                &MovementSpeed,
                &AirAcceleration,
                &JumpImpulse,
                &Rotation,
                &mut LinearVelocity,
                Has<Grounded>,
            ),
            With<Player>,
        >,
    ) {
        let (speed, air_acceleration, jump_impulse, rotation, mut linear_velocity, is_grounded) =
            player.into_inner();

        // Build a horizontal input direction (local space, -Z is forward).
        let mut direction = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            direction.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        // World-space move direction, rotated by the body's yaw so it's relative to
        // where the player faces.
        let world_dir = if direction != Vec3::ZERO {
            rotation.0 * direction.normalize()
        } else {
            Vec3::ZERO
        };

        if is_grounded {
            // Snap horizontal velocity straight to the target so movement starts and
            // stops instantly — the crisp, responsive feel of an FPS.
            let target = world_dir * speed.0;
            linear_velocity.x = target.x;
            linear_velocity.z = target.z;
        } else if world_dir != Vec3::ZERO {
            // Airborne: nudge horizontal velocity toward the input with limited
            // authority, so you can steer the jump. Momentum is otherwise preserved
            // (no ground snap), and steering can't push us past the ground speed.
            let delta = world_dir * air_acceleration.0 * time.delta_secs();
            linear_velocity.x += delta.x;
            linear_velocity.z += delta.z;

            let horizontal = Vec2::new(linear_velocity.x, linear_velocity.z);
            if horizontal.length() > speed.0 {
                let clamped = horizontal.normalize() * speed.0;
                linear_velocity.x = clamped.x;
                linear_velocity.z = clamped.y;
            }
        }

        if is_grounded && keyboard.just_pressed(KeyCode::Space) {
            linear_velocity.y = jump_impulse.0;
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Player::spawn);
        app.add_systems(
            Update,
            (Player::yaw, Player::update_grounded, Player::movement).chain(),
        );
    }
}
