//! Biome stuff for LCE.

mod biome;
mod climate;
mod colormap;
mod registry;
mod worldgen;

pub use biome::Biome;
pub use climate::Climate;
pub use colormap::ColorMap;
pub use registry::BiomeRegistry;
pub use worldgen::Worldgen;

#[rustfmt::skip]
use {
    bevy::prelude::*,
    leafwing_manifest::{
        asset_state::SimpleAssetState,
        plugin::{ManifestPlugin, RegisterManifest},
    },
};

/// Plugin to load the biome registry.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct LceBiomePlugin;

impl Plugin for LceBiomePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SimpleAssetState>()
            .add_plugins(ManifestPlugin::<SimpleAssetState>::default())
            .register_manifest::<BiomeRegistry>("registry/biome.toml");
    }
}
