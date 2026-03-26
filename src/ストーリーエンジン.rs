// src/ストーリーエンジン.rs
// 宣言的ストーリー定義 + 評価エンジン

use bevy::prelude::*;
use std::collections::HashMap;

// =============================================================================
// ゲーム内時間
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ゲーム日付 {
    pub 月: u32,
    pub 日: u32,
}

impl ゲーム日付 {
    pub fn new(月: u32, 日: u32) -> Self { Self { 月, 日 } }

    /// 翌日を返す (簡易: 各月30日固定)
    pub fn 翌日(&self) -> Self {
        if self.日 >= 30 {
            Self { 月: self.月 + 1, 日: 1 }
        } else {
            Self { 月: self.月, 日: self.日 + 1 }
        }
    }

    /// 日付の前後比較
    pub fn 以降か(&self, other: &Self) -> bool {
        (self.月, self.日) >= (other.月, other.日)
    }
}

impl std::fmt::Display for ゲーム日付 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}月{}日", self.月, self.日)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum 時間帯 {
    朝,   // 6:00-12:00
    昼,   // 12:00-17:00
    夕,   // 17:00-19:00
    夜,   // 19:00-6:00
}

#[derive(Resource)]
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
    /// 次の時間帯へ進める
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

    /// 翌日の朝まで進める (睡眠)
    pub fn 日送り(&mut self) {
        self.日付 = self.日付.翌日();
        self.経過日数 += 1;
        self.時間帯 = 時間帯::朝;
    }
}

// =============================================================================
// フラグストア
// =============================================================================

#[derive(Resource, Default)]
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

