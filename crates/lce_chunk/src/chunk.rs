use bevy::prelude::*;
use lce_biome::BiomeId;
use lce_palette::collection::PVec;

/// A 16x by 16z portion of the world.
///
/// Chunks are the method used by the game to divide maps into manageable
/// pieces, those pieces are then further broken down into sections. In `Ru4J`,
/// chunks are [`Components`](Component), and sections exist as
/// [`SubChunks`](crate::subchunk::SubChunk).
///
/// Sub-chunks aren't stored inline with chunks, they're stored as [`Children`]
/// [`Entities`](Entity) of the chunk.
#[derive(Component, Debug, Default)]
#[require(Children, Transform, InheritedVisibility)]
pub struct Chunk {
    biomes: PVec<BiomeId>,
}

impl Chunk {
    /// Returns the biomes stored in the [`Chunk`].
    #[must_use]
    pub const fn biomes(&self) -> &PVec<BiomeId> {
        &self.biomes
    }
}
