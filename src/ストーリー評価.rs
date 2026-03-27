// src/ストーリー評価.rs
// Bevy システム関数 — ストーリー評価・効果適用・エリア判定・通知更新・操作

use bevy::prelude::*;
use crate::ストーリーエンジン::*;
use crate::UI設定::*;
use crate::カメラ制御::カメラ操作;

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
    mut アプリ状態: ResMut<アプリ状態>,
    mut 会話: ResMut<会話ストア>,
    カメラ: Query<&Transform, With<カメラ操作>>,
) {
    // 選択肢の決定済み効果があれば適用 (F調査の結果もここで処理)
    if let Some(効果群) = 選択肢.決定済み効果.take() {
        効果適用(
            &効果群, &時間, &mut フラグ, &mut ドキュメント,
            &mut メッセージ, &mut カレンダー, &mut 通知, &mut クエスト,
            &mut 人物, &mut タイムライン, &mut エリア, &mut 選択肢,
            &mut 天候, &mut アプリ状態, Some(&mut 会話), None,
        );
        return;
    }

    // 選択肢・会話表示中はイベント評価を停止
    if 選択肢.表示中 || 会話.表示中 { return; }

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
                イベント条件::アイテム所持(名前) => {
                    フラグ.オン(&format!("アイテム_{}", 名前))
                }
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
            &mut 天候, &mut アプリ状態, Some(&mut 会話), None,
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
    アプリ状態: &mut アプリ状態,
    mut 会話: Option<&mut 会話ストア>,
    mut 調査: Option<&mut 調査ポイントストア>,
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
            イベント効果::地図マーカー追加 { 座標, 名前 } => {
                // マーカーをエリアとして小半径で登録
                エリア.追加(名前, *座標, 2.0);
            }
            イベント効果::タブ切替(名前) => {
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
            イベント効果::会話開始(台詞群) => {
                if let Some(ref mut 会話) = 会話 {
                    会話.表示中 = true;
                    会話.台詞群 = 台詞群.clone();
                    会話.現在位置 = 0;
                }
            }
            イベント効果::調査ポイント登録 { 名前, 座標, 半径, 効果群: pt効果群 } => {
                if let Some(ref mut 調査) = 調査 {
                    調査.ポイント群.push(調査ポイント {
                        名前: 名前.clone(),
                        座標: *座標,
                        半径: *半径,
                        効果群: pt効果群.clone(),
                        使用済み: false,
                    });
                }
            }
        }
    }
}

