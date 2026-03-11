mod voxel_world;
mod meshing;
mod camera_controller;
mod chunk_system;
mod collision;
mod ui_logic;

use bevy::prelude::*;
use ui_logic::{setup_ui, handle_button_interaction, update_ui_state};
use camera_controller::handle_unity_camera;
use chunk_system::{manage_chunks, process_chunk_tasks};

fn main() {
    App::new()
        // Window Setup
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "One O Net - Cyber Studio v0.9".into(),
                resolution: (1280.0, 720.0).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        // Startup
        .add_systems(Startup, setup_ui)
        // Update Loop
        .add_systems(Update, (
            handle_button_interaction,
            update_ui_state,
            handle_unity_camera,
            manage_chunks,
            process_chunk_tasks,
        ))
        .run();
}
