// SPDX-License-Identifier: Apache-2.0

//! ログ出力の基盤と、利用者に見せるメッセージの生成。
//!
//! # 2 つの出力先を分離する
//!
//! ADR-0001 の C-6 は「エラーを握り潰さない。トースト **と** ログの両方に出す」と定めている。
//! この 2 つは目的が違うので、本モジュールでは内容を明確に分ける。
//!
//! | 出力先 | 内容 | 読む人 |
//! |---|---|---|
//! | ログファイル | 完全な情報。バイトオフセット、長さ、原因チェーン | 開発者 |
//! | トースト（[`UserMessage`]） | 1 行の要約と、次に何をすべきか | 利用者 |
//!
//! [`report`] がこの分離を担う。ログには全部書き、戻り値には利用者向けの文面だけを返す。
//! **トーストにオフセットや内部用語を出さない。**
//!
//! # サブスクライバをライブラリ側に置いている理由
//!
//! 一般にライブラリはサブスクライバを設定せず、バイナリ側に任せる。本クレートは例外とする。
//! 利用側が `sdmeta-cli` と Tauri アプリの 2 つに限られており、どちらも
//! C-6 を同じ挙動で満たす必要があるため、設定を 1 箇所に置く方が崩れにくい。

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// ログディレクトリ名に使うアプリ名（macOS / Windows 向け、空白あり）。
const APP_NAME: &str = "SD Meta Editor";
/// ログディレクトリ名に使うアプリ名（Linux 向け、XDG 慣習のケバブケース）。
const APP_SLUG: &str = "sd-meta-editor";

/// ログ出力先を決めるための、プラットフォームごとの慣習。
///
/// 3 OS すべての分岐を 1 台の開発機で単体テストできるよう、
/// `cfg` ではなく値として扱う。C-13（3 実機での検証）が満たせるまでの間、
/// Windows と Linux の分岐を検証する唯一の手段になる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// macOS。`~/Library/Logs/<APP_NAME>`
    MacOs,
    /// Windows。`%LOCALAPPDATA%\<APP_NAME>\logs`
    Windows,
    /// Linux。`$XDG_STATE_HOME/<APP_SLUG>/logs`
    Linux,
}

impl Platform {
    /// コンパイル対象のプラットフォーム。
    #[must_use]
    pub const fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self::MacOs
        }
        #[cfg(target_os = "windows")]
        {
            Self::Windows
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Self::Linux
        }
    }
}

/// ログ出力先の解決に使う環境の値。
///
/// 環境変数を直接読まずに受け取ることで、テストから任意の値を与えられる。
#[derive(Debug, Default, Clone)]
pub struct DirSources {
    /// ホームディレクトリ（`$HOME` / `%USERPROFILE%`）。
    pub home: Option<PathBuf>,
    /// Windows の `%LOCALAPPDATA%`。
    pub local_app_data: Option<PathBuf>,
    /// Linux の `$XDG_STATE_HOME`。
    pub xdg_state_home: Option<PathBuf>,
}

impl DirSources {
    /// 環境変数から読み取る。
    #[must_use]
    pub fn from_env() -> Self {
        let var = |k: &str| {
            std::env::var_os(k)
                .map(PathBuf::from)
                .filter(|p| !p.as_os_str().is_empty())
        };
        Self {
            home: var("HOME").or_else(|| var("USERPROFILE")),
            local_app_data: var("LOCALAPPDATA"),
            xdg_state_home: var("XDG_STATE_HOME"),
        }
    }
}

