// src/セーブ.rs
// セーブ/ロード: ゲーム状態をJSONファイルに保存・復元
// 本体の型に Serialize/Deserialize が付いているので、別構造体は不要

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::ストーリーエンジン::*;
use crate::カメラ制御::カメラ操作;

const セーブファイルパス: &str = "save.json";

// =============================================================================
// セーブデータ: 本体の型を直接使う + 位置・発火済みを追加
// =============================================================================

#[derive(Serialize, Deserialize)]
pub struct セーブデータ {
    pub ゲーム時間: ゲーム時間,
    pub フラグ: フラグストア,
    pub ドキュメント: ドキュメントストア,
    pub メッセージ: メッセージストア,
    pub カレンダー: カレンダーストア,
    pub クエスト: クエストストア,
    pub 人物: 人物ストア,
    pub タイムライン: タイムラインストア,
    pub 天候: 天候ストア,
    pub 発火済み: HashMap<String, bool>,
    pub 調査済みポイント: Vec<String>,
    pub プレイヤー位置: [f32; 3],
}

// =============================================================================
// セーブシステム (F5)
// =============================================================================

pub fn セーブシステム(
    keys: Res<ButtonInput<KeyCode>>,
    時間: Res<ゲーム時間>,
    フラグ: Res<フラグストア>,
    エンジン: Res<ストーリーエンジン>,
    ドキュメント: Res<ドキュメントストア>,
    メッセージ: Res<メッセージストア>,
    カレンダー: Res<カレンダーストア>,
    クエスト: Res<クエストストア>,
    人物: Res<人物ストア>,
    タイムライン: Res<タイムラインストア>,
    天候: Res<天候ストア>,
    調査: Res<調査ポイントストア>,
    mut 通知: ResMut<通知ストア>,
    カメラ: Query<&Transform, With<カメラ操作>>,
) {
    if !keys.just_pressed(KeyCode::F5) { return; }

    let pos = カメラ.get_single().map(|t| t.translation).unwrap_or(Vec3::ZERO);

    let data = セーブデータ {
        ゲーム時間: ゲーム時間 {
            日付: 時間.日付,
            時間帯: 時間.時間帯,
            経過日数: 時間.経過日数,
        },
        フラグ: フラグストア {
            フラグ群: フラグ.フラグ群.clone(),
            カウンター群: フラグ.カウンター群.clone(),
        },
        ドキュメント: ドキュメントストア { 文書群: ドキュメント.文書群.clone() },
        メッセージ: メッセージストア { メッセージ群: メッセージ.メッセージ群.clone() },
        カレンダー: カレンダーストア { 予定群: カレンダー.予定群.clone() },
        クエスト: クエストストア { クエスト群: クエスト.クエスト群.clone() },
        人物: 人物ストア { 人物群: 人物.人物群.clone() },
        タイムライン: タイムラインストア { エントリ群: タイムライン.エントリ群.clone() },
        天候: 天候ストア { 現在: 天候.現在 },
        発火済み: エンジン.発火済み.clone(),
        調査済みポイント: 調査.ポイント群.iter()
            .filter(|p| p.使用済み)
            .map(|p| p.名前.clone())
            .collect(),
        プレイヤー位置: [pos.x, pos.y, pos.z],
    };

    match serde_json::to_string_pretty(&data) {
        Ok(json) => match std::fs::write(セーブファイルパス, json) {
            Ok(_) => 通知.追加("セーブしました (F5)"),
            Err(e) => 通知.追加(&format!("セーブ失敗: {}", e)),
        },
        Err(e) => 通知.追加(&format!("シリアライズ失敗: {}", e)),
    }
}

// =============================================================================
// ロードシステム (F9)
// =============================================================================

pub fn ロードシステム(
    keys: Res<ButtonInput<KeyCode>>,
    mut 時間: ResMut<ゲーム時間>,
    mut フラグ: ResMut<フラグストア>,
    mut エンジン: ResMut<ストーリーエンジン>,
    mut ドキュメント: ResMut<ドキュメントストア>,
    mut メッセージ: ResMut<メッセージストア>,
    mut カレンダー: ResMut<カレンダーストア>,
    mut クエスト: ResMut<クエストストア>,
    mut 人物: ResMut<人物ストア>,
    mut タイムライン: ResMut<タイムラインストア>,
    mut 天候: ResMut<天候ストア>,
    mut 調査: ResMut<調査ポイントストア>,
    mut 通知: ResMut<通知ストア>,
    mut カメラ: Query<&mut Transform, With<カメラ操作>>,
) {
    if !keys.just_pressed(KeyCode::F9) { return; }

    let json = match std::fs::read_to_string(セーブファイルパス) {
        Ok(s) => s,
        Err(e) => { 通知.追加(&format!("ロード失敗: {}", e)); return; }
    };

    let data: セーブデータ = match serde_json::from_str(&json) {
        Ok(d) => d,
        Err(e) => { 通知.追加(&format!("パース失敗: {}", e)); return; }
    };

    // 本体型を直接代入
    時間.日付 = data.ゲーム時間.日付;
    時間.時間帯 = data.ゲーム時間.時間帯;
    時間.経過日数 = data.ゲーム時間.経過日数;

    フラグ.フラグ群 = data.フラグ.フラグ群;
    フラグ.カウンター群 = data.フラグ.カウンター群;

    ドキュメント.文書群 = data.ドキュメント.文書群;
    メッセージ.メッセージ群 = data.メッセージ.メッセージ群;
    カレンダー.予定群 = data.カレンダー.予定群;
    クエスト.クエスト群 = data.クエスト.クエスト群;
    人物.人物群 = data.人物.人物群;
    タイムライン.エントリ群 = data.タイムライン.エントリ群;
    天候.現在 = data.天候.現在;

    エンジン.発火済み = data.発火済み;

    // 調査ポイントの使用済み復元
    for name in &data.調査済みポイント {
        if let Some(p) = 調査.ポイント群.iter_mut().find(|p| &p.名前 == name) {
            p.使用済み = true;
        }
    }

    // プレイヤー位置復元
    if let Ok(mut t) = カメラ.get_single_mut() {
        t.translation = Vec3::new(data.プレイヤー位置[0], data.プレイヤー位置[1], data.プレイヤー位置[2]);
    }

    通知.追加("ロードしました (F9)");
}
