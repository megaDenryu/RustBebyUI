#[path = "ボクセル世界.rs"]
mod ボクセル世界;
#[path = "メッシュ生成.rs"]
mod メッシュ生成;
#[path = "カメラ制御.rs"]
mod カメラ制御;
#[path = "チャンク管理.rs"]
mod チャンク管理;
#[path = "衝突判定.rs"]
mod 衝突判定;
#[path = "UI設定.rs"]
mod UI設定;
#[path = "四面体世界.rs"]
mod 四面体世界;
#[path = "四面体メッシュ生成.rs"]
mod 四面体メッシュ生成;
#[path = "四面体チャンク管理.rs"]
mod 四面体チャンク管理;

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use UI設定::{UI初期化, ボタン操作処理, UI状態更新, FPS更新, アプリ状態, エディタビュー};
use カメラ制御::カメラ操作処理;
use チャンク管理::{チャンク管理処理, チャンクタスク処理, チャンク描画データ};
use 四面体チャンク管理::{四面体チャンク管理処理, 四面体チャンクタスク処理, 四面体チャンクマーカー};

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
        .add_systems(Startup, UI初期化)
        // 更新ループ
        .add_systems(Update, (
            ボタン操作処理,
            UI状態更新,
            カメラ操作処理,
            FPS更新,
            // 立方体モード (Scene tab)
            チャンク管理処理.run_if(立方体モードか),
            チャンクタスク処理.run_if(立方体モードか),
            // 四面体モード (Tetra tab)
            四面体チャンク管理処理.run_if(四面体モードか),
            四面体チャンクタスク処理.run_if(四面体モードか),
            // 表示切替
            チャンク表示切替,
        ))
        .run();
}
