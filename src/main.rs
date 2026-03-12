mod voxel_world;
mod meshing;
mod camera_controller;
mod chunk_system;
mod collision;
mod ui_logic;
mod tetra_world;
mod tetra_meshing;
mod tetra_chunk_system;

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use ui_logic::{setup_ui, handle_button_interaction, update_ui_state, update_fps_counter, AppState, EditorView};
use camera_controller::handle_unity_camera;
use chunk_system::{manage_chunks, process_chunk_tasks, ChunkRenderData};
use tetra_chunk_system::{manage_tetra_chunks, process_tetra_chunk_tasks, TetraChunkMarker};

fn is_cube_mode(state: Res<AppState>) -> bool {
    state.active_view == EditorView::Scene
}

fn is_tetra_mode(state: Res<AppState>) -> bool {
    state.active_view == EditorView::Tetra
}

fn toggle_chunk_visibility(
    state: Res<AppState>,
    mut cube_query: Query<&mut Visibility, (With<ChunkRenderData>, Without<TetraChunkMarker>)>,
    mut tetra_query: Query<&mut Visibility, (With<TetraChunkMarker>, Without<ChunkRenderData>)>,
) {
    if state.is_changed() || state.is_added() {
        let is_cube = state.active_view == EditorView::Scene;
        for mut vis in &mut cube_query {
            *vis = if is_cube { Visibility::Visible } else { Visibility::Hidden };
        }
        
        let is_tetra = state.active_view == EditorView::Tetra;
        for mut vis in &mut tetra_query {
            *vis = if is_tetra { Visibility::Visible } else { Visibility::Hidden };
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "One O Net - Cyber Studio v0.9".into(),
                resolution: (1280.0, 720.0).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
        // Startup
        .add_systems(Startup, setup_ui)
        // Update Loop
        .add_systems(Update, (
            handle_button_interaction,
            update_ui_state,
            handle_unity_camera,
            update_fps_counter,
            // 立方体モード (Scene tab)
            manage_chunks.run_if(is_cube_mode),
            process_chunk_tasks.run_if(is_cube_mode),
            // 四面体モード (Code tab)
            manage_tetra_chunks.run_if(is_tetra_mode),
            process_tetra_chunk_tasks.run_if(is_tetra_mode),
            // 表示切替
            toggle_chunk_visibility,
        ))
        .run();
}
