# Contributing to SD Meta Editor

[日本語版はこちら](CONTRIBUTING.ja.md)

The project is still in its **design phase** — no application code exists yet. At this stage the two
most valuable contributions are **feedback on the design** and **sample images**.

---

## 1. Sample images (most welcome)

The quality of this tool depends almost entirely on **how many real-world generators we have actually
seen output from**. If you have images produced by a tool we do not handle yet, please send them our way.

Read the policy in [tests/fixtures/README.md](tests/fixtures/README.md) before you do.
Fixture images are committed **with their metadata intact** — that metadata is the thing under test.
Do not submit an image containing anything you would not want published.

## 2. Feedback on the design

- If you find a path through which metadata survives removal, we treat it as a **security issue**, not
  an ordinary bug. Use the "Metadata leak" issue template.
- To disagree with a design decision, read the relevant record under [docs/adr/](docs/adr/) first, then
  open an issue referring to it.

> The ADRs are written in Japanese. An English translation may be added later; until then, the
> rationale for every decision lives in those files and the issue tracker is fine in either language.

## 3. Design invariants

These are premises of the project. Changing any of them requires a new ADR.

| Invariant | Source |
|---|---|
| Never modify the original file (unless explicitly opted in) | FR-6 |
| The default save path never re-encodes pixel data | FR-6 |
| ICC profiles, EXIF Orientation and C2PA are always preserved | FR-9, ADR-0011 |
| Unknown keys and chunks must remain removable | FR-3 |
| Re-read and verify the file after saving | FR-7 |
| Never swallow an error | ADR-0001 C-6 |
| Never put CPU work on the UI thread | ADR-0001 C-1 |
| A keystroke must not cause IPC | ADR-0001 C-2 |
| Lossless writing counts as verified only after testing on all three OSes, on real hardware | ADR-0009 C-13 |

> Using an AI coding agent on this repository? Point it at [AGENTS.md](AGENTS.md) first.

## 4. How we use ADRs

Design decisions are recorded in [docs/adr/](docs/adr/).

- One decision per file
- **An ADR that has been Accepted is never rewritten.** When a decision changes, write a new ADR and
  mark the old one `Superseded`
- Start from [template.md](docs/adr/template.md)

## 5. Coding conventions (once implementation starts)

- Rust: code must pass `cargo fmt` and `cargo clippy`. Lint levels are set in the workspace
  `Cargo.toml` under `[workspace.lints]` — do not pass `-D warnings`, it overrides the deliberate
  distinction between `deny` (blocking) and `warn` (advisory)
- TypeScript: formatter and linter will be fixed when implementation begins
- Commit messages: [Conventional Commits](https://www.conventionalcommits.org/)

### Checks to run before submitting

**This repository has no CI** ([ADR-0007](docs/adr/0007-ci-abolished-local-builds.md)). Nothing will
stop a broken change automatically, so please run these locally before opening a pull request.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

Record the results in the "verification" section of your pull request.

### Build targets and where they are built

| Target | Built on |
|---|---|
| macOS 14.6+ (Apple Silicon / arm64) | the developer's Mac |
| Windows 11 | real Windows hardware |
| Ubuntu 24.04 LTS (x86_64) | real Ubuntu hardware |

A Mac cannot produce Linux or Windows bundles, so each platform is built on its own machine.
Supported OS versions, and what happens with older ones, are covered in
[ADR-0009](docs/adr/0009-local-builds-all-three-os.md) and
[ADR-0013](docs/adr/0013-minimum-macos-version.md).

If your change touches the write path, invariant C-13 applies: it must be verified on all three
machines. The checklist is [docs/verification.md](docs/verification.md).

## 6. Development flow

One issue, one branch, one pull request. **Nothing is pushed directly to `main`.**

### Branch names

```
<type>/<issue number>-<short slug>

docs/59-pr-workflow
feat/10-quote-aware-tokenizer
fix/21-png-idat-copy
```

`<type>` is the same set as the Conventional Commits prefix, so there is nothing extra to remember.
The issue number makes the target obvious from the branch name alone.

### Pull requests

- Reference the issue with `Closes #<n>` so it closes on merge
- Fill in the "Verification" section — **this repository has no CI**
  ([ADR-0007](docs/adr/0007-ci-abolished-local-builds.md)), so the checks you ran there are the only
  evidence that anything works
- **Squash merge.** One issue becomes one commit on `main`, which keeps the history one-to-one with
  the issue list. Write the squashed message as a Conventional Commit whose body explains *why*
- Delete the branch after merging

### Issues labelled `needs-hardware`

These require macOS, Windows 11 **and** Ubuntu 24.04 on real machines (invariant C-13). They carry a
checklist instead of code, so they are closed by working through that checklist and recording the
results in the issue — **no branch, no pull request.** See
[docs/roadmap.md](docs/roadmap.md) for which issues these are.

### Where the work comes from

[docs/roadmap.md](docs/roadmap.md) holds phases P0–P8. Work the phase you are in, in dependency
order; a phase does not start until the previous one meets its exit criteria.

## 7. Contact

Please use GitHub Issues for bug reports and feature requests.

## 8. Licensing of contributions

This project is released under the [Apache License 2.0](LICENSE).

Under **§5 (Submission of Contributions)** of Apache-2.0, any contribution you intentionally submit
for inclusion in this project is **licensed under the same terms as the project**, unless you state
otherwise explicitly. For that reason this project does **not** ask for a separate CLA
(Contributor License Agreement).

By opening a pull request you represent that you have the right to submit the work under those terms.

### Header for new files

Add a single SPDX identifier line to every new source file.

```rust
// SPDX-License-Identifier: Apache-2.0
```

### The NOTICE file

`NOTICE` carries the attribution notices required by Apache-2.0 §4(d). If you bring in third-party
code, add its attribution there.
