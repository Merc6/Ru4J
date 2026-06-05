use bevy::{
    asset::RenderAssetUsages,
    math::{I16Vec3, U8Vec3},
    prelude::*,
};
use lce_asset::BlockAtlasData;
use lce_block::{Block, BlockId, BlockRegistry};
use lce_palette::collection::PVec;
use leafwing_manifest::manifest::Manifest;

/// A 16x by 16z by 16y section within a [`Chunk`](crate::chunk::Chunk).
///
/// Sub-chunks are how `Ru4J` divides chunks into sections, the reason this is
/// done is similar to why the game world is broken into chunks. Chunks - while
/// still a relatively small subsection of the world - are very large;
/// fortunately, chunks never - or very rarely - need to be operated on in
/// whole, and for that reason sub-chunks exist.
#[derive(Component, Debug, Default)]
#[require(Transform, InheritedVisibility)]
pub struct SubChunk {
    /// How blocks are stored in the [`SubChunk`].
    ///
    /// Under-the-hood, this uses [`PVec`] - a special collection built for
    /// storing large quantities of items with relatively little variance.
    /// Roughly four unique-block-sets can exist on the stack before being
    /// moved onto the heap.
    pub storage: PVec<BlockId>,
}

impl SubChunk {
    /// The length along any axis of a [`SubChunk`].
    pub const SIZE: usize = 16;

    /// The area of a [`SubChunk`].
    pub const SIZE_3: usize = Self::SIZE * Self::SIZE * Self::SIZE;

    /// Converts a [`Chunk`](crate::chunk::Chunk)-relative position to an index
    /// in a [`SubChunk's`](SubChunk) [`storage`](SubChunk::storage).
    ///
    /// # Example
    ///
    /// ```rust
    /// use lce_chunk::SubChunk;
    /// use lce_palette::collection::PVec;
    ///
    /// let origin = (0, 0, 0);
    /// let origin_idx = SubChunk::index(origin.0, origin.1, origin.2);
    ///
    /// let mut storage = PVec::<u8>::new();
    /// storage.push(1);
    ///
    /// assert_eq!(
    ///     origin_idx, 0,
    ///     "(0, 0, 0) should be the first item's index in storage"
    /// );
    ///
    /// assert_eq!(
    ///     storage.get(origin_idx),
    ///     Some(&1),
    ///     "1 should be the first stored item"
    /// );
    /// ```
    #[must_use]
    pub fn index(x: u8, y: u8, z: u8) -> usize {
        usize::from(y) * Self::SIZE * Self::SIZE + usize::from(x) * Self::SIZE + usize::from(z)
    }

    /// Converts a [`SubChunk's`](SubChunk) [`storage`](SubChunk::storage) index
    /// into a [`Chunk`](crate::chunk::Chunk)-relative position.
    ///
    /// # Panics
    ///
    /// Panics if the `index` is out of bounds for what may exist within a
    /// `SubChunk`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lce_chunk::SubChunk;
    /// use lce_palette::collection::PVec;
    ///
    /// # fn main() {
    /// #     doc().expect("there should be at least one value");
    /// # }
    /// #
    /// # fn doc() -> Option<()> {
    /// let mut storage = PVec::<u8>::new();
    /// storage.push(1);
    ///
    /// let (index, _) = storage.iter().enumerate().next()?;
    ///
    /// assert_eq!(
    ///     SubChunk::coords(index),
    ///     (0, 0, 0),
    ///     "the first item in storage should be at the origin"
    /// );
    /// #
    /// # Some(())
    /// # }
    /// ```
    #[must_use]
    pub fn coords(index: usize) -> (u8, u8, u8) {
        let z = index % Self::SIZE;
        let x = (index / Self::SIZE) % Self::SIZE;
        let y = index / (Self::SIZE * Self::SIZE);

        let (x, y, z) = (
            u8::try_from(x).expect("the chunk-relative `x` should cast into a `u8`"),
            u8::try_from(y).expect("the chunk-relative `y` should cast into a `u8`"),
            u8::try_from(z).expect("the chunk-relative `z` should cast into a `u8`"),
        );

        (x, y, z)
    }

