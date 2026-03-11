use bevy::prelude::*;
use std::collections::HashMap;

use bevy::input::mouse::MouseMotion;

// =============================================================================
// Constants: Theme & Nature (Brighter and Vibrant)
// =============================================================================
const COLOR_VS_BG: Color = Color::srgb(0.12, 0.12, 0.12);
const COLOR_VS_SIDEBAR: Color = Color::srgb(0.15, 0.15, 0.15);
const COLOR_VS_ACTIVITY: Color = Color::srgb(0.2, 0.2, 0.2);
const COLOR_VS_STATUS: Color = Color::srgb(0.0, 0.48, 0.8);
const COLOR_VS_BORDER: Color = Color::srgb(0.18, 0.18, 0.18);
const COLOR_VS_TEXT: Color = Color::srgb(0.8, 0.8, 0.82);
const COLOR_VS_ACCENT: Color = Color::srgb(0.0, 0.58, 1.0);
const COLOR_BLENDER_ORANGE: Color = Color::srgb(0.9, 0.47, 0.13);

use crate::domain;
use crate::meshing;

// =============================================================================
// Settings & Config
// =============================================================================
pub const RENDER_DISTANCE: i32 = 4; // カメラ周囲 何チャンク先まで描画するか

#[derive(Resource)]
pub struct ChunkManager {
    // 描画済みチャンクの座標とEntityIDを管理
    pub loaded_chunks: HashMap<(i32, i32, i32), Entity>,
}

#[derive(Component)]
pub struct ChunkRenderData {
    pub cx: i32,
    pub cy: i32,
    pub cz: i32,
}

// =============================================================================
// State & Components
// =============================================================================
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum EditorView {
    #[default] Scene, // Unity Scene View
    Code,  // VSCode Editor
    System, // System Logs
}

#[derive(Resource)]
pub struct AppState {
    pub active_view: EditorView,
}

#[derive(Component)]
pub struct TabButton(pub EditorView);

#[derive(Component)]
pub struct ViewContainer(pub EditorView);

#[derive(Component)]
pub struct UnityCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub speed: f32,
}

