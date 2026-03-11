mod ui_logic;
mod domain;
mod meshing;


use bevy::prelude::*;
use ui_logic::{setup_ui, handle_button_interaction, update_scene_objects, animate_timeline, update_ui_state};

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
            update_scene_objects,
            animate_timeline,
            update_ui_state,
            ui_logic::handle_unity_camera,
            ui_logic::manage_chunks,
        ))
        .run();
}
