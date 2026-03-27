// src/デバッグ.rs
// F3デバッグオーバーレイ + ワールド読込待機

use bevy::prelude::*;
use crate::カメラ制御::カメラ操作;
use crate::チャンク管理::チャンク管理者;
use crate::ボクセル世界::チャンクのワールドサイズ;
use crate::地形生成::地形高さ;
use crate::ストーリーエンジン::*;
use crate::UI設定::ゲームフォント;

// =============================================================================
// リソース・コンポーネント
// =============================================================================

#[derive(Resource)]
pub struct デバッグ状態 {
    pub 表示中: bool,
    pub ワールド読込完了: bool,
}

impl Default for デバッグ状態 {
    fn default() -> Self {
        Self { 表示中: false, ワールド読込完了: false }
    }
}

#[derive(Component)]
pub struct デバッグオーバーレイ;

#[derive(Component)]
pub struct ローディング表示;

// =============================================================================
// 初期化 (Startup)
// =============================================================================

pub fn デバッグUI初期化(mut commands: Commands) {
    commands.insert_resource(デバッグ状態::default());

    // デバッグオーバーレイ (画面左上、初期非表示)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(8.0)),
            row_gap: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        Visibility::Hidden,
        デバッグオーバーレイ,
    ));

    // ローディング表示 (画面中央)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ローディング表示,
    ));
}

// =============================================================================
// F3トグル
// =============================================================================

pub fn デバッグトグル(
    keys: Res<ButtonInput<KeyCode>>,
    mut デバッグ: ResMut<デバッグ状態>,
    mut query: Query<&mut Visibility, With<デバッグオーバーレイ>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        デバッグ.表示中 = !デバッグ.表示中;
        for mut vis in &mut query {
            *vis = if デバッグ.表示中 { Visibility::Visible } else { Visibility::Hidden };
        }
    }
}

// =============================================================================
// デバッグオーバーレイ更新
// =============================================================================

pub fn デバッグ表示更新(
    mut commands: Commands,
    デバッグ: Res<デバッグ状態>,
    font: Res<ゲームフォント>,
    camera_query: Query<(&Transform, &カメラ操作)>,
    manager: Res<チャンク管理者>,
    時間: Res<ゲーム時間>,
    天候: Res<天候ストア>,
    エリア: Res<エリアストア>,
    フラグ: Res<フラグストア>,
    エンジン: Res<ストーリーエンジン>,
    query: Query<(Entity, &デバッグオーバーレイ)>,
) {
    if !デバッグ.表示中 { return; }

    let Ok((cam_transform, cam)) = camera_query.get_single() else { return; };
    let pos = cam_transform.translation;
    let ground = 地形高さ(pos.x, pos.z);
    let cx = (pos.x / チャンクのワールドサイズ).floor() as i32;
    let cy = (pos.y / チャンクのワールドサイズ).floor() as i32;
    let cz = (pos.z / チャンクのワールドサイズ).floor() as i32;

    let loaded = manager.読込済み.len();
    let pending = manager.読込中.len();

    let mode = if cam.飛行中 { "Flying" } else if cam.接地中 { "Walking | Grounded" } else { "Walking | Airborne" };

    let 天候名 = match 天候.現在 {
        天候種別::晴れ => "Clear", 天候種別::曇り => "Cloudy",
        天候種別::雨 => "Rain", 天候種別::嵐 => "Storm", 天候種別::霧 => "Fog",
    };
    let 時間帯名 = match 時間.時間帯 {
        時間帯::朝 => "Morning", 時間帯::昼 => "Noon",
        時間帯::夕 => "Evening", 時間帯::夜 => "Night",
    };

    let area_name = エリア.現在のエリア.as_deref().unwrap_or("(none)");
    let flag_count = フラグ.フラグ群.values().filter(|v| **v).count();
    let fired_count = エンジン.発火済み.len();

    let lines = vec![
        ("=== DEBUG (F3) ===".to_string(), true),
        (format!("Pos: ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z), false),
        (format!("Ground: {:.1}  Delta: {:.1}", ground, pos.y - ground), false),
        (format!("Chunk: ({}, {}, {})", cx, cy, cz), false),
        (format!("Facing: yaw={:.2} pitch={:.2}", cam.yaw, cam.pitch), false),
        (format!("Mode: {}  VSpd: {:.1}", mode, cam.垂直速度), false),
        ("".to_string(), false),
        (format!("Chunks: {} loaded | {} pending", loaded, pending), false),
        (format!("World ready: {}", デバッグ.ワールド読込完了), false),
        ("".to_string(), false),
        (format!("{} {} | {}月{}日 {}", 天候名, 時間帯名, 時間.日付.月, 時間.日付.日, area_name), false),
        (format!("Flags: {} on | Events: {} fired", flag_count, fired_count), false),
    ];

    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            for (text, accent) in &lines {
                let color = if *accent { Color::srgb(0.0, 0.7, 1.0) } else { Color::srgb(0.8, 0.9, 0.8) };
                parent.spawn((Text::new(text.clone()), font.テキスト(10.0), TextColor(color)));
            }
        });
    }
}

// =============================================================================
// ワールド読込待機
// =============================================================================

pub fn ワールド読込判定(
    mut デバッグ: ResMut<デバッグ状態>,
    camera_query: Query<&Transform, With<カメラ操作>>,
    manager: Res<チャンク管理者>,
) {
    if デバッグ.ワールド読込完了 { return; }

    let Ok(cam_transform) = camera_query.get_single() else { return; };
    let pos = cam_transform.translation;
    let cx = (pos.x / チャンクのワールドサイズ).floor() as i32;
    let cy = (pos.y / チャンクのワールドサイズ).floor() as i32;
    let cz = (pos.z / チャンクのワールドサイズ).floor() as i32;

    // プレイヤー直下とその下のチャンクが読込済みか
    let 足元あり = manager.読込済み.contains_key(&(cx, cy, cz));
    let 直下あり = manager.読込済み.contains_key(&(cx, cy - 1, cz));

    if 足元あり && 直下あり {
        デバッグ.ワールド読込完了 = true;
        info!("World loaded: player chunks ready at ({}, {}, {})", cx, cy, cz);
    }
}

pub fn ローディング表示更新(
    mut commands: Commands,
    デバッグ: Res<デバッグ状態>,
    font: Res<ゲームフォント>,
    query: Query<(Entity, &ローディング表示)>,
) {
    if !デバッグ.is_changed() { return; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        if !デバッグ.ワールド読込完了 {
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Node { padding: UiRect::all(Val::Px(20.0)), ..default() },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                )).with_children(|p| {
                    p.spawn((Text::new("Loading world..."), font.テキスト(18.0), TextColor(Color::srgb(0.0, 0.7, 1.0))));
                });
            });
        }
    }
}