// =============================================================================
// UI & Scene Setup
// =============================================================================
pub fn setup_ui(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(AppState { active_view: EditorView::Scene });
    commands.insert_resource(ChunkManager { loaded_chunks: HashMap::new() });

    // --- 1. Camera System (Unity Style) ---
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.5, 0.7, 1.0)), // Beautiful blue sky
            ..default()
        },
        Transform::from_xyz(8.0, 15.0, 8.0).looking_at(Vec3::new(10.0, 12.0, 10.0), Vec3::Y),
        UnityCamera { yaw: -0.4, pitch: -0.3, sensitivity: 0.002, speed: 10.0 }, // スピード少し自然に
        // マイクラ風のアプローチ：描画境界を霧で隠す
        DistanceFog {
            color: Color::srgb(0.6, 0.75, 0.9), // Bright sky blue fog
            falloff: FogFalloff::Linear { 
                start: (RENDER_DISTANCE as f32 * domain::CHUNK_WORLD_SIZE) * 0.4, 
                end: (RENDER_DISTANCE as f32 * domain::CHUNK_WORLD_SIZE) * 0.9
            },
            ..default()
        }
    ));

    // --- 2. Environment (Sun & Lights) ---
    commands.spawn((
        DirectionalLight {
            illuminance: 25_000.0, // Bright sunlight
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.98, 0.9),
            ..default()
        },
        Transform::from_xyz(50.0, 100.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Fill Light for brighter ambient look
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.8, 0.9, 1.0),
        brightness: 400.0, 
    });

    // --- 3. UI Layout (VSCode Frame) ---
    // (UI setup...省略せずにそのまま残します)
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() },
    )).with_children(|root| {
        root.spawn((
            Node { flex_grow: 1.0, width: Val::Percent(100.0), flex_direction: FlexDirection::Row, ..default() },
        )).with_children(|body| {
            // Activity Bar
            body.spawn((Node { width: Val::Px(50.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(COLOR_VS_ACTIVITY)));

            // Sidebar
            body.spawn((
                Node { width: Val::Px(240.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, border: UiRect::right(Val::Px(1.0)), ..default() },
                BackgroundColor(COLOR_VS_SIDEBAR), BorderColor(COLOR_VS_BORDER),
            )).with_children(|sb| {
                sb.spawn((Node { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(12.0)), ..default() }))
                    .with_child((Text::new("VOXEL HIERARCHY"), TextFont { font_size: 11.0, ..default() }, TextColor(COLOR_VS_ACCENT)));
                let items = ["﹂📦 Dynamic Chunks", "﹂☀️ Sunlight"];
                for item in items {
                    sb.spawn((Node { width: Val::Percent(100.0), height: Val::Px(22.0), padding: UiRect::left(Val::Px(10.0)), align_items: AlignItems::Center, ..default() }))
                        .with_child((Text::new(item), TextFont { font_size: 13.0, ..default() }, TextColor(COLOR_VS_TEXT)));
                }
            });

            // Editor
            body.spawn((
                Node { flex_grow: 1.0, height: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() },
            )).with_children(|ed| {
                // Tab Bar
                ed.spawn((Node { width: Val::Percent(100.0), height: Val::Px(35.0), ..default() }, BackgroundColor(COLOR_VS_SIDEBAR)))
                    .with_children(|tabs| {
                        let data = [("Scene View", EditorView::Scene), ("domain.rs", EditorView::Code)];
                        for (name, view) in data {
                            tabs.spawn((
                                Button,
                                Node { width: Val::Px(120.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::right(Val::Px(1.0)), ..default() },
                                BorderColor(COLOR_VS_BORDER), TabButton(view),
                            )).with_child((Text::new(name), TextFont { font_size: 13.0, ..default() }, TextColor(COLOR_VS_TEXT)));
                        }
                    });

                // Window Switcher
                ed.spawn((Node { flex_grow: 1.0, width: Val::Percent(100.0), ..default() })).with_children(|container| {
                    container.spawn((
                        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                        ViewContainer(EditorView::Scene),
                    )).with_children(|view| {
                        view.spawn((Node { position_type: PositionType::Absolute, top: Val::Px(10.0), left: Val::Px(10.0), ..default() }))
                            .with_child((Text::new("UNITY MODE :: HOLD RIGHT MOUSE TO MOVE"), TextFont { font_size: 11.0, ..default() }, TextColor(COLOR_VS_ACCENT)));
                    });

                    container.spawn((
                        Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), padding: UiRect::all(Val::Px(20.0)), ..default() },
                        BackgroundColor(COLOR_VS_BG), ViewContainer(EditorView::Code),
                    )).with_child((Text::new("// Voxel Engine\npub fn chunk_system() {\n    // Dynamic loading\n}"), TextFont { font_size: 14.0, ..default() }, TextColor(COLOR_VS_TEXT)));
                });
            });
        });

        // Status Bar
        root.spawn((
            Node { width: Val::Percent(100.0), height: Val::Px(22.0), padding: UiRect::horizontal(Val::Px(10.0)), align_items: AlignItems::Center, ..default() },
            BackgroundColor(COLOR_VS_STATUS),
        )).with_child((Text::new("● Engine: Voxel High-Res | Dynamic Streaming: Active"), TextFont { font_size: 12.0, ..default() }, TextColor(Color::WHITE)));
    });
}

// =============================================================================
// Systems
// =============================================================================

