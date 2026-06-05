//! A strategy to divide the world into manageable segments.
//!
//! # Note
//!
//! This API is completely unstable and subject to change.

use bevy::prelude::*;

mod chunk;
mod subchunk;

pub use chunk::Chunk;
use leafwing_manifest::asset_state::SimpleAssetState;
pub use subchunk::SubChunk;

/// The [`Plugin`] that implements chunking behavior.
///
/// For more details on chunking see [`Chunk`] and [`SubChunk`].
#[derive(Clone, Copy, Debug, Eq, Default, Hash, PartialEq)]
pub struct LceChunkPlugin;

impl Plugin for LceChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            SubChunk::generate_mesh.run_if(
                in_state(lce_asset::BlockAssetLoadState::Finished)
                    .and(in_state(SimpleAssetState::Ready)),
            ),
        );
    }
}
