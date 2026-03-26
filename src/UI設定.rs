// src/UI設定.rs
// UIレイアウト構築・イベント処理・マルチウィンドウ

use bevy::prelude::*;
use crate::ストーリーエンジン::*;

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
const サイバー薄文字色: Color = Color::srgb(0.5, 0.5, 0.6);
const サイバー重要色: Color = Color::srgb(1.0, 0.3, 0.3);

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
pub struct サイドバー手がかり表示;

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
            ];
            for (name, view) in タブ定義 {
                tabs.spawn((
                    Button,
                    Node {
                        width: Val::Px(120.0), height: Val::Percent(100.0),
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
            column_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(サイバーステータス色),
    ))
    .with_child((Text::new(""), TextFont { font_size: 11.0, ..default() }, TextColor(Color::WHITE), 日付表示))
    .with_child((Text::new(""), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーテキスト色), エリア名表示))
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

// =============================================================================
// ストーリーデータ → UI 同期システム
// =============================================================================

/// ステータスバーのエリア名表示を更新
pub fn エリア名表示更新(
    エリア: Res<エリアストア>,
    mut query: Query<&mut Text, With<エリア名表示>>,
) {
    if エリア.is_changed() {
        let 名前 = エリア.現在のエリア.as_deref().unwrap_or("");
        for mut text in &mut query {
            text.0 = 名前.to_string();
        }
    }
}

/// ステータスバーの日付表示を更新
pub fn 日付表示更新(
    時間: Res<ゲーム時間>,
    mut query: Query<&mut Text, With<日付表示>>,
) {
    if 時間.is_changed() {
        let 帯名 = match 時間.時間帯 {
            時間帯::朝 => "朝",
            時間帯::昼 => "昼",
            時間帯::夕 => "夕",
            時間帯::夜 => "夜",
        };
        for mut text in &mut query {
            text.0 = format!("{} [{}] | Day {}", 時間.日付, 帯名, 時間.経過日数 + 1);
        }
    }
}

/// カレンダーウィンドウの内容を更新
pub fn カレンダー表示更新(
    mut commands: Commands,
    カレンダー: Res<カレンダーストア>,
    時間: Res<ゲーム時間>,
    query: Query<(Entity, &カレンダー内容表示)>,
) {
    if !カレンダー.is_changed() && !時間.is_changed() { return; }

    for (entity, _) in &query {
        // 既存の子を削除して再構築
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            // 現在月の予定を日付順に表示
            let 月 = 時間.日付.月;
            let mut 日付一覧: Vec<_> = カレンダー.予定群.keys()
                .filter(|d| d.月 == 月)
                .copied()
                .collect();
            日付一覧.sort_by_key(|d| (d.月, d.日));

            // 月ヘッダー
            parent.spawn((
                Text::new(format!("── {}月 ──", 月)),
                TextFont { font_size: 13.0, ..default() },
                TextColor(サイバーアクセント色),
            ));

            for 日付 in 日付一覧 {
                let 予定群 = カレンダー.取得(&日付);
                for 予定 in 予定群 {
                    let is_today = 日付 == 時間.日付;
                    let マーカー = if is_today { "▶ " } else { "  " };
                    let color = if 予定.重要 { サイバー重要色 }
                        else if is_today { サイバーアクセント色 }
                        else { サイバーテキスト色 };

                    parent.spawn((
                        Text::new(format!("{}{}日  {}", マーカー, 日付.日, 予定.内容)),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(color),
                    ));
                }
            }
        });
    }
}

/// メッセージウィンドウの内容を更新
pub fn メッセージ表示更新(
    mut commands: Commands,
    mut メッセージ: ResMut<メッセージストア>,
    query: Query<(Entity, &メッセージ内容表示)>,
) {
    if !メッセージ.is_changed() { return; }

    // 表示時に全て既読にする
    for m in &mut メッセージ.メッセージ群 {
        m.既読 = true;
    }

    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            if メッセージ.メッセージ群.is_empty() {
                parent.spawn((
                    Text::new("メッセージはありません"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(サイバー薄文字色),
                ));
                return;
            }

            for msg in メッセージ.メッセージ群.iter().rev() {
                // 送信者ヘッダー
                parent.spawn((
                    Text::new(format!("From: {} ({})", msg.送信者, msg.日付)),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(サイバーアクセント色),
                ));
                // 本文
                parent.spawn((
                    Text::new(msg.本文.clone()),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(サイバーテキスト色),
                ));
                // 区切り線
                parent.spawn((
                    Text::new("─────────────────"),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(サイバーボーダー色),
                ));
            }
        });
    }
}

