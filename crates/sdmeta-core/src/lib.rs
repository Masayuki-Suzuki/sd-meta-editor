// SPDX-License-Identifier: Apache-2.0

//! # sdmeta-core
//!
//! SD Meta Editor の中核ライブラリ。GUI には一切依存しない。
//!
//! **実装中。** 現時点で入っているのは [`error`] のみ。設計は以下を参照すること。
//!
//! - 要件定義書: `docs/requirements.md`
//! - 実装ロードマップ: `docs/roadmap.md`
//! - 設計判断: `docs/adr/`
//!
//! ## モジュール構成
//!
//! | モジュール | 責務 | 状態 |
//! |---|---|---|
//! | [`error`]   | クレート共通のエラー型（ADR-0001 C-6） | **実装済み** |
//! | `container` | JPEG / PNG / WebP のチャンク・セグメント走査とロスレス書き出し | 未着手 |
//! | `adapter`   | ツール別のキー → 正規フィールドへの写像（A1111 系、ComfyUI、NovelAI ほか） | 未着手 |
//! | `model`     | 正規フィールドモデル（要件定義書 FR-3） | 未着手 |
//! | `parser`    | 引用符対応トークナイザ（要件定義書 FR-4） | 未着手 |
//! | `preset`    | 削除プリセットの定義と適用（要件定義書 FR-5） | 未着手 |
//! | `stealth`   | アルファ LSB ステガノグラフィの検出・除去（ADR-0004） | 未着手 |
//! | `verify`    | 保存後の自己検証（要件定義書 FR-7） | 未着手 |
//!
//! ## 不変条件
//!
//! - 既定の書き出しパスで画素データをデコード・再エンコードしない
//! - 唯一の例外は stealth 除去時の PNG 再エンコード（ADR-0004）
//! - 未知のキー・チャンクも削除可能であること
//! - ICC プロファイル・EXIF Orientation・C2PA は常に保持する（FR-9、ADR-0011）
//! - 信頼できない入力でパニックしない。すべて [`Error`] として返す

pub mod error;

pub use error::{Container, Error, Protected, Result};
