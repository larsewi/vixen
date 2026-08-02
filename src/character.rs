use avian3d::prelude::*;
use bevy::{ecs::query::Has, prelude::*};

pub struct CharacterControllerPlugin;

/// Marker for an entity moved by the kinematic character controller.
///
/// Requires [`CustomPositionIntegration`] so Avian doesn't also integrate the
/// body's velocity into its position; move-and-slide is the only thing that
/// moves it. The speculative margin is zeroed so we don't push bodies we merely
/// pass close to.
#[derive(Component)]
#[require(
    RigidBody::Kinematic,
    CustomPositionIntegration,
    SpeculativeMargin(0.0)
)]
pub struct CharacterController;

/// How a [`CharacterController`] accelerates, jumps and falls.
#[derive(Component)]
pub struct CharacterMovementSettings {
    /// Horizontal acceleration, in m/s^2. Together with `damping` this decides
    /// the top speed, which settles at `acceleration / damping`.
    pub acceleration: f32,
    /// Coefficient of the exponential decay applied to horizontal velocity, so
    /// the character coasts to a stop when there's no input.
    pub damping: f32,
    /// Upward velocity applied on jump, in m/s.
    pub jump_impulse: f32,
    /// Gravity for this character. Kinematic bodies are unaffected by the
    /// global [`Gravity`] resource, so the controller applies its own.
    pub gravity: Vec3,
    /// Cap on speed along gravity, in m/s, so a long fall doesn't accelerate
    /// forever.
    pub terminal_velocity: f32,
}

impl Default for CharacterMovementSettings {
    fn default() -> Self {
        Self {
            acceleration: 50.0,
            damping: 10.0,
            jump_impulse: 7.0,
            // Heavier than real gravity: a floaty jump feels sluggish in first
            // person.
            gravity: Vec3::new(0.0, -9.81 * 2.0, 0.0),
            terminal_velocity: 50.0,
        }
    }
}

/// How a [`CharacterController`] detects the ground beneath it.
#[derive(Component)]
pub struct GroundDetection {
    /// The shape cast straight down to look for ground. Usually a slightly
    /// smaller copy of the character's own collider, so brushing a wall doesn't
    /// register as ground.
    pub cast_shape: Collider,
    /// How far down to look, in meters.
    pub max_distance: f32,
    /// The steepest surface, in radians from level, that still counts as ground
    /// rather than a wall. The character can only jump from ground, and won't
    /// slide down it.
    pub max_angle: f32,
}

impl GroundDetection {
    /// Ground detection with `cast_shape`, looking 0.2 m down and treating
    /// anything up to 30 degrees from level as ground.
    pub fn new(cast_shape: Collider) -> Self {
        Self {
            cast_shape,
            max_distance: 0.2,
            max_angle: std::f32::consts::FRAC_PI_6,
        }
    }
}

/// Marker present while a [`CharacterController`] stands on ground that isn't
/// too steep.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// A movement request for the character controller, written by whatever drives
/// the character (see `Player::input`) and applied on the next physics tick.
#[derive(Message)]
pub enum MovementAction {
    /// Accelerate along a horizontal world-space direction, given as `(x, z)`
    /// with a length of at most 1.
    Move(Vec2),
    /// Jump, if grounded.
    Jump,
}

/// What [`CharacterController::move_and_slide`] needs from each character. Characters
/// without [`GroundDetection`] still slide, they just get no slope handling.
type MoveAndSlideData = (
    Entity,
    Option<&'static GroundDetection>,
    &'static Position,
    &'static Rotation,
    &'static mut Transform,
    &'static mut LinearVelocity,
    &'static Collider,
);

