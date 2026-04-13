use serde::{Deserialize, Serialize};

/// A biome's climate data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Climate {
    temperature: f32,
    downfall: f32,
    precipitation: bool,
}

impl Climate {
    const RAIN_TEMP_THRESH: f32 = 0.15;
    const HUMID_THRESH: f32 = 0.85;

    /// Constructs a new [`Climate`].
    #[must_use]
    pub const fn new(temperature: f32, downfall: f32, precipitation: bool) -> Self {
        Self {
            temperature,
            downfall,
            precipitation,
        }
    }

    /// Returns `true` if the [`Climate`] is humid.
    #[must_use]
    pub const fn is_humid(&self) -> bool {
        self.downfall > Self::HUMID_THRESH
    }

    /// Returns the [`Climate's`](Climate) temperature.
    #[must_use]
    pub const fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Returns the [`Climate's`](Climate) downfall.
    #[must_use]
    pub const fn downfall(&self) -> f32 {
        self.downfall
    }

    /// Returns `true` if the precipitation can occur in the [`Climate`].
    #[must_use]
    pub const fn can_precipitate(&self) -> bool {
        self.precipitation
    }

    /// Returns `true` if rain can fall in the [`Climate`].
    #[must_use]
    pub const fn is_rainable(&self) -> bool {
        self.precipitation && (self.temperature >= Self::RAIN_TEMP_THRESH)
    }

    /// Returns `true` if snow can fall in the [`Climate`].
    #[must_use]
    pub const fn is_snowable(&self) -> bool {
        self.precipitation && (self.temperature < Self::RAIN_TEMP_THRESH)
    }
}
