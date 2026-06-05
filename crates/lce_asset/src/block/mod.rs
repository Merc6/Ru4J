//! Block asset management

use bevy::{
    asset::LoadedFolder,
    image::{ImageSampler, ImageSamplerDescriptor},
    prelude::*,
};

/// The plugin responsible for initializing block assets.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct BlockAssetLoadingPlugin;

impl Plugin for BlockAssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<BlockAssetLoadState>()
            .add_systems(OnEnter(BlockAssetLoadState::LoadingFolder), load_textures)
            .add_systems(
                Update,
                check_textures.run_if(in_state(BlockAssetLoadState::LoadingFolder)),
            )
            .add_systems(OnEnter(BlockAssetLoadState::BuildingAtlas), setup);
    }
}

/// Loading states for the block assets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum BlockAssetLoadState {
    /// The folder is currently being parsed for textures.
    #[default]
    LoadingFolder,

    /// The atlas is being constructed from the textures in the folder.
    BuildingAtlas,

    /// The atlas has been constructed, and block-assets are ready to be used.
    Finished,
}

#[derive(Resource, Default)]
struct BlockSpriteFolder(Handle<LoadedFolder>);

/// Data regarding the block texture-atlas.
#[derive(Resource)]
pub struct BlockAtlasData {
    /// handle to the layout for the block texture-atlas.
    pub layout: Handle<TextureAtlasLayout>,

    /// The mapping of texture-handles to their related area-index.
    pub sources: TextureAtlasSources,

    /// The handle to the texture-atlas image.
    pub texture: Handle<Image>,
}

#[expect(clippy::needless_pass_by_value)]
fn load_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BlockSpriteFolder(
        asset_server.load_folder("textures/blocks"),
    ));
}

#[expect(clippy::needless_pass_by_value)]
fn check_textures(
    mut next_state: ResMut<NextState<BlockAssetLoadState>>,
    block_sprite_folder: Res<BlockSpriteFolder>,
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
) {
    for () in events
        .read()
        .filter(|ev| ev.is_loaded_with_dependencies(&block_sprite_folder.0))
        .map(drop)
    {
        next_state.set(BlockAssetLoadState::BuildingAtlas);
    }
}

#[expect(clippy::needless_pass_by_value)]
fn setup(
    mut next_state: ResMut<NextState<BlockAssetLoadState>>,
    mut commands: Commands,
    block_sprite_handles: Res<BlockSpriteFolder>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let loaded_folder = loaded_folders.get(&block_sprite_handles.0).expect(
        "Block sprites should be loaded at this point; Shouldn't be allowed to \
         advance to this state until they are loaded.",
    );

    let (texture_atlas, sources, texture) = create_texture_atlas(
        loaded_folder,
        None,
        Some(ImageSampler::Descriptor(ImageSamplerDescriptor {
            label: Some("voxel_nearest_sampler".into()),
            mag_filter: bevy::image::ImageFilterMode::Nearest,
            min_filter: bevy::image::ImageFilterMode::Linear,
            mipmap_filter: bevy::image::ImageFilterMode::Linear,
            address_mode_u: bevy::image::ImageAddressMode::ClampToEdge,
            address_mode_v: bevy::image::ImageAddressMode::ClampToEdge,
            address_mode_w: bevy::image::ImageAddressMode::ClampToEdge,
            ..default()
        })),
        &mut textures,
    );

    commands.insert_resource(BlockAtlasData {
        layout: texture_atlases.add(texture_atlas),
        sources,
        texture,
    });

    next_state.set(BlockAssetLoadState::Finished);
}

fn create_texture_atlas(
    folder: &LoadedFolder,
    padding: Option<UVec2>,
    sampling: Option<ImageSampler>,
    textures: &mut ResMut<Assets<Image>>,
) -> (TextureAtlasLayout, TextureAtlasSources, Handle<Image>) {
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    texture_atlas_builder.padding(padding.unwrap_or_default());

    for handle in &folder.handles {
        let id = handle.id().typed_unchecked::<Image>();

        let Some(texture) = textures.get(id) else {
            warn!(
                "{} did not resolve to an `Image` asset.",
                handle.path().unwrap()
            );

            continue;
        };

        texture_atlas_builder.add_texture(Some(id), texture);
    }

    let (texture_atlas_layout, texture_atlas_sources, texture) =
        texture_atlas_builder.build().unwrap();
    let texture = textures.add(texture);

    let image = textures.get_mut(&texture).unwrap();
    image.sampler = sampling.unwrap_or_default();

    (texture_atlas_layout, texture_atlas_sources, texture)
}