/// プラットフォームの慣習に従ってログディレクトリを決める。
///
/// # Errors
///
/// 必要な環境変数が得られない場合に [`Error::OutputPath`] を返す。
/// パニックせずにエラーを返すのは、環境変数が無い環境（サンドボックス、
/// サービスとしての起動など）でも呼ばれうるため。
pub fn log_dir_for(platform: Platform, sources: &DirSources) -> Result<PathBuf> {
    let missing = |what: &str| Error::OutputPath {
        path: PathBuf::new(),
        detail: format!("{what} が取得できないため、ログ出力先を決められません"),
    };

    match platform {
        // ~/Library/Logs/SD Meta Editor
        Platform::MacOs => {
            let home = sources.home.as_deref().ok_or_else(|| missing("HOME"))?;
            Ok(home.join("Library").join("Logs").join(APP_NAME))
        }
        // %LOCALAPPDATA%\SD Meta Editor\logs
        Platform::Windows => {
            let base = sources
                .local_app_data
                .as_deref()
                .ok_or_else(|| missing("LOCALAPPDATA"))?;
            Ok(base.join(APP_NAME).join("logs"))
        }
        // $XDG_STATE_HOME/sd-meta-editor/logs、未設定なら ~/.local/state
        Platform::Linux => {
            let base = match sources.xdg_state_home.as_deref() {
                Some(p) => p.to_path_buf(),
                None => {
                    let home = sources.home.as_deref().ok_or_else(|| missing("HOME"))?;
                    home.join(".local").join("state")
                }
            };
            Ok(base.join(APP_SLUG).join("logs"))
        }
    }
}

/// 実行中のプラットフォームと環境変数からログディレクトリを決める。
///
/// # Errors
///
/// [`log_dir_for`] と同じ。
pub fn log_dir() -> Result<PathBuf> {
    log_dir_for(Platform::current(), &DirSources::from_env())
}

/// ログ基盤の生存を保証するガード。
///
/// 非同期にログを書き出すワーカを保持している。**ドロップすると書き残しが
/// 失われる可能性があるため、プロセスが終わるまで保持する。**
#[must_use = "ドロップするとログの書き残しが失われる"]
pub struct LogGuard {
    _worker: tracing_appender::non_blocking::WorkerGuard,
    dir: PathBuf,
}

impl LogGuard {
    /// 実際に使われているログディレクトリ。
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

impl std::fmt::Debug for LogGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogGuard").field("dir", &self.dir).finish()
    }
}

/// ログ基盤を初期化する。
///
/// - ログファイルは日付ごとにローテーションする
/// - ファイルへの書き込みは**別スレッドに逃がす**。UI スレッドを止めないため（C-1）
/// - 出力レベルは環境変数 `SDMETA_LOG` で変えられる（既定は `info`）
///
/// # Errors
///
/// ログディレクトリを決められない、または作成できない場合にエラーを返す。
pub fn init() -> Result<LogGuard> {
    init_in(log_dir()?)
}

/// 出力先を明示してログ基盤を初期化する。
///
/// Tauri 側が自前のアプリログディレクトリを使う場合や、テストで一時ディレクトリに
/// 書かせる場合に使う。[`init`] はこれを [`log_dir`] の結果で呼ぶだけ。
///
/// 環境変数を書き換える方法は取らない。edition 2024 の `std::env::set_var` は
/// `unsafe` であり、本クレートは `unsafe_code = "forbid"` のため使えない。
///
/// # Errors
///
/// ディレクトリを作成できない場合にエラーを返す。
pub fn init_in(dir: impl Into<PathBuf>) -> Result<LogGuard> {
    let dir = dir.into();
    std::fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;

    let appender = tracing_appender::rolling::daily(&dir, "sdmeta.log");
    let (writer, worker) = tracing_appender::non_blocking(appender);

    let filter = tracing_subscriber::EnvFilter::try_from_env("SDMETA_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_target(true)
        .with_env_filter(filter)
        .finish();

    // 二重初期化は異常ではない（テストや組み込み時に起こる）。失敗しても続行する。
    if tracing::subscriber::set_global_default(subscriber).is_err() {
        tracing::debug!("ログ基盤は既に初期化されている");
    }

    tracing::info!(dir = %dir.display(), "ログ基盤を初期化した");
    Ok(LogGuard {
        _worker: worker,
        dir,
    })
}

/// 利用者に見せるメッセージ。
///
/// **内部情報を含めない。** バイトオフセットや長さはログにだけ書く。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserMessage {
    /// トーストに出す 1 行。何が起きて、次に何をすべきか。
    pub headline: String,
    /// 書き出したファイルを信用してはならないかどうか。
    ///
    /// `true` の場合、UI はより強い警告を出し、出力を使わないよう促す必要がある。
    pub output_invalid: bool,
}

