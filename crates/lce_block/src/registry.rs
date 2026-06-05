use bevy::prelude::*;
use hashbrown::HashMap;
use leafwing_manifest::{
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
};
use serde::{Deserialize, Serialize};

use crate::block::Block;

/// Stores mapping of [`BlockId`] to [`Block`].
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Asset, TypePath, Resource,
)]
pub struct BlockRegistry {
    blocks: HashMap<Id<Block>, Block>,
}

impl BlockRegistry {
    /// The [`BlockId`] for the air block.
    #[must_use]
    pub const fn air_id() -> Id<Block> {
        Id::from_name("Ru4J:air")
    }

    /// Returns `true` if `blk` is an air block.
    #[must_use]
    pub fn is_air(blk: Id<Block>) -> bool {
        blk == Self::air_id()
    }
}

impl From<RawBlockRegistry> for BlockRegistry {
    fn from(value: RawBlockRegistry) -> Self {
        Self {
            blocks: value
                .block
                .iter()
                .map(|block| (Id::from_name(block.ident()), block.clone()))
                .collect(),
        }
    }
}

impl Manifest for BlockRegistry {
    type Item = Block;
    type RawItem = Block;
    type ConversionError = std::convert::Infallible;
    type RawManifest = RawBlockRegistry;

    const FORMAT: ManifestFormat = ManifestFormat::Toml;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        _world: &mut bevy::ecs::world::World,
    ) -> Result<Self, Self::ConversionError> {
        Ok(Self::from(raw_manifest))
    }

    fn get(&self, id: leafwing_manifest::identifier::Id<Self::Item>) -> Option<&Self::Item> {
        self.blocks.get(&id)
    }

    fn get_by_name(&self, name: impl std::borrow::Borrow<str>) -> Option<&Self::Item> {
        self.blocks.get(&Id::from_name(name.borrow()))
    }
}

#[doc(hidden)]
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Asset, TypePath, Resource,
)]
pub struct RawBlockRegistry {
    block: Vec<Block>,
}
