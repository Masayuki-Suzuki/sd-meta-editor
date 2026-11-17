# SD Meta Editor

> **Status: pre-alpha — design phase. No application code has been written yet.**

A fast, cross-platform desktop app for **inspecting, editing and reliably stripping** the generation metadata embedded in AI-generated images (Stable Diffusion and friends).

[日本語の README はこちら](README.ja.md)

---

## Why

Most people reaching for a tool like this are not trying to *edit* metadata — they are trying to **make sure it is gone** before they publish an image. Prompts, LoRA names, merge recipes and sampler settings all leak far more about how an image was made than people expect.

SD Meta Editor is built around that goal. Its core promise is not "you can edit the fields", it is **"you can be confident it is actually gone."**

## Design goals

| Goal | How |
|---|---|
| **Nothing is left behind** | Covers the easy-to-miss vectors: `zTXt`, `eXIf`, XMP, MakerNote, GPS, embedded EXIF thumbnails, and alpha-channel LSB steganography (stealth PNG) |
| **Disclosure is kept** | C2PA / Content Credentials are **never stripped**. Hiding how an image was made is not the same as hiding *that* it is AI-generated ([ADR-0011](docs/adr/0011-c2pa-not-removed.md)) |
| **No quality loss** | Metadata lives outside the pixel data. Saving rewrites only the container chunks and copies the image bitstream verbatim — no decode, no re-encode |
| **Fast** | A lossless save is roughly the cost of a file copy |
| **Never freezes** | All I/O and decoding runs off the UI thread. No IPC round-trip per keystroke |
| **Proves itself** | After every save the file is read back and verified to contain nothing |

## Planned features (v1)

- JPEG / PNG / WebP — drag & drop, file dialog, OS file association, clipboard
- Structured parsing of A1111 / Forge / Forge Neo parameters
- Raw chunk view with keep/remove toggles for ComfyUI and unknown formats
- Deletion presets (prompts, LoRA, seed, model & environment, extensions, everything)
- Batch processing across a folder
- stealth PNG detection and removal
- Original format preserved on export, with an `_edited` filename suffix
- Optional re-encode mode with configurable quality

## Known limitations

**Invisible in-pixel watermarks such as SynthID cannot be removed by this tool.** Stripping metadata does not make an image untraceable. We would rather say this plainly than let the tool imply otherwise.

**Editing invalidates a C2PA signature.** A C2PA manifest hashes the byte ranges of the asset it
signs, so rewriting any other metadata breaks that binding. We keep the manifest rather than delete
it, but a verifier will report the image as modified after signing. The app warns you before editing
a file that carries Content Credentials.

**Releases are not code-signed.** macOS Gatekeeper and Windows SmartScreen will warn you. See
[Installation](#installation).

## Architecture

```
crates/sdmeta-core/   Pure Rust. Parsing, editing, lossless writing. UI-independent, unit tested
crates/sdmeta-cli/    Command-line interface for batch work and format research
app/                  Tauri 2 shell (Rust backend + React/TypeScript frontend)
```

The hard part — byte-level container surgery — lives in a GUI-independent crate so it can be tested without a GUI, and so the CLI comes for free.

## Supported platforms

| Platform | Supported version |
|---|---|
| macOS on **Apple Silicon** (arm64) | **14.6 or later** ([ADR-0013](docs/adr/0013-minimum-macos-version.md)) |
| Windows | **11 only** |
| Linux (x86_64) | **Ubuntu 24.04 LTS** |

Every build is produced and tested on real hardware for its own platform
([ADR-0009](docs/adr/0009-local-builds-all-three-os.md)). Older OS versions are out of scope: we
will consider them if a contributor owns the machine and can build **and test** on it themselves.

**Intel Macs are not supported.** The macOS build targets `aarch64-apple-darwin` only and will not
launch on an Intel machine — see [ADR-0008](docs/adr/0008-apple-silicon-only.md) for the reasoning.

## Installation

There are no releases yet. When there are, they will be published **only on GitHub Releases**
([ADR-0012](docs/adr/0012-github-only-distribution.md)) — no Homebrew, no winget — with SHA-256
checksums attached. Builds are **not code-signed**, because a paid certificate is not justified at
this scale ([ADR-0010](docs/adr/0010-no-code-signing.md)), so each OS will warn you once:

| OS | What you will see | How to proceed |
|---|---|---|
| macOS | "cannot be opened because the developer cannot be verified" | System Settings → Privacy & Security → **Open Anyway**. Control-clicking the app no longer works on macOS 15 and later. |
| Windows 11 | "Windows protected your PC" | **More info** → **Run anyway** |
| Ubuntu | AppImage will not start | `chmod +x SD_Meta_Editor-*.AppImage` |

## Documentation

- [Requirements](docs/requirements.md)
- [Architecture Decision Records](docs/adr/README.md)
- [Metadata format research](docs/research/metadata-formats.md)
- [3-OS verification checklist](docs/verification.md) — the only quality gate now that CI is gone; kept up to date as implementation proceeds

## Status and roadmap

Design is complete through ADR-0008; implementation has not started. The planned order of work is listed at the end of the [requirements document](docs/requirements.md#8-実装の着手順推奨).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Sample images from generators we do not yet handle are especially welcome — see the fixture policy in [tests/fixtures/README.md](tests/fixtures/README.md).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

Apache-2.0 was chosen over MIT for its explicit patent grant and its contribution clause (§5), which means contributions are licensed under the same terms without requiring a separate CLA.