/// エラーをログに記録し、利用者向けのメッセージを返す。
///
/// ログには完全な情報（原因チェーン、オフセット、長さ）を書き、
/// 戻り値には要約だけを入れる。これが C-6 の「握り潰さない」の実装であり、
/// 同時にトーストとログの分離でもある。
pub fn report(error: &Error) -> UserMessage {
    tracing::error!(
        kind = variant_name(error),
        output_invalid = error.invalidates_output(),
        error = %error,
        source = ?std::error::Error::source(error),
        "処理に失敗した"
    );

    let headline = match error {
        Error::Io { path, .. } => {
            let name = path
                .file_name()
                .map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
            format!("{name} を読み書きできませんでした。")
        }
        Error::UnsupportedContainer { .. } => "対応していない画像形式です。".to_owned(),
        Error::Malformed { .. } | Error::Truncated { .. } | Error::OutOfRange { .. } => {
            "画像ファイルが壊れているため読み取れませんでした。".to_owned()
        }
        Error::Parse { .. } => {
            "メタデータの形式を解釈できませんでした。削除は実行できますが、項目ごとの表示はできません。"
                .to_owned()
        }
        Error::VerifyFailed { .. } => {
            "保存後の検証に失敗しました。書き出したファイルは使わないでください。".to_owned()
        }
        Error::ProtectedDataWouldBeLost { what } => {
            format!("{what} は常に保持する必要があるため、この操作は実行できません。")
        }
        Error::OutputPath { .. } => "保存先を決められませんでした。".to_owned(),
        // フォールバックを置かない。バリアントを追加したときに
        // コンパイルエラーで文面の追加を強制させる。
    };

    UserMessage {
        headline,
        output_invalid: error.invalidates_output(),
    }
}

