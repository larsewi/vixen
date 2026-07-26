use bevy::color::{ColorToComponents, LinearRgba, palettes::tailwind};

/// World-space edge length of one block, in meters. Voxel logic stays on an integer
/// grid; only the generated geometry, colliders, and chunk offsets are scaled by
/// this, so shrinking it makes the whole world finer-grained. Tune here.
pub const BLOCK_SIZE: f32 = 0.2;

/// The kind of material occupying a single cell of the voxel grid.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
}

impl Block {
    /// Whether this block fills its cell. Air is the only non-solid block, so
    /// it alone lets a neighboring face show through during meshing.
    pub fn is_solid(self) -> bool {
        self != Block::Air
    }

    /// Linear RGBA vertex color for every face of this block, taken from Bevy's
    /// Tailwind palette and converted from sRGB to linear (mesh vertex colors are
    /// interpreted as linear). Multiplied by the chunk material's white `base_color`,
    /// so these are the colors seen in-world. `Air` never reaches meshing, so its
    /// value is unused.
    pub fn color(self) -> [f32; 4] {
        let srgb = match self {
            Block::Air => bevy::color::Srgba::default(),
            Block::Grass => tailwind::GREEN_600,
            Block::Dirt => tailwind::AMBER_800,
            Block::Stone => tailwind::STONE_500,
        };
        LinearRgba::from(srgb).to_f32_array()
    }
}
