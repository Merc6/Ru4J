use std::borrow::Cow;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{Climate, ColorMap, Worldgen, colormap::RawColorMap};

/// A region of the world with distinct geographical features, plants, mobs,
/// temperatures, humidity-levels, colors, and more.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Biome {
    ident: Cow<'static, str>,
    climate: Climate,
    worldgen: Worldgen,
    colormap: ColorMap,
}

impl Biome {
    /// The identifier of the [`Biome`].
    ///
    /// # Note
    /// This includes the namespace of the identifier.
    #[must_use]
    pub fn ident(&self) -> &str {
        &self.ident
    }
}

impl From<&RawBiome> for Biome {
    fn from(value: &RawBiome) -> Self {
        Self {
            ident: value.ident.clone(),
            climate: value.climate,
            worldgen: value.worldgen,
            colormap: ColorMap::from(value.colormap),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Asset, TypePath)]
pub struct RawBiome {
    ident: Cow<'static, str>,
    climate: Climate,
    worldgen: Worldgen,
    colormap: RawColorMap,
}

impl RawBiome {
    pub fn ident(&self) -> &str {
        &self.ident
    }
}
