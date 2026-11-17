# Contributing to SD Meta Editor

現在は**設計段階**です。アプリケーションコードはまだありません。
この段階で最も価値のある貢献は、**設計への指摘**と**サンプル画像の提供**です。

---

## 1. サンプル画像の提供（最も歓迎します）

本ツールの品質は、**どれだけ多様な生成ツールの実物を知っているか**でほぼ決まります。
未対応のツールで生成した画像をお持ちであれば、ぜひご提供ください。

提供前に [tests/fixtures/README.md](tests/fixtures/README.md) のポリシーを必ずお読みください。
フィクスチャ画像は**メタデータを含んだまま**コミットされます（それが検証対象のため）。
公開したくない情報を含む画像は提供しないでください。

## 2. 設計への指摘

- 削除し損ねる経路を見つけた場合は、**Security Issue として扱います**（単なるバグではありません）
- 設計判断への異論は [docs/adr/](docs/adr/) の該当 ADR を参照のうえ Issue でお願いします

## 3. 設計上の不変条件

以下はこのプロジェクトの前提です。変更には新しい ADR が必要です。

| 不変条件 | 根拠 |
|---|---|
| 元ファイルは書き換えない（明示的オプトインを除く） | FR-6 |
| 既定の保存は再エンコードしない | FR-6 |
| ICC プロファイルと EXIF Orientation は常に保持する | FR-9 |
| 未知のキー・チャンクも削除可能にする | FR-3 |
| 保存後に読み直して検証する | FR-7 |
| エラーを握り潰さない | ADR-0001 C-6 |
| CPU 処理を UI スレッドに置かない | ADR-0001 C-1 |
| キー入力で IPC を発生させない | ADR-0001 C-2 |
| ロスレス書き出しは Windows 実機で検証するまで完了と見なさない | ADR-0007 C-12 |

## 4. ADR の運用

設計判断は [docs/adr/](docs/adr/) に記録します。

- 1 ファイル 1 決定
- **Accepted になった ADR は書き換えない。** 判断が変わったら新しい ADR を起こし、旧 ADR を `Superseded` にする
- 新規作成は [template.md](docs/adr/template.md) をコピーする

## 5. コーディング規約（実装開始後）

- Rust: `cargo fmt` と `cargo clippy -- -D warnings` を通すこと
- TypeScript: フォーマッタ・リンタは実装開始時に確定する
- コミットメッセージ: [Conventional Commits](https://www.conventionalcommits.org/ja/)

### 提出前のローカルチェック

**このリポジトリには CI がありません**（[ADR-0007](docs/adr/0007-ci-abolished-local-builds.md)）。
自動で止めてくれる仕組みが無いため、以下を手元で通してから Pull Request を出してください。

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

実行結果は Pull Request の「検証方法」欄に記載してください。

### ビルド対象と実行環境

| ターゲット | 実行環境 |
|---|---|
| macOS (Apple Silicon / arm64) | 開発機の Mac（主対象） |
| Linux (x86_64) | Mac 上の Linux コンテナ |
| Windows | Windows 実機（現在は保留） |

macOS から Linux / Windows のバンドルは生成できません。理由は
[ADR-0007](docs/adr/0007-ci-abolished-local-builds.md) に記載しています。

## 6. 連絡先

バグ報告・機能要望は GitHub Issues へお願いします。

## 7. ライセンスと貢献の扱い

本プロジェクトは [Apache License 2.0](LICENSE) で公開しています。

Apache-2.0 の**第 5 条（Submission of Contributions）**により、あなたが意図的に本プロジェクトへ
送った貢献は、別段の明示的な合意がない限り、**本ライセンスと同一条件でライセンスされます。**
そのため本プロジェクトでは**別途の CLA（Contributor License Agreement）を求めません。**

送信した貢献についてその条件で提供する権利を有していることを、Pull Request の時点で
表明したものとみなします。

### 新規ファイルのヘッダ

新しいソースファイルには SPDX 識別子を 1 行入れてください。

```rust
// SPDX-License-Identifier: Apache-2.0
```

### NOTICE ファイル

`NOTICE` は Apache-2.0 第 4 条 (d) に基づく帰属表示です。
サードパーティ由来のコードを取り込んだ場合は、その帰属表示をここに追記してください。
