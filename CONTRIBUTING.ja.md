# SD Meta Editor への貢献

[English version](CONTRIBUTING.md)

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
| ICC プロファイル・EXIF Orientation・C2PA は常に保持する | FR-9, ADR-0011 |
| 未知のキー・チャンクも削除可能にする | FR-3 |
| 保存後に読み直して検証する | FR-7 |
| エラーを握り潰さない | ADR-0001 C-6 |
| CPU 処理を UI スレッドに置かない | ADR-0001 C-1 |
| キー入力で IPC を発生させない | ADR-0001 C-2 |
| ロスレス書き出しは 3 OS の実機で検証するまで完了と見なさない | ADR-0009 C-13 |

> このリポジトリで AI コーディングエージェントを使う場合は、まず [AGENTS.md](AGENTS.md)（英語）を読ませてください。

## 4. ADR の運用

設計判断は [docs/adr/](docs/adr/) に記録します。

- 1 ファイル 1 決定
- **Accepted になった ADR は書き換えない。** 判断が変わったら新しい ADR を起こし、旧 ADR を `Superseded` にする
- 新規作成は [template.md](docs/adr/template.md) をコピーする

## 5. コーディング規約（実装開始後）

- Rust: `cargo fmt` と `cargo clippy` を通すこと。lint のレベルはワークスペースの
  `Cargo.toml` の `[workspace.lints]` で決めている。`-D warnings` は付けない
  （`deny`＝即ブロックと `warn`＝助言、という意図的な区別を上書きしてしまう）
- TypeScript: フォーマッタ・リンタは実装開始時に確定する
- コミットメッセージ: [Conventional Commits](https://www.conventionalcommits.org/ja/)

### 提出前のローカルチェック

**このリポジトリには CI がありません**（[ADR-0007](docs/adr/0007-ci-abolished-local-builds.md)）。
自動で止めてくれる仕組みが無いため、以下を手元で通してから Pull Request を出してください。

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

実行結果は Pull Request の「検証方法」欄に記載してください。

### ビルド対象と実行環境

| ターゲット | 実行環境 |
|---|---|
| macOS 14.6 以降 (Apple Silicon / arm64) | 開発機の Mac |
| Windows 11 | Windows 実機 |
| Ubuntu 24.04 LTS (x86_64) | Ubuntu 実機 |

macOS から Linux / Windows のバンドルは生成できないため、それぞれの実機でビルドします。
対応 OS バージョンと、それより古いバージョンの扱いは
[ADR-0009](docs/adr/0009-local-builds-all-three-os.md) と
[ADR-0013](docs/adr/0013-minimum-macos-version.md) に記載しています。

書き出しパスに触る変更には不変条件 C-13 が適用され、3 実機すべてでの検証が必要です。
チェックリストは [docs/verification.md](docs/verification.md) です。

## 6. 開発フロー

1 Issue = 1 ブランチ = 1 PR。**`main` への直接 push はしない。**

### ブランチ名

```
<type>/<issue番号>-<短いスラグ>

docs/59-pr-workflow
feat/10-quote-aware-tokenizer
fix/21-png-idat-copy
```

`<type>` は Conventional Commits の接頭辞と同じ語彙なので、覚えることは増えません。
Issue 番号を含めることで、ブランチ名だけで対象が分かります。

### Pull Request

- `Closes #<n>` で Issue を参照する（マージ時に自動クローズされる）
- 「検証方法」欄を埋める。**このリポジトリには CI が無い**ため
  （[ADR-0007](docs/adr/0007-ci-abolished-local-builds.md)）、そこに書いた実行結果が
  動作することの唯一の証拠になる
- **squash マージ**。1 Issue が `main` の 1 コミットになり、履歴と Issue 一覧が 1 対 1 で対応する。
  squash 後のメッセージは Conventional Commits で書き、本文に**なぜ**を残す
- マージ後にブランチを削除する

### `needs-hardware` ラベルの Issue

macOS・Windows 11・Ubuntu 24.04 の実機が必要な Issue（不変条件 C-13）です。
コードではなくチェックリストを持つため、チェックリストを消化して結果を Issue に記録することで
閉じます。**ブランチも PR も作りません。** 該当 Issue は
[docs/roadmap.md](docs/roadmap.md) を参照してください。

### 作業の出どころ

[docs/roadmap.md](docs/roadmap.md) にフェーズ P0〜P8 があります。
現在のフェーズを依存順に進めます。前のフェーズが完了条件を満たすまで次に進みません。

## 7. 連絡先

バグ報告・機能要望は GitHub Issues へお願いします。

## 8. ライセンスと貢献の扱い

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