/// エリア判定: プレイヤー位置から現在のエリアを更新
pub fn エリア判定システム(
    mut エリア: ResMut<エリアストア>,
    mut 通知: ResMut<通知ストア>,
    カメラ: Query<&Transform, With<カメラ操作>>,
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

/// キーボードによる時間進行・タブ切替・ヘルプ・会話
pub fn 操作システム(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut 時間: ResMut<ゲーム時間>,
    mut 通知: ResMut<通知ストア>,
    mut 選択肢: ResMut<選択肢ストア>,
    mut 会話: ResMut<会話ストア>,
    mut ヘルプ: ResMut<ヘルプ表示>,
    mut アプリ状態: ResMut<アプリ状態>,
    mut フラグ: ResMut<フラグストア>,
    mut ドキュメント: ResMut<ドキュメントストア>,
    mut メッセージ: ResMut<メッセージストア>,
    mut カレンダー: ResMut<カレンダーストア>,
    mut クエスト: ResMut<クエストストア>,
    mut 天候: ResMut<天候ストア>,
) {
    // --- ヘルプトグル (最優先) ---
    if keys.just_pressed(KeyCode::KeyH) {
        ヘルプ.表示中 = !ヘルプ.表示中;
        return;
    }
    if ヘルプ.表示中 {
        if keys.just_pressed(KeyCode::Escape) { ヘルプ.表示中 = false; }
        return;
    }

    // --- 会話モード ---
    if 会話.表示中 {
        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space)
            || keys.just_pressed(KeyCode::KeyF) || mouse.just_pressed(MouseButton::Left)
        {
            会話.現在位置 += 1;
            if 会話.現在位置 >= 会話.台詞群.len() {
                会話.表示中 = false;
                会話.台詞群.clear();
                会話.現在位置 = 0;
            }
        }
        if keys.just_pressed(KeyCode::Escape) {
            会話.表示中 = false;
            会話.台詞群.clear();
        }
        return;
    }

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
                選択肢.決定済み効果 = Some(効果群); // 次フレームでストーリー評価システムが適用
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

// =============================================================================
// UI 表示更新システム群
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

pub fn 日付表示更新(
    時間: Res<ゲーム時間>,
    mut query: Query<&mut Text, With<日付表示>>,
) {
    if 時間.is_changed() {
        let 帯名 = match 時間.時間帯 {
            時間帯::朝 => "朝", 時間帯::昼 => "昼", 時間帯::夕 => "夕", 時間帯::夜 => "夜",
        };
        for mut text in &mut query {
            text.0 = format!("{} [{}] | Day {}", 時間.日付, 帯名, 時間.経過日数 + 1);
        }
    }
}

pub fn カレンダー表示更新(
    mut commands: Commands,
    カレンダー: Res<カレンダーストア>,
    時間: Res<ゲーム時間>,
    query: Query<(Entity, &カレンダー内容表示)>,
) {
    if !カレンダー.is_changed() && !時間.is_changed() { return; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            let 月 = 時間.日付.月;
            let mut 日付一覧: Vec<_> = カレンダー.予定群.keys().filter(|d| d.月 == 月).copied().collect();
            日付一覧.sort_by_key(|d| (d.月, d.日));
            parent.spawn((Text::new(format!("── {}月 ──", 月)), TextFont { font_size: 13.0, ..default() }, TextColor(サイバーアクセント色)));
            for 日付 in 日付一覧 {
                for 予定 in カレンダー.取得(&日付) {
                    let is_today = 日付 == 時間.日付;
                    let マーカー = if is_today { "▶ " } else { "  " };
                    let color = if 予定.重要 { サイバー重要色 } else if is_today { サイバーアクセント色 } else { サイバーテキスト色 };
                    parent.spawn((Text::new(format!("{}{}日  {}", マーカー, 日付.日, 予定.内容)), TextFont { font_size: 12.0, ..default() }, TextColor(color)));
                }
            }
        });
    }
}

pub fn メッセージ表示更新(
    mut commands: Commands,
    mut メッセージ: ResMut<メッセージストア>,
    query: Query<(Entity, &メッセージ内容表示)>,
) {
    if !メッセージ.is_changed() { return; }
    for m in &mut メッセージ.メッセージ群 { m.既読 = true; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            if メッセージ.メッセージ群.is_empty() {
                parent.spawn((Text::new("メッセージはありません"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバー薄文字色)));
                return;
            }
            for msg in メッセージ.メッセージ群.iter().rev() {
                parent.spawn((Text::new(format!("From: {} ({})", msg.送信者, msg.日付)), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーアクセント色)));
                parent.spawn((Text::new(msg.本文.clone()), TextFont { font_size: 12.0, ..default() }, TextColor(サイバーテキスト色)));
                parent.spawn((Text::new("─────────────────"), TextFont { font_size: 10.0, ..default() }, TextColor(サイバーボーダー色)));
            }
        });
    }
}

