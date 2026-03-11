use bevy::prelude::*;
use crate::voxel_world::{self};

// プレイヤーの発光マーカー
#[derive(Component)]
pub struct PlayerLight;

// =============================================================================
// Constants: Theme & Nature (Premium Cyber Theme)
// =============================================================================
const COLOR_CYBER_BG: Color = Color::srgb(0.05, 0.07, 0.12);
const COLOR_CYBER_SIDEBAR: Color = Color::srgb(0.1, 0.1, 0.12);
const COLOR_CYBER_ACTIVITY: Color = Color::srgb(0.12, 0.12, 0.15);
const COLOR_CYBER_STATUS: Color = Color::srgb(0.0, 0.3, 0.6);
const COLOR_CYBER_BORDER: Color = Color::srgb(0.2, 0.2, 0.3);
const COLOR_CYBER_TEXT: Color = Color::srgb(0.85, 0.85, 0.9);
const COLOR_CYBER_ACCENT: Color = Color::srgb(0.0, 0.7, 1.0);
const COLOR_CYBER_GLASS: Color = Color::srgba(1.0, 1.0, 1.0, 0.05);

use crate::camera_controller::UnityCamera;
use crate::chunk_system::{ChunkManager, VoxelMaterial};
use crate::tetra_chunk_system::TetraChunkManager;

// =============================================================================
// State & Components
// =============================================================================
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum EditorView {
    #[default] Scene, // Cube World
    Tetra,  // Tetra World
    Code,  // VSCode Editor
}

#[derive(Resource)]
pub struct AppState {
    pub active_view: EditorView,
}

#[derive(Component)]
pub struct TabButton(pub EditorView);

#[derive(Component)]
pub struct ViewContainer(pub EditorView);

