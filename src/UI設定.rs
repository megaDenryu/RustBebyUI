// src/UI設定.rs
// UIレイアウト構築・テーマ定数・コンポーネント定義

use bevy::prelude::*;
use crate::ストーリーエンジン::*;

// =============================================================================
// サイバーテーマ配色
// =============================================================================
pub const サイバー背景色: Color = Color::srgb(0.05, 0.07, 0.12);
pub const サイバーサイドバー色: Color = Color::srgb(0.1, 0.1, 0.12);
pub const サイバーアクティビティ色: Color = Color::srgb(0.12, 0.12, 0.15);
pub const サイバーステータス色: Color = Color::srgb(0.0, 0.3, 0.6);
pub const サイバーボーダー色: Color = Color::srgb(0.2, 0.2, 0.3);
pub const サイバーテキスト色: Color = Color::srgb(0.85, 0.85, 0.9);
pub const サイバーアクセント色: Color = Color::srgb(0.0, 0.7, 1.0);
pub const サイバー薄文字色: Color = Color::srgb(0.5, 0.5, 0.6);
pub const サイバー重要色: Color = Color::srgb(1.0, 0.3, 0.3);

// =============================================================================
// 状態・コンポーネント
// =============================================================================

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum エディタビュー {
    #[default] シーン,
    四面体,
    カレンダー,
    メッセージ,
    ドキュメント,
    マップ,
    人物,
    タイムライン,
    インベントリ,
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

#[derive(Component)]
pub struct 日付表示;

#[derive(Component)]
pub struct カレンダー内容表示;

#[derive(Component)]
pub struct メッセージ内容表示;

#[derive(Component)]
pub struct ドキュメント一覧表示;

#[derive(Component)]
pub struct エリア名表示;

#[derive(Component)]
pub struct 通知テキスト;

#[derive(Component)]
pub struct 選択肢オーバーレイ;

#[derive(Component)]
pub struct マップ内容表示;

#[derive(Component)]
pub struct 人物内容表示;

#[derive(Component)]
pub struct タイムライン内容表示;

#[derive(Component)]
pub struct インベントリ内容表示;

#[derive(Component)]
pub struct サイドバー手がかり表示;

#[derive(Component)]
pub struct ヘルプオーバーレイ;

#[derive(Component)]
pub struct 会話オーバーレイ;

#[derive(Component)]
pub struct 天候表示;

#[derive(Component)]
pub struct タブテキスト(pub エディタビュー);

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
        // 本体
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

    // 通知オーバーレイ (画面上部中央)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            left: Val::Percent(30.0),
            width: Val::Percent(40.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        通知テキスト,
    ));

    // 選択肢オーバーレイ (画面中央)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(30.0),
            left: Val::Percent(25.0),
            width: Val::Percent(50.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(6.0),
            ..default()
        },
        選択肢オーバーレイ,
    ));

    // 会話オーバーレイ (画面下部)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.0),
            left: Val::Percent(10.0),
            width: Val::Percent(80.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(4.0),
            ..default()
        },
        会話オーバーレイ,
    ));

    // ヘルプオーバーレイ (画面中央, 大きめ)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(10.0),
            left: Val::Percent(15.0),
            width: Val::Percent(70.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(4.0),
            ..default()
        },
        ヘルプオーバーレイ,
    ));
}

