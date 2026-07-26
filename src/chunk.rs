use bevy::prelude::*;

use crate::block::Block;

/// Horizontal footprint of a chunk, in blocks (X and Z).
pub const CHUNK_SIZE: usize = 16;
/// Vertical extent of the world, in blocks. A chunk is a full column, so there is
/// no vertical chunk stacking — only horizontal neighbours matter.
pub const WORLD_HEIGHT: usize = 64;

/// Marker for a spawned, meshed chunk entity.
#[derive(Component)]
pub struct Chunk;

/// Dense voxel storage for one `CHUNK_SIZE × CHUNK_SIZE × WORLD_HEIGHT` column.
pub struct ChunkData {
    blocks: Vec<Block>,
}

impl ChunkData {
    /// A column filled entirely with air, ready to be carved by terrain generation.
    pub fn empty() -> Self {
        Self {
            blocks: vec![Block::Air; CHUNK_SIZE * CHUNK_SIZE * WORLD_HEIGHT],
        }
    }

    /// Flat index for a local coordinate, laid out y-major so vertical columns are
    /// strided and horizontal slices are contiguous.
    fn index(x: usize, y: usize, z: usize) -> usize {
        (y * CHUNK_SIZE + z) * CHUNK_SIZE + x
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        self.blocks[Self::index(x, y, z)]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        self.blocks[Self::index(x, y, z)] = block;
    }
}
