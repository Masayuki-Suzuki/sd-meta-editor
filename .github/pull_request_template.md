## Summary

<!-- What changed, and why. -->

## Related issues / ADRs

<!-- e.g. Closes #12 / implements ADR-0003 -->

## Verification

<!-- How you checked this. If you added a fixture, say so. -->

This repository has **no CI** ([ADR-0007](https://github.com/Masayuki-Suzuki/sd-meta-editor/blob/main/docs/adr/0007-ci-abolished-local-builds.md)), so nothing
runs these for you. Tick what you ran:

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] Not applicable (no Rust code touched)

### Hardware tested on

Required for any change to the write path (invariant C-13). Each platform is built and tested on real
hardware — see [ADR-0009](https://github.com/Masayuki-Suzuki/sd-meta-editor/blob/main/docs/adr/0009-local-builds-all-three-os.md).

- [ ] macOS (Apple Silicon)
- [ ] Windows 11
- [ ] Ubuntu 24.04 LTS
- [ ] Not applicable (write path untouched)

## Design invariants

Confirm this change does not break any invariant below. **If it does, it needs a new ADR** — see
[CONTRIBUTING.md](https://github.com/Masayuki-Suzuki/sd-meta-editor/blob/main/CONTRIBUTING.md) ([日本語](https://github.com/Masayuki-Suzuki/sd-meta-editor/blob/main/CONTRIBUTING.ja.md)).

Tick every box, or mark the whole section N/A for a docs-only change.

- [ ] The original file is never modified (unless explicitly opted in) — FR-6
- [ ] The default save path does not re-encode pixel data — FR-6
- [ ] ICC profiles and EXIF Orientation are preserved — FR-9
- [ ] Unknown keys and chunks remain removable — FR-3
- [ ] The file is re-read and verified after saving — FR-7
- [ ] No error is swallowed — ADR-0001 C-6
- [ ] No CPU work runs on the UI thread — ADR-0001 C-1
- [ ] No keystroke causes IPC — ADR-0001 C-2
- [ ] Lossless writing is verified on all three OSes, on real hardware — ADR-0009 C-13
- [ ] Not applicable (documentation only)
