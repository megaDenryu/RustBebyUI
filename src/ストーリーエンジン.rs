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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum 時間帯 {
    朝,
    昼,
    夕,
    夜,
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
    pub 重要: bool,
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
// クエストストア
// =============================================================================

#[derive(Clone, Debug)]
pub struct クエスト {
    pub 名前: String,
    pub 説明: String,
    pub 完了: bool,
}

#[derive(Resource, Default)]
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

#[derive(Clone, Debug)]
pub struct 人物 {
    pub 名前: String,
    pub 説明: String,
}

#[derive(Resource, Default)]
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

#[derive(Clone, Debug)]
pub struct タイムラインエントリ {
    pub 日付: ゲーム日付,
    pub 時間帯: 時間帯,
    pub テキスト: String,
}

#[derive(Resource, Default)]
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
}

// =============================================================================
// 天候
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum 天候種別 {
    #[default]
    晴れ,
    曇り,
    雨,
    嵐,
    霧,
}

#[derive(Resource, Default)]
pub struct 天候ストア {
    pub 現在: 天候種別,
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

// =============================================================================
// 評価システム (Bevy System)
// =============================================================================

pub fn ストーリー評価システム(
    mut エンジン: ResMut<ストーリーエンジン>,
    時間: Res<ゲーム時間>,
    mut フラグ: ResMut<フラグストア>,
    mut ドキュメント: ResMut<ドキュメントストア>,
    mut メッセージ: ResMut<メッセージストア>,
    mut カレンダー: ResMut<カレンダーストア>,
    mut 通知: ResMut<通知ストア>,
    mut クエスト: ResMut<クエストストア>,
    mut 人物: ResMut<人物ストア>,
    mut タイムライン: ResMut<タイムラインストア>,
    mut エリア: ResMut<エリアストア>,
    mut 選択肢: ResMut<選択肢ストア>,
    mut 天候: ResMut<天候ストア>,
    mut アプリ状態: ResMut<crate::UI設定::アプリ状態>,
    カメラ: Query<&Transform, With<crate::カメラ制御::カメラ操作>>,
) {
    // 選択肢表示中はイベント評価を停止
    if 選択肢.表示中 { return; }

    let プレイヤー位置 = カメラ.get_single().map(|t| t.translation).unwrap_or(Vec3::ZERO);

    let mut 発火対象: Vec<usize> = Vec::new();

    for (i, イベント) in エンジン.イベント群.iter().enumerate() {
        if イベント.一度きり && エンジン.発火済み.get(&イベント.識別子).copied().unwrap_or(false) {
            continue;
        }

        let 全条件満了 = イベント.条件群.iter().all(|条件| {
            match 条件 {
                イベント条件::日付当日(日付) => 時間.日付 == *日付,
                イベント条件::日付以降(日付) => 時間.日付.以降か(日付),
                イベント条件::時間帯一致(帯) => 時間.時間帯 == *帯,
                イベント条件::場所範囲 { 座標, 半径 } => {
                    プレイヤー位置.distance(*座標) <= *半径
                }
                イベント条件::エリア内(名前) => {
                    エリア.現在のエリア.as_deref() == Some(名前.as_str())
                }
                イベント条件::フラグオン(名前) => フラグ.オン(名前),
                イベント条件::フラグオフ(名前) => !フラグ.オン(名前),
                イベント条件::カウンター以上(名前, 値) => フラグ.カウンター取得(名前) >= *値,
                イベント条件::天候一致(種別) => 天候.現在 == *種別,
                イベント条件::アイテム所持(_名前) => false, // 将来実装
            }
        });

        if 全条件満了 {
            発火対象.push(i);
        }
    }

    for &i in &発火対象 {
        let イベント = エンジン.イベント群[i].clone();

        if イベント.一度きり {
            エンジン.発火済み.insert(イベント.識別子.clone(), true);
        }

        // タイムラインに自動記録
        タイムライン.追記(時間.日付, 時間.時間帯, &イベント.識別子);

        効果適用(
            &イベント.効果群, &時間, &mut フラグ, &mut ドキュメント,
            &mut メッセージ, &mut カレンダー, &mut 通知, &mut クエスト,
            &mut 人物, &mut タイムライン, &mut エリア, &mut 選択肢,
            &mut 天候, &mut アプリ状態,
        );
    }
}

/// 効果群を適用する (選択肢の結果からも呼ばれるため関数化)
pub fn 効果適用(
    効果群: &[イベント効果],
    時間: &ゲーム時間,
    フラグ: &mut フラグストア,
    ドキュメント: &mut ドキュメントストア,
    メッセージ: &mut メッセージストア,
    カレンダー: &mut カレンダーストア,
    通知: &mut 通知ストア,
    クエスト: &mut クエストストア,
    人物: &mut 人物ストア,
    タイムライン: &mut タイムラインストア,
    エリア: &mut エリアストア,
    選択肢: &mut 選択肢ストア,
    天候: &mut 天候ストア,
    アプリ状態: &mut crate::UI設定::アプリ状態,
) {
    for 効果 in 効果群 {
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
            イベント効果::クエスト追加 { 名前, 説明 } => {
                クエスト.追加(名前, 説明);
            }
            イベント効果::クエスト完了(名前) => {
                クエスト.完了にする(名前);
            }
            イベント効果::人物追加 { 名前, 説明 } => {
                人物.追加(名前, 説明);
            }
            イベント効果::タイムライン追記(テキスト) => {
                タイムライン.追記(時間.日付, 時間.時間帯, テキスト);
            }
            イベント効果::エリア登録 { 名前, 座標, 半径 } => {
                エリア.追加(名前, *座標, *半径);
            }
            イベント効果::地図マーカー追加 { .. } => {
                // Map ウィンドウ実装時に対応
            }
            イベント効果::タブ切替(名前) => {
                use crate::UI設定::エディタビュー;
                let ビュー = match 名前.as_str() {
                    "World" | "シーン" => Some(エディタビュー::シーン),
                    "Tetra" | "四面体" => Some(エディタビュー::四面体),
                    "Calendar" | "カレンダー" => Some(エディタビュー::カレンダー),
                    "Messages" | "メッセージ" => Some(エディタビュー::メッセージ),
                    "Documents" | "ドキュメント" => Some(エディタビュー::ドキュメント),
                    "Map" | "マップ" => Some(エディタビュー::マップ),
                    "Profiles" | "人物" => Some(エディタビュー::人物),
                    "Timeline" | "タイムライン" => Some(エディタビュー::タイムライン),
                    "Inventory" | "インベントリ" => Some(エディタビュー::インベントリ),
                    _ => None,
                };
                if let Some(v) = ビュー { アプリ状態.現在のビュー = v; }
            }
            イベント効果::選択肢表示 { タイトル, 選択肢群 } => {
                選択肢.表示中 = true;
                選択肢.タイトル = タイトル.clone();
                選択肢.選択肢群 = 選択肢群.clone();
                選択肢.カーソル位置 = 0;
            }
            イベント効果::天候変更(種別) => {
                天候.現在 = *種別;
            }
        }
    }
}

