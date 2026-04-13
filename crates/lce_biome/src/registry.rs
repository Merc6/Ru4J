use bevy::prelude::*;
use hashbrown::HashMap;
use leafwing_manifest::{
    identifier::Id,
    manifest::{Manifest, ManifestFormat},
};
use serde::{Deserialize, Serialize};

use crate::{Biome, biome::RawBiome};

/// Contains all loaded biome information.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Asset, TypePath, Resource)]
pub struct BiomeRegistry {
    biomes: HashMap<Id<Biome>, Biome>,
}

impl From<RawBiomeRegistry> for BiomeRegistry {
    fn from(value: RawBiomeRegistry) -> Self {
        Self {
            biomes: value
                .biome
                .iter()
                .map(|biome| (Id::from_name(biome.ident()), biome.into()))
                .collect(),
        }
    }
}

impl Manifest for BiomeRegistry {
    type Item = Biome;
    type RawItem = RawBiome;
    type ConversionError = std::convert::Infallible;
    type RawManifest = RawBiomeRegistry;

    const FORMAT: ManifestFormat = ManifestFormat::Toml;

    fn from_raw_manifest(
        raw_manifest: Self::RawManifest,
        _world: &mut bevy::ecs::world::World,
    ) -> Result<Self, Self::ConversionError> {
        Ok(Self::from(raw_manifest))
    }

    fn get(&self, id: leafwing_manifest::identifier::Id<Self::Item>) -> Option<&Self::Item> {
        self.biomes.get(&id)
    }

    fn get_by_name(&self, name: impl std::borrow::Borrow<str>) -> Option<&Self::Item> {
        self.biomes.get(&Id::from_name(name.borrow()))
    }
}

#[doc(hidden)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Asset, TypePath, Resource)]
pub struct RawBiomeRegistry {
    biome: Vec<RawBiome>,
}
