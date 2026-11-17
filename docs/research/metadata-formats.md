# 調査メモ: 生成 AI 画像のメタデータ格納方式

**Status:** 調査中（実地検証前）
**Date:** 2026-11-12

本ドキュメントは推測を含む。**実物のコーパスによる検証が完了した項目のみ「確認済」とマークする。**

---

## 1. 調査方針

キー名のレベルで共通するパラメータは存在しない。したがって「共通パラメータの集合」を探すのではなく、
**意味ロール（正規フィールド）への写像表をツールごとに作る**ことを調査の成果物とする。

調査は `sdmeta-cli` の読み取り専用ダンプ機能で行い、実行結果をそのままコーパス化する。

---

## 2. ツール別の格納方式

| ツール | 格納場所 | 形式 | 検証状況 |
|---|---|---|---|
| A1111 / Forge / Forge Neo / reForge / SD.Next | PNG `tEXt: parameters` / JPEG・WebP の EXIF `UserComment` | 独自テキスト（下記 3 章） | **サンプル取得済**（Forge Neo） |
| ComfyUI | PNG `tEXt: prompt` + `tEXt: workflow` | JSON（ノードグラフ） | 未検証 |
| SwarmUI | `sui_image_params` | JSON | 未検証 |
| InvokeAI | `invokeai_metadata` | JSON | 未検証 |
| Fooocus | 独自 JSON、または A1111 形式（設定で切替） | JSON / テキスト | 未検証 |
| NovelAI | `Software` / `Comment` / `Description` ＋**アルファ LSB ステガノグラフィ** | JSON + 画素内 | 未検証 |
| Draw Things (macOS) | 独自形式 | 未調査 | 未検証 |
| Midjourney | **ほぼ何も書かない**（Discord 配信時に剥がれる） | — | 未検証 |
| DALL·E / Adobe Firefly / Google 系 | C2PA マニフェスト（JUMBF） | 署名付き | 未検証 |

---

## 3. A1111 系テキスト形式の構造

3 つの論理パートからなる単一のテキストブロブ。

```
<ポジティブプロンプト>
Negative prompt: <ネガティブプロンプト>
Steps: 18, Sampler: Res Multistep, ... , Version: neo-2.31
```

3 行目（パラメータ行）はカンマ区切りの `Key: Value` 列。

### 3.1 パース上の落とし穴

**値がカンマを含む。** `Lora hashes` は二重引用符で囲まれ、内部に多数のカンマを持つ。
フィクスチャ（`tests/fixtures/text/forge-neo-params.txt`）では **単一フィールド内に 44 個のカンマ**が存在する。

> 素朴な `split(",")` を使うと 45 件の LoRA がすべて不正なキーとして解釈され、パーサが破綻する。
> 既存の類似ツールで「テキストボックスが反応しない」「Save が無反応」という症状が報告されるのは、
> この種の入力で例外が発生し、それが握り潰されている可能性が高い。

→ **引用符を認識するトークナイザが必須**（要件定義書 FR-4）。

**キーの語彙は閉じていない。** Forge / Neo は固有キーとして
`Shift`, `Beta schedule alpha`, `Beta schedule beta`, `Module 1`, `Module 2`, `Hires Shift`, `RNG` が出現する。
さらに ADetailer 拡張が `ADetailer ` 接頭辞で 8 キー、Hires 系が `Hires ` 接頭辞で 6 キーを追加する。

**値は何でも入りうる。** `Model` にはマージレシピが括弧ネスト付きでそのまま入る（`0.5(A+B(2) + C) + 0.5(D_merge)` のような形）。
LoRA ハッシュも桁数が揃わず、`0x` 接頭辞が付くものが混在する。
→ 値のバリデーションは行わない。

### 3.2 ツール固有キーの写像例

