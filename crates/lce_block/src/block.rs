use std::path::Path;

use crate::texture::BlockTextures;

/// A placeable, destroyable, and interact-able geometry in the world.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    ident: String,

    #[serde(rename = "faces")]
    #[serde(default)]
    textures: BlockTextures,
}

impl Block {
    /// Returns the name of this block.
    #[must_use]
    pub fn ident(&self) -> &str {
        &self.ident
    }

    /// Returns the texture-mappings for each face.
    #[must_use]
    pub const fn textures(&self) -> &BlockTextures {
        &self.textures
    }

    /// Returns the texture used for the front of this block.
    #[must_use]
    pub fn texture_front(&self) -> &Path {
        match self.textures {
            BlockTextures::SharedFace(ref p) => p,
            BlockTextures::AllFaces { ref front, .. } => front,
            BlockTextures::Air => panic!("attept to get texture of invalid state"),
        }
    }

    /// Returns the texture used for the back of this block.
    #[must_use]
    pub fn texture_back(&self) -> &Path {
        match self.textures {
            BlockTextures::SharedFace(ref p) => p,
            BlockTextures::AllFaces { ref back, .. } => back,
            BlockTextures::Air => panic!("attept to get texture of invalid state"),
        }
    }

    /// Returns the texture used for the left of this block.
    #[must_use]
    pub fn texture_left(&self) -> &Path {
        match self.textures {
            BlockTextures::SharedFace(ref p) => p,
            BlockTextures::AllFaces { ref left, .. } => left,
            BlockTextures::Air => panic!("attept to get texture of invalid state"),
        }
    }

    /// Returns the texture used for the right of this block.
    #[must_use]
    pub fn texture_right(&self) -> &Path {
        match self.textures {
            BlockTextures::SharedFace(ref p) => p,
            BlockTextures::AllFaces { ref right, .. } => right,
            BlockTextures::Air => panic!("attept to get texture of invalid state"),
        }
    }

    /// Returns the texture used for the top of this block.
    #[must_use]
    pub fn texture_top(&self) -> &Path {
        match self.textures {
            BlockTextures::SharedFace(ref p) => p,
            BlockTextures::AllFaces { ref top, .. } => top,
            BlockTextures::Air => panic!("attept to get texture of invalid state"),
        }
    }

    /// Returns the texture used for the bottom of this block.
    #[must_use]
    pub fn texture_bottom(&self) -> &Path {
        match self.textures {
            BlockTextures::SharedFace(ref p) => p,
            BlockTextures::AllFaces { ref bottom, .. } => bottom,
            BlockTextures::Air => panic!("attept to get texture of invalid state"),
        }
    }
}
