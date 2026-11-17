# ADR-0007: CI を廃止し、ビルドは開発機でのローカル実行に切り替える

- Status: Accepted
- 日付: 2026-11-17

## Context

初回 push で GitHub Actions の CI（`ci.yml`、3 OS マトリクスで fmt / clippy / test）が
3 OS すべて失敗した。失敗理由はコードではなくアカウント側である。

```
The job was not started because your account is locked due to a billing issue.
```

ジョブはステップを 1 つも実行せずに 3 秒で終了した。これを機に、現時点で CI が
提供している価値とコストを見直した。

**価値がほぼ無い。** テストは 0 件、`sdmeta-core` は doc コメントのみで実装が無い。
現状の CI が検出できる不具合は、ローカルで `cargo fmt --check` と `cargo clippy` を
打てば同じく検出できる。実際、初回コミットに含まれていた末尾改行の欠落
（`fix: ソースファイルの末尾改行欠落を修正し Cargo.lock を追跡下に置く`）は
CI ではなくローカル実行で発見した。

**コストが大きい。** private リポジトリの Actions は実行時間を消費し、
分数倍率は `macos-latest` が 10 倍、`windows-latest` が 2 倍である。
実装が無い段階で 3 OS マトリクスを push ごとに回す構成は消費が見合わない。

**クロスビルドが成立しない。** Tauri の Linux バンドル（`.deb` / `.AppImage`）は
glibc・GTK3・WebKitGTK 4.1・libsoup の Linux 共有ライブラリへリンクするため、
macOS から直接生成できない。Windows バンドル（`.msi` / `.exe`）も WiX に依存し
同様である。「Mac で Linux 向けもビルドする」は Linux 環境を用意しない限り不可能で、
実質的な選択肢は Linux コンテナか VM になる。

## Decision

**GitHub Actions の CI を廃止する。** `.github/workflows/ci.yml` を削除する。

ビルドはプラットフォームごとに実行環境を分ける。

| ターゲット | 実行環境 | 状態 |
|---|---|---|
| macOS (universal) | 開発機の Mac でローカルビルド | **主対象** |
| Linux (x86_64) | Mac 上の Linux コンテナ | 必要になった時点で着手 |
| Windows | Windows 実機でローカルビルド | **保留** |

`release.yml` は macOS ジョブのみを残した雛形として保持するが、当面使用しない。
Windows / Linux のジョブは意図的に含めない。

品質ゲートはローカル実行に移す。コミット前に以下を通すことを `CONTRIBUTING.md` に明記する。

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 却下した案

**3 OS マトリクスを維持する** — 実装もテストも無い段階では検出するものが無い。
`macos-latest` 10 倍・`windows-latest` 2 倍の倍率を払う理由が現時点では無い。

**リポジトリを public にして Actions の無料枠を使う** — public リポジトリの Actions は
分数無料なので課金問題は回避できる。しかし一度 public にすると GitHub 側のキャッシュ、
フォーク、各種クローラに残り巻き戻せない。中身の確認が済む前に公開する判断は取らない。
なおアカウントロックが Actions 全体に及んでいる場合は、public でも解決しない可能性がある。

**macOS から Linux 向けにクロスコンパイルする** — Tauri のバンドル処理が Linux の
ネイティブツールとライブラリに依存するため成立しない。`sdmeta-core` 単体
（GUI を含まない純 Rust）であれば `cross` 等で Linux ターゲットのテストは可能だが、
配布物は作れない。

**Windows ビルドを Actions で回す** — 倍率 2 倍で macOS より安いが、Windows 実機が
あるならローカルビルドの方が速く、費用もかからない。保留とする。

## Consequences

- **品質ゲートが開発者の手元に依存する。** 強制力が無くなるため、`CONTRIBUTING.md` に
  実行すべきコマンドを明記し、PR テンプレートの「検証方法」欄で申告させる
- ADR-0001 の「3 OS で CI スクリーンショット比較を行う」および
  「`tauri build --target universal-apple-darwin` を CI に組み込む」は、本 ADR で
  **ローカルビルドに読み替える**。ADR-0001 自体は書き換えない（ADR 運用ルール）
- ADR-0005 の「`cargo-deny` による CI チェックを導入する」も当面ローカル実行とする
- macOS の universal ビルドには `x86_64-apple-darwin` ターゲットの追加が必要
  （現状 `aarch64-apple-darwin` のみ）
- Windows 固有の挙動（開いているファイルの上書き不可、アトミックな置き換えの差異、
  パス区切り）が検証されないまま進む。**ロスレス書き出しパスを実装した時点で、
  Windows 実機での検証を必須とする**。この制約を C-12 として追加する

## 派生制約

- **C-12**: ロスレス書き出しパスは、Windows 実機でファイルの上書き・リネーム・
  同時オープンの挙動を検証するまで「動作確認済み」と見なさない

## 再検討の条件

以下のいずれかが成立した時点で CI の再導入を検討する。

- アカウントの課金問題が解消される
- リポジトリを public にする（分数無料になる）
- `sdmeta-core` に回帰テストが蓄積し、ローカル実行だけでは退行を防げなくなる
