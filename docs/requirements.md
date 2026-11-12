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
- C2PA マニフェストの構造的解釈（v1 ではチャンク単位の削除対象に含めるのみ）
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
| Resources | LoRA・Embedding・VAE・Upscaler の一覧＋トグル |
| Raw | 全チャンク／全 EXIF タグのダンプ（読取専用＋チャンク単位トグル） |

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

| # | プリセット | 対象 |
|---|---|---|
| 1 | メタデータ全削除 | 全チャンク・全 EXIF・stealth（ICC と Orientation は保持） |
| 2 | ポジティブプロンプト削除 | `prompt.positive` |
| 3 | ネガティブプロンプト削除 | `prompt.negative`（`Negative prompt:` 行ごと） |
| 4 | プロンプト全削除 | 2 + 3 |
| 5 | LoRA 削除 | `resources.lora[]` ＋**プロンプト本文内の `<lora:name:weight>` タグ**の両方 |
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
- C2PA JUMBF（PNG `caBX` / JPEG `APP11`）

また、**ファイル名そのものにプロンプト断片が含まれる場合**は警告を出し、リネームを任意で提案する（強制しない）。

### FR-9 保護対象

ICC プロファイルと EXIF Orientation は、「全削除」を含むすべてのプリセットで既定保持する。これらを剥ぐと色が変わる／画像が回転する事故になる。

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

---

## 7. 未決事項

- ~~ライセンス~~ → **Apache-2.0 で確定**（[ADR-0005](adr/0005-license-apache-2.0.md)）
- コード署名の方針（Apple Developer Program 加入の有無、Windows 証明書）
- C2PA を v1 でチャンク削除対象に含めるかの最終確認
- ファイル名リネーム提案の UI 設計
- 配布チャネル（GitHub Releases のみか、Homebrew / winget 等も対象とするか）

---

## 8. 実装の着手順（推奨）

1. **読み取り専用のダンプ CLI**（`sdmeta-cli`）から着手する。GUI を一切作らずに `sdmeta-core` のパーサを検証でき、手元の画像を流すだけで実地調査（[docs/research/metadata-formats.md](research/metadata-formats.md)）が同時に完了する
2. 生成されたコーパスを `tests/fixtures/` に回帰テストとして残す
3. ロスレス書き出しパスを実装し、Verify で往復検証する
4. stealth 検出・除去を追加する
5. Tauri シェルと GUI を載せる
6. 一括処理を追加する