pub fn ドキュメント表示更新(
    mut commands: Commands,
    mut ドキュメント: ResMut<ドキュメントストア>,
    query: Query<(Entity, &ドキュメント一覧表示)>,
) {
    if !ドキュメント.is_changed() { return; }
    for d in &mut ドキュメント.文書群 { d.既読 = true; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            if ドキュメント.文書群.is_empty() {
                parent.spawn((Text::new("発見した文書はありません"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバー薄文字色)));
                return;
            }
            for doc in &ドキュメント.文書群 {
                let カテゴリ名 = match doc.カテゴリ { ドキュメントカテゴリ::手紙=>"手紙", ドキュメントカテゴリ::日記=>"日記", ドキュメントカテゴリ::地図=>"地図", ドキュメントカテゴリ::公文書=>"公文書", ドキュメントカテゴリ::メモ=>"メモ" };
                parent.spawn((Text::new(format!("[{}] {}", カテゴリ名, doc.タイトル)), TextFont { font_size: 13.0, ..default() }, TextColor(サイバーアクセント色)));
                parent.spawn((Text::new(doc.本文.clone()), TextFont { font_size: 12.0, ..default() }, TextColor(サイバーテキスト色)));
                parent.spawn((Text::new(""), TextFont { font_size: 6.0, ..default() }, TextColor(サイバーボーダー色)));
            }
        });
    }
}

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
                parent.spawn((Node { padding: UiRect::all(Val::Px(8.0)), margin: UiRect::bottom(Val::Px(4.0)), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85))))
                    .with_child((Text::new(n.テキスト.clone()), TextFont { font_size: 13.0, ..default() }, TextColor(サイバーアクセント色)));
            }
        });
    }
}

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
            parent.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(16.0)), row_gap: Val::Px(8.0), ..default() }, BackgroundColor(Color::srgba(0.03, 0.05, 0.1, 0.95))))
                .with_children(|panel| {
                    panel.spawn((Text::new(選択肢.タイトル.clone()), TextFont { font_size: 14.0, ..default() }, TextColor(サイバーアクセント色)));
                    for (i, item) in 選択肢.選択肢群.iter().enumerate() {
                        let is_sel = i == 選択肢.カーソル位置;
                        let marker = if is_sel { "▶ " } else { "  " };
                        let color = if is_sel { サイバーアクセント色 } else { サイバーテキスト色 };
                        panel.spawn((Text::new(format!("{}{}", marker, item.テキスト)), TextFont { font_size: 13.0, ..default() }, TextColor(color)));
                    }
                    panel.spawn((Text::new("[↑↓] 選択  [Enter/F] 決定  [Esc] 戻る"), TextFont { font_size: 10.0, ..default() }, TextColor(サイバー薄文字色)));
                });
        });
    }
}

pub fn サイドバー更新(
    mut commands: Commands,
    フラグ: Res<フラグストア>,
    ドキュメント: Res<ドキュメントストア>,
    メッセージ: Res<メッセージストア>,
    クエスト: Res<クエストストア>,
    query: Query<(Entity, &サイドバー手がかり表示)>,
) {
    if !フラグ.is_changed() && !ドキュメント.is_changed() && !メッセージ.is_changed() && !クエスト.is_changed() { return; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            parent.spawn((Text::new(format!("Docs: {}  Msg: {}", ドキュメント.文書群.len(), メッセージ.メッセージ群.len())), TextFont { font_size: 11.0, ..default() }, TextColor(サイバー薄文字色)));
            let active: Vec<_> = クエスト.クエスト群.iter().filter(|q| !q.完了).collect();
            if !active.is_empty() {
                parent.spawn((Text::new("── Quests ──"), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーアクセント色)));
                for q in &active { parent.spawn((Text::new(format!("  ◇ {}", q.名前)), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーテキスト色))); }
            }
            let done: Vec<_> = クエスト.クエスト群.iter().filter(|q| q.完了).collect();
            for q in &done { parent.spawn((Text::new(format!("  ◆ {} ✓", q.名前)), TextFont { font_size: 11.0, ..default() }, TextColor(サイバー薄文字色))); }
            parent.spawn((Text::new("── Clues ──"), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーアクセント色)));
            let mut flags: Vec<_> = フラグ.フラグ群.iter().filter(|(_, v)| **v).map(|(k, _)| k.clone()).collect();
            flags.sort();
            if flags.is_empty() { parent.spawn((Text::new("  (none yet)"), TextFont { font_size: 11.0, ..default() }, TextColor(サイバー薄文字色))); }
            else { for f in &flags { parent.spawn((Text::new(format!("  * {}", f)), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーテキスト色))); } }
        });
    }
}