impl CharacterController {
    /// Toggles the [`Grounded`] marker by casting the ground shape straight down.
    ///
    /// Reads the physics [`Position`]/[`Rotation`] rather than the [`Transform`], which
    /// lags behind them when interpolation is enabled.
    fn update_grounded(
        mut commands: Commands,
        characters: Query<
            (Entity, &GroundDetection, &Position, &Rotation),
            With<CharacterController>,
        >,
        spatial_query: SpatialQuery,
    ) {
        for (entity, ground_detection, position, rotation) in &characters {
            let up = rotation * Vec3::Y;
            let down = Dir3::new(-up).unwrap_or(Dir3::NEG_Y);

            let hit = spatial_query.cast_shape(
                &ground_detection.cast_shape,
                position.0,
                rotation.0,
                down,
                &ShapeCastConfig::from_max_distance(ground_detection.max_distance),
                &SpatialQueryFilter::from_excluded_entities([entity]),
            );

            // Grounded if we hit a surface whose normal isn't too steep. `normal1` is
            // the normal of the surface we hit, already in world space.
            let is_grounded =
                hit.is_some_and(|hit| hit.normal1.angle_between(up) <= ground_detection.max_angle);

            if is_grounded {
                commands.entity(entity).insert(Grounded);
            } else {
                commands.entity(entity).remove::<Grounded>();
            }
        }
    }

    /// Accelerates the character along gravity, up to its terminal velocity.
    fn apply_gravity(
        time: Res<Time>,
        mut characters: Query<(&CharacterMovementSettings, &mut LinearVelocity)>,
    ) {
        for (settings, mut linear_velocity) in &mut characters {
            let gravity_direction = settings.gravity.normalize_or_zero();

            // Already falling as fast as we're allowed to.
            if linear_velocity.dot(gravity_direction) > settings.terminal_velocity {
                continue;
            }

            let new_velocity = linear_velocity.0 + settings.gravity * time.delta_secs();
            if new_velocity.dot(gravity_direction) < settings.terminal_velocity {
                linear_velocity.0 = new_velocity;
            } else {
                linear_velocity.0 = gravity_direction * settings.terminal_velocity;
            }
        }
    }

    /// Applies the [`MovementAction`]s written this frame.
    fn movement(
        time: Res<Time>,
        mut actions: MessageReader<MovementAction>,
        mut characters: Query<(
            &CharacterMovementSettings,
            &mut LinearVelocity,
            Has<Grounded>,
        )>,
    ) {
        for action in actions.read() {
            for (settings, mut linear_velocity, is_grounded) in &mut characters {
                match action {
                    MovementAction::Move(direction) => {
                        let delta = *direction * settings.acceleration * time.delta_secs();
                        linear_velocity.x += delta.x;
                        linear_velocity.z += delta.y;
                    }
                    MovementAction::Jump => {
                        if is_grounded {
                            linear_velocity.y = settings.jump_impulse;
                        }
                    }
                }
            }
        }
    }

    /// Slows down horizontal movement.
    fn apply_movement_damping(
        time: Res<Time>,
        mut characters: Query<(&CharacterMovementSettings, &mut LinearVelocity)>,
    ) {
        for (settings, mut linear_velocity) in &mut characters {
            // Approximate exponential decay. `LinearDamping` would do this for us, but
            // it would also dampen falling and jumping.
            let decay = 1.0 / (1.0 + time.delta_secs() * settings.damping);
            linear_velocity.x *= decay;
            linear_velocity.z *= decay;
        }
    }

