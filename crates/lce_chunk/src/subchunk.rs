use bevy::{asset::RenderAssetUsages, prelude::*};
use lce_palette::container::PVec;

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
    pub storage: PVec<u8>,
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
    /// use lce_palette::container::PVec;
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
    /// use lce_palette::container::PVec;
    ///
    /// # fn main() {
    /// #     doc().expect("there should be at least one value");
    /// # }
    /// #
    /// # fn doc() -> Option<()> {
    /// let mut storage = PVec::<u8>::new();
    /// storage.push(1);
    ///
    /// let (index, _) = storage.index_iter().next()?;
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
    ) {
        const FACE_OFFSETS: [[i16; 3]; 6] = [
            [0, 1, 0],  // +y
            [0, -1, 0], // -y
            [1, 0, 0],  // +x
            [-1, 0, 0], // -x
            [0, 0, 1],  // +z
            [0, 0, -1], // -z
        ];

        for (entity, subchunk) in query {
            info!("creating sub-chunk mesh");

            let mut positions = Vec::new();
            let mut indices = Vec::new();
            let mut normals = Vec::new();

            // Stub until blocks exist.
            for (idx, _block) in subchunk.storage.index_iter().filter(|(_, blk)| **blk != 0) {
                let (x, y, z) = Self::coords(idx);

                let (fx, fy, fz) = (f32::from(x), f32::from(y), f32::from(z));

                for [dx, dy, dz] in FACE_OFFSETS {
                    let nx = i16::from(x) + dx;
                    let ny = i16::from(y) + dy;
                    let nz = i16::from(z) + dz;

                    {
                        let valid_range = 0..i16::try_from(Self::SIZE).expect(
                            "The length of a sub-chunks axes should not exceed i16 precision",
                        );

                        if !(valid_range.contains(&nx)
                            && valid_range.contains(&ny)
                            && valid_range.contains(&nz))
                        {
                            continue;
                        }
                    }

                    if subchunk
                        .storage
                        .get(Self::index(
                            nx.try_into().expect("`n` should fit in an `u8`"),
                            ny.try_into().expect("`n` should fit in an `u8`"),
                            nz.try_into().expect("`n` should fit in an `u8`"),
                        ))
                        .unwrap_or(&0)
                        != &0
                    {
                        continue;
                    }

                    let (dx, dy, dz) = (f32::from(dx), f32::from(dy), f32::from(dz));
                    let cur_idx = u32::try_from(positions.len())
                        .expect("The number of positions should not exceed u32 precision");

                    #[rustfmt::skip]
                    positions.extend_from_slice(
                        &if dy != 0. {[
                            [fx,      fy + dy, fz     ],
                            [fx + 1., fy + dy, fz     ],
                            [fx + 1., fy + dy, fz + 1.],
                            [fx,      fy + dy, fz + 1.],
                        ]} else if dx != 0. {[
                            [fx + dx, fy,      fz     ],
                            [fx + dx, fy,      fz + 1.],
                            [fx + dx, fy + 1., fz + 1.],
                            [fx + dx, fy + 1., fz     ],
                        ]} else {[
                            [fx,      fy,      fz + dz],
                            [fx,      fy + 1., fz + dz],
                            [fx + 1., fy + 1., fz + dz],
                            [fx + 1., fy,      fz + dz],
                        ]}
                    );

                    normals.extend_from_slice(&[[dx, dy, dz]].repeat(4));

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

            info!(
                "generated sub-chunk with: indices: {}, tris: {}, quads: {}",
                indices.len(),
                indices.len() / 3,
                indices.len() / 6
            );

            let mesh = Mesh::new(
                bevy::mesh::PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(bevy::mesh::Indices::U32(indices));

            commands.entity(entity).insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
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
                storage: PVec::filled(1, 3),
            });
        });

        app.update();
    }
}
