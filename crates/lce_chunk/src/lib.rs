//! A strategy to divide the world into manageable segments.

use bevy::prelude::*;

mod chunk;
mod subchunk;

pub use chunk::Chunk;
pub use subchunk::SubChunk;

/// The [`Plugin`] that implements chunking behavior.
///
/// For more details on chunking see [`Chunk`].
#[derive(Clone, Copy, Debug, Eq, Default, Hash, PartialEq)]
pub struct LceChunkPlugin;

impl Plugin for LceChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, SubChunk::generate_mesh);
    }
}