    pub(crate) fn generate_mesh(
        query: Query<(Entity, &Self), Added<Self>>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        asset_server: Res<AssetServer>,
        block_sprites: Res<BlockAtlasData>,
        block_registry: Res<BlockRegistry>,
        texture_atlases: Res<Assets<TextureAtlasLayout>>,
    ) {
        let atlas_size = texture_atlases
            .get(&block_sprites.layout)
            .unwrap()
            .size
            .as_vec2();

        for (entity, subchunk) in query {
            let mut positions = Vec::new();
            let mut indices = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();

            for (idx, &block_id) in subchunk
                .storage
                .iter()
                .enumerate()
                .filter(|(_, blk)| !BlockRegistry::is_air(**blk))
            {
                let (x, y, z) = Self::coords(idx);

                let block = block_registry.get(block_id).unwrap();

                for face in Face::ALL {
                    let [dx, dy, dz] = face.unpack_offsets();

                    let nx = i16::from(x) + dx;
                    let ny = i16::from(y) + dy;
                    let nz = i16::from(z) + dz;

                    let valid_range = 0..(i16::try_from(Self::SIZE).unwrap());

                    if valid_range.contains(&nx)
                        && valid_range.contains(&ny)
                        && valid_range.contains(&nz)
                    {
                        if subchunk
                            .storage
                            .get(Self::index(
                                nx.try_into().expect("`nx` should fit in an `u8`"),
                                ny.try_into().expect("`ny` should fit in an `u8`"),
                                nz.try_into().expect("`nz` should fit in an `u8`"),
                            ))
                            .is_some_and(|&id| id != BlockRegistry::air_id())
                        {
                            continue;
                        }
                    } else {
                        // neighbor checks
                    }

                    let block_texture_handle = face.texture_for(&asset_server, block).unwrap();

                    let atlas_index = block_sprites
                        .sources
                        .handle(block_sprites.layout.clone(), &block_texture_handle)
                        .unwrap();

                    let rect = atlas_index.texture_rect(&texture_atlases).unwrap();

                    let u = rect.min.with_y(rect.max.x).as_vec2() / atlas_size;
                    let v = rect.max.with_x(rect.min.y).as_vec2() / atlas_size;
                    uvs.extend_from_slice(&face.uvs(u, v));

                    let cur_idx = u32::try_from(positions.len())
                        .expect("The number of positions should not exceed u32 precision");

                    positions.extend_from_slice(&face.positions(U8Vec3::new(x, y, z).as_vec3()));
                    normals.extend_from_slice(&[face.offset().as_vec3(); 4]);

                    indices.extend_from_slice(&[
                        cur_idx,
                        cur_idx + 3,
                        cur_idx + 1,
                        cur_idx + 1,
                        cur_idx + 3,
                        cur_idx + 2,
                    ]);
                }
            }

            if positions.is_empty() {
                continue;
            }

            let mesh = Mesh::new(
                bevy::mesh::PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_indices(bevy::mesh::Indices::U32(indices));

            commands.entity(entity).insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(block_sprites.texture.clone()),
                    perceptual_roughness: 1.0,
                    anisotropy_rotation: 0.5,
                    anisotropy_strength: 1.0,
                    ..default()
                })),
            ));
        }
    }
}

#[cfg(test)]
mod test {
    use bevy::{mesh::MeshPlugin, prelude::*};

    use super::*;

    #[test]
    fn coordinate_conversion_index_first() {
        for y in 0..SubChunk::SIZE {
            for x in 0..SubChunk::SIZE {
                for z in 0..SubChunk::SIZE {
                    let (x, y, z) = (
                        u8::try_from(x).expect("the chunk-relative `x` should cast into a `u8`"),
                        u8::try_from(y).expect("the chunk-relative `y` should cast into a `u8`"),
                        u8::try_from(z).expect("the chunk-relative `z` should cast into a `u8`"),
                    );

                    let idx = SubChunk::index(x, y, z);
                    assert_eq!(SubChunk::coords(idx), (x, y, z));
                }
            }
        }
    }

    #[test]
    fn coordinate_conversion_coords_first() {
        for idx in 0..SubChunk::SIZE_3 {
            let (x, y, z) = SubChunk::coords(idx);
            assert_eq!(SubChunk::index(x, y, z), idx);
        }
    }

