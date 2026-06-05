//! The entry-point for Rust-port of 4jcraft.

use bevy::{color::palettes::css::BLACK, pbr::DefaultOpaqueRendererMethod, prelude::*};
use lce_block::BlockId;
use lce_palette::collection::PVec;
use leafwing_manifest::{asset_state::SimpleAssetState, plugin::ManifestPlugin};

#[rustfmt::skip]
use {
    lce_asset::BlockAssetLoadingPlugin,
    lce_biome::LceBiomePlugin,
    lce_block::LceBlockPlugin,
    lce_chunk::LceChunkPlugin,
};

fn main() -> AppExit {
    let mut minecraft = App::new();

    minecraft.add_plugins((
        DefaultPlugins,
        ManifestPlugin::<SimpleAssetState>::default(),
    ));

    minecraft.add_plugins((
        BlockAssetLoadingPlugin,
        LceBiomePlugin,
        LceBlockPlugin,
        LceChunkPlugin,
    ));

    minecraft
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .insert_resource(ClearColor(Color::Srgba(BLACK)))
        .insert_resource(GlobalAmbientLight::NONE);

    #[cfg(feature = "dev")]
    {
        use bevy::{
            camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
            dev_tools::fps_overlay::FpsOverlayPlugin,
            pbr::ScatteringMedium,
        };
        use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

        minecraft.add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::default(),
            FpsOverlayPlugin::default(),
            FreeCameraPlugin,
        ));

        minecraft.add_systems(
            Startup,
            |mut commands: Commands, mut scattering_medium: ResMut<Assets<ScatteringMedium>>| {
                use bevy::{
                    anti_alias::fxaa::Fxaa,
                    camera::Exposure,
                    core_pipeline::tonemapping::Tonemapping,
                    light::{
                        AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, FogVolume,
                        VolumetricFog, VolumetricLight, light_consts::lux,
                    },
                    pbr::{Atmosphere, ScatteringMedium, ScreenSpaceReflections},
                    post_process::bloom::Bloom,
                };

                commands.spawn((
                    Camera3d::default(),
                    Transform::from_xyz(8.0, 16.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
                    FreeCamera {
                        sensitivity: 0.2,
                        friction: 25.0,
                        walk_speed: 3.0,
                        run_speed: 9.0,
                        ..default()
                    },
                    Atmosphere::earthlike(scattering_medium.add(ScatteringMedium::default())),
                    Exposure { ev100: 13.0 },
                    Tonemapping::AcesFitted,
                    Bloom::OLD_SCHOOL,
                    AtmosphereEnvironmentMapLight::default(),
                    VolumetricFog {
                        ambient_intensity: 0.0,
                        ..default()
                    },
                    Msaa::Off,
                    Fxaa::default(),
                    ScreenSpaceReflections::default(),
                ));

                commands.spawn((
                    DirectionalLight {
                        shadows_enabled: true,
                        illuminance: lux::DIRECT_SUNLIGHT,
                        ..default()
                    },
                    Transform::from_xyz(1.0, 0.4, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
                    VolumetricLight,
                    CascadeShadowConfigBuilder {
                        first_cascade_far_bound: 0.3,
                        maximum_distance: 15.0,
                        ..default()
                    }
                    .build(),
                ));

                commands.spawn((
                    FogVolume::default(),
                    Transform::from_scale(Vec3::new(10.0, 1.0, 10.0))
                        .with_translation(Vec3::Y * 0.5),
                ));
            },
        );
    }

    minecraft.add_systems(
        OnEnter(SimpleAssetState::Ready),
        |mut commands: Commands| {
            let mut storage = PVec::filled(BlockId::from_name("Ru4J:bedrock"), 16 * 16);

            for _ in 0..(16 * 16 * 2) {
                storage.push(BlockId::from_name("Ru4J:dirt"));
            }

            for _ in 0..(16 * 16) {
                storage.push(BlockId::from_name("Ru4J:grass"));
            }

            const RENDER_DIAMETER: usize = 8;

            for x in 0..RENDER_DIAMETER {
                for z in 0..RENDER_DIAMETER {
                    let x = x * 16;
                    let z = z * 16;

                    commands.spawn((
                        lce_chunk::Chunk::default(),
                        Transform::from_xyz(x as f32, 0.0, z as f32),
                        children![lce_chunk::SubChunk {
                            storage: storage.clone()
                        }],
                    ));
                }
            }
        },
    );

    minecraft.run()
}