fn アクティビティバー構築(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Px(55.0), height: Val::Percent(100.0),
            border: UiRect::right(Val::Px(1.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::top(Val::Px(8.0)),
            row_gap: Val::Px(4.0),
            ..default()
        },
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
        // タイトル
        sb.spawn(Node { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(16.0)), ..default() })
            .with_child((Text::new("INVESTIGATION"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバーアクセント色)));

        // 手がかり一覧 (動的更新)
        sb.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::horizontal(Val::Px(12.0)),
                row_gap: Val::Px(2.0),
                ..default()
            },
            サイドバー手がかり表示,
        ));
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
            let タブ定義 = [
                ("World",     エディタビュー::シーン),
                ("Tetra",     エディタビュー::四面体),
                ("Calendar",  エディタビュー::カレンダー),
                ("Messages",  エディタビュー::メッセージ),
                ("Documents", エディタビュー::ドキュメント),
                ("Map",       エディタビュー::マップ),
                ("Profiles",  エディタビュー::人物),
                ("Timeline",  エディタビュー::タイムライン),
                ("Inventory", エディタビュー::インベントリ),
            ];
            for (name, view) in タブ定義 {
                tabs.spawn((
                    Button,
                    Node {
                        width: Val::Px(95.0), height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::right(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(サイバーボーダー色),
                    タブボタン(view),
                )).with_child((Text::new(name), TextFont { font_size: 13.0, ..default() }, TextColor(サイバーテキスト色), タブテキスト(view)));
            }
        });

        // ビューコンテナ
        ed.spawn(Node { flex_grow: 1.0, width: Val::Percent(100.0), ..default() }).with_children(|container| {
            // 3Dビュー (透過: ワールドが見える)
            ビュー構築(container, エディタビュー::シーン,  "WORLD :: RIGHT-DRAG TO NAVIGATE");
            ビュー構築(container, エディタビュー::四面体, "TETRA :: TETRAHEDRAL VOXEL WORLD");

            // カレンダービュー
            container.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0), height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(サイバー背景色),
                ビューコンテナ(エディタビュー::カレンダー),
            )).with_children(|cal| {
                cal.spawn((Text::new("CALENDAR"), TextFont { font_size: 14.0, ..default() }, TextColor(サイバーアクセント色)));
                cal.spawn((
                    Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), ..default() },
                    カレンダー内容表示,
                ));
            });

            // メッセージビュー
            container.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0), height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(サイバー背景色),
                ビューコンテナ(エディタビュー::メッセージ),
            )).with_children(|msg| {
                msg.spawn((Text::new("MESSAGES"), TextFont { font_size: 14.0, ..default() }, TextColor(サイバーアクセント色)));
                msg.spawn((
                    Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(8.0), ..default() },
                    メッセージ内容表示,
                ));
            });

            // ドキュメントビュー
            container.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0), height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(サイバー背景色),
                ビューコンテナ(エディタビュー::ドキュメント),
            )).with_children(|doc| {
                doc.spawn((Text::new("DOCUMENTS"), TextFont { font_size: 14.0, ..default() }, TextColor(サイバーアクセント色)));
                doc.spawn((
                    Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(6.0), ..default() },
                    ドキュメント一覧表示,
                ));
            });

            // マップビュー
            情報ビュー構築(container, エディタビュー::マップ, "MAP", マップ内容表示);

            // 人物ビュー
            情報ビュー構築(container, エディタビュー::人物, "PROFILES", 人物内容表示);

            // タイムラインビュー
            情報ビュー構築(container, エディタビュー::タイムライン, "TIMELINE", タイムライン内容表示);

            // インベントリビュー
            情報ビュー構築(container, エディタビュー::インベントリ, "INVENTORY", インベントリ内容表示);
        });
    });
}

/// 汎用的な情報ウィンドウの構築ヘルパー
fn 情報ビュー構築(parent: &mut ChildBuilder, view: エディタビュー, title: &str, marker: impl Component) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0), height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(24.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(サイバー背景色),
        ビューコンテナ(view),
    )).with_children(|area| {
        area.spawn((Text::new(title), TextFont { font_size: 14.0, ..default() }, TextColor(サイバーアクセント色)));
        area.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            marker,
        ));
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
            column_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(サイバーステータス色),
    ))
    .with_child((Text::new(""), TextFont { font_size: 11.0, ..default() }, TextColor(Color::WHITE), 日付表示))
    .with_child((Text::new(""), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーテキスト色), エリア名表示))
    .with_child((Text::new(""), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーテキスト色), 天候表示))
    .with_child((Text::new("[H] Help"), TextFont { font_size: 10.0, ..default() }, TextColor(サイバー薄文字色)))
    .with_child((Text::new("FPS: --"), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーアクセント色), FPSカウンター));
}
