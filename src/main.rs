mod ui_logic;

use bevy::prelude::*;
use ui_logic::{setup_ui, handle_button_interaction};

fn main() {
    App::new()
        // ウィンドウの初期設定 (Window Plugin の構成)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Bevy UI - Sample Window".into(),
                resolution: (800u32, 600u32).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        // UI 設定 (View 構築)
        .add_systems(Startup, setup_ui)
        // システム (Orchestrator 登録)
        .add_systems(Update, handle_button_interaction)
        .run();
}
