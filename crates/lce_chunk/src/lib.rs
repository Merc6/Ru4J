//! A strategy to divide the world into manageable segments.

#![feature(stmt_expr_attributes)]

use bevy::prelude::*;

mod chunk;
mod subchunk;

pub use chunk::Chunk;
pub use subchunk::SubChunk;

#[derive(Clone, Copy, Debug, Eq, Default, Hash, PartialEq)]
pub struct LceChunkPlugin;

impl Plugin for LceChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, SubChunk::generate_mesh);
    }
}
