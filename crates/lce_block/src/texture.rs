use std::path::PathBuf;

/// Texture-mappings stored as paths.
///
/// Encodes no texture, a unified texture for each face, and a face for each
/// side of the block.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum BlockTextures {
    /// No texture.
    #[default]
    Air,

    /// A texture for each face of a block.
    SharedFace(PathBuf),

    /// A mapping of textures to each face of a block.
    AllFaces {
        /// A texture for the top face of the block.
        top: PathBuf,

        /// A texture for the front face of the block.
        front: PathBuf,

        /// A texture for the left face of the block.
        left: PathBuf,

        /// A texture for the right face of the block.
        right: PathBuf,

        /// A texture for the back face of the block.
        back: PathBuf,

        /// A texture for the bottom face of the block.
        bottom: PathBuf,
    },
}