pub fn マップ表示更新(
    mut commands: Commands,
    エリア: Res<エリアストア>,
    query: Query<(Entity, &マップ内容表示)>,
) {
    if !エリア.is_changed() { return; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            if エリア.エリア群.is_empty() { parent.spawn((Text::new("未探索"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバー薄文字色))); return; }
            parent.spawn((Text::new("── 発見済みエリア ──"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバーアクセント色)));
            for area in &エリア.エリア群 {
                let is_cur = エリア.現在のエリア.as_deref() == Some(area.名前.as_str());
                let m = if is_cur { "▶ " } else { "  " };
                let c = if is_cur { サイバーアクセント色 } else { サイバーテキスト色 };
                parent.spawn((Text::new(format!("{}{} ({:.0},{:.0},{:.0})", m, area.名前, area.座標.x, area.座標.y, area.座標.z)), TextFont { font_size: 12.0, ..default() }, TextColor(c)));
            }
            if let Some(ref n) = エリア.現在のエリア { parent.spawn((Text::new(format!("\n現在地: {}", n)), TextFont { font_size: 13.0, ..default() }, TextColor(サイバーアクセント色))); }
            else { parent.spawn((Text::new("\n現在地: (エリア外)"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバー薄文字色))); }
        });
    }
}

pub fn 人物表示更新(
    mut commands: Commands,
    人物: Res<人物ストア>,
    query: Query<(Entity, &人物内容表示)>,
) {
    if !人物.is_changed() { return; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            if 人物.人物群.is_empty() { parent.spawn((Text::new("まだ誰にも会っていない"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバー薄文字色))); return; }
            for (i, p) in 人物.人物群.iter().enumerate() {
                parent.spawn((Text::new(format!("#{} {}", i+1, p.名前)), TextFont { font_size: 14.0, ..default() }, TextColor(サイバーアクセント色)));
                parent.spawn((Text::new(p.説明.clone()), TextFont { font_size: 12.0, ..default() }, TextColor(サイバーテキスト色)));
                parent.spawn((Text::new(""), TextFont { font_size: 4.0, ..default() }, TextColor(サイバーボーダー色)));
            }
        });
    }
}

pub fn タイムライン表示更新(
    mut commands: Commands,
    タイムライン: Res<タイムラインストア>,
    query: Query<(Entity, &タイムライン内容表示)>,
) {
    if !タイムライン.is_changed() { return; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            if タイムライン.エントリ群.is_empty() { parent.spawn((Text::new("まだ何も起きていない"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバー薄文字色))); return; }
            for entry in タイムライン.エントリ群.iter().rev() {
                let 帯 = match entry.時間帯 { 時間帯::朝=>"朝", 時間帯::昼=>"昼", 時間帯::夕=>"夕", 時間帯::夜=>"夜" };
                parent.spawn((Text::new(format!("[{} {}] {}", entry.日付, 帯, entry.テキスト)), TextFont { font_size: 11.0, ..default() }, TextColor(サイバーテキスト色)));
            }
        });
    }
}

pub fn インベントリ表示更新(
    mut commands: Commands,
    フラグ: Res<フラグストア>,
    query: Query<(Entity, &インベントリ内容表示)>,
) {
    if !フラグ.is_changed() { return; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|parent| {
            parent.spawn((Text::new("── 所持品 ──"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバーアクセント色)));
            let mut items: Vec<_> = フラグ.フラグ群.iter().filter(|(k, v)| k.starts_with("アイテム_") && **v).map(|(k, _)| k.strip_prefix("アイテム_").unwrap_or(k).to_string()).collect();
            items.sort();
            if items.is_empty() { parent.spawn((Text::new("  何も持っていない"), TextFont { font_size: 12.0, ..default() }, TextColor(サイバー薄文字色))); }
            else { for item in &items { parent.spawn((Text::new(format!("  ◆ {}", item)), TextFont { font_size: 12.0, ..default() }, TextColor(サイバーテキスト色))); } }
        });
    }
}