// =============================================================================
// UI Setup (Declarative SengenUI Style)
// =============================================================================
pub fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AppState { active_view: EditorView::Scene });
    commands.insert_resource(ChunkManager { 
        loaded_chunks: std::collections::HashMap::new(),
        loading_chunks: std::collections::HashSet::new(),
    });

    commands.insert_resource(TetraChunkManager {
        loaded_chunks: std::collections::HashMap::new(),
        loading_chunks: std::collections::HashSet::new(),
    });

    let fog_color = Color::srgb(0.53, 0.72, 0.9); // 青空

    // ボクセルの質感を定義
    let voxel_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: false, 
        perceptual_roughness: 0.9,
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });
    commands.insert_resource(VoxelMaterial(voxel_mat));

    // --- 1. Camera System ---
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(fog_color), 
            ..default()
        },
        Transform::from_xyz(8.0, 25.0, 8.0).looking_at(Vec3::new(10.0, 23.0, 10.0), Vec3::Y),
        UnityCamera { yaw: -0.4, pitch: -0.3, sensitivity: 0.002, speed: 5.0, velocity_y: 0.0, is_grounded: false, is_flying: false },
        DistanceFog {
            color: fog_color, 
            falloff: FogFalloff::Linear { 
                start: (8.0 * voxel_world::チャンクのワールドサイズ), 
                end: (19.0 * voxel_world::チャンクのワールドサイズ) 
            },
            ..default()
        }
    )).with_children(|parent| {
        // プレイヤー発光: 洞窟内でも視認できるポイントライト
        parent.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.95, 0.8),
                intensity: 80_000.0,
                range: 25.0,
                shadows_enabled: false, // パフォーマンスのため影なし
                ..default()
            },
            Transform::from_xyz(0.0, -0.5, 0.0), // カメラの少し下
            PlayerLight,
        ));
    });

    // --- 2. Sunlight ---
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0, // 明るい太陽光
            shadows_enabled: true,
            shadow_depth_bias: 0.1,
            shadow_normal_bias: 0.2,
            color: Color::srgb(1.0, 0.98, 0.9), // 暖かい日光
            ..default()
        },
        Transform::from_xyz(60.0, 120.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6, 0.7, 0.9), // 空色の環境光
        brightness: 800.0, // 大幅に明るく
    });

    // --- 3. UI Layout ---
    commands.spawn((
        Node { 
            width: Val::Percent(100.0), 
            height: Val::Percent(100.0), 
            flex_direction: FlexDirection::Column, 
            ..default() 
        },
    )).with_children(|root| {
        root.spawn((
            Node { 
                flex_grow: 1.0, 
                width: Val::Percent(100.0), 
                flex_direction: FlexDirection::Row, 
                ..default() 
            },
        )).with_children(|body| {
            // Activity Bar (Glass effect)
            body.spawn((
                Node { width: Val::Px(55.0), height: Val::Percent(100.0), border: UiRect::right(Val::Px(1.0)), ..default() }, 
                BackgroundColor(COLOR_CYBER_ACTIVITY),
                BorderColor(COLOR_CYBER_BORDER),
            ));

            // Sidebar
            body.spawn((
                Node { 
                    width: Val::Px(260.0), 
                    height: Val::Percent(100.0), 
                    flex_direction: FlexDirection::Column, 
                    border: UiRect::right(Val::Px(1.0)), 
                    ..default() 
                },
                BackgroundColor(COLOR_CYBER_SIDEBAR), BorderColor(COLOR_CYBER_BORDER),
            )).with_children(|sb| {
                sb.spawn(Node { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(16.0)), ..default() })
                    .with_child((Text::new("VOXEL ENGINE PRO"), TextFont { font_size: 12.0, ..default() }, TextColor(COLOR_CYBER_ACCENT)));
                
                let items = ["﹂📁 World Data", "﹂🛠️ Materials", "﹂✨ Post Process"];
                for item in items {
                    sb.spawn((
                        Node { 
                            width: Val::Percent(100.0), 
                            height: Val::Px(28.0), 
                            padding: UiRect::left(Val::Px(12.0)), 
                            align_items: AlignItems::Center, 
                            ..default() 
                        },
                    )).with_child((Text::new(item), TextFont { font_size: 13.0, ..default() }, TextColor(COLOR_CYBER_TEXT)));
                }
            });

            // Editor Area
            body.spawn((
                Node { flex_grow: 1.0, height: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() },
            )).with_children(|ed| {
                // Tab Bar
                ed.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(40.0), ..default() }, 
                    BackgroundColor(COLOR_CYBER_SIDEBAR),
                    BorderColor(COLOR_CYBER_BORDER),
                ))
                    .with_children(|tabs| {
                        let data = [("Cube World", EditorView::Scene), ("Tetra World", EditorView::Tetra), ("Code Editor", EditorView::Code)];
                        for (name, view) in data {
                            tabs.spawn((
                                Button,
                                Node { 
                                    width: Val::Px(150.0), 
                                    height: Val::Percent(100.0), 
                                    justify_content: JustifyContent::Center, 
                                    align_items: AlignItems::Center, 
                                    border: UiRect::right(Val::Px(1.0)), 
                                    ..default() 
                                },
                                BorderColor(COLOR_CYBER_BORDER), TabButton(view),
                            )).with_child((Text::new(name), TextFont { font_size: 13.0, ..default() }, TextColor(COLOR_CYBER_TEXT)));
                        }
                    });

                // Window Switcher
                ed.spawn(Node { flex_grow: 1.0, width: Val::Percent(100.0), ..default() }).with_children(|container| {
                    container.spawn((
                        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                        ViewContainer(EditorView::Scene),
                    )).with_children(|view| {
                        view.spawn(Node { position_type: PositionType::Absolute, top: Val::Px(15.0), left: Val::Px(15.0), ..default() })
                            .with_child((Text::new("CYBER MODE :: RIGHT-DRAG TO NAVIGATE"), TextFont { font_size: 11.0, ..default() }, TextColor(COLOR_CYBER_ACCENT)));
                    });

                    container.spawn((
                        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                        ViewContainer(EditorView::Tetra),
                    )).with_children(|view| {
                        view.spawn(Node { position_type: PositionType::Absolute, top: Val::Px(15.0), left: Val::Px(15.0), ..default() })
                            .with_child((Text::new("TETRA MODE :: TETRAHEDRAL VOXEL WORLD"), TextFont { font_size: 11.0, ..default() }, TextColor(COLOR_CYBER_ACCENT)));
                    });

                    container.spawn((
                        Node { 
                            position_type: PositionType::Absolute, 
                            width: Val::Percent(100.0), 
                            height: Val::Percent(100.0), 
                            padding: UiRect::all(Val::Px(24.0)), 
                            ..default() 
                        },
                        BackgroundColor(COLOR_CYBER_BG), ViewContainer(EditorView::Code),
                    )).with_child((Text::new("// Voxel Engine Core\n// Cube + Tetra dual mode\n\nfn initialize_world() {\n    let config = WorldConfig::default();\n    render_streamer.start(config);\n}"), TextFont { font_size: 15.0, ..default() }, TextColor(COLOR_CYBER_TEXT)));
                });
            });
        });

        // Status Bar
        root.spawn((
            Node { 
                width: Val::Percent(100.0), 
                height: Val::Px(24.0), 
                padding: UiRect::horizontal(Val::Px(12.0)), 
                align_items: AlignItems::Center, 
                ..default() 
            },
            BackgroundColor(COLOR_CYBER_STATUS),
        )).with_child((
            Text::new("CYBER STUDIO | LOD ENABLED | "), 
            TextFont { font_size: 11.0, ..default() }, 
            TextColor(Color::WHITE)
        )).with_child((
            Text::new("FPS: --"), 
            TextFont { font_size: 11.0, ..default() }, 
            TextColor(COLOR_CYBER_ACCENT),
            FPSCounter
        ));
    });
}

#[derive(Component)]
pub struct FPSCounter;

pub fn update_fps_counter(
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    mut query: Query<&mut Text, With<FPSCounter>>,
) {
    if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            for mut text in &mut query {
                text.0 = format!("FPS: {:.1}", value);
            }
        }
    }
}

pub fn handle_button_interaction(
    mut state: ResMut<AppState>,
    mut q: Query<(&Interaction, &mut BackgroundColor, Option<&TabButton>), With<Button>>,
) {
    for (int, mut bg, tab) in &mut q {
        match *int {
            Interaction::Pressed => {
                *bg = BackgroundColor(COLOR_CYBER_ACCENT);
                if let Some(t) = tab { state.active_view = t.0; }
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            Interaction::None => *bg = BackgroundColor(Color::NONE),
        }
    }
}

pub fn update_ui_state(
    state: Res<AppState>,
    mut q_view: Query<(&mut Node, &ViewContainer)>,
    mut q_tab: Query<(&mut BackgroundColor, &TabButton), With<Button>>,
) {
    for (mut node, container) in &mut q_view {
        node.display = if container.0 == state.active_view { Display::Flex } else { Display::None };
    }
    for (mut bg, tab) in &mut q_tab {
        if tab.0 == state.active_view { *bg = BackgroundColor(COLOR_CYBER_BG); }
    }
}
