//! The entry-point for Rust-port of 4jcraft.

use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    dev_tools::fps_overlay::FpsOverlayPlugin,
};
#[cfg(feature = "dev")]
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

fn main() -> AppExit {
    let mut app = App::new();

    app.add_plugins((DefaultPlugins, lce_chunk::LceChunkPlugin));

    #[cfg(feature = "dev")]
    app.add_plugins((
        EguiPlugin::default(),
        FreeCameraPlugin,
        WorldInspectorPlugin::new(),
        FpsOverlayPlugin::default(),
    ));

    // Exists such that the world inspector and fps overlay are visible; should
    // be removed once the player camera is added.
    #[cfg(feature = "dev")]
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((FreeCamera::default(), Camera3d::default()));
    });

    app.run()
}
