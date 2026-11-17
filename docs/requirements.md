# SD Meta Editor 要件定義書

**Version:** 1.0
**Date:** 2026-11-12
**Status:** Draft（コーディング未着手）

---

## 1. 目的

生成 AI 画像に埋め込まれたメタデータを**確実に削除**し、必要に応じて部分的に編集した上で、**無劣化・高速に**書き出すクロスプラットフォームデスクトップアプリケーション。

主要価値は「編集できること」ではなく、**「公開しても安全だと確信できること」**である。

## 2. ターゲットユーザーとユースケース

| 優先度 | ユースケース |
|---|---|
| **最高** | SNS や作品投稿サイトに公開する前に、プロンプト・LoRA・生成環境を消したい |
| 高 | 複数枚をまとめて消したい（公開直前の一括処理） |
| 中 | モデル名は見せたいが LoRA は隠したい等の部分開示 |
| 中 | カメラ由来の GPS・個人情報を消したい |
| 低 | メタデータを閲覧・確認したい（ビューア用途） |

## 3. スコープ

### v1 に含む

- JPEG / PNG / WebP の読み込み・メタデータ表示・編集・書き出し
- A1111 / Forge / Forge Neo 形式の構造化パース
- ComfyUI / 未知形式の Raw 表示＋チャンク単位トグル（ADR-0002）
- stealth PNG 検出・除去（ADR-0004）
- 一括処理（プリセット適用限定、ADR-0003）
- 保存後の自己検証（Verify）
- 設定画面

### v2 以降に送る

- ComfyUI workflow JSON のノード単位編集
- AVIF / HEIC 対応
- C2PA マニフェストの構造的解釈（v1 では Raw 表示のみ。削除はしない → [ADR-0011](adr/0011-c2pa-not-removed.md)）
- 自動更新、アダプタのプラグイン化

### 明確に対象外

- **SynthID 等の画素内不可視透かしの除去**。README とアプリ内に明記する

---

## 4. 機能要件

### FR-1 入力

- ウィンドウ全体へのドラッグ&ドロップ（Tauri ネイティブイベント / ADR-0001 C-4）
- ファイルダイアログ
- OS のファイル関連付け
- クリップボード貼り付け
- 形式判定は**拡張子ではなくマジックバイト**で行う

### FR-2 表示

左ペインにプレビュー（フィット / 100% / ズーム、透過はチェッカーボード背景）、右ペインにタブ構成。

| タブ | 内容 |
|---|---|
| Prompt | 正／負プロンプトの複数行テキストボックス（Undo/Redo、文字数カウント） |
| Parameters | 正規フィールドごとの行表示＋保持/削除トグル |
| Resources | LoRA・Embedding・VAE・Upscaler の**サマリ**（件数と代表例）＋「一覧を開く」ボタン |
| Raw | チャンク／EXIF タグの**サマリ**＋「一覧を開く」ボタン |

件数が多くなる一覧（LoRA、Raw チャンク、ComfyUI workflow）は右ペインに収めず、
**詳細シート（FR-2.1）として別画面に開く。**

### FR-2.1 詳細シート（別画面）

一覧性が要る情報は、モーダルまたは全面オーバーレイの「詳細シート」で表示する（[ADR-0006](adr/0006-detail-sheet-for-long-lists.md)）。
LoRA 一覧・Raw ダンプ・ComfyUI workflow（ADR-0002）はいずれも同一のシート UI を共有する。

右ペインは 3 ペインレイアウトの一部で幅が限られる。LoRA は数十件に達することがあり、
狭いタブ内に押し込むとスクロールと誤操作が増える。一覧は画面を広く使って出す。

**共通要件**

- ESC と明示的な閉じるボタンの両方で閉じられること
- 開いている間もプレビューの文脈が失われないこと（タイトルに対象ファイル名を表示）
- 件数が 20 を超える場合は**インクリメンタル検索フィルタ**を表示する
- 「すべて残す」「すべて消す」の一括操作を備える
- **行単位でメモ化し、1 行のトグルが一覧全体の再レンダを起こさないこと**（ADR-0001 C-2 の趣旨）

**LoRA 一覧シートの要件**

- **LoRA 1 件ごとに保持/削除のトグルを持つ**（プリセット 5 は全件を一括で OFF にするショートカット）
- 各行に以下を表示する
  - LoRA 名
  - 重み（プロンプトの `<lora:name:weight>` から取得できた場合）
  - ハッシュ（`Lora hashes` から取得できた場合）
  - **検出元バッジ**: `プロンプト` / `Lora hashes` / `両方`