/// 天候に応じてフォグ色と環境光を変更
pub fn 天候ビジュアル更新(
    天候: Res<天候ストア>,
    mut fog_query: Query<&mut DistanceFog>,
    mut ambient: ResMut<AmbientLight>,
) {
    if !天候.is_changed() { return; }
    let (fog_color, brightness) = match 天候.現在 {
        天候種別::晴れ => (Color::srgb(0.53, 0.72, 0.9), 800.0),
        天候種別::曇り => (Color::srgb(0.45, 0.50, 0.55), 400.0),
        天候種別::雨   => (Color::srgb(0.30, 0.35, 0.40), 250.0),
        天候種別::嵐   => (Color::srgb(0.15, 0.15, 0.20), 120.0),
        天候種別::霧   => (Color::srgb(0.60, 0.60, 0.60), 350.0),
    };
    for mut fog in &mut fog_query {
        fog.color = fog_color;
    }
    ambient.brightness = brightness;
}

/// ヘルプオーバーレイの表示
pub fn ヘルプ表示更新(
    mut commands: Commands,
    ヘルプ: Res<ヘルプ表示>,
    query: Query<(Entity, &ヘルプオーバーレイ)>,
) {
    if !ヘルプ.is_changed() { return; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        if !ヘルプ.表示中 { continue; }
        commands.entity(entity).with_children(|parent| {
            parent.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(20.0)), row_gap: Val::Px(3.0), ..default() }, BackgroundColor(Color::srgba(0.03, 0.05, 0.1, 0.95))))
                .with_children(|p| {
                    let lines = [
                        ("=== CONTROLS ===", true),
                        ("", false),
                        ("W/A/S/D    移動", false),
                        ("Right-drag カメラ回転", false),
                        ("Space      ジャンプ / 浮上", false),
                        ("Shift      ダッシュ / 潜水", false),
                        ("Q/E        飛行モード", false),
                        ("R          位置リセット", false),
                        ("", false),
                        ("T          時間帯を進める", false),
                        ("N          翌朝まで睡眠", false),
                        ("F / 左クリック  周囲を調べる", false),
                        ("", false),
                        ("1-9        タブ切替", false),
                        ("  1:World 2:Tetra 3:Calendar", false),
                        ("  4:Messages 5:Documents", false),
                        ("  6:Map 7:Profiles 8:Timeline 9:Inventory", false),
                        ("", false),
                        ("↑↓ Enter   選択肢操作", false),
                        ("Enter/Space 会話送り", false),
                        ("Esc        キャンセル / 閉じる", false),
                        ("", false),
                        ("F5         セーブ", false),
                        ("F9         ロード", false),
                        ("H          このヘルプを閉じる", false),
                    ];
                    for (text, accent) in lines {
                        let color = if accent { サイバーアクセント色 } else { サイバーテキスト色 };
                        p.spawn((Text::new(text), TextFont { font_size: 12.0, ..default() }, TextColor(color)));
                    }
                });
        });
    }
}

/// 会話オーバーレイの表示
pub fn 会話表示更新(
    mut commands: Commands,
    会話: Res<会話ストア>,
    query: Query<(Entity, &会話オーバーレイ)>,
) {
    if !会話.is_changed() { return; }
    for (entity, _) in &query {
        commands.entity(entity).despawn_descendants();
        if !会話.表示中 || 会話.現在位置 >= 会話.台詞群.len() { continue; }
        let 台詞 = &会話.台詞群[会話.現在位置];
        commands.entity(entity).with_children(|parent| {
            parent.spawn((Node { flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(16.0)), row_gap: Val::Px(6.0), ..default() }, BackgroundColor(Color::srgba(0.02, 0.04, 0.08, 0.92))))
                .with_children(|p| {
                    p.spawn((Text::new(台詞.話者.clone()), TextFont { font_size: 12.0, ..default() }, TextColor(サイバーアクセント色)));
                    p.spawn((Text::new(台詞.本文.clone()), TextFont { font_size: 14.0, ..default() }, TextColor(サイバーテキスト色)));
                    p.spawn((Text::new(format!("[Enter/Space] 次へ ({}/{})", 会話.現在位置 + 1, 会話.台詞群.len())), TextFont { font_size: 10.0, ..default() }, TextColor(サイバー薄文字色)));
                });
        });
    }
}

