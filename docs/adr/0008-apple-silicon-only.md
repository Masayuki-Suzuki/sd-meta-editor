# ADR-0008: macOS 版は Apple Silicon 限定とし、universal ビルドを行わない

- Status: Accepted
- 日付: 2026-11-17
- 関連: [ADR-0001](0001-gui-stack-tauri2.md) の派生制約 C-7 を置き換える

## Context

ADR-0001 の C-7 では `tauri build --target universal-apple-darwin` を前提にしていた。
ADR-0007 でこれをローカルビルドに読み替えた際、universal ビルドには
`x86_64-apple-darwin` ターゲットの追加が必要である点を Consequences に記録した。

ここで Intel Mac を対象に含める必要性そのものを再検討した。

**Intel Mac の位置づけ。** Apple Silicon は 2020 年 11 月の導入から 6 年が経過している。
Apple は macOS 26 Tahoe を Intel Mac に対応する最後のメジャーリリースとして announce しており、
Rosetta についても段階的な縮小が予告されている。新規に開発するデスクトップアプリで
Intel Mac を対象に含める理由は薄い。

**本アプリのユーザー層。** 想定ユーザーは生成 AI 画像を扱う層である。Mac で画像生成を
行う場合は MPS を使う Apple Silicon が実質的な前提になる。残余ケースとして
「別マシンで生成した画像を Intel Mac で受け取って編集するだけ」という利用は
あり得るが、規模は小さいと判断する。

**universal のコスト。** universal バイナリは Rust バイナリを 2 アーキテクチャ分
含むため、サイズがほぼ倍になる。NFR-8（バイナリサイズ 20MB 以内）に対して
この増分を払う価値がない。ビルド時間も倍になり、`lipo` による結合工程が増える。

**判断の可逆性。** 後から Intel 対応を戻す場合のコストは `rustup target add
x86_64-apple-darwin` とビルドターゲットの変更だけで済む。安価かつ可逆な判断である。

## Decision

**macOS 版のビルドターゲットを `aarch64-apple-darwin` 単体とする。**

- `universal-apple-darwin` は使用しない
- `x86_64-apple-darwin` ターゲットは導入しない
- 動作環境として「Apple Silicon 搭載 Mac」を README に明記する
- ADR-0001 の派生制約 **C-7 を本 ADR の C-7′ で置き換える**

### C-7′. macOS ビルドは Apple Silicon 単体

```
tauri build --target aarch64-apple-darwin
```

## ADR-0001 の Status を変更しない理由

ADR 運用ルールでは「判断が変わった場合は旧 ADR の Status を `Superseded` に変更する」
と定めている。本 ADR はこれに従わず、ADR-0001 を `Accepted` のまま残す。

ADR-0001 の決定事項は「GUI スタックに Tauri 2 を採用する」であり、この判断は
変わっていない。変更したのは派生制約 C-7 という一部分である。ADR-0001 全体を
`Superseded` にすると「Tauri 2 の採用を撤回した」と読めてしまい、履歴として
誤った情報を残す。したがって本 ADR で置き換え範囲を明示する方法を取る。

ADR-0007 でも同じ理由で ADR-0001 / ADR-0005 を書き換えずに読み替えを宣言している。

## 却下した案

**universal ビルドを維持する** — バイナリサイズが倍、ビルド時間も倍になる。
対象ユーザーがほぼ存在しない構成にこのコストは払えない。

**Intel 専用ビルドを別アーティファクトとして配布する** — サイズ問題は回避できるが、
**Intel Mac の実機が無いため動作検証ができない。** 検証していないバイナリを
配布すると、起動しない・保存が壊れるといった不具合を発見できないまま公開することになる。
本アプリは画像ファイルを書き出すツールであり、検証できない環境向けの配布はしない。

**Rosetta 前提で x86_64 単体ビルドにする** — Apple Silicon で動かすために毎回
エミュレーションを通すことになり、方向が逆である。

## Consequences

- **Intel Mac では起動しない。** README の動作環境と、将来のリリースノートに明記する
- NFR-8（20MB 以内）に対して余裕が生まれる。universal を捨てた分を
  機能やデバッグシンボルに充てられる
- `rustup target add x86_64-apple-darwin` は不要になった（ADR-0007 の Consequences から取り下げ）
- Linux 向けのアーキテクチャ選択は本 ADR の対象外。Linux デスクトップは x86_64 が
  主流のため、ADR-0007 のとおりコンテナ側で別途決める（要件定義書 §7 の未決事項）

## 再検討の条件

- Intel Mac ユーザーからの要望が実際に寄せられた場合。ただし検証用の実機が確保できない
  限り、提供しても「未検証」の扱いを明記する
