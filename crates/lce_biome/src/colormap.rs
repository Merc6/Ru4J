use bevy::color::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct RawColorMap {
    pub grass: u32,
    pub foliage: u32,
    pub sky: u32,
}

/// Biome color blending data.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ColorMap {
    grass: Color,
    foliage: Color,
    sky: Color,
}

impl ColorMap {
    /// Constructs a new [`ColorMap`].
    #[must_use]
    pub const fn new(grass: Color, foliage: Color, sky: Color) -> Self {
        Self {
            grass,
            foliage,
            sky,
        }
    }

    /// The grass color tint for the [`ColorMap`].
    #[must_use]
    pub const fn grass(&self) -> &Color {
        &self.grass
    }

    /// The foliage color tint for the [`ColorMap`].
    #[must_use]
    pub const fn foliage(&self) -> &Color {
        &self.foliage
    }

    /// The sky color-tint for the [`ColorMap`].
    #[must_use]
    pub const fn sky(&self) -> &Color {
        &self.sky
    }
}

impl From<RawColorMap> for ColorMap {
    fn from(value: RawColorMap) -> Self {
        Self {
            grass: Color::srgb_u8(
                unsafe { ((value.grass >> 16) & 0xFF).try_into().unwrap_unchecked() },
                unsafe { ((value.grass >> 8) & 0xFF).try_into().unwrap_unchecked() },
                unsafe { (value.grass & 0xFF).try_into().unwrap_unchecked() },
            ),
            foliage: Color::srgb_u8(
                unsafe { ((value.foliage >> 16) & 0xFF).try_into().unwrap_unchecked() },
                unsafe { ((value.foliage >> 8) & 0xFF).try_into().unwrap_unchecked() },
                unsafe { (value.foliage & 0xFF).try_into().unwrap_unchecked() },
            ),
            sky: Color::srgb_u8(
                unsafe { ((value.sky >> 16) & 0xFF).try_into().unwrap_unchecked() },
                unsafe { ((value.sky >> 8) & 0xFF).try_into().unwrap_unchecked() },
                unsafe { (value.sky & 0xFF).try_into().unwrap_unchecked() },
            ),
        }
    }
}
