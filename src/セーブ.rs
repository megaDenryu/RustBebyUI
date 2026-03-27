// src/セーブ.rs
// セーブ/ロード: ゲーム状態をJSONファイルに保存・復元

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::ストーリーエンジン::*;
use crate::UI設定::*;
use crate::カメラ制御::カメラ操作;

const セーブファイルパス: &str = "save.json";

// =============================================================================
// セーブデータ構造 (シリアライズ可能)
// =============================================================================

#[derive(Serialize, Deserialize)]
pub struct セーブデータ {
    // 時間
    pub 月: u32,
    pub 日: u32,
    pub 時間帯: u8, // 0=朝,1=昼,2=夕,3=夜
    pub 経過日数: u32,

    // プレイヤー位置
    pub 位置x: f32,
    pub 位置y: f32,
    pub 位置z: f32,

    // フラグ
    pub フラグ群: HashMap<String, bool>,
    pub カウンター群: HashMap<String, i32>,

    // 発火済みイベント
    pub 発火済み: HashMap<String, bool>,

    // ドキュメント
    pub 文書群: Vec<セーブドキュメント>,

    // メッセージ
    pub メッセージ群: Vec<セーブメッセージ>,

    // カレンダー追加分
    pub カレンダー追加: Vec<セーブカレンダー予定>,

    // クエスト
    pub クエスト群: Vec<セーブクエスト>,

    // 人物
    pub 人物群: Vec<セーブ人物>,

    // 天候
    pub 天候: u8, // 0=晴,1=曇,2=雨,3=嵐,4=霧

    // 調査ポイント使用済み
    pub 調査済みポイント: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct セーブドキュメント {
    pub タイトル: String,
    pub 本文: String,
    pub カテゴリ: u8, // 0=手紙,1=日記,2=地図,3=公文書,4=メモ
}

#[derive(Serialize, Deserialize)]
pub struct セーブメッセージ {
    pub 送信者: String,
    pub 本文: String,
    pub 月: u32,
    pub 日: u32,
}

#[derive(Serialize, Deserialize)]
pub struct セーブカレンダー予定 {
    pub 月: u32,
    pub 日: u32,
    pub 内容: String,
    pub 重要: bool,
}

#[derive(Serialize, Deserialize)]
pub struct セーブクエスト {
    pub 名前: String,
    pub 説明: String,
    pub 完了: bool,
}

#[derive(Serialize, Deserialize)]
pub struct セーブ人物 {
    pub 名前: String,
    pub 説明: String,
}

// =============================================================================
// セーブ/ロード システム
// =============================================================================

fn 時間帯to番号(帯: &時間帯) -> u8 {
    match 帯 { 時間帯::朝=>0, 時間帯::昼=>1, 時間帯::夕=>2, 時間帯::夜=>3 }
}

fn 番号to時間帯(n: u8) -> 時間帯 {
    match n { 0=>時間帯::朝, 1=>時間帯::昼, 2=>時間帯::夕, _=>時間帯::夜 }
}

fn 天候to番号(天: &天候種別) -> u8 {
    match 天 { 天候種別::晴れ=>0, 天候種別::曇り=>1, 天候種別::雨=>2, 天候種別::嵐=>3, 天候種別::霧=>4 }
}

fn 番号to天候(n: u8) -> 天候種別 {
    match n { 0=>天候種別::晴れ, 1=>天候種別::曇り, 2=>天候種別::雨, 3=>天候種別::嵐, _=>天候種別::霧 }
}

fn カテゴリto番号(c: &ドキュメントカテゴリ) -> u8 {
    match c { ドキュメントカテゴリ::手紙=>0, ドキュメントカテゴリ::日記=>1, ドキュメントカテゴリ::地図=>2, ドキュメントカテゴリ::公文書=>3, ドキュメントカテゴリ::メモ=>4 }
}

fn 番号toカテゴリ(n: u8) -> ドキュメントカテゴリ {
    match n { 0=>ドキュメントカテゴリ::手紙, 1=>ドキュメントカテゴリ::日記, 2=>ドキュメントカテゴリ::地図, 3=>ドキュメントカテゴリ::公文書, _=>ドキュメントカテゴリ::メモ }
}

/// F5でセーブ
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
    天候: Res<天候ストア>,
    調査: Res<調査ポイントストア>,
    mut 通知: ResMut<通知ストア>,
    カメラ: Query<&Transform, With<カメラ操作>>,
) {
    if !keys.just_pressed(KeyCode::F5) { return; }

    let pos = カメラ.get_single().map(|t| t.translation).unwrap_or(Vec3::ZERO);

    let data = セーブデータ {
        月: 時間.日付.月,
        日: 時間.日付.日,
        時間帯: 時間帯to番号(&時間.時間帯),
        経過日数: 時間.経過日数,
        位置x: pos.x, 位置y: pos.y, 位置z: pos.z,
        フラグ群: フラグ.フラグ群.clone(),
        カウンター群: フラグ.カウンター群.clone(),
        発火済み: エンジン.発火済み.clone(),
        文書群: ドキュメント.文書群.iter().map(|d| セーブドキュメント {
            タイトル: d.タイトル.clone(), 本文: d.本文.clone(), カテゴリ: カテゴリto番号(&d.カテゴリ),
        }).collect(),
        メッセージ群: メッセージ.メッセージ群.iter().map(|m| セーブメッセージ {
            送信者: m.送信者.clone(), 本文: m.本文.clone(), 月: m.日付.月, 日: m.日付.日,
        }).collect(),
        カレンダー追加: カレンダー.予定群.iter().flat_map(|(日付, 予定群)| {
            予定群.iter().map(move |p| セーブカレンダー予定 {
                月: 日付.月, 日: 日付.日, 内容: p.内容.clone(), 重要: p.重要,
            })
        }).collect(),
        クエスト群: クエスト.クエスト群.iter().map(|q| セーブクエスト {
            名前: q.名前.clone(), 説明: q.説明.clone(), 完了: q.完了,
        }).collect(),
        人物群: 人物.人物群.iter().map(|p| セーブ人物 {
            名前: p.名前.clone(), 説明: p.説明.clone(),
        }).collect(),
        天候: 天候to番号(&天候.現在),
        調査済みポイント: 調査.ポイント群.iter().filter(|p| p.使用済み).map(|p| p.名前.clone()).collect(),
    };

    match serde_json::to_string_pretty(&data) {
        Ok(json) => {
            match std::fs::write(セーブファイルパス, json) {
                Ok(_) => 通知.追加("セーブしました (F5)"),
                Err(e) => 通知.追加(&format!("セーブ失敗: {}", e)),
            }
        }
        Err(e) => 通知.追加(&format!("シリアライズ失敗: {}", e)),
    }
}

