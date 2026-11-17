// SPDX-License-Identifier: Apache-2.0

// CLI は標準出力・標準エラーへの出力が本務なので、ワークスペースの禁止をここで解除する。
// `sdmeta-core` 側では禁止のまま（ライブラリが勝手に出力してはならない）。
// Cargo は `lints.workspace = true` とマニフェスト側の個別上書きを併用できないため、
// クレート属性で解除している。
#![allow(clippy::print_stdout, clippy::print_stderr)]

//! SD Meta Editor の CLI。
//!
//! **現時点では実装は未着手。**
//!
//! 最初に実装するのは読み取り専用の `dump` サブコマンド（全チャンク・全 EXIF タグの列挙）。
//! これにより GUI を作らずにパーサを検証でき、同時に
//! `docs/research/metadata-formats.md` の実地調査が進む。

fn main() {}
