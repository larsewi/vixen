use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Cube;

impl Cube {
    pub fn spawn(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        commands.spawn((
            Cube,
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            // A fixed, floating platform. Its top face is at y = 1.0 — the surface
            // the player lands and walks on. There is no floor, so walking off the
            // edge means falling into the void.
            Transform::from_xyz(0.0, 0.5, 0.0),
            RigidBody::Static,
            Collider::cuboid(1.0, 1.0, 1.0),
        ));
    }
}
