use noise::{NoiseFn, Perlin};

use crate::block::Block;
use crate::chunk::{CHUNK_SIZE, ChunkData, WORLD_HEIGHT};

/// Lowest and highest surface a column can reach, leaving headroom below (bedrock
/// of stone) and above (air) the generated band.
const MIN_HEIGHT: i32 = 8;
const MAX_HEIGHT: i32 = WORLD_HEIGHT as i32 - 8;

/// How far apart, in blocks, the noise field varies. Larger => broader, gentler hills.
const FEATURE_SIZE: f64 = 48.0;

/// Generate the block data for the chunk at chunk-grid coordinate `(chunk_x, chunk_z)`.
///
/// A 2D Perlin heightmap picks a surface height per column: the top block is grass,
/// the three below are dirt, and everything deeper is stone. Above the surface is air.
pub fn generate(chunk_x: i32, chunk_z: i32, seed: u32) -> ChunkData {
    let perlin = Perlin::new(seed);
    let mut data = ChunkData::empty();

    debug_assert!(MIN_HEIGHT >= 3, "dirt band needs room below the surface");

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            // World-space block coordinates for this column.
            let wx = chunk_x * CHUNK_SIZE as i32 + x as i32;
            let wz = chunk_z * CHUNK_SIZE as i32 + z as i32;

            // Perlin returns ~[-1, 1]; remap to [0, 1] then into the height band.
            let noise = perlin.get([wx as f64 / FEATURE_SIZE, wz as f64 / FEATURE_SIZE]);
            let t = (noise + 1.0) / 2.0;
            let height = MIN_HEIGHT + (t * (MAX_HEIGHT - MIN_HEIGHT) as f64).round() as i32;

            for y in 0..=height {
                let block = if y == height {
                    Block::Grass
                } else if y >= height - 3 {
                    Block::Dirt
                } else {
                    Block::Stone
                };
                data.set(x, y as usize, z, block);
            }
        }
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_have_grass_on_air_within_the_height_band() {
        let data = generate(0, 0, 1337);

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // Find the surface: the highest solid block in the column.
                let surface = (0..WORLD_HEIGHT)
                    .rev()
                    .find(|&y| data.get(x, y, z).is_solid())
                    .expect("every column has ground");

                assert!(
                    (MIN_HEIGHT as usize..=MAX_HEIGHT as usize).contains(&surface),
                    "surface {surface} outside height band",
                );
                assert_eq!(data.get(x, surface, z), Block::Grass, "top block is grass");
                assert!(
                    !data.get(x, surface + 1, z).is_solid(),
                    "grass sits directly below air",
                );
            }
        }
    }
}