- トグルを OFF にした場合、**検出されたすべての箇所から除去する。**
  プロンプト本文内の `<lora:name:weight>` タグと `Lora hashes` のエントリの両方が対象
- 片方にしか存在しない LoRA（ハッシュ計算が無効だった、Hires や ADetailer のプロンプトにのみ出現する等）
  も 1 行として扱い、存在する側からのみ除去する
- Embedding・VAE・Upscaler も同一シート内のセクションとして同じ粒度で扱う

**プロンプト本文との整合**

LoRA のトグル操作は**プロンプトのテキストボックスを即時に書き換える**（WYSIWYG）。
バックエンドへの往復は行わず、フロントエンド内で完結させる（ADR-0001 C-2）。

- 書き換えはプロンプト編集と**同一の Undo スタック**に積む。トグル操作も Ctrl+Z で戻せること
- ユーザーが手編集でタグを消した LoRA は、次回の再解析時に検出元から外れる
- 保存前の差分プレビューでは、タグ除去による本文の変化も差分として表示する


### FR-3 メタデータモデル

ツール別アダプタで**正規フィールドモデル**へ写像する。

```
prompt.positive / prompt.negative
model.name / model.hash / model.recipe
sampling.seed / sampling.steps / sampling.sampler / sampling.cfg
sampling.scheduler / sampling.denoise
geometry.base_size / geometry.upscale
resources.lora[] / resources.embedding[] / resources.vae / resources.upscaler
generator.name / generator.version
extensions.*   （ADetailer 等のツール固有）
unknown.*      （未知キー。削除可能であること）
raw            （原文バイト列を常に保持）
```

**キー名のレベルで共通するパラメータは存在しない。** 共通なのは意味ロールのみである。実在性の確度は以下のとおり。

| 階層 | ロール | 備考 |
|---|---|---|
| A: ほぼ必ず存在 | positive prompt / seed / model 識別子 / 生成時サイズ | |
| B: 多くに存在 | negative prompt / steps / sampler / CFG / LoRA 参照 | negative は CFG=1 系で欠落しうる |
| C: ツール固有 | scheduler, Shift, RNG, Module N, ADetailer *, Hires * | **共通化を試みてはいけない領域** |

**未知キーも必ず削除可能とする。**「知らないから触らない」は漏洩防止ツールとして失格である。

### FR-4 パーサ

- **引用符を認識するトークナイザを自前実装する。** A1111 形式のパラメータ行はカンマ区切りの `Key: Value` 列だが、`Lora hashes: "a: h1, b: h2, ..."` のように値が二重引用符で囲まれ内部にカンマを含む。素朴な `split(",")` は破綻する（`tests/fixtures/text/forge-neo-params.txt` 参照：単一フィールド内に 44 個のカンマ）
- 値はバリデーションせず**不透明文字列**として扱う。`Model` にはマージレシピが括弧ネスト付きでそのまま入り、ハッシュも桁数・形式が揃わない（`0x` 接頭辞付きなど）
- 既知キーのホワイトリスト方式は採らない
- 解析失敗時は例外で落とさず Raw フォールバックし、**ファイルを素通しで保存できる**こと

### FR-5 削除プリセット

単一処理・一括処理で共通。複数選択可。「全削除」のみ排他。

**「全削除」は文字どおりの全削除ではない。** FR-9 の保護対象（ICC・EXIF Orientation・C2PA）は
残る。UI 上でこれを明示し、利用者が「全部消えた」と誤解しないようにする。

| # | プリセット | 対象 |
|---|---|---|
| 1 | メタデータ全削除 | 全チャンク・全 EXIF・stealth（ICC・Orientation・C2PA は保持。FR-9 参照） |
| 2 | ポジティブプロンプト削除 | `prompt.positive` |
| 3 | ネガティブプロンプト削除 | `prompt.negative`（`Negative prompt:` 行ごと） |
| 4 | プロンプト全削除 | 2 + 3 |
| 5 | LoRA 削除 | `resources.lora[]` ＋**プロンプト本文内の `<lora:name:weight>` タグ**の両方。全件一括。個別指定は FR-2.1 の LoRA 一覧シートで行う |
| 6 | Seed 削除 | `sampling.seed` ＋ Variation seed / Seed resize 系 |
| 7 | 解像度・アップスケール情報削除 | `geometry.base_size` ＋ `geometry.upscale` |
| 8 | モデル・環境情報削除 | `model.*` ＋ `resources.vae` ＋ `generator.*` ＋ RNG |
| 9 | 拡張機能情報削除 | `extensions.*` ＋ `sampling.denoise` |
| 10 | カスタム | ユーザー定義の組み合わせを保存 |