#[derive(Clone, Debug)]
pub struct ドキュメント {
    pub タイトル: String,
    pub 本文: String,
    pub カテゴリ: ドキュメントカテゴリ,
    pub 既読: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ドキュメントカテゴリ {
    手紙,
    日記,
    地図,
    公文書,
    メモ,
}

#[derive(Resource, Default)]
pub struct ドキュメントストア {
    pub 文書群: Vec<ドキュメント>,
}

impl ドキュメントストア {
    pub fn 追加(&mut self, タイトル: &str, 本文: &str, カテゴリ: ドキュメントカテゴリ) {
        // 重複追加しない
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

#[derive(Clone, Debug)]
pub struct メッセージ {
    pub 送信者: String,
    pub 本文: String,
    pub 日付: ゲーム日付,
    pub 既読: bool,
}

#[derive(Resource, Default)]
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

#[derive(Clone, Debug)]
pub struct カレンダー予定 {
    pub 内容: String,
    pub 重要: bool, // 重要な予定は赤表示など
}

#[derive(Resource, Default)]
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
// ストーリーイベント定義 (宣言的)
// =============================================================================

/// イベント発火の条件
#[derive(Clone, Debug)]
pub enum イベント条件 {
    日付当日(ゲーム日付),
    日付以降(ゲーム日付),
    時間帯一致(時間帯),
    場所範囲 { 座標: Vec3, 半径: f32 },
    フラグオン(String),
    フラグオフ(String),
    カウンター以上(String, i32),
}

/// イベント発火時の効果
#[derive(Clone, Debug)]
pub enum イベント効果 {
    フラグ設定(String),
    フラグ解除(String),
    カウンター加算(String, i32),
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
    // 将来拡張:
    // ワールド変更 { ... },
    // BGM変更(String),
    // 画面演出(演出種別),
}

/// 一つのストーリーイベント
#[derive(Clone, Debug)]
pub struct ストーリーイベント {
    pub 識別子: String,
    pub 条件群: Vec<イベント条件>,   // すべてANDで評価
    pub 効果群: Vec<イベント効果>,
    pub 一度きり: bool,             // trueなら一度発火したら再発火しない
}

// =============================================================================
// ストーリーエンジン (Resource)
// =============================================================================

#[derive(Resource)]
pub struct ストーリーエンジン {
    pub イベント群: Vec<ストーリーイベント>,
    pub 発火済み: HashMap<String, bool>,
}

impl Default for ストーリーエンジン {
    fn default() -> Self {
        Self {
            イベント群: Vec::new(),
            発火済み: HashMap::new(),
        }
    }
}

impl ストーリーエンジン {
    /// イベントを登録する
    pub fn イベント登録(&mut self, イベント: ストーリーイベント) {
        self.イベント群.push(イベント);
    }

    /// 複数イベントを一括登録
    pub fn イベント群登録(&mut self, イベント群: Vec<ストーリーイベント>) {
        self.イベント群.extend(イベント群);
    }
}

// =============================================================================
// 評価システム (Bevy System)
// =============================================================================

/// 毎フレーム、全イベントの条件を評価し、満たされたイベントの効果を適用する
pub fn ストーリー評価システム(
    mut エンジン: ResMut<ストーリーエンジン>,
    時間: Res<ゲーム時間>,
    mut フラグ: ResMut<フラグストア>,
    mut ドキュメント: ResMut<ドキュメントストア>,
    mut メッセージ: ResMut<メッセージストア>,
    mut カレンダー: ResMut<カレンダーストア>,
    mut 通知: ResMut<通知ストア>,
    カメラ: Query<&Transform, With<crate::カメラ制御::カメラ操作>>,
) {
    let プレイヤー位置 = カメラ.get_single().map(|t| t.translation).unwrap_or(Vec3::ZERO);

    // 発火すべきイベントのインデックスを収集
    let mut 発火対象: Vec<usize> = Vec::new();

    for (i, イベント) in エンジン.イベント群.iter().enumerate() {
        // 一度きりイベントで既に発火済みならスキップ
        if イベント.一度きり && エンジン.発火済み.get(&イベント.識別子).copied().unwrap_or(false) {
            continue;
        }

        // 全条件を評価 (AND)
        let 全条件満了 = イベント.条件群.iter().all(|条件| {
            match 条件 {
                イベント条件::日付当日(日付) => 時間.日付 == *日付,
                イベント条件::日付以降(日付) => 時間.日付.以降か(日付),
                イベント条件::時間帯一致(帯) => 時間.時間帯 == *帯,
                イベント条件::場所範囲 { 座標, 半径 } => {
                    プレイヤー位置.distance(*座標) <= *半径
                }
                イベント条件::フラグオン(名前) => フラグ.オン(名前),
                イベント条件::フラグオフ(名前) => !フラグ.オン(名前),
                イベント条件::カウンター以上(名前, 値) => フラグ.カウンター取得(名前) >= *値,
            }
        });

        if 全条件満了 {
            発火対象.push(i);
        }
    }

    // 効果を適用
    for &i in &発火対象 {
        let イベント = エンジン.イベント群[i].clone();

        if イベント.一度きり {
            エンジン.発火済み.insert(イベント.識別子.clone(), true);
        }

        for 効果 in &イベント.効果群 {
            match 効果 {
                イベント効果::フラグ設定(名前) => フラグ.設定(名前),
                イベント効果::フラグ解除(名前) => フラグ.解除(名前),
                イベント効果::カウンター加算(名前, 値) => フラグ.カウンター加算(名前, *値),
                イベント効果::通知(テキスト) => 通知.追加(テキスト),
                イベント効果::ドキュメント追加 { タイトル, 本文, カテゴリ } => {
                    ドキュメント.追加(タイトル, 本文, カテゴリ.clone());
                }
                イベント効果::メッセージ追加 { 送信者, 本文 } => {
                    メッセージ.追加(送信者, 本文, 時間.日付);
                }
                イベント効果::カレンダー追記 { 日付, 内容, 重要 } => {
                    カレンダー.追記(*日付, 内容, *重要);
                }
            }
        }
    }
}

/// 通知の残り時間を更新し、期限切れを削除する
pub fn 通知更新システム(
    time: Res<Time>,
    mut 通知: ResMut<通知ストア>,
) {
    let dt = time.delta_secs();
    for n in &mut 通知.通知群 {
        n.残り秒数 -= dt;
    }
    通知.通知群.retain(|n| n.残り秒数 > 0.0);
}