// 動的チャンク管理システム (Minecraft Style Open World)
pub fn manage_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<UnityCamera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return };
    
    // カメラの現在チャンク座標を計算
    let cam_p = camera_transform.translation;
    let center_cx = (cam_p.x / domain::CHUNK_WORLD_SIZE).floor() as i32;
    // 今回は平面オープンワールドを想定し、cy=0固定付近でロードする。必要ならY軸のチャンクも管理します。
    let center_cy = 0; 
    let center_cz = (cam_p.z / domain::CHUNK_WORLD_SIZE).floor() as i32;

    let mut needed_chunks = std::collections::HashSet::new();

    // 描画範囲内のチャンクをリストアップ
    for dx in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for dz in -RENDER_DISTANCE..=RENDER_DISTANCE {
            if dx * dx + dz * dz > RENDER_DISTANCE * RENDER_DISTANCE { continue; } // 円形にロード
            needed_chunks.insert((center_cx + dx, center_cy, center_cz + dz));
        }
    }

    // 1. 不要になったチャンクをアンロード（破棄）
    chunk_manager.loaded_chunks.retain(|&pos, &mut entity| {
        if needed_chunks.contains(&pos) {
            true // 維持
        } else {
            commands.entity(entity).despawn_recursive(); // メッシュごと破棄
            false // マネージャから削除
        }
    });

    // 2. 新しく必要なチャンクをロード（生成）
    // （※本来は別スレッドや非同期タスクで生成すべきですが、今回は同期処理で簡易的に生成します）
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: false, 
        perceptual_roughness: 0.9,
        alpha_mode: AlphaMode::Opaque, // 透過による描画順バグを防ぐため不透明に固定
        cull_mode: None,              // 洞窟内部（裏側）も描画されるように
        ..default()
    });

    for &pos in &needed_chunks {
        if !chunk_manager.loaded_chunks.contains_key(&pos) {
            // Layer 1: ドメインによる純粋な地形生成
            let new_chunk = domain::Chunk::new_hilly_terrain(pos.0, pos.1, pos.2);
            // Layer 2: アダプターによるメッシュ変換
            let mesh = meshing::generate_naive_mesh(&new_chunk);
            
            // Layer 3: ECS上にデプロイ
            let entity = commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(
                    pos.0 as f32 * domain::CHUNK_WORLD_SIZE,
                    pos.1 as f32 * domain::CHUNK_WORLD_SIZE,
                    pos.2 as f32 * domain::CHUNK_WORLD_SIZE,
                ),
                ChunkRenderData { cx: pos.0, cy: pos.1, cz: pos.2 },
            )).id();

            chunk_manager.loaded_chunks.insert(pos, entity);
        }
    }
}

// Unity Style Camera Controller (WASD + Right Drag)
pub fn handle_unity_camera(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut UnityCamera)>,
) {
    let (mut transform, mut cam) = query.single_mut();

    // Unity Scene View: Movement and Rotation only active while HOLDING Right Click
    if mouse_button.pressed(MouseButton::Right) {
        // 1. Rotation
        for ev in mouse_motion.read() {
            cam.yaw -= ev.delta.x * cam.sensitivity;
            cam.pitch -= ev.delta.y * cam.sensitivity;
        }

        // 2. Movement (WASD + QE)
        let move_speed = cam.speed * time.delta_secs() * (if keys.pressed(KeyCode::ShiftLeft) { 3.0 } else { 1.0 });
        let mut velocity = Vec3::ZERO;
        let forward = *transform.forward();
        let right = *transform.right();

        if keys.pressed(KeyCode::KeyW) { velocity += forward; }
        if keys.pressed(KeyCode::KeyS) { velocity -= forward; }
        if keys.pressed(KeyCode::KeyA) { velocity -= right; }
        if keys.pressed(KeyCode::KeyD) { velocity += right; }
        if keys.pressed(KeyCode::KeyQ) { velocity -= Vec3::Y; }
        if keys.pressed(KeyCode::KeyE) { velocity += Vec3::Y; }

        // --- 3. Buoyancy (Underwater) ---
        if transform.translation.y < domain::WATER_LEVEL {
            velocity += Vec3::Y * 0.5; // 浮力で少し浮き上がる
            if keys.pressed(KeyCode::Space) {
                velocity += Vec3::Y * 1.5; // 水中での上昇
            }
        }

        if velocity.length() > 0.0 {
            transform.translation += velocity.normalize() * move_speed;
        }
    }

    cam.pitch = cam.pitch.clamp(-1.5, 1.5);
    transform.rotation = Quat::from_axis_angle(Vec3::Y, cam.yaw) * Quat::from_axis_angle(Vec3::X, cam.pitch);
}

pub fn handle_button_interaction(
    mut state: ResMut<AppState>,
    mut q: Query<(&Interaction, &mut BackgroundColor, Option<&TabButton>), With<Button>>,
) {
    for (int, mut bg, tab) in &mut q {
        match *int {
            Interaction::Pressed => {
                *bg = BackgroundColor(COLOR_VS_ACCENT);
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
        if tab.0 == state.active_view { *bg = BackgroundColor(COLOR_VS_BG); }
    }
}

pub fn update_scene_objects() {}
pub fn animate_timeline() {}
