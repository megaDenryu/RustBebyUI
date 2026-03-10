# Bevy UI 技術考察レポート

> [1.94.0 2026-03-02 時点の Rust/Bevy 環境下での考察]

## 1. Bevy における UI 表現の仕組み

Bevy の UI システムは、エンジンのコアアーキテクチャである **ECS (Entity Component System)** に完全に統合されています。
すべての UI 要素（ボタン、テキスト、コンテナ）は **Entity** であり、その見た目やレイアウト（Flexbox/Grid）は **Component** (例: `Node`, `Text`, `UiImage`) によって定義されます。

### 特徴
- **ECS 直結:** UI 要素の状態（ボタンのホバー、入力テキスト）は通常の ECS System でクエリして操作できます。
- **Taffy レイアウト:** Web 標準に近い Flexbox や CSS-Grid 形式でレイアウトを計算します。
- **宣言的 vs 手続き的:** 標準の Bevy API は「命令的（手続き的）」ですが、近年（2026年時点）は「宣言的」あるいは「フルエントな（流れるような）」記述をサポートする拡張が成熟しています。

## 2. メソッドチェーン（フルエント API）の可否

結論から述べますと、**「標準機能では不十分だが、拡張ライブラリ（Crate）を用いれば SengenUI のようなエレメントのメソッドチェーンが可能」** です。

### SengenUI との比較
| 特徴 | SengenUI (TypeScript) | Bevy UI (Rust) |
|---|---|---|
| **基本構文** | `div().child(span())` | `commands.spawn(NodeBundle).with_children(...)` |
| **拡張構文** | メソッドチェーン (Fluent) | **Sickle UI** や **Famiq** 等で Fluent API 化が可能 |
| **状態管理** | Orchestrator (Class) | **ECS Systems / Components / Observors** |

### Bevy でのメソッドチェーンの例 (Sickle UI 等を想定)
```rust
// 概念的なイメージ（SengenUI に近い書き方）
ui_builder.column(|column| {
    column.spawn(ButtonBundle::default())
        .with_text("送信")
        .on_click(|world| { /* イベントハンドラ */ })
        .style(|msg| {
            msg.width(percent(100.0))
               .padding(px(10.0))
        });
});
```

## 3. 推奨されるアプローチ

2026 年時点で Bevy UI を開発するなら、以下のいずれかを選択するのが主流です。

1.  **Sickle UI / Famiq (Fluent API):** SengenUI に最も近い体験。メソッドチェーンで UI を構築する。
2.  **Bevy Feathers (Official Extension):** Bevy 0.18 から登場した公式のウィジェットセット。標準の ECS らしさを保ちつつ、ボタンやスライダーなどの部品を簡単に配置できる。
3.  **bevy_egui (Immediate Mode):** デバッグツールやツール開発に最適。見た目の柔軟性は低いが、実装が極めて速い。

## 4. 総評
SengenUI のような「関数型で UI を定義し、状態はオーケストレーターに任せる」という設計思想は、Bevy においても **「UI Plugin」** と **「Observer/System」** の組み合わせで高い次元で再現可能です。
今回は、できるだけメソッドチェーンや宣言的な記述を取り入れた構成でサンプルを作っていく方針とします。
