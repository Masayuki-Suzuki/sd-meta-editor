# ADR-0012: 配布チャネルを GitHub Releases のみとする

- Status: Accepted
- 日付: 2026-11-17
- 関連: [ADR-0010](0010-no-code-signing.md)（コード署名を行わない）

## Context

要件定義書 §7 で「GitHub Releases のみか、Homebrew / winget 等も対象とするか」を
未決としていた。

パッケージマネージャ経由の配布には利点がある。導入が 1 コマンドで済み、
更新も利用者側の習慣に乗る。一方で登録と維持にコストがかかる。

- **Homebrew cask**: 公式リポジトリへの登録には一定の知名度が要求される。
  独自 tap を作れば回避できるが、利用者に tap の追加を求めることになり、
  1 コマンドで済む利点が薄れる。リリースごとに cask の更新 PR が必要
- **winget**: マニフェストを microsoft/winget-pkgs へ PR する。
  リリースごとに更新が必要。**署名されていない配布物は審査で問題になりうる**
  （[ADR-0010](0010-no-code-signing.md) のとおり署名しない方針）
- どちらもリリース手順が増える。CI を廃止している（[ADR-0007](0007-ci-abolished-local-builds.md)）ため、
  この更新作業は手動になる

## Decision

**配布は GitHub Releases のみとする。**

- 各リリースに 3 プラットフォームの成果物を添付する
  - macOS (Apple Silicon): `.dmg` または `.app.tar.gz`
  - Windows 11: `.msi`
  - Ubuntu 24.04: `.deb` と AppImage
- SHA-256 チェックサムを併せて添付する（[ADR-0010](0010-no-code-signing.md)）
- Homebrew / winget / Flatpak / Snap には登録しない

## 却下した案

**Homebrew cask（独自 tap）** — tap の追加を求める時点で「1 コマンドで入る」
利点が失われる。GitHub Releases からダウンロードする手間とほぼ変わらない。

**winget への登録** — 署名なしの配布物であることが審査上の障害になりうる。
署名方針（ADR-0010）を変えない限り、着手しても通らない可能性がある。

**Flatpak / Snap** — Linux 側の対象を Ubuntu 24.04 に絞っている
（[ADR-0009](0009-local-builds-all-three-os.md)）状況で、
ディストリ非依存のパッケージ形式を維持する意味が薄い。

**自前のウェブサイトで配布する** — ドメインとホスティングの維持コストが発生し、
かつ署名のない配布物を独自サイトで配ると信頼性がさらに下がる。
GitHub の Releases ページの方が出自が明確である。

## Consequences

- 利用者は毎回 Releases ページから手動でダウンロードする。
  自動更新も提供しない（Tauri の updater は署名鍵の管理が必要であり、
  v1 のスコープ外とする）
- 新バージョンの通知手段がない。README に Watch / Releases の購読を案内する
- リリース手順が単純になる。3 OS でビルドし、チェックサムを付けて
  Releases にアップロードするだけで完結する

## 再検討の条件

- 利用者から導入方法に関する要望が繰り返し寄せられた場合
- コード署名を行うことになった場合（ADR-0010 の再検討条件と連動する）。
  署名があれば winget の障害が消える
