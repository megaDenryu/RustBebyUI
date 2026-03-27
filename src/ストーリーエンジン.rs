// src/ストーリーエンジン.rs
// 宣言的ストーリー定義 — データ構造・型定義のみ

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// =============================================================================
// ゲーム内時間
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ゲーム日付 {
    pub 月: u32,
    pub 日: u32,
}

impl ゲーム日付 {
    pub fn new(月: u32, 日: u32) -> Self { Self { 月, 日 } }

    pub fn 翌日(&self) -> Self {
        if self.日 >= 30 {
            Self { 月: self.月 + 1, 日: 1 }
        } else {
            Self { 月: self.月, 日: self.日 + 1 }
        }
    }

    pub fn 以降か(&self, other: &Self) -> bool {
        (self.月, self.日) >= (other.月, other.日)
    }
}

impl std::fmt::Display for ゲーム日付 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}月{}日", self.月, self.日)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum 時間帯 {
    朝,
    昼,
    夕,
    夜,
}

#[derive(Resource, Serialize, Deserialize)]
pub struct ゲーム時間 {
    pub 日付: ゲーム日付,
    pub 時間帯: 時間帯,
    pub 経過日数: u32,
}

impl Default for ゲーム時間 {
    fn default() -> Self {
        Self {
            日付: ゲーム日付::new(4, 1),
            時間帯: 時間帯::朝,
            経過日数: 0,
        }
    }
}

impl ゲーム時間 {
    pub fn 時間帯送り(&mut self) {
        self.時間帯 = match self.時間帯 {
            時間帯::朝 => 時間帯::昼,
            時間帯::昼 => 時間帯::夕,
            時間帯::夕 => 時間帯::夜,
            時間帯::夜 => {
                self.日付 = self.日付.翌日();
                self.経過日数 += 1;
                時間帯::朝
            }
        };
    }

    pub fn 日送り(&mut self) {
        self.日付 = self.日付.翌日();
        self.経過日数 += 1;
        self.時間帯 = 時間帯::朝;
    }
}

// =============================================================================
// フラグストア
// =============================================================================

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct フラグストア {
    pub フラグ群: HashMap<String, bool>,
    pub カウンター群: HashMap<String, i32>,
}

impl フラグストア {
    pub fn オン(&self, 名前: &str) -> bool {
        self.フラグ群.get(名前).copied().unwrap_or(false)
    }
    pub fn 設定(&mut self, 名前: &str) {
        self.フラグ群.insert(名前.to_string(), true);
    }
    pub fn 解除(&mut self, 名前: &str) {
        self.フラグ群.insert(名前.to_string(), false);
    }
    pub fn カウンター取得(&self, 名前: &str) -> i32 {
        self.カウンター群.get(名前).copied().unwrap_or(0)
    }
    pub fn カウンター加算(&mut self, 名前: &str, 値: i32) {
        *self.カウンター群.entry(名前.to_string()).or_insert(0) += 値;
    }
}

