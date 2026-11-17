# Architecture Decision Records

SD Meta Editor の設計判断の記録。採用した案だけでなく、**却下した案とその理由**を残すことを目的とする。

## 索引

| ADR | タイトル | Status | 日付 |
|---|---|---|---|
| [0001](0001-gui-stack-tauri2.md) | GUI スタックに Tauri 2 を採用する | Accepted | 2026-11-12 |
| [0002](0002-comfyui-raw-only.md) | ComfyUI 画像は Raw 表示＋保持/削除トグルのみ対応する | Accepted | 2026-11-12 |
| [0003](0003-batch-preset-only.md) | バッチ処理を v1 に導入する（プリセット適用限定） | Accepted | 2026-11-12 |
| [0004](0004-stealth-png-removal.md) | stealth PNG の検出・除去を v1 必須機能とする | Accepted | 2026-11-12 |
| [0005](0005-license-apache-2.0.md) | ライセンスに Apache License 2.0 を採用する | Accepted | 2026-11-12 |
| [0006](0006-detail-sheet-for-long-lists.md) | 件数の多い一覧は詳細シート（別画面）で表示する | Accepted | 2026-11-12 |
| [0007](0007-ci-abolished-local-builds.md) | CI を廃止し、ビルドは開発機でのローカル実行に切り替える | Accepted | 2026-11-17 |
| [0008](0008-apple-silicon-only.md) | macOS 版は Apple Silicon 限定とし、universal ビルドを行わない | Accepted | 2026-11-17 |
| [0009](0009-local-builds-all-three-os.md) | 3 OS すべてを実機ローカルビルドで対象とし、対応 OS バージョンを確定する | Accepted | 2026-11-17 |
| [0010](0010-no-code-signing.md) | コード署名を行わず、OS の警告の回避手順を案内する | Accepted | 2026-11-17 |
| [0011](0011-c2pa-not-removed.md) | C2PA / Content Credentials は削除対象に含めない | Accepted | 2026-11-17 |
| [0012](0012-github-only-distribution.md) | 配布チャネルを GitHub Releases のみとする | Accepted | 2026-11-17 |
| [0013](0013-minimum-macos-version.md) | 最低対応 macOS バージョンを 14.6 とする | Accepted | 2026-11-17 |

## 運用ルール

- 1 ファイル 1 決定。採番は連番 4 桁
- Status は `Proposed` / `Accepted` / `Superseded by ADR-XXXX` / `Deprecated`
- **一度 Accepted にした ADR は書き換えない。** 判断が変わった場合は新しい ADR を起こし、旧 ADR の Status を `Superseded` に変更する
- 新規作成は [template.md](template.md) をコピーする

## 言語について

本 ADR 群は日本語で記述している。英訳は必要に応じて後から追加する。
