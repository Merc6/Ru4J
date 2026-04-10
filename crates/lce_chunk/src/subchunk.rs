use bevy::{asset::RenderAssetUsages, prelude::*};
use lce_palette::container::PVec;

#[derive(Component, Debug, Default)]
#[require(Transform, InheritedVisibility)]
pub struct SubChunk {
    pub storage: PVec<u8>,
}

impl SubChunk {
    pub const SIZE: usize = 16;
    pub const SIZE_3: usize = Self::SIZE * Self::SIZE * Self::SIZE;

    #[must_use]
    pub const fn index(x: usize, y: usize, z: usize) -> usize {
        y * Self::SIZE * Self::SIZE + x * Self::SIZE + z
    }

    #[must_use]
    pub const fn coords(index: usize) -> (usize, usize, usize) {
        let z = index % Self::SIZE;
        let x = (index / Self::SIZE) % Self::SIZE;
        let y = index / (Self::SIZE * Self::SIZE);

        (x, y, z)
    }

    pub(crate) fn generate_mesh(
        query: Query<(Entity, &Self), Added<Self>>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        const FACE_OFFSETS: [[i32; 3]; 6] = [
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

                let (fx, fy, fz) = (x as f32, y as f32, z as f32);

                for [dx, dy, dz] in FACE_OFFSETS {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let nz = z as i32 + dz;

                    {
                        let valid_range = 0..(Self::SIZE as i32);

                        if !(valid_range.contains(&nx)
                            && valid_range.contains(&ny)
                            && valid_range.contains(&nz))
                        {
                            continue;
                        }
                    }

                    if subchunk
                        .storage
                        .get(Self::index(nx as usize, ny as usize, nz as usize))
                        .unwrap_or(&0)
                        != &0
                    {
                        continue;
                    }

                    let cur_idx: u32 = positions.len() as u32;

                    #[rustfmt::skip]
                    if dy != 0 {
                        positions.extend_from_slice(&[
                            [fx,      fy + dy as f32, fz     ],
                            [fx + 1., fy + dy as f32, fz     ],
                            [fx + 1., fy + dy as f32, fz + 1.],
                            [fx,      fy + dy as f32, fz + 1.],
                        ]);
                    } else if dx != 0 {
                        positions.extend_from_slice(&[
                            [fx + dx as f32, fy,      fz     ],
                            [fx + dx as f32, fy,      fz + 1.],
                            [fx + dx as f32, fy + 1., fz + 1.],
                            [fx + dx as f32, fy + 1., fz     ],
                        ]);
                    } else {
                        positions.extend_from_slice(&[
                            [fx,      fy,      fz + dz as f32],
                            [fx,      fy + 1., fz + dz as f32],
                            [fx + 1., fy + 1., fz + dz as f32],
                            [fx + 1., fy,      fz + dz as f32],
                        ]);
                    }

                    normals.extend_from_slice(&[
                        [dx as f32, dy as f32, dz as f32],
                        [dx as f32, dy as f32, dz as f32],
                        [dx as f32, dy as f32, dz as f32],
                        [dx as f32, dy as f32, dz as f32],
                    ]);

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
