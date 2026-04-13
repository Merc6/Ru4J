use serde::{Deserialize, Serialize};

/// Biome modifiers to world-gen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Worldgen {
    depth: f32,
    scale: f32,
}

impl Worldgen {
    /// Constructs a new [`Worldgen`].
    #[must_use]
    pub const fn new(depth: f32, scale: f32) -> Self {
        Self { depth, scale }
    }

    /// The depth modifier for the [`Worldgen`].
    #[must_use]
    pub const fn depth(self) -> f32 {
        self.depth
    }

    /// The scale modifier for the [`Worldgen`].
    #[must_use]
    pub const fn scale(self) -> f32 {
        self.scale
    }
}