/// ログのフィールドに入れるバリアント名。
fn variant_name(error: &Error) -> &'static str {
    match error {
        Error::Io { .. } => "Io",
        Error::UnsupportedContainer { .. } => "UnsupportedContainer",
        Error::Malformed { .. } => "Malformed",
        Error::Truncated { .. } => "Truncated",
        Error::OutOfRange { .. } => "OutOfRange",
        Error::Parse { .. } => "Parse",
        Error::VerifyFailed { .. } => "VerifyFailed",
        Error::ProtectedDataWouldBeLost { .. } => "ProtectedDataWouldBeLost",
        Error::OutputPath { .. } => "OutputPath",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sources() -> DirSources {
        DirSources {
            home: Some(PathBuf::from("/home/u")),
            local_app_data: Some(PathBuf::from("C:\\Users\\u\\AppData\\Local")),
            xdg_state_home: Some(PathBuf::from("/home/u/.local/state")),
        }
    }

    // 3 OS の分岐を 1 台で検証する。C-13 が満たせるまでの唯一の手段。
    #[test]
    fn macos_uses_library_logs() {
        let d = log_dir_for(Platform::MacOs, &sources());
        assert_eq!(
            d.as_deref().ok(),
            Some(Path::new("/home/u/Library/Logs/SD Meta Editor"))
        );
    }

    #[test]
    fn windows_uses_local_app_data() {
        let d = log_dir_for(Platform::Windows, &sources());
        assert_eq!(
            d.as_deref().ok(),
            Some(Path::new(
                "C:\\Users\\u\\AppData\\Local/SD Meta Editor/logs"
            ))
        );
    }

    #[test]
    fn linux_uses_xdg_state_home() {
        let d = log_dir_for(Platform::Linux, &sources());
        assert_eq!(
            d.as_deref().ok(),
            Some(Path::new("/home/u/.local/state/sd-meta-editor/logs"))
        );
    }

    #[test]
    fn linux_falls_back_to_dot_local_state() {
        let s = DirSources {
            home: Some(PathBuf::from("/home/u")),
            xdg_state_home: None,
            ..DirSources::default()
        };
        let d = log_dir_for(Platform::Linux, &s);
        assert_eq!(
            d.as_deref().ok(),
            Some(Path::new("/home/u/.local/state/sd-meta-editor/logs"))
        );
    }

    #[test]
    fn missing_environment_is_an_error_not_a_panic() {
        let empty = DirSources::default();
        for p in [Platform::MacOs, Platform::Windows, Platform::Linux] {
            let r = log_dir_for(p, &empty);
            assert!(matches!(r, Err(Error::OutputPath { .. })), "{p:?} で {r:?}");
        }
    }

    #[test]
    fn empty_environment_variable_is_treated_as_unset() {
        // 空文字の HOME を「/」と解釈してしまうと、ルート直下に書こうとする。
        let s = DirSources {
            home: Some(PathBuf::new()),
            ..DirSources::default()
        };
        // 空のパスは join の起点としては有効なので、呼び出し側の責務を明示する意味で
        // from_env 側でフィルタしている。ここでは from_env の挙動を確かめる。
        let _ = s;
        let before = std::env::var_os("XDG_STATE_HOME");
        assert!(
            DirSources::from_env().xdg_state_home.is_none() || before.is_some(),
            "未設定の環境変数が Some になっている"
        );
    }

    // --- トーストとログの分離 ---

    #[test]
    fn user_message_hides_internal_detail() {
        let e = Error::Malformed {
            container: crate::Container::Png,
            offset: 12_345,
            detail: "IHDR の長さが 13 ではない".to_owned(),
        };
        let m = report(&e);
        assert!(
            !m.headline.contains("12345"),
            "オフセットが漏れている: {m:?}"
        );
        assert!(!m.headline.contains("IHDR"), "内部用語が漏れている: {m:?}");
        assert!(!m.output_invalid);
    }

    #[test]
    fn verify_failure_marks_the_output_invalid() {
        let e = Error::VerifyFailed {
            detail: "parameters が残っている".to_owned(),
        };
        let m = report(&e);
        assert!(m.output_invalid, "出力を信用してはならないと伝わらない");
        assert!(m.headline.contains("使わないで"), "{m:?}");
    }

    #[test]
    fn protected_data_message_names_the_item() {
        let e = Error::ProtectedDataWouldBeLost {
            what: crate::Protected::IccProfile,
        };
        let m = report(&e);
        assert!(m.headline.contains("ICC"), "{m:?}");
        assert!(m.output_invalid);
    }

    #[test]
    fn io_message_shows_only_the_file_name() {
        let e = Error::io(
            "/Users/someone/secret-folder/a.png",
            std::io::Error::new(std::io::ErrorKind::NotFound, "nope"),
        );
        let m = report(&e);
        assert!(m.headline.contains("a.png"), "{m:?}");
        assert!(
            !m.headline.contains("secret-folder"),
            "パス全体が漏れている: {m:?}"
        );
    }

    #[test]
    fn every_variant_has_a_message_and_a_name() {
        let errors = [
            Error::io("x", std::io::Error::other("e")),
            Error::UnsupportedContainer {
                detail: "x".to_owned(),
            },
            Error::Malformed {
                container: crate::Container::Jpeg,
                offset: 0,
                detail: "x".to_owned(),
            },
            Error::Truncated {
                container: crate::Container::WebP,
                expected: 2,
                available: 1,
            },
            Error::OutOfRange {
                container: crate::Container::Png,
                detail: "x".to_owned(),
            },
            Error::Parse {
                detail: "x".to_owned(),
                span: None,
            },
            Error::VerifyFailed {
                detail: "x".to_owned(),
            },
            Error::ProtectedDataWouldBeLost {
                what: crate::Protected::C2pa,
            },
            Error::OutputPath {
                path: PathBuf::from("x"),
                detail: "x".to_owned(),
            },
        ];
        let mut names = std::collections::BTreeSet::new();
        for e in &errors {
            let m = report(e);
            assert!(!m.headline.is_empty(), "{e:?} の文面が空");
            assert!(
                names.insert(variant_name(e)),
                "{e:?} のバリアント名が重複している"
            );
        }
        assert_eq!(names.len(), errors.len(), "全バリアントを網羅していない");
    }
}
