use bevy::prelude::*;
use crate::ボクセル世界::{self};

// プレイヤーの発光マーカー
#[derive(Component)]
pub struct プレイヤーライト;

// =============================================================================
// 定数: テーマ & 自然 (プレミアムサイバーテーマ)
// =============================================================================
const サイバー背景色: Color = Color::srgb(0.05, 0.07, 0.12);
const サイバーサイドバー色: Color = Color::srgb(0.1, 0.1, 0.12);
const サイバーアクティビティ色: Color = Color::srgb(0.12, 0.12, 0.15);
const サイバーステータス色: Color = Color::srgb(0.0, 0.3, 0.6);
const サイバーボーダー色: Color = Color::srgb(0.2, 0.2, 0.3);
const サイバーテキスト色: Color = Color::srgb(0.85, 0.85, 0.9);
const サイバーアクセント色: Color = Color::srgb(0.0, 0.7, 1.0);
const サイバーガラス色: Color = Color::srgba(1.0, 1.0, 1.0, 0.05);

use crate::カメラ制御::カメラ操作;
use crate::チャンク管理::{チャンク管理者, ボクセル素材};
use crate::四面体チャンク管理::四面体チャンク管理者;

// =============================================================================
// 状態 & コンポーネント
// =============================================================================
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum エディタビュー {
    #[default] シーン,  // 立方体ワールド
    四面体,             // 四面体ワールド
    コード,             // VSCodeエディタ
}

#[derive(Resource)]
pub struct アプリ状態 {
    pub 現在のビュー: エディタビュー,
}

#[derive(Component)]
pub struct タブボタン(pub エディタビュー);

#[derive(Component)]
pub struct ビューコンテナ(pub エディタビュー);