/// F9でロード
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

    // 時間
    時間.日付 = ゲーム日付::new(data.月, data.日);
    時間.時間帯 = 番号to時間帯(data.時間帯);
    時間.経過日数 = data.経過日数;

    // 位置
    if let Ok(mut t) = カメラ.get_single_mut() {
        t.translation = Vec3::new(data.位置x, data.位置y, data.位置z);
    }

    // フラグ
    フラグ.フラグ群 = data.フラグ群;
    フラグ.カウンター群 = data.カウンター群;

    // 発火済み
    エンジン.発火済み = data.発火済み;

    // ドキュメント
    ドキュメント.文書群 = data.文書群.into_iter().map(|d| ドキュメント {
        タイトル: d.タイトル, 本文: d.本文, カテゴリ: 番号toカテゴリ(d.カテゴリ), 既読: true,
    }).collect();

    // メッセージ
    メッセージ.メッセージ群 = data.メッセージ群.into_iter().map(|m| メッセージ {
        送信者: m.送信者, 本文: m.本文, 日付: ゲーム日付::new(m.月, m.日), 既読: true,
    }).collect();

    // カレンダー (初期状態をリセットしてから追加)
    カレンダー.予定群.clear();
    crate::サンプルストーリー::カレンダー初期化(&mut カレンダー);
    for p in data.カレンダー追加 {
        // 初期データと重複しないよう、既にあるものは追加しない
        let 日付 = ゲーム日付::new(p.月, p.日);
        let existing = カレンダー.取得(&日付);
        if !existing.iter().any(|e| e.内容 == p.内容) {
            カレンダー.追記(日付, &p.内容, p.重要);
        }
    }

    // クエスト
    クエスト.クエスト群 = data.クエスト群.into_iter().map(|q| クエスト {
        名前: q.名前, 説明: q.説明, 完了: q.完了,
    }).collect();

    // 人物
    人物.人物群 = data.人物群.into_iter().map(|p| 人物 {
        名前: p.名前, 説明: p.説明,
    }).collect();

    // 天候
    天候.現在 = 番号to天候(data.天候);

    // 調査ポイントの使用済み復元
    for name in &data.調査済みポイント {
        if let Some(p) = 調査.ポイント群.iter_mut().find(|p| &p.名前 == name) {
            p.使用済み = true;
        }
    }

    通知.追加("ロードしました (F9)");
}