/// ドキュメントウィンドウの内容を更新
pub fn ドキュメント表示更新(
    mut commands: Commands,
    mut ドキュメント: ResMut<ドキュメントストア>,
    query: Query<(Entity, &ドキュメント一覧表示)>,
) {
    if !ドキュメント.is_changed() { return; }

    // 表示時に全て既読にする
    for d in &mut ドキュメント.文書群 {
        d.既読 = true;
    }

    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            if ドキュメント.文書群.is_empty() {
                parent.spawn((
                    Text::new("発見した文書はありません"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(サイバー薄文字色),
                ));
                return;
            }

            for doc in &ドキュメント.文書群 {
                let カテゴリ名 = match doc.カテゴリ {
                    ドキュメントカテゴリ::手紙 => "手紙",
                    ドキュメントカテゴリ::日記 => "日記",
                    ドキュメントカテゴリ::地図 => "地図",
                    ドキュメントカテゴリ::公文書 => "公文書",
                    ドキュメントカテゴリ::メモ => "メモ",
                };

                // タイトル
                parent.spawn((
                    Text::new(format!("[{}] {}", カテゴリ名, doc.タイトル)),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(サイバーアクセント色),
                ));
                // 本文
                parent.spawn((
                    Text::new(doc.本文.clone()),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(サイバーテキスト色),
                ));
                // 区切り
                parent.spawn((
                    Text::new(""),
                    TextFont { font_size: 6.0, ..default() },
                    TextColor(サイバーボーダー色),
                ));
            }
        });
    }
}

/// 通知オーバーレイの表示
pub fn 通知表示更新(
    mut commands: Commands,
    通知: Res<通知ストア>,
    query: Query<(Entity, &通知テキスト)>,
) {
    if !通知.is_changed() { return; }

    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            for n in &通知.通知群 {
                parent.spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                )).with_child((
                    Text::new(n.テキスト.clone()),
                    TextFont { font_size: 13.0, ..default() },
                    TextColor(サイバーアクセント色),
                ));
            }
        });
    }
}

/// 選択肢オーバーレイの表示
pub fn 選択肢表示更新(
    mut commands: Commands,
    選択肢: Res<選択肢ストア>,
    query: Query<(Entity, &選択肢オーバーレイ)>,
) {
    if !選択肢.is_changed() { return; }

    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();

        if !選択肢.表示中 { continue; }

        commands.entity(entity).with_children(|parent| {
            // 背景パネル
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.05, 0.1, 0.95)),
                BorderColor(サイバーアクセント色),
            )).with_children(|panel| {
                // タイトル
                panel.spawn((
                    Text::new(選択肢.タイトル.clone()),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(サイバーアクセント色),
                ));

                // 選択肢一覧
                for (i, item) in 選択肢.選択肢群.iter().enumerate() {
                    let is_selected = i == 選択肢.カーソル位置;
                    let marker = if is_selected { "▶ " } else { "  " };
                    let color = if is_selected { サイバーアクセント色 } else { サイバーテキスト色 };

                    panel.spawn((
                        Text::new(format!("{}{}", marker, item.テキスト)),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(color),
                    ));
                }

                // 操作ヒント
                panel.spawn((
                    Text::new("[↑↓] 選択  [Enter/F] 決定  [Esc] 戻る"),
                    TextFont { font_size: 10.0, ..default() },
                    TextColor(サイバー薄文字色),
                ));
            });
        });
    }
}

/// サイドバーの手がかり・クエスト一覧を更新
pub fn サイドバー更新(
    mut commands: Commands,
    フラグ: Res<フラグストア>,
    ドキュメント: Res<ドキュメントストア>,
    メッセージ: Res<メッセージストア>,
    クエスト: Res<クエストストア>,
    query: Query<(Entity, &サイドバー手がかり表示)>,
) {
    if !フラグ.is_changed() && !ドキュメント.is_changed()
        && !メッセージ.is_changed() && !クエスト.is_changed() { return; }

    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            // 統計
            parent.spawn((
                Text::new(format!("Docs: {}  Msg: {}", ドキュメント.文書群.len(), メッセージ.メッセージ群.len())),
                TextFont { font_size: 11.0, ..default() },
                TextColor(サイバー薄文字色),
            ));

            // クエスト
            let active_quests: Vec<_> = クエスト.クエスト群.iter().filter(|q| !q.完了).collect();
            if !active_quests.is_empty() {
                parent.spawn((
                    Text::new("── Quests ──"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(サイバーアクセント色),
                ));
                for q in &active_quests {
                    parent.spawn((
                        Text::new(format!("  ◇ {}", q.名前)),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(サイバーテキスト色),
                    ));
                }
            }

            // 完了済みクエスト
            let done_quests: Vec<_> = クエスト.クエスト群.iter().filter(|q| q.完了).collect();
            if !done_quests.is_empty() {
                for q in &done_quests {
                    parent.spawn((
                        Text::new(format!("  ◆ {} ✓", q.名前)),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(サイバー薄文字色),
                    ));
                }
            }

            // 手がかり
            parent.spawn((
                Text::new("── Clues ──"),
                TextFont { font_size: 11.0, ..default() },
                TextColor(サイバーアクセント色),
            ));

            let mut フラグ一覧: Vec<_> = フラグ.フラグ群.iter()
                .filter(|(_, v)| **v)
                .map(|(k, _)| k.clone())
                .collect();
            フラグ一覧.sort();

            if フラグ一覧.is_empty() {
                parent.spawn((
                    Text::new("  (none yet)"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(サイバー薄文字色),
                ));
            } else {
                for 名前 in &フラグ一覧 {
                    parent.spawn((
                        Text::new(format!("  * {}", 名前)),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(サイバーテキスト色),
                    ));
                }
            }
        });
    }
}