// =============================================================================
// UI初期化 (宣言的SengenUIスタイル)
// =============================================================================
pub fn UI初期化(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(アプリ状態 { 現在のビュー: エディタビュー::シーン });
    commands.insert_resource(チャンク管理者 {
        読込済み: std::collections::HashMap::new(),
        読込中: std::collections::HashSet::new(),
    });

    commands.insert_resource(四面体チャンク管理者 {
        読込済み: std::collections::HashMap::new(),
        読込中: std::collections::HashSet::new(),
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
    commands.insert_resource(ボクセル素材(voxel_mat));

    // --- 1. カメラシステム ---
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(fog_color),
            ..default()
        },
        Transform::from_xyz(8.0, 25.0, 8.0).looking_at(Vec3::new(10.0, 23.0, 10.0), Vec3::Y),
        カメラ操作 { yaw: -0.4, pitch: -0.3, 感度: 0.002, 速度: 5.0, 垂直速度: 0.0, 接地中: false, 飛行中: false },
        DistanceFog {
            color: fog_color,
            falloff: FogFalloff::Linear {
                start: (8.0 * ボクセル世界::チャンクのワールドサイズ),
                end: (19.0 * ボクセル世界::チャンクのワールドサイズ)
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
            プレイヤーライト,
        ));
    });

    // --- 2. 太陽光 ---
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
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
        brightness: 800.0,
    });

    // --- 3. UIレイアウト ---
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
            // アクティビティバー (ガラスエフェクト)
            body.spawn((
                Node { width: Val::Px(55.0), height: Val::Percent(100.0), border: UiRect::right(Val::Px(1.0)), ..default() },
                BackgroundColor(サイバーアクティビティ色),
                BorderColor(サイバーボーダー色),
            ));

            // サイドバー
            body.spawn((
                Node {
                    width: Val::Px(260.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::right(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(サイバーサイドバー色), BorderColor(サイバーボーダー色),
            )).with_children(|sb| {
                sb.spawn(Node { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(16.0)), ..default() })
                    .with_child((Text::new("VOXEL ENGINE PRO"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバーアクセント色)));

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
                    )).with_child((Text::new(item), TextFont { font_size: 13.0, ..default() }, TextColor(サイバーテキスト色)));
                }
            });

            // エディタエリア
            body.spawn((
                Node { flex_grow: 1.0, height: Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default() },
            )).with_children(|ed| {
                // タブバー
                ed.spawn((
                    Node { width: Val::Percent(100.0), height: Val::Px(40.0), ..default() },
                    BackgroundColor(サイバーサイドバー色),
                    BorderColor(サイバーボーダー色),
                ))
                    .with_children(|tabs| {
                        let data = [("Cube World", エディタビュー::シーン), ("Tetra World", エディタビュー::四面体), ("Code Editor", エディタビュー::コード)];
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
                                BorderColor(サイバーボーダー色), タブボタン(view),
                            )).with_child((Text::new(name), TextFont { font_size: 13.0, ..default() }, TextColor(サイバーテキスト色)));
                        }
                    });

                // ウィンドウスイッチャー
                ed.spawn(Node { flex_grow: 1.0, width: Val::Percent(100.0), ..default() }).with_children(|container| {
                    container.spawn((
                        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                        ビューコンテナ(エディタビュー::シーン),
                    )).with_children(|view| {
                        view.spawn(Node { position_type: PositionType::Absolute, top: Val::Px(15.0), left: Val::Px(15.0), ..default() })
                            .with_child((Text::new("CYBER MODE :: RIGHT-DRAG TO NAVIGATE"), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーアクセント色)));
                    });

                    container.spawn((
                        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                        ビューコンテナ(エディタビュー::四面体),
                    )).with_children(|view| {
                        view.spawn(Node { position_type: PositionType::Absolute, top: Val::Px(15.0), left: Val::Px(15.0), ..default() })
                            .with_child((Text::new("TETRA MODE :: TETRAHEDRAL VOXEL WORLD"), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーアクセント色)));
                    });

                    container.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(24.0)),
                            ..default()
                        },
                        BackgroundColor(サイバー背景色), ビューコンテナ(エディタビュー::コード),
                    )).with_child((Text::new("// Voxel Engine Core\n// Cube + Tetra dual mode\n\nfn initialize_world() {\n    let config = WorldConfig::default();\n    render_streamer.start(config);\n}"), TextFont { font_size: 15.0, ..default() }, TextColor(サイバーテキスト色)));
                });
            });
        });

        // ステータスバー
        root.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                padding: UiRect::horizontal(Val::Px(12.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(サイバーステータス色),
        )).with_child((
            Text::new("CYBER STUDIO | LOD ENABLED | "),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::WHITE)
        )).with_child((
            Text::new("FPS: --"),
            TextFont { font_size: 11.0, ..default() },
            TextColor(サイバーアクセント色),
            FPSカウンター
        ));
    });
}

#[derive(Component)]
pub struct FPSカウンター;

pub fn FPS更新(
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    mut query: Query<&mut Text, With<FPSカウンター>>,
) {
    if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            for mut text in &mut query {
                text.0 = format!("FPS: {:.1}", value);
            }
        }
    }
}

pub fn ボタン操作処理(
    mut state: ResMut<アプリ状態>,
    mut q: Query<(&Interaction, &mut BackgroundColor, Option<&タブボタン>), With<Button>>,
) {
    for (int, mut bg, tab) in &mut q {
        match *int {
            Interaction::Pressed => {
                *bg = BackgroundColor(サイバーアクセント色);
                if let Some(t) = tab { state.現在のビュー = t.0; }
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            Interaction::None => *bg = BackgroundColor(Color::NONE),
        }
    }
}

pub fn UI状態更新(
    state: Res<アプリ状態>,
    mut q_view: Query<(&mut Node, &ビューコンテナ)>,
    mut q_tab: Query<(&mut BackgroundColor, &タブボタン), With<Button>>,
) {
    for (mut node, container) in &mut q_view {
        node.display = if container.0 == state.現在のビュー { Display::Flex } else { Display::None };
    }
    for (mut bg, tab) in &mut q_tab {
        if tab.0 == state.現在のビュー { *bg = BackgroundColor(サイバー背景色); }
    }
}
