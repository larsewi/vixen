use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::block::{BLOCK_SIZE, Block};
use crate::chunk::{CHUNK_SIZE, WORLD_HEIGHT};

/// One cube face: the neighbor direction to test for culling, the outward normal,
/// and the four corner offsets (relative to the block's minimum corner) wound
/// counter-clockwise when viewed from outside, so they front-face with Bevy's
/// default back-face culling. Triangulated as [0, 1, 2, 0, 2, 3].
struct Face {
    dir: [i32; 3],
    normal: [f32; 3],
    corners: [[f32; 3]; 4],
}

const FACES: [Face; 6] = [
    // +X
    Face {
        dir: [1, 0, 0],
        normal: [1.0, 0.0, 0.0],
        corners: [
            [1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
        ],
    },
    // -X
    Face {
        dir: [-1, 0, 0],
        normal: [-1.0, 0.0, 0.0],
        corners: [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
        ],
    },
    // +Y
    Face {
        dir: [0, 1, 0],
        normal: [0.0, 1.0, 0.0],
        corners: [
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
    },
    // -Y
    Face {
        dir: [0, -1, 0],
        normal: [0.0, -1.0, 0.0],
        corners: [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
    },
    // +Z
    Face {
        dir: [0, 0, 1],
        normal: [0.0, 0.0, 1.0],
        corners: [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
    },
    // -Z
    Face {
        dir: [0, 0, -1],
        normal: [0.0, 0.0, -1.0],
        corners: [
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
    },
];

const FACE_UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

/// Build a culled mesh for a single chunk. `sample` returns the block at a
/// chunk-local coordinate, and may be called with coordinates outside the chunk
/// (negative or `>= CHUNK_SIZE`/`WORLD_HEIGHT`); it is responsible for resolving
/// those into neighboring chunks or air, so faces at chunk borders are only
/// emitted where a solid block truly meets air. Vertex positions are chunk-local;
/// the chunk entity's `Transform` places them in the world.
pub fn build_mesh(sample: impl Fn(i32, i32, i32) -> Block) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for y in 0..WORLD_HEIGHT as i32 {
        for z in 0..CHUNK_SIZE as i32 {
            for x in 0..CHUNK_SIZE as i32 {
                let block = sample(x, y, z);
                if !block.is_solid() {
                    continue;
                }
                let color = block.color();

                for face in &FACES {
                    // Only emit the face if the block it faces isn't solid.
                    if sample(x + face.dir[0], y + face.dir[1], z + face.dir[2]).is_solid() {
                        continue;
                    }

                    let base = positions.len() as u32;
                    for (corner, uv) in face.corners.iter().zip(FACE_UVS.iter()) {
                        positions.push([
                            (x as f32 + corner[0]) * BLOCK_SIZE,
                            (y as f32 + corner[1]) * BLOCK_SIZE,
                            (z as f32 + corner[2]) * BLOCK_SIZE,
                        ]);
                        normals.push(face.normal);
                        uvs.push(*uv);
                        colors.push(color);
                    }
                    indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lone_block_emits_all_six_faces() {
        // A single solid block at the chunk origin, everything else air.
        let mesh = build_mesh(|x, y, z| {
            if (x, y, z) == (0, 0, 0) {
                Block::Stone
            } else {
                Block::Air
            }
        });

        // 6 faces × 4 vertices, 6 faces × 6 indices.
        assert_eq!(mesh.count_vertices(), 24);
        let Some(Indices::U32(indices)) = mesh.indices() else {
            panic!("expected U32 indices");
        };
        assert_eq!(indices.len(), 36);
    }

    #[test]
    fn fully_enclosed_solid_emits_no_faces() {
        // Solid everywhere, including out-of-bounds neighbours: every face borders
        // another solid block, so culling removes all of them.
        let mesh = build_mesh(|_, _, _| Block::Stone);
        assert_eq!(mesh.count_vertices(), 0);
    }
}
