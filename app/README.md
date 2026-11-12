# app/ — Tauri 2 シェル（未スキャフォールド）

このディレクトリには Tauri 2 アプリケーション（`src-tauri/` + フロントエンド）が入る予定です。
**現時点では空です。**

## スキャフォールド前に読むこと

- [ADR-0001](../docs/adr/0001-gui-stack-tauri2.md) — とくに派生制約 C-1 〜 C-7
- [要件定義書](../docs/requirements.md)

## 着手タイミング

[要件定義書 第 8 章](../docs/requirements.md) のとおり、`sdmeta-core` と `sdmeta-cli` で
パース・ロスレス書き出し・stealth 除去が検証できてから着手します。
GUI を先に作ると、検証されていないコアの上に UI を載せることになります。

## スキャフォールド時の注意

- フロントエンドは React 19 + TypeScript + Vite
- `dragDropEnabled` の設定は C-4 を参照（Tauri ネイティブ D&D と HTML5 D&D は排他）
- プレビューは IPC ではなくカスタム URI スキームで配信する（C-3）
- `capabilities/*.json` は最小権限で記述する（C-5）