| Forge Neo のキー | 正規フィールド |
|---|---|
| `Steps` | `sampling.steps` |
| `Sampler` | `sampling.sampler` |
| `Schedule type` | `sampling.scheduler` |
| `CFG scale` | `sampling.cfg` |
| `Seed` | `sampling.seed` |
| `Size` | `geometry.base_size` |
| `Model` / `Model hash` | `model.name` / `model.hash` |
| `Module 1` / `Module 2` | `resources.vae` / `resources.text_encoder` |
| `Denoising strength` | `sampling.denoise` |
| `Lora hashes` | `resources.lora[]` |
| `Hires upscale` / `Hires upscaler` / `Hires steps` | `geometry.upscale` |
| `ADetailer *` | `extensions.adetailer.*` |
| `Shift` / `RNG` / `Beta schedule *` | `extensions.forge.*` |
| `Version` | `generator.version` |

注: 旧 A1111 では VAE は `VAE` / `VAE hash` キーだった。Forge では `Module N` に置き換わっている。
**同じ意味ロールがツールによって別のキー名を持つ**典型例。

---

## 4. 見落としやすい漏洩経路

| 経路 | リスク |
|---|---|
| EXIF 埋め込みサムネイル（IFD1） | 編集前の画像の縮小版が別途埋まる。トリミング前が復元できる |
| PNG `zTXt`（zlib 圧縮テキスト） | `tEXt` だけ見ていると取りこぼす |
| PNG `eXIf` | PNG にも EXIF が入る |
| XMP（`APP1` / `XMP ` チャンク） | 一部ツールが二重記録する |
| MakerNote / GPS | カメラ由来。img2img の元画像由来で混入しうる |
| **stealth PNG（アルファ LSB）** | チャンク削除では一切消えない（ADR-0004） |
| **C2PA / Content Credentials** | JUMBF として埋まる（PNG `caBX` / JPEG `APP11`）。通常の EXIF・tEXt 走査では見つからない。**削除対象外**（[ADR-0011](../adr/0011-c2pa-not-removed.md)）だが、検出と内容表示のために位置の把握は必要 |
| **ファイル名そのもの** | ComfyUI の filename prefix 等にプロンプト断片が入る。`_edited` サフィックス方式では元のファイル名が温存されるため盲点になる |

### LoRA 名とモデル名の暴露度について

`Lora hashes` フィールドには LoRA が数十件単位で列挙されることがある
（フィクスチャでは実物の構造に合わせて 45 件を再現している）。
一般に、この一覧の名称だけから以下が読み取れてしまう。

- 自前学習の有無と、その実施時期（学習ジョブ ID 形式の名称がそのまま残る）
- スタイル調整に用いた LoRA の種類（命名規則から用途が推測できる）
- 体型や顔パーツを調整するスライダ系 LoRA の構成

**LoRA 名はプロンプト本文以上に「何をどう作ったか」を暴露する。**
長期間かけて調整した画風は、LoRA の構成と重みの組み合わせそのものが成果物であり、
その一覧が画像に添付されたまま公開されると**制作者の中核的なノウハウが丸ごと流出する。**
削除プリセット 5（LoRA 削除）の優先度が高い根拠はここにある。

同様に `Model` のマージレシピは配合そのもの、`Module 2` はテキストエンコーダ構成まで露出する。
これらは削除プリセット 8（モデル・環境情報削除）の根拠。

> **注意:** 本節は形式の一般的な性質についての分析である。
> `tests/fixtures/text/forge-neo-params.txt` は構造のみを再現した合成データであり、
> **実在する LoRA 名・モデル名はリポジトリに一切含めていない。**

---

## 5. 除去できないもの

**SynthID**（Google Imagen / Gemini 系）は画素内に埋め込まれる不可視透かしであり、
本ツールでは除去できない。README とアプリ内に明記すること。

「全部消えた」と誤認させることは、保存後検証（FR-7）の思想に反する。

---

## 6. コーパス収集の進め方

1. `sdmeta-cli dump <path>` で全チャンク・全 EXIF タグを列挙する（実装予定）
2. ツール別・フォーマット別に 1 枚以上を `tests/fixtures/images/` に配置する
3. 出現したキーの分布を集計し、本ドキュメントの写像表を更新する
4. 未対応ツールの画像は「未知形式」として Raw 表示＋全削除のみ提供し、アダプタは後から追加する
