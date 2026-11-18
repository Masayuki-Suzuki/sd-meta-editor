// SPDX-License-Identifier: Apache-2.0

//! `sdmeta-core` のエラー型。
//!
//! # 設計上の制約
//!
//! - **公開 API は `anyhow::Error` や `Box<dyn Error>` を返さない。** GUI は
//!   エラー種別で分岐してトーストの文面を変える必要があり（ADR-0001 C-6）、
//!   型が潰れていると分岐できない
//! - **パニックしない。** 本クレートは信頼できないファイルをパースする。
//!   破損・切り詰め・不正な長さ宣言はすべて [`Error`] として返す
//! - [`Error`] は `#[non_exhaustive]`。バリアントの追加を破壊的変更にしない

use std::ops::Range;
use std::path::PathBuf;

/// `sdmeta-core` の [`Result`](std::result::Result) 別名。
pub type Result<T> = std::result::Result<T, Error>;

/// 扱うコンテナ形式。エラーがどの形式の処理で起きたかを示す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Container {
    /// PNG。
    Png,
    /// JPEG。
    Jpeg,
    /// WebP。
    WebP,
}

impl std::fmt::Display for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::WebP => "WebP",
        };
        f.write_str(s)
    }
}

/// 既定で保持しなければならないデータ（FR-9）。
///
/// 保持の理由は項目ごとに異なる。ICC と Orientation は剥ぐと表示が壊れるため、
/// C2PA は剥ぐべきでないため（ADR-0011）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protected {
    /// ICC プロファイル。失うと色が変わる。
    IccProfile,
    /// EXIF Orientation。失うと画像が回転する。
    ExifOrientation,
    /// C2PA / Content Credentials。AI 生成物であることの開示情報。
    C2pa,
}

impl std::fmt::Display for Protected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::IccProfile => "ICC プロファイル",
            Self::ExifOrientation => "EXIF Orientation",
            Self::C2pa => "C2PA / Content Credentials",
        };
        f.write_str(s)
    }
}

/// `sdmeta-core` のエラー。
///
/// バリアントが種別そのものなので、呼び出し側は `match` で分岐できる。
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// ファイルの入出力に失敗した。どのパスで失敗したかを必ず含む。
    #[error("{path}: {source}")]
    Io {
        /// 対象のパス。
        path: PathBuf,
        /// 元の入出力エラー。
        #[source]
        source: std::io::Error,
    },

    /// 画像として認識できなかった、または本アプリが扱わない形式だった。
    #[error("対応していない形式です: {detail}")]
    UnsupportedContainer {
        /// 判定の根拠（検出したマジックバイトなど）。
        detail: String,
    },

    /// コンテナの構造が壊れている。
    #[error("{container} の構造が壊れています（オフセット {offset}）: {detail}")]
    Malformed {
        /// 処理中だった形式。
        container: Container,
        /// 問題を検出したバイトオフセット。
        offset: u64,
        /// 何が期待と違ったか。
        detail: String,
    },

    /// ファイルが途中で終わっている。
    #[error("{container} が途中で終わっています（{expected} バイト必要、残り {available} バイト）")]
    Truncated {
        /// 処理中だった形式。
        container: Container,
        /// 必要だったバイト数。
        expected: u64,
        /// 実際に残っていたバイト数。
        available: u64,
    },

    /// ファイル由来の長さ・オフセットが扱える範囲を超えている。
    ///
    /// チャンク長や EXIF のオフセットは入力ファイルが申告する値であり、
    /// 信用できない。`as` による切り捨てで「もっともらしい誤った長さ」を
    /// 作らないよう、範囲外は必ずこのエラーにする。
    #[error("{container} の値が範囲外です: {detail}")]
    OutOfRange {
        /// 処理中だった形式。
        container: Container,
        /// どの値がどの範囲を超えたか。
        detail: String,
    },

    /// メタデータのテキストを解釈できなかった。
    #[error("メタデータを解釈できませんでした: {detail}")]
    Parse {
        /// 解釈に失敗した理由。
        detail: String,
        /// 原文中の位置。特定できない場合は `None`。
        span: Option<Range<usize>>,
    },

    /// 保存後の読み直し（FR-7）で、期待した状態になっていなかった。
    ///
    /// 削除したはずのものが残っている、または保持すべきものが消えている。
    /// **書き出し結果を信用してはならない**ことを意味する。
    #[error("保存後の検証に失敗しました: {detail}")]
    VerifyFailed {
        /// 何が期待と違ったか。
        detail: String,
    },

    /// 保護対象（FR-9）を失う操作が要求された。
    #[error("{what} は保持しなければならないため、この操作は実行できません")]
    ProtectedDataWouldBeLost {
        /// 失われるはずだった保護対象。
        what: Protected,
    },

    /// 出力先を決められなかった。
    #[error("出力先を決められませんでした（{path}）: {detail}")]
    OutputPath {
        /// 試みた出力先。
        path: PathBuf,
        /// 失敗の理由（連番が尽きた、書き込み権限がない、など）。
        detail: String,
    },
}

impl Error {
    /// 入出力エラーにパスを添えて [`Error::Io`] を作る。
    ///
    /// `std::io::Error` は失敗したパスを持たないため、呼び出し側で補う。
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    /// 書き出した結果を信用できないエラーかどうか。
    ///
    /// `true` の場合、出力ファイルは破棄すべきであり、
    /// 利用者に「消えていない可能性がある」と伝える必要がある。
    pub fn invalidates_output(&self) -> bool {
        matches!(
            self,
            Self::VerifyFailed { .. } | Self::ProtectedDataWouldBeLost { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_is_send_sync_static() {
        // Tauri のコマンドは別スレッドで動く（ADR-0001 C-1）。
        // エラーを跨がせるため Send + Sync + 'static が必要。
        fn assert_bounds<T: Send + Sync + 'static>() {}
        assert_bounds::<Error>();
    }

    #[test]
    fn error_implements_std_error() {
        fn assert_std_error<T: std::error::Error>() {}
        assert_std_error::<Error>();
    }

    #[test]
    fn variants_can_be_matched() {
        let e = Error::Truncated {
            container: Container::Png,
            expected: 16,
            available: 4,
        };
        assert!(matches!(e, Error::Truncated { .. }));
        assert!(!e.invalidates_output());

        let e = Error::VerifyFailed {
            detail: "parameters が残っている".to_owned(),
        };
        assert!(e.invalidates_output());
    }

    #[test]
    fn io_error_keeps_the_path() {
        let e = Error::io(
            "/tmp/a.png",
            std::io::Error::new(std::io::ErrorKind::NotFound, "no such file"),
        );
        let msg = e.to_string();
        assert!(msg.contains("/tmp/a.png"), "パスが含まれていない: {msg}");
        assert!(msg.contains("no such file"), "原因が含まれていない: {msg}");
    }

    #[test]
    fn source_chain_is_reachable() {
        use std::error::Error as _;
        let e = Error::io(
            "/tmp/a.png",
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        );
        assert!(e.source().is_some(), "source が辿れない");
    }

    #[test]
    fn protected_display_names_the_item() {
        let e = Error::ProtectedDataWouldBeLost {
            what: Protected::C2pa,
        };
        assert!(e.to_string().contains("C2PA"), "{e}");
    }

    #[test]
    fn container_display() {
        assert_eq!(Container::Png.to_string(), "PNG");
        assert_eq!(Container::Jpeg.to_string(), "JPEG");
        assert_eq!(Container::WebP.to_string(), "WebP");
    }
}
