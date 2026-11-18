// SPDX-License-Identifier: Apache-2.0

//! ログが実際にファイルへ書かれることを確かめる結合テスト。
//!
//! 単体テストと別プロセスで動く必要がある。`set_global_default` は
//! プロセスごとに 1 回しか成功しないため。

// テストでは失敗時にパニックするのが正しい挙動であり、`expect` の方が
// 失敗理由が読める。ワークスペースの `expect_used = "deny"` をここで解除する。
#![allow(clippy::expect_used)]

use std::path::PathBuf;

/// プロセス ID で区切った一時ディレクトリ。外部クレートに依存しない。
fn temp_dir() -> PathBuf {
    std::env::temp_dir().join(format!("sdmeta-logging-test-{}", std::process::id()))
}

#[test]
fn init_in_creates_the_directory_and_writes_the_log() {
    let dir = temp_dir();
    let _ = std::fs::remove_dir_all(&dir);

    {
        let guard = sdmeta_core::logging::init_in(&dir).expect("ログ基盤の初期化に失敗");
        assert_eq!(guard.dir(), dir.as_path(), "ガードが出力先を保持していない");

        // report() の経路でログが流れることを確かめる。
        let msg = sdmeta_core::logging::report(&sdmeta_core::Error::VerifyFailed {
            detail: "テスト用の詳細メッセージ".to_owned(),
        });
        assert!(msg.output_invalid, "出力無効のフラグが立っていない");
        // ガードのドロップで書き残しが flush される。
    }

    assert!(
        dir.is_dir(),
        "ログディレクトリが作られていない: {}",
        dir.display()
    );

    let mut written = String::new();
    let mut files = 0_usize;
    for entry in std::fs::read_dir(&dir).expect("ログディレクトリを読めない") {
        let entry = entry.expect("ディレクトリエントリを読めない");
        files += 1;
        written.push_str(&std::fs::read_to_string(entry.path()).expect("ログを読めない"));
    }

    assert!(files > 0, "ログファイルが作られていない");
    assert!(
        written.contains("ログ基盤を初期化した"),
        "初期化のログが入っていない:\n{written}"
    );
    assert!(
        written.contains("テスト用の詳細メッセージ"),
        "エラーの詳細がログに入っていない。トーストにしか出ていない可能性がある:\n{written}"
    );
    assert!(
        written.contains("VerifyFailed"),
        "バリアント名がログに入っていない:\n{written}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