補足：プリセット 7 について、`Size` は**生成時のベースサイズ**であり Hires 適用後の実ファイル寸法とは異なる。Hires 使用時は非自明な情報なので削除に意味があるが、未使用時はファイル自体から自明であり削除は実質的に無意味となる。

### FR-6 書き出し

- **既定はロスレス**
  - PNG: チャンク書き換え＋IDAT バイト列コピー
  - JPEG: APP セグメント書き換え＋スキャンデータコピー
  - WebP: RIFF チャンク差し替え＋ビットストリームコピー（VP8X 拡張形式への昇格とフラグ操作を含む）
- 書き戻しは**再シリアライズではなく span ベースの差分除去**。原文の空白・キー順序・引用符を保持する。完全な再構築は他ツールで読めなくなるリスクがあるため行わない
- ファイル名は `[元ファイル名]_edited.[元拡張子]`。サフィックスは設定変更可。衝突時は連番
- **元ファイルは書き換えない**（上書きは明示的オプトインのみ）
- 一時ファイルに書いて atomic rename する
- 再エンコードモードを明示的に ON にした場合のみ、品質・圧縮レベル設定が有効になる
- 唯一の例外は stealth 除去時の PNG 再エンコード（ADR-0004）

### FR-7 保存後検証（Verify）

書き出したファイルを読み直し、**残存メタデータ 0 件および stealth 未検出**を確認して結果を UI に表示する。常時 ON。

### FR-8 漏洩経路の網羅

以下をすべて削除対象として扱えること。

- EXIF 埋め込みサムネイル（IFD1）
- PNG `zTXt`（zlib 圧縮テキスト）
- PNG `eXIf`
- XMP（`APP1` / `XMP ` チャンク）
- MakerNote
- GPS
- stealth PNG（アルファ LSB）

C2PA JUMBF（PNG `caBX` / JPEG `APP11`）は**削除対象に含めない**。FR-9 の保護対象とする（[ADR-0011](adr/0011-c2pa-not-removed.md)）。

また、**ファイル名そのものにプロンプト断片が含まれる場合**は警告を出す。
出力ファイル名は FR-6 のとおり `[元ファイル名]_edited.[元拡張子]` に固定で、
**リネームの提案は行わない**（元ファイル名を変えたい場合は利用者が手で行う）。

### FR-9 保護対象

以下は「全削除」を含むすべてのプリセットで既定保持する。

| 保護対象 | 保持する理由 |
|---|---|
| ICC プロファイル | 剥ぐと色が変わる |
| EXIF Orientation | 剥ぐと画像が回転する |
| C2PA / Content Credentials | AI 生成物であることは開示すべき情報であり、隠す対象ではない（[ADR-0011](adr/0011-c2pa-not-removed.md)） |

前 2 者は「剥ぐと事故になる」ため、C2PA は「剥ぐべきでない」ため保護する。理由は異なるが扱いは同じ。

なお本アプリの編集は **C2PA の署名検証を失敗させる**（hard binding のハッシュが変わるため）。
これは避けられないので、C2PA を検出したら編集前に警告を表示し、内容を Raw シートで表示する。

### FR-10 一括処理

フォルダ／複数ファイル選択、プリセット適用、進捗表示、キャンセル、部分失敗サマリ。対象フィールドが存在しないファイルは no-op として扱いエラーにしない。

### FR-11 設定

出力先、サフィックス文字列、衝突時の挙動、再エンコードモードと品質、既定プリセット（起動時自動適用の可否を含む）、テーマ、言語（日/英）。

### FR-12 エラー処理

`Result` を握り潰す経路を作らない。トースト表示とログファイル出力を必須とする（ADR-0001 C-6）。

---

## 5. 非機能要件

| ID | 項目 | 目標値 |
|---|---|---|
| NFR-1 | D&D → プレビュー表示 | 4K 画像で 150ms 以内 |
| NFR-2 | メタデータ解析 → パネル表示 | 20ms 以内 |
| NFR-3 | ロスレス保存 | 10MB 画像で 50ms 以内 |
| NFR-4 | stealth 除去を伴う保存 | 10MB PNG で 500ms 以内（再エンコード込み） |
| NFR-5 | キー入力レイテンシ | 16ms 以内。入力中の IPC 発生ゼロ |
| NFR-6 | UI スレッドのブロッキング | 常に 0ms |
| NFR-7 | 一括処理 | 100 枚のロスレス削除を 10 秒以内 |
| NFR-8 | バイナリサイズ | 20MB 以内 |

性能はアーキテクチャ制約 C-1〜C-3（ADR-0001）の遵守によって担保する。