    #[test]
    fn mesh_generation() {
        let mut app = App::new();

        app.add_plugins((
            AssetPlugin::default(),
            MeshPlugin,
            MaterialPlugin::<StandardMaterial>::default(),
        ))
        .add_systems(PreUpdate, SubChunk::generate_mesh)
        .add_systems(
            PostUpdate,
            |sub_chunk_mesh: Single<&Mesh3d, With<SubChunk>>, meshes: Res<Assets<Mesh>>| {
                let mesh = meshes
                    .get(sub_chunk_mesh.id())
                    .expect("mesh should exist for the subchunk");

                let indices = mesh.indices().expect("mesh should have indices");

                // 7 quads because of how our culling works.
                //
                // This first starts out as 3 cubes in a row, which is 18 quads
                // total. All the quads on chunk boarders are culled, leaving
                // 11 quads. Quads that cannot be seen are then culled, leaving
                // 7 quads.
                assert_eq!(indices.len() / 6, 7, "there should be seven quads");
            },
        )
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(SubChunk {
                storage: PVec::filled(BlockId::from_name("Ru4J:grass"), 3),
            });
        });

        app.update();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Face {
    PosY,
    NegY,
    PosX,
    NegX,
    PosZ,
    NegZ,
}

impl Face {
    pub const ALL: [Self; 6] = [
        Self::PosY,
        Self::NegY,
        Self::PosX,
        Self::NegX,
        Self::PosZ,
        Self::NegZ,
    ];

    #[must_use]
    const fn offset(self) -> I16Vec3 {
        match self {
            Self::PosY => I16Vec3::Y,
            Self::NegY => I16Vec3::NEG_Y,
            Self::PosX => I16Vec3::X,
            Self::NegX => I16Vec3::NEG_X,
            Self::PosZ => I16Vec3::Z,
            Self::NegZ => I16Vec3::NEG_Z,
        }
    }

    #[must_use]
    const fn unpack_offsets(self) -> [i16; 3] {
        let offs = self.offset();
        [offs.x, offs.y, offs.z]
    }

    #[must_use]
    fn texture_for(self, asset_server: &AssetServer, blk: &Block) -> Option<Handle<Image>> {
        let asset_path = match self {
            Self::PosY => blk.texture_top(),
            Self::NegY => blk.texture_bottom(),
            Self::PosX => blk.texture_right(),
            Self::NegX => blk.texture_left(),
            Self::PosZ => blk.texture_front(),
            Self::NegZ => blk.texture_back(),
        }
        .to_owned();

        asset_server.get_handle(asset_path)
    }

    fn uvs(self, u: Vec2, v: Vec2) -> [Vec2; 4] {
        let uv00 = Vec2::new(u.x, v.x);
        let uv01 = Vec2::new(u.x, v.y);
        let uv10 = Vec2::new(u.y, v.x);
        let uv11 = Vec2::new(u.y, v.y);

        match self {
            Self::PosY => [uv10, uv00, uv01, uv11],
            Self::NegY => [uv10, uv11, uv01, uv00],
            Self::PosX => [uv01, uv11, uv10, uv00],
            Self::NegX => [uv01, uv00, uv10, uv11],
            Self::PosZ => [uv01, uv00, uv10, uv11],
            Self::NegZ => [uv01, uv11, uv10, uv00],
        }
    }

    fn positions(self, block_pos: Vec3) -> [Vec3; 4] {
        match self {
            Self::PosX => [
                block_pos + Vec3::X,
                block_pos + Vec3::ONE - Vec3::Y,
                block_pos + Vec3::ONE,
                block_pos + Vec3::ONE - Vec3::Z,
            ],
            Self::NegX => [
                block_pos,
                block_pos + Vec3::Y,
                block_pos + Vec3::ONE - Vec3::X,
                block_pos + Vec3::Z,
            ],
            Self::PosY => [
                block_pos + Vec3::Y,
                block_pos + Vec3::ONE - Vec3::Z,
                block_pos + Vec3::ONE,
                block_pos + Vec3::ONE - Vec3::X,
            ],
            Self::NegY => [
                block_pos,
                block_pos + Vec3::Z,
                block_pos + Vec3::ONE - Vec3::Y,
                block_pos + Vec3::X,
            ],
            Self::PosZ => [
                block_pos + Vec3::Z,
                block_pos + Vec3::ONE - Vec3::X,
                block_pos + Vec3::ONE,
                block_pos + Vec3::ONE - Vec3::Y,
            ],
            Self::NegZ => [
                block_pos,
                block_pos + Vec3::X,
                block_pos + Vec3::ONE - Vec3::Z,
                block_pos + Vec3::Y,
            ],
        }
    }
}
