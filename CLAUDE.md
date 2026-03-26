# CLAUDE.md - RustBebyUI プロジェクトガイド

## プロジェクト概要
Bevy 0.15 (Rust製ゲームエンジン) を使ったボクセルワールドエディタ。
ウィンドウタイトル: "One O Net - Cyber Studio v0.9"

## 技術スタック
- **言語:** Rust (Edition 2024)
- **フレームワーク:** Bevy 0.15 (ECS アーキテクチャ)
- **主要クレート:** noise (地形生成), futures-lite (非同期処理)

## ビルド・実行
```bash
cargo build          # ビルド
cargo run            # 実行
cargo check          # 型チェック
cargo clippy         # Lint
cargo fmt            # フォーマット
```

## ソースコード構成 (src/)
- `main.rs` — エントリポイント、プラグイン登録
- `voxel_world.rs` — ボクセル型定義 (9種) と Perlin ノイズ地形生成
- `ui_logic.rs` — サイバーテーマ VS Code 風 UI (アクティビティバー、サイドバー、タブ、ステータスバー)
- `camera_controller.rs` — 飛行/歩行モード切替、水中物理
- `chunk_system.rs` — 立方体チャンクの LOD 管理・非同期ロード
- `meshing.rs` — Greedy meshing アルゴリズム
- `tetra_world.rs` — 四面体ボクセルシステム (立方体を5四面体に分割)
- `tetra_chunk_system.rs` — 四面体チャンク管理
- `tetra_meshing.rs` — 四面体メッシュ生成
- `collision.rs` — 衝突判定 (7点チェック)

## アーキテクチャ方針
- 仕様書駆動開発 (SDD): `_doc/` 配下に仕様・設計ドキュメント
- Bevy ECS パターン: Component 定義 → Plugin 化 → System 実装
- 二重レンダリング: 立方体ボクセルと四面体ボクセルをタブで切替

## コーディング規約
- 識別子は Rust 慣習 (snake_case, PascalCase)
- コメント・仕様書・ドメインロジックは日本語
- 新機能追加時は `_doc/` に仕様を先に定義