/// エリア判定: プレイヤー位置から現在のエリアを更新
pub fn エリア判定システム(
    mut エリア: ResMut<エリアストア>,
    mut 通知: ResMut<通知ストア>,
    カメラ: Query<&Transform, With<crate::カメラ制御::カメラ操作>>,
) {
    let Ok(transform) = カメラ.get_single() else { return };
    let pos = transform.translation;

    let mut 新エリア: Option<String> = None;
    for area in &エリア.エリア群 {
        if pos.distance(area.座標) <= area.半径 {
            新エリア = Some(area.名前.clone());
            break;
        }
    }

    if 新エリア != エリア.現在のエリア {
        if let Some(ref 名前) = 新エリア {
            通知.追加(&format!("── {} ──", 名前));
        }
        エリア.現在のエリア = 新エリア;
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

/// キーボードによる時間進行とタブ切替
pub fn 操作システム(
    keys: Res<ButtonInput<KeyCode>>,
    mut 時間: ResMut<ゲーム時間>,
    mut 通知: ResMut<通知ストア>,
    mut 選択肢: ResMut<選択肢ストア>,
    mut アプリ状態: ResMut<crate::UI設定::アプリ状態>,
    // 選択肢決定時に効果を適用するためのリソース群
    mut フラグ: ResMut<フラグストア>,
    mut ドキュメント: ResMut<ドキュメントストア>,
    mut メッセージ: ResMut<メッセージストア>,
    mut カレンダー: ResMut<カレンダーストア>,
    mut クエスト: ResMut<クエストストア>,
    mut 人物: ResMut<人物ストア>,
    mut タイムライン: ResMut<タイムラインストア>,
    mut エリア: ResMut<エリアストア>,
    mut 天候: ResMut<天候ストア>,
) {
    use crate::UI設定::エディタビュー;

    // --- 選択肢モード ---
    if 選択肢.表示中 {
        if keys.just_pressed(KeyCode::ArrowUp) {
            if 選択肢.カーソル位置 > 0 { 選択肢.カーソル位置 -= 1; }
        }
        if keys.just_pressed(KeyCode::ArrowDown) {
            if 選択肢.カーソル位置 + 1 < 選択肢.選択肢群.len() {
                選択肢.カーソル位置 += 1;
            }
        }
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyF) {
            let idx = 選択肢.カーソル位置;
            if idx < 選択肢.選択肢群.len() {
                let 効果群 = 選択肢.選択肢群[idx].効果群.clone();
                選択肢.表示中 = false;
                選択肢.選択肢群.clear();
                効果適用(
                    &効果群, &時間, &mut フラグ, &mut ドキュメント,
                    &mut メッセージ, &mut カレンダー, &mut 通知, &mut クエスト,
                    &mut 人物, &mut タイムライン, &mut エリア, &mut 選択肢,
                    &mut 天候, &mut アプリ状態,
                );
            }
        }
        if keys.just_pressed(KeyCode::Escape) {
            選択肢.表示中 = false;
            選択肢.選択肢群.clear();
        }
        return; // 選択肢モード中は他の操作を無視
    }

    // --- 時間進行 ---
    if keys.just_pressed(KeyCode::KeyT) {
        時間.時間帯送り();
        let 帯名 = match 時間.時間帯 {
            時間帯::朝 => "朝",
            時間帯::昼 => "昼",
            時間帯::夕 => "夕方",
            時間帯::夜 => "夜",
        };
        通知.追加(&format!("{}  {}", 時間.日付, 帯名));
    }
    if keys.just_pressed(KeyCode::KeyN) {
        時間.日送り();
        通知.追加(&format!("翌朝 — {}", 時間.日付));
    }

    // --- タブ切替 ---
    if keys.just_pressed(KeyCode::Digit1) { アプリ状態.現在のビュー = エディタビュー::シーン; }
    if keys.just_pressed(KeyCode::Digit2) { アプリ状態.現在のビュー = エディタビュー::四面体; }
    if keys.just_pressed(KeyCode::Digit3) { アプリ状態.現在のビュー = エディタビュー::カレンダー; }
    if keys.just_pressed(KeyCode::Digit4) { アプリ状態.現在のビュー = エディタビュー::メッセージ; }
    if keys.just_pressed(KeyCode::Digit5) { アプリ状態.現在のビュー = エディタビュー::ドキュメント; }
    if keys.just_pressed(KeyCode::Digit6) { アプリ状態.現在のビュー = エディタビュー::マップ; }
    if keys.just_pressed(KeyCode::Digit7) { アプリ状態.現在のビュー = エディタビュー::人物; }
    if keys.just_pressed(KeyCode::Digit8) { アプリ状態.現在のビュー = エディタビュー::タイムライン; }
    if keys.just_pressed(KeyCode::Digit9) { アプリ状態.現在のビュー = エディタビュー::インベントリ; }
}
