use std::collections::HashMap;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::block::{BLOCK_SIZE, Block};
use crate::chunk::{CHUNK_SIZE, Chunk, ChunkData, WORLD_HEIGHT};
use crate::{mesher, terrain};

/// Width of the generated world, in chunks. The grid is centred on the origin.
const WORLD_CHUNKS: i32 = 8;

pub struct WorldPlugin;

/// Resolve the block at a global block coordinate by locating its owning chunk in
/// `map`. Coordinates outside the vertical range or the finite chunk grid read as
/// air, so the world's underside isn't drawn and its outer edges become cliffs.
fn sample_world(map: &HashMap<IVec2, ChunkData>, gx: i32, gy: i32, gz: i32) -> Block {
    if gy < 0 || gy >= WORLD_HEIGHT as i32 {
        return Block::Air;
    }
    let coord = IVec2::new(
        gx.div_euclid(CHUNK_SIZE as i32),
        gz.div_euclid(CHUNK_SIZE as i32),
    );
    match map.get(&coord) {
        Some(data) => data.get(
            gx.rem_euclid(CHUNK_SIZE as i32) as usize,
            gy as usize,
            gz.rem_euclid(CHUNK_SIZE as i32) as usize,
        ),
        None => Block::Air,
    }
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let half = WORLD_CHUNKS / 2;

    // A fresh random world each run. Logged so a world you like can be reproduced
    // by hard-coding this value.
    let seed: u32 = rand::random();
    info!("world seed: {seed}");

    // Generate every chunk's block data up front so meshing can sample across chunk
    // borders and cull faces between adjacent solid chunks.
    let mut map: HashMap<IVec2, ChunkData> = HashMap::new();
    for cz in -half..half {
        for cx in -half..half {
            map.insert(IVec2::new(cx, cz), terrain::generate(cx, cz, seed));
        }
    }

    // One shared white material for the whole world; per-block color comes from the
    // mesh's vertex colors.
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    for (&coord, _) in &map {
        // Sampler in chunk-local space: shift local coords into global space, then
        // look them up in the world map (this transparently reaches into neighbours).
        let origin_x = coord.x * CHUNK_SIZE as i32;
        let origin_z = coord.y * CHUNK_SIZE as i32;
        let mesh = mesher::build_mesh(|x, y, z| {
            sample_world(&map, origin_x + x, y, origin_z + z)
        });

        // A fully empty chunk (no faces) yields no collider; skip it.
        let Some(collider) = Collider::trimesh_from_mesh(&mesh) else {
            continue;
        };

        commands.spawn((
            Chunk,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(origin_x as f32 * BLOCK_SIZE, 0.0, origin_z as f32 * BLOCK_SIZE),
            RigidBody::Static,
            collider,
        ));
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_world);
    }
}
