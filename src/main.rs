#[path = "ボクセル世界.rs"]
mod ボクセル世界;
#[path = "地形生成.rs"]
mod 地形生成;
#[path = "メッシュ生成.rs"]
mod メッシュ生成;
#[path = "カメラ制御.rs"]
mod カメラ制御;
#[path = "チャンク管理.rs"]
mod チャンク管理;
#[path = "衝突判定.rs"]
mod 衝突判定;
#[path = "シーン初期化.rs"]
mod シーン初期化;
#[path = "UI設定.rs"]
mod UI設定;
#[path = "四面体世界.rs"]
mod 四面体世界;
#[path = "四面体メッシュ生成.rs"]
mod 四面体メッシュ生成;
#[path = "四面体チャンク管理.rs"]
mod 四面体チャンク管理;
#[path = "ストーリーエンジン.rs"]
mod ストーリーエンジン;
#[path = "サンプルストーリー.rs"]
mod サンプルストーリー;

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use UI設定::{アプリ状態, エディタビュー};
use チャンク管理::チャンク描画データ;
use 四面体チャンク管理::四面体チャンクマーカー;

fn 立方体モードか(state: Res<アプリ状態>) -> bool {
    state.現在のビュー == エディタビュー::シーン
}

fn 四面体モードか(state: Res<アプリ状態>) -> bool {
    state.現在のビュー == エディタビュー::四面体
}

fn チャンク表示切替(
    state: Res<アプリ状態>,
    mut cube_query: Query<&mut Visibility, (With<チャンク描画データ>, Without<四面体チャンクマーカー>)>,
    mut tetra_query: Query<&mut Visibility, (With<四面体チャンクマーカー>, Without<チャンク描画データ>)>,
) {
    if state.is_changed() || state.is_added() {
        let is_cube = state.現在のビュー == エディタビュー::シーン;
        for mut vis in &mut cube_query {
            *vis = if is_cube { Visibility::Visible } else { Visibility::Hidden };
        }
        let is_tetra = state.現在のビュー == エディタビュー::四面体;
        for mut vis in &mut tetra_query {
            *vis = if is_tetra { Visibility::Visible } else { Visibility::Hidden };
        }
    }
}

/// ストーリーエンジンの初期化 (Startupシステム)
fn ストーリー初期化(
    mut commands: Commands,
) {
    // リソース群を初期化
    let mut カレンダー = ストーリーエンジン::カレンダーストア::default();
    サンプルストーリー::カレンダー初期化(&mut カレンダー);

    let mut エリア = ストーリーエンジン::エリアストア::default();
    サンプルストーリー::エリア初期化(&mut エリア);

    let mut エンジン = ストーリーエンジン::ストーリーエンジン::default();
    エンジン.イベント群登録(サンプルストーリー::第一章イベント群());

    commands.insert_resource(ストーリーエンジン::ゲーム時間::default());
    commands.insert_resource(ストーリーエンジン::フラグストア::default());
    commands.insert_resource(ストーリーエンジン::ドキュメントストア::default());
    commands.insert_resource(ストーリーエンジン::メッセージストア::default());
    commands.insert_resource(カレンダー);
    commands.insert_resource(ストーリーエンジン::通知ストア::default());
    commands.insert_resource(ストーリーエンジン::クエストストア::default());
    commands.insert_resource(ストーリーエンジン::人物ストア::default());
    commands.insert_resource(ストーリーエンジン::タイムラインストア::default());
    commands.insert_resource(エリア);
    commands.insert_resource(ストーリーエンジン::選択肢ストア::default());
    commands.insert_resource(ストーリーエンジン::天候ストア::default());
    commands.insert_resource(エンジン);
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
        // 起動時
        .add_systems(Startup, (
            シーン初期化::シーン初期化,
            UI設定::UI初期化,
            ストーリー初期化,
        ))
        // 更新ループ: コア
        .add_systems(Update, (
            UI設定::ボタン操作処理,
            UI設定::UI状態更新,
            カメラ制御::カメラ操作処理,
            UI設定::FPS更新,
            チャンク管理::チャンク管理処理.run_if(立方体モードか),
            チャンク管理::チャンクタスク処理.run_if(立方体モードか),
            四面体チャンク管理::四面体チャンク管理処理.run_if(四面体モードか),
            四面体チャンク管理::四面体チャンクタスク処理.run_if(四面体モードか),
            チャンク表示切替,
        ))
        // 更新ループ: ストーリーエンジン + 操作
        .add_systems(Update, (
            ストーリーエンジン::ストーリー評価システム,
            ストーリーエンジン::通知更新システム,
            ストーリーエンジン::エリア判定システム,
            ストーリーエンジン::操作システム,
        ))
        // 更新ループ: UI ↔ ストーリーデータ同期
        .add_systems(Update, (
            UI設定::日付表示更新,
            UI設定::エリア名表示更新,
            UI設定::カレンダー表示更新,
            UI設定::メッセージ表示更新,
            UI設定::ドキュメント表示更新,
            UI設定::通知表示更新,
            UI設定::選択肢表示更新,
            UI設定::サイドバー更新,
        ))
        .run();
}