// =============================================================================
// ドキュメントストア
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ドキュメント {
    pub タイトル: String,
    pub 本文: String,
    pub カテゴリ: ドキュメントカテゴリ,
    pub 既読: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ドキュメントカテゴリ {
    手紙,
    日記,
    地図,
    公文書,
    メモ,
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct ドキュメントストア {
    pub 文書群: Vec<ドキュメント>,
}

impl ドキュメントストア {
    pub fn 追加(&mut self, タイトル: &str, 本文: &str, カテゴリ: ドキュメントカテゴリ) {
        if self.文書群.iter().any(|d| d.タイトル == タイトル) { return; }
        self.文書群.push(ドキュメント {
            タイトル: タイトル.to_string(),
            本文: 本文.to_string(),
            カテゴリ,
            既読: false,
        });
    }

    pub fn 未読数(&self) -> usize {
        self.文書群.iter().filter(|d| !d.既読).count()
    }
}

// =============================================================================
// メッセージストア
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct メッセージ {
    pub 送信者: String,
    pub 本文: String,
    pub 日付: ゲーム日付,
    pub 既読: bool,
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct メッセージストア {
    pub メッセージ群: Vec<メッセージ>,
}

impl メッセージストア {
    pub fn 追加(&mut self, 送信者: &str, 本文: &str, 日付: ゲーム日付) {
        self.メッセージ群.push(メッセージ {
            送信者: 送信者.to_string(),
            本文: 本文.to_string(),
            日付,
            既読: false,
        });
    }

    pub fn 未読数(&self) -> usize {
        self.メッセージ群.iter().filter(|m| !m.既読).count()
    }
}

// =============================================================================
// カレンダーストア
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct カレンダー予定 {
    pub 内容: String,
    pub 重要: bool,
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct カレンダーストア {
    pub 予定群: HashMap<ゲーム日付, Vec<カレンダー予定>>,
}

impl カレンダーストア {
    pub fn 追記(&mut self, 日付: ゲーム日付, 内容: &str, 重要: bool) {
        self.予定群.entry(日付).or_default().push(カレンダー予定 {
            内容: 内容.to_string(),
            重要,
        });
    }

    pub fn 取得(&self, 日付: &ゲーム日付) -> &[カレンダー予定] {
        self.予定群.get(日付).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

// =============================================================================
// 通知システム
// =============================================================================

#[derive(Clone, Debug)]
pub struct ゲーム通知 {
    pub テキスト: String,
    pub 残り秒数: f32,
}

#[derive(Resource, Default)]
pub struct 通知ストア {
    pub 通知群: Vec<ゲーム通知>,
}

impl 通知ストア {
    pub fn 追加(&mut self, テキスト: &str) {
        self.通知群.push(ゲーム通知 {
            テキスト: テキスト.to_string(),
            残り秒数: 5.0,
        });
    }
}

// =============================================================================
// クエストストア
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct クエスト {
    pub 名前: String,
    pub 説明: String,
    pub 完了: bool,
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct クエストストア {
    pub クエスト群: Vec<クエスト>,
}

impl クエストストア {
    pub fn 追加(&mut self, 名前: &str, 説明: &str) {
        if self.クエスト群.iter().any(|q| q.名前 == 名前) { return; }
        self.クエスト群.push(クエスト {
            名前: 名前.to_string(),
            説明: 説明.to_string(),
            完了: false,
        });
    }

    pub fn 完了にする(&mut self, 名前: &str) {
        if let Some(q) = self.クエスト群.iter_mut().find(|q| q.名前 == 名前) {
            q.完了 = true;
        }
    }
}

// =============================================================================
// 人物ストア
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct 人物 {
    pub 名前: String,
    pub 説明: String,
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct 人物ストア {
    pub 人物群: Vec<人物>,
}

impl 人物ストア {
    pub fn 追加(&mut self, 名前: &str, 説明: &str) {
        if self.人物群.iter().any(|p| p.名前 == 名前) { return; }
        self.人物群.push(人物 {
            名前: 名前.to_string(),
            説明: 説明.to_string(),
        });
    }
}

// =============================================================================
// タイムラインストア
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct タイムラインエントリ {
    pub 日付: ゲーム日付,
    pub 時間帯: 時間帯,
    pub テキスト: String,
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct タイムラインストア {
    pub エントリ群: Vec<タイムラインエントリ>,
}

impl タイムラインストア {
    pub fn 追記(&mut self, 日付: ゲーム日付, 時間帯: 時間帯, テキスト: &str) {
        self.エントリ群.push(タイムラインエントリ {
            日付,
            時間帯,
            テキスト: テキスト.to_string(),
        });
    }
}

// =============================================================================
// エリアストア
// =============================================================================

#[derive(Clone, Debug)]
pub struct エリア定義 {
    pub 名前: String,
    pub 座標: Vec3,
    pub 半径: f32,
}

#[derive(Resource, Default)]
pub struct エリアストア {
    pub エリア群: Vec<エリア定義>,
    pub 現在のエリア: Option<String>,
}

impl エリアストア {
    pub fn 追加(&mut self, 名前: &str, 座標: Vec3, 半径: f32) {
        if self.エリア群.iter().any(|a| a.名前 == 名前) { return; }
        self.エリア群.push(エリア定義 {
            名前: 名前.to_string(),
            座標,
            半径,
        });
    }
}

// =============================================================================
// 選択肢システム
// =============================================================================

#[derive(Clone, Debug)]
pub struct 選択肢項目 {
    pub テキスト: String,
    pub 効果群: Vec<イベント効果>,
}

#[derive(Resource, Default)]
pub struct 選択肢ストア {
    pub 表示中: bool,
    pub タイトル: String,
    pub 選択肢群: Vec<選択肢項目>,
    pub カーソル位置: usize,
    /// 選択肢が決定された時、効果群をここに格納。ストーリー評価システムが次フレームで適用する
    pub 決定済み効果: Option<Vec<イベント効果>>,
}

// =============================================================================
// 天候
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum 天候種別 {
    #[default]
    晴れ,
    曇り,
    雨,
    嵐,
    霧,
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct 天候ストア {
    pub 現在: 天候種別,
}

// =============================================================================
// 会話ストア
// =============================================================================

#[derive(Clone, Debug)]
pub struct 台詞 {
    pub 話者: String,
    pub 本文: String,
}

#[derive(Resource, Default)]
pub struct 会話ストア {
    pub 表示中: bool,
    pub 台詞群: Vec<台詞>,
    pub 現在位置: usize,
}

// =============================================================================
// 調査ポイントストア
// =============================================================================

#[derive(Clone, Debug)]
pub struct 調査ポイント {
    pub 名前: String,
    pub 座標: Vec3,
    pub 半径: f32,
    pub 効果群: Vec<イベント効果>,
    pub 使用済み: bool,
}

#[derive(Resource, Default)]
pub struct 調査ポイントストア {
    pub ポイント群: Vec<調査ポイント>,
}

// =============================================================================
// ヘルプ表示
// =============================================================================

#[derive(Resource, Default)]
pub struct ヘルプ表示 {
    pub 表示中: bool,
}

// =============================================================================
// ストーリーイベント定義 (宣言的)
// =============================================================================

/// イベント発火の条件
#[derive(Clone, Debug)]
pub enum イベント条件 {
    日付当日(ゲーム日付),
    日付以降(ゲーム日付),
    時間帯一致(時間帯),
    場所範囲 { 座標: Vec3, 半径: f32 },
    エリア内(String),
    フラグオン(String),
    フラグオフ(String),
    カウンター以上(String, i32),
    天候一致(天候種別),
    アイテム所持(String),
}

/// イベント発火時の効果
#[derive(Clone, Debug)]
pub enum イベント効果 {
    // --- 状態管理 ---
    フラグ設定(String),
    フラグ解除(String),
    カウンター加算(String, i32),

    // --- 情報配信 ---
    通知(String),
    ドキュメント追加 {
        タイトル: String,
        本文: String,
        カテゴリ: ドキュメントカテゴリ,
    },
    メッセージ追加 {
        送信者: String,
        本文: String,
    },
    カレンダー追記 {
        日付: ゲーム日付,
        内容: String,
        重要: bool,
    },

    // --- ゲーム進行 ---
    クエスト追加 { 名前: String, 説明: String },
    クエスト完了(String),
    人物追加 { 名前: String, 説明: String },
    タイムライン追記(String),

    // --- 空間 ---
    エリア登録 { 名前: String, 座標: Vec3, 半径: f32 },
    地図マーカー追加 { 座標: Vec3, 名前: String },

    // --- UI ---
    タブ切替(String),
    選択肢表示 {
        タイトル: String,
        選択肢群: Vec<選択肢項目>,
    },

    // --- 環境 ---
    天候変更(天候種別),

    // --- 会話・調査 ---
    会話開始(Vec<台詞>),
    調査ポイント登録 {
        名前: String,
        座標: Vec3,
        半径: f32,
        効果群: Vec<イベント効果>,
    },

    // --- 将来実装 ---
    // ワールド変更 { 座標: Vec3, ボクセル種別: ... },
    // プレイヤー移動(Vec3),
    // 画面演出(演出種別),
    // BGM変更(String),
    // SE再生(String),
    // カメラ演出 { 方向: Vec3, 秒数: f32 },
    // エンディング(String),
}

/// 一つのストーリーイベント
#[derive(Clone, Debug)]
pub struct ストーリーイベント {
    pub 識別子: String,
    pub 条件群: Vec<イベント条件>,
    pub 効果群: Vec<イベント効果>,
    pub 一度きり: bool,
}

// =============================================================================
// ストーリーエンジン (Resource)
// =============================================================================

#[derive(Resource, Default)]
pub struct ストーリーエンジン {
    pub イベント群: Vec<ストーリーイベント>,
    pub 発火済み: HashMap<String, bool>,
}

impl ストーリーエンジン {
    pub fn イベント登録(&mut self, イベント: ストーリーイベント) {
        self.イベント群.push(イベント);
    }

    pub fn イベント群登録(&mut self, イベント群: Vec<ストーリーイベント>) {
        self.イベント群.extend(イベント群);
    }
}
