// src/UI設定.rs
// UIレイアウト構築・イベント処理

use bevy::prelude::*;

// =============================================================================
// サイバーテーマ配色
// =============================================================================
const サイバー背景色: Color = Color::srgb(0.05, 0.07, 0.12);
const サイバーサイドバー色: Color = Color::srgb(0.1, 0.1, 0.12);
const サイバーアクティビティ色: Color = Color::srgb(0.12, 0.12, 0.15);
const サイバーステータス色: Color = Color::srgb(0.0, 0.3, 0.6);
const サイバーボーダー色: Color = Color::srgb(0.2, 0.2, 0.3);
const サイバーテキスト色: Color = Color::srgb(0.85, 0.85, 0.9);
const サイバーアクセント色: Color = Color::srgb(0.0, 0.7, 1.0);

// =============================================================================
// 状態・コンポーネント
// =============================================================================

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum エディタビュー {
    #[default] シーン,
    四面体,
    コード,
}

#[derive(Resource)]
pub struct アプリ状態 {
    pub 現在のビュー: エディタビュー,
}

#[derive(Component)]
pub struct タブボタン(pub エディタビュー);

#[derive(Component)]
pub struct ビューコンテナ(pub エディタビュー);

#[derive(Component)]
pub struct FPSカウンター;

// =============================================================================
// UI構築 (Startup System)
// =============================================================================

pub fn UI初期化(mut commands: Commands) {
    commands.insert_resource(アプリ状態 { 現在のビュー: エディタビュー::シーン });

    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        ..default()
    }).with_children(|root| {
        // 本体 (アクティビティバー + サイドバー + エディタ)
        root.spawn(Node {
            flex_grow: 1.0,
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        }).with_children(|body| {
            アクティビティバー構築(body);
            サイドバー構築(body);
            エディタエリア構築(body);
        });

        // ステータスバー
        ステータスバー構築(root);
    });
}

fn アクティビティバー構築(parent: &mut ChildBuilder) {
    parent.spawn((
        Node { width: Val::Px(55.0), height: Val::Percent(100.0), border: UiRect::right(Val::Px(1.0)), ..default() },
        BackgroundColor(サイバーアクティビティ色),
        BorderColor(サイバーボーダー色),
    ));
}

fn サイドバー構築(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Px(260.0), height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            border: UiRect::right(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(サイバーサイドバー色),
        BorderColor(サイバーボーダー色),
    )).with_children(|sb| {
        sb.spawn(Node { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(16.0)), ..default() })
            .with_child((Text::new("VOXEL ENGINE PRO"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバーアクセント色)));

        for item in ["﹂📁 World Data", "﹂🛠️ Materials", "﹂✨ Post Process"] {
            sb.spawn(Node {
                width: Val::Percent(100.0), height: Val::Px(28.0),
                padding: UiRect::left(Val::Px(12.0)),
                align_items: AlignItems::Center,
                ..default()
            }).with_child((Text::new(item), TextFont { font_size: 13.0, ..default() }, TextColor(サイバーテキスト色)));
        }
    });
}

fn エディタエリア構築(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        flex_grow: 1.0, height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        ..default()
    }).with_children(|ed| {
        // タブバー
        ed.spawn((
            Node { width: Val::Percent(100.0), height: Val::Px(40.0), ..default() },
            BackgroundColor(サイバーサイドバー色),
            BorderColor(サイバーボーダー色),
        )).with_children(|tabs| {
            for (name, view) in [("Cube World", エディタビュー::シーン), ("Tetra World", エディタビュー::四面体), ("Code Editor", エディタビュー::コード)] {
                tabs.spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0), height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::right(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(サイバーボーダー色),
                    タブボタン(view),
                )).with_child((Text::new(name), TextFont { font_size: 13.0, ..default() }, TextColor(サイバーテキスト色)));
            }
        });

        // ビューコンテナ
        ed.spawn(Node { flex_grow: 1.0, width: Val::Percent(100.0), ..default() }).with_children(|container| {
            ビュー構築(container, エディタビュー::シーン,  "CYBER MODE :: RIGHT-DRAG TO NAVIGATE");
            ビュー構築(container, エディタビュー::四面体, "TETRA MODE :: TETRAHEDRAL VOXEL WORLD");

            // コードエディタビュー
            container.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0), height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    ..default()
                },
                BackgroundColor(サイバー背景色),
                ビューコンテナ(エディタビュー::コード),
            )).with_child((
                Text::new("// Voxel Engine Core\n// Cube + Tetra dual mode\n\nfn initialize_world() {\n    let config = WorldConfig::default();\n    render_streamer.start(config);\n}"),
                TextFont { font_size: 15.0, ..default() },
                TextColor(サイバーテキスト色),
            ));
        });
    });
}

fn ビュー構築(parent: &mut ChildBuilder, view: エディタビュー, label: &str) {
    parent.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
        ビューコンテナ(view),
    )).with_children(|v| {
        v.spawn(Node { position_type: PositionType::Absolute, top: Val::Px(15.0), left: Val::Px(15.0), ..default() })
            .with_child((Text::new(label), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーアクセント色)));
    });
}

fn ステータスバー構築(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0), height: Val::Px(24.0),
            padding: UiRect::horizontal(Val::Px(12.0)),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(サイバーステータス色),
    ))
    .with_child((Text::new("CYBER STUDIO | LOD ENABLED | "), TextFont { font_size: 11.0, ..default() }, TextColor(Color::WHITE)))
    .with_child((Text::new("FPS: --"), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーアクセント色), FPSカウンター));
}

// =============================================================================
// イベント処理システム
// =============================================================================

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