    /// Sweeps each character through the world along its velocity, sliding along any
    /// surface it runs into.
    ///
    /// The sweep starts from the physics [`Position`], not the [`Transform`], because
    /// interpolation leaves the `Transform` somewhere between the last two physics
    /// steps; starting from it would drag the character backwards every tick. The result
    /// goes into the `Transform`, which Avian copies back into `Position` when it steps
    /// physics right after this.
    ///
    /// We assume the character is a root entity whose collider sits on the same entity.
    fn move_and_slide(
        mut characters: Query<MoveAndSlideData, With<CharacterController>>,
        move_and_slide: MoveAndSlide,
        time: Res<Time>,
    ) {
        for (
            entity,
            ground_detection,
            position,
            rotation,
            mut transform,
            mut linear_velocity,
            collider,
        ) in &mut characters
        {
            let up = rotation * Vec3::Y;
            let velocity = linear_velocity.0;
            let mut hit_ground_or_ceiling = false;

            let MoveAndSlideOutput {
                position: new_position,
                projected_velocity,
            } = move_and_slide.move_and_slide(
                collider,
                position.0,
                rotation.0,
                velocity,
                time.delta(),
                &MoveAndSlideConfig::default(),
                &SpatialQueryFilter::from_excluded_entities([entity]),
                |hit| {
                    // Called for every surface we touch along the sweep. We use it to
                    // stop the character sliding down ground it's standing on, and to
                    // stop it riding up surfaces too steep to walk on.
                    let Some(ground_detection) = ground_detection else {
                        return MoveAndSlideHitResponse::Accept;
                    };

                    let normal = *hit.normal;
                    let is_ground = up.angle_between(*normal) <= ground_detection.max_angle;
                    let is_ceiling = is_ground && up.dot(*normal) < 0.0;

                    // Split the input velocity so we can tell how much of it is trying
                    // to climb versus move along the surface.
                    let [horizontal_component, vertical_component] =
                        split_into_components(velocity, up);

                    let horizontal_decomposition =
                        decompose_hit_velocity(horizontal_component, normal, up);
                    let decomposition = decompose_hit_velocity(*hit.velocity, normal, up);

                    // Intent is what the input asks for; the other two are what the
                    // slide would actually do. Small thresholds keep noise out.
                    let slipping_intent =
                        up.dot(horizontal_decomposition.vertical_tangent) < -0.001;
                    let slipping = up.dot(decomposition.vertical_tangent) < -0.001;
                    let climbing_intent = up.dot(vertical_component) > 0.0;
                    let climbing = up.dot(decomposition.vertical_tangent) > 0.0;

                    *hit.velocity = if !is_ground && climbing && !climbing_intent {
                        // Too steep to climb: drop the upward motion the forward motion
                        // would otherwise induce.
                        decomposition.horizontal_tangent + decomposition.normal_part
                    } else if is_ground && slipping && !slipping_intent {
                        // Standing on ground: don't let it slide us downhill.
                        decomposition.horizontal_tangent + decomposition.normal_part
                    } else {
                        // Otherwise allow the full slide, climbing and slipping included.
                        decomposition.horizontal_tangent
                            + decomposition.vertical_tangent
                            + decomposition.normal_part
                    };

                    if is_ground || is_ceiling {
                        hit_ground_or_ceiling = true;
                    }

                    MoveAndSlideHitResponse::Accept
                },
            );

            transform.translation = new_position;

            // Adopt the swept velocity along the up-axis, so landing on a slope doesn't
            // accumulate speed into the ground and a jump doesn't stick to a ceiling.
            if hit_ground_or_ceiling {
                let velocity_along_up = linear_velocity.dot(up);
                let new_velocity_along_up = projected_velocity.dot(up);
                linear_velocity.0 += (new_velocity_along_up - velocity_along_up) * up;
            }
        }
    }
}

/// A velocity split into parts relative to a collision normal and an up-direction, used
/// to work out how much of it is climbing, slipping, or moving freely along the surface.
struct VelocityDecomposition {
    /// The part pushing straight into the surface.
    normal_part: Vec3,
    /// The part along the surface, perpendicular to the up-direction.
    horizontal_tangent: Vec3,
    /// The part along the surface, parallel to the up-direction.
    vertical_tangent: Vec3,
}

fn decompose_hit_velocity(velocity: Vec3, normal: Dir3, up: Vec3) -> VelocityDecomposition {
    let normal_part = normal * normal.dot(velocity);
    let tangent_part = velocity - normal_part;

    let horizontal_tangent_dir = normal.cross(up).normalize_or_zero();
    let horizontal_tangent = tangent_part.dot(horizontal_tangent_dir) * horizontal_tangent_dir;
    let vertical_tangent = tangent_part - horizontal_tangent;

    VelocityDecomposition {
        normal_part,
        horizontal_tangent,
        vertical_tangent,
    }
}

/// Splits a vector into its horizontal and vertical components, in that order, relative
/// to an `up` direction.
fn split_into_components(v: Vec3, up: Vec3) -> [Vec3; 2] {
    let vertical = up * v.dot(up);
    [v - vertical, vertical]
}

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<MovementAction>();

        // Move in `FixedUpdate` so behaviour doesn't depend on the frame rate, and so
        // the new position is in place before Avian steps physics in `FixedPostUpdate`.
        app.add_systems(
            FixedUpdate,
            (
                CharacterController::update_grounded,
                CharacterController::apply_gravity,
                CharacterController::movement,
                CharacterController::apply_movement_damping,
                CharacterController::move_and_slide,
            )
                .chain(),
        );
    }
}