---

## 6. アーキテクチャ決定索引

| ADR | 決定 |
|---|---|
| [0001](adr/0001-gui-stack-tauri2.md) | GUI スタックに Tauri 2 を採用。Rust コア＋React 19/TS。派生制約 C-1〜C-7 |
| [0002](adr/0002-comfyui-raw-only.md) | ComfyUI は Raw 表示＋チャンク単位トグルのみ。JSON 内部編集は v2 |
| [0003](adr/0003-batch-preset-only.md) | バッチ処理を v1 導入。プリセット適用限定。派生制約 C-8・C-9 |
| [0004](adr/0004-stealth-png-removal.md) | stealth PNG の検出・除去を v1 必須。ロスレス原則の唯一の例外 |
| [0005](adr/0005-license-apache-2.0.md) | ライセンスに Apache-2.0 を採用。CLA は不要。LICENSE は逐語、著作権表示は NOTICE に分離 |
| [0006](adr/0006-detail-sheet-for-long-lists.md) | 件数の多い一覧は詳細シートで別画面表示。LoRA・Raw・workflow で UI を共有。派生制約 C-10・C-11 |
| [0007](adr/0007-ci-abolished-local-builds.md) | CI を廃止しローカルビルドへ。macOS を主対象、Windows は保留。派生制約 C-12 |
| [0008](adr/0008-apple-silicon-only.md) | macOS 版は Apple Silicon 限定。universal ビルドを行わない。C-7 を C-7′ で置換 |
| [0009](adr/0009-local-builds-all-three-os.md) | 3 OS すべて実機ローカルビルド。Windows 11 / Ubuntu 24.04 に確定。C-12 を C-13 で置換 |
| [0010](adr/0010-no-code-signing.md) | コード署名は行わず回避手順を案内。SHA-256 チェックサムを添付 |
| [0011](adr/0011-c2pa-not-removed.md) | C2PA は削除せず FR-9 の保護対象へ。署名検証は失敗する点を警告 |
| [0012](adr/0012-github-only-distribution.md) | 配布は GitHub Releases のみ。Homebrew / winget は対象外 |
| [0013](adr/0013-minimum-macos-version.md) | 最低対応 macOS は 14.6。技術的下限ではなく検証済みの下限 |

---

## 7. 未決事項

- ~~ライセンス~~ → **Apache-2.0 で確定**（[ADR-0005](adr/0005-license-apache-2.0.md)）
- ~~ビルド・CI 体制~~ → **CI は廃止、ローカルビルドで確定**（[ADR-0007](adr/0007-ci-abolished-local-builds.md)）
- ~~Windows 対応の再開時期~~ / ~~Linux ビルドのコンテナ構成~~ → **3 OS すべて実機ローカルビルドで確定**（[ADR-0009](adr/0009-local-builds-all-three-os.md)）
- ~~コード署名の方針~~ → **v1 では署名せず、回避手順の案内と SHA-256 チェックサムで対応**（[ADR-0010](adr/0010-no-code-signing.md)）
- ~~C2PA を削除対象に含めるか~~ → **含めない。FR-9 の保護対象とする**（[ADR-0011](adr/0011-c2pa-not-removed.md)）
- ~~ファイル名リネーム提案の UI~~ → **提案は行わない。出力名は FR-6 の `_edited` サフィックス固定**
- ~~配布チャネル~~ → **GitHub Releases のみ**（[ADR-0012](adr/0012-github-only-distribution.md)）
- ~~macOS の最低対応バージョン~~ → **14.6 で確定**（[ADR-0013](adr/0013-minimum-macos-version.md)）
- ~~3 OS 検証チェックリストの設計~~ → **[docs/verification.md](verification.md) に作成**。実装の進行に応じて随時更新する

**設計段階の未決事項はすべて解消した。** 以降の論点は実装着手後に出てくるものに限る。

---

## 8. 実装の着手順（推奨）

> フェーズと Issue 単位への細分化は [docs/roadmap.md](roadmap.md) にある。本節はその根拠。

1. **読み取り専用のダンプ CLI**（`sdmeta-cli`）から着手する。GUI を一切作らずに `sdmeta-core` のパーサを検証でき、手元の画像を流すだけで実地調査（[docs/research/metadata-formats.md](research/metadata-formats.md)）が同時に完了する
2. 生成されたコーパスを `tests/fixtures/` に回帰テストとして残す
3. ロスレス書き出しパスを実装し、Verify で往復検証する
4. stealth 検出・除去を追加する
5. Tauri シェルと GUI を載せる
6. 一括処理を追加する
