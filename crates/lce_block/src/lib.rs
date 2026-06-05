//! blocks.

mod block;
mod registry;
mod texture;

pub use block::Block;
pub use registry::BlockRegistry;
pub use texture::BlockTextures;

#[rustfmt::skip]
use {
    bevy::prelude::*,
    leafwing_manifest::{
        plugin::{RegisterManifest},
    },
};

/// An Identifier that allows access into the [biome registry](BiomeRegistry).
pub type BlockId = leafwing_manifest::identifier::Id<Block>;

/// Plugin to load the block registry.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct LceBlockPlugin;

impl Plugin for LceBlockPlugin {
    fn build(&self, app: &mut App) {
        app.register_manifest::<BlockRegistry>("registry/block.toml");
    }
}