/// 天候ステータス表示
pub fn 天候表示更新(
    天候: Res<天候ストア>,
    mut query: Query<&mut Text, With<天候表示>>,
) {
    if !天候.is_changed() { return; }
    let 名前 = match 天候.現在 {
        天候種別::晴れ => "☀ 晴れ",
        天候種別::曇り => "☁ 曇り",
        天候種別::雨 => "🌧 雨",
        天候種別::嵐 => "⛈ 嵐",
        天候種別::霧 => "🌫 霧",
    };
    for mut text in &mut query { text.0 = 名前.to_string(); }
}

/// 未読バッジ: タブテキストに未読数を表示
pub fn 未読バッジ更新(
    メッセージ: Res<メッセージストア>,
    ドキュメント: Res<ドキュメントストア>,
    mut query: Query<(&mut Text, &タブテキスト)>,
) {
    if !メッセージ.is_changed() && !ドキュメント.is_changed() { return; }
    let msg_unread = メッセージ.未読数();
    let doc_unread = ドキュメント.未読数();
    for (mut text, tab) in &mut query {
        text.0 = match tab.0 {
            エディタビュー::メッセージ => {
                if msg_unread > 0 { format!("Messages({})", msg_unread) } else { "Messages".into() }
            }
            エディタビュー::ドキュメント => {
                if doc_unread > 0 { format!("Docs({})", doc_unread) } else { "Documents".into() }
            }
            エディタビュー::シーン => "World".into(),
            エディタビュー::四面体 => "Tetra".into(),
            エディタビュー::カレンダー => "Calendar".into(),
            エディタビュー::マップ => "Map".into(),
            エディタビュー::人物 => "Profiles".into(),
            エディタビュー::タイムライン => "Timeline".into(),
            エディタビュー::インベントリ => "Inventory".into(),
        };
    }
}

/// F調査システム: 周囲の調査ポイントを調べる
pub fn 調査システム(
    keys: Res<ButtonInput<KeyCode>>,
    mut 調査: ResMut<調査ポイントストア>,
    mut 通知: ResMut<通知ストア>,
    mut 選択肢: ResMut<選択肢ストア>,
    会話: Res<会話ストア>,
    ヘルプ: Res<ヘルプ表示>,
    カメラ: Query<&Transform, With<カメラ操作>>,
) {
    // モーダル中は無効
    if 選択肢.表示中 || 会話.表示中 || ヘルプ.表示中 { return; }

    if keys.just_pressed(KeyCode::KeyF) {
        let Ok(cam_t) = カメラ.get_single() else { return };
        let pos = cam_t.translation;

        let mut found = false;
        for point in &mut 調査.ポイント群 {
            if !point.使用済み && pos.distance(point.座標) <= point.半径 {
                point.使用済み = true;
                // 効果群を選択肢ストアの決定済み効果に入れて、次フレームでストーリー評価が適用
                選択肢.決定済み効果 = Some(point.効果群.clone());
                found = true;
                break;
            }
        }
        if !found {
            通知.追加("周囲に調べられるものはない。");
        }
    }
}

/// 昼夜サイクル: 時間帯に応じて太陽光の色・角度を変更
pub fn 昼夜サイクル更新(
    時間: Res<ゲーム時間>,
    mut sun_query: Query<(&mut DirectionalLight, &mut Transform), Without<crate::カメラ制御::カメラ操作>>,
) {
    if !時間.is_changed() { return; }
    let (color, illuminance, sun_y, sun_x) = match 時間.時間帯 {
        時間帯::朝 => (Color::srgb(1.0, 0.85, 0.7), 10_000.0, 80.0, 30.0),   // 低い朝日
        時間帯::昼 => (Color::srgb(1.0, 0.98, 0.9), 15_000.0, 120.0, 60.0),   // 高い太陽
        時間帯::夕 => (Color::srgb(1.0, 0.6, 0.3), 8_000.0, 80.0, -30.0),     // オレンジの夕日
        時間帯::夜 => (Color::srgb(0.3, 0.3, 0.5), 1_000.0, 40.0, -60.0),     // 月明かり
    };
    for (mut light, mut transform) in &mut sun_query {
        light.color = color;
        light.illuminance = illuminance;
        *transform = Transform::from_xyz(sun_x, sun_y, 40.0).looking_at(Vec3::ZERO, Vec3::Y);
    }
}
