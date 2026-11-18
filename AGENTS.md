# AGENTS.md

Context for AI coding agents working in this repository. Read this before changing anything.

Humans: start with [README.md](README.md) and [CONTRIBUTING.md](CONTRIBUTING.md) instead.

## What this project is

A desktop app (Tauri 2 + Rust + React) for inspecting, editing and **reliably removing** the
generation metadata that Stable Diffusion and similar tools embed in JPEG / PNG / WebP images.

The product insight that drives every design decision: **most people reaching for this kind of tool
want the metadata gone, not edited.** The core value is being able to trust that it is actually gone.
Competing tools freeze on large metadata blobs; not freezing is a requirement, not a nice-to-have.

## Current state — read this before you claim anything works

**The project is in its design phase. There is no implementation.**

- `crates/sdmeta-core/src/lib.rs` and `crates/sdmeta-cli/src/main.rs` contain doc comments and
  `fn main() {}` — nothing else
- `app/` is not scaffolded. There is no Tauri project, no frontend
- `cargo test` reports 0 tests
- Nothing has been released

Do not write documentation, changelog entries or commit messages that imply otherwise.

## Five rules that override your defaults

### 1. Never commit real metadata from a real image

Test fixtures must be **synthetic**. This already went wrong once: a real parameter string from the
project owner's own generation was committed and had to be purged from history.

LoRA names, merge recipes, model hashes and training job IDs expose how an image was made — for many
users that is the core of their work. Reproduce the *structure* that breaks parsers, invent every
name and hash. See [tests/fixtures/README.md](tests/fixtures/README.md).

`tests/fixtures/text/forge-neo-params.txt` is the reference: 31 keys, 74 commas, 44 of them inside a
single quoted `Lora hashes` value, 45 LoRA entries — all fake.

### 2. Never rewrite an Accepted ADR

Design decisions live in [docs/adr/](docs/adr/) and are **append-only once Accepted**. When a
decision changes, write a new ADR and state in it exactly what it replaces.

If only a *derived constraint* of an older ADR changes, do **not** mark that whole ADR `Superseded` —
that would misrepresent history. ADR-0008, ADR-0009 and ADR-0013 all follow this pattern and explain
the deviation in their own text. Follow it.

### 3. One issue, one branch, one pull request — and never push to `main`

Work comes from [docs/roadmap.md](docs/roadmap.md), registered as GitHub issues. For each one:

```
git switch -c <type>/<issue number>-<short slug>   # e.g. feat/10-quote-aware-tokenizer
```

Open a pull request with `Closes #<n>` and fill in its Verification section. Merges are **squash**,
so one issue becomes one commit on `main`. Full rules: [CONTRIBUTING.md](CONTRIBUTING.md) §6.

**Still ask before committing or pushing.** Make the edits, then stop and report what changed. The
repository owner decides when work enters history, and one approval covers one commit — being on a
branch does not change that.

Issues labelled `needs-hardware` get **no branch and no pull request**; they are closed by working
through their checklist on real machines, which you cannot do (see rule 4).

### 4. There is no CI — you are the gate

[ADR-0007](docs/adr/0007-ci-abolished-local-builds.md) removed GitHub Actions. Run this before
reporting any Rust change as done:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

If your change touches the write path, invariant **C-13** applies: it is not verified until it has
run on macOS, Windows 11 and Ubuntu 24.04 **on real hardware**. See [docs/verification.md](docs/verification.md).
You cannot satisfy C-13 yourself — say so instead of claiming verification.

### 5. Report what you actually checked

Say plainly when something is untested, unverified or assumed. This is a tool that rewrites people's
image files; an optimistic report is worse than no report.

## Design invariants

Breaking any of these requires a new ADR, not a code change.

| Invariant | Source |
|---|---|
| Never modify the original file (unless explicitly opted in) | FR-6 |
| The default save path never decodes or re-encodes pixel data | FR-6 |
| ICC profiles, EXIF Orientation and C2PA are always preserved | FR-9, ADR-0011 |
| Unknown keys and chunks must remain removable | FR-3 |
| Re-read and verify the file after saving | FR-7 |
| Never swallow an error — toast **and** log file | ADR-0001 C-6 |
| Never put CPU work on the UI thread | ADR-0001 C-1 |
| A keystroke must never cause IPC | ADR-0001 C-2 |
| Never send preview images over IPC or as base64 | ADR-0001 C-3 |
| Lossless writing is verified only on all three OSes, on real hardware | ADR-0009 C-13 |

The one documented exception to losslessness is PNG re-encoding when removing stealth alpha-LSB
steganography ([ADR-0004](docs/adr/0004-stealth-png-removal.md)).

## Writing Rust here

Formatting and the obvious non-idiomatic patterns are already settled: `rustfmt.toml` fixes style and
`[workspace.lints]` in the root `Cargo.toml` makes the mechanical rules **fail the build**. `unsafe`
is `forbid`-level, so it cannot be re-enabled with `#[allow]`. Run `cargo clippy` and it enforces
itself — do not weaken those lint levels to make your code compile.

What the compiler cannot decide for you is below. These are the decisions, not the style.

**Error types.** `sdmeta-core` returns typed errors (`thiserror`). `sdmeta-cli` may use `anyhow`
internally. **Never expose `anyhow::Error` or `Box<dyn Error>` from the library's public API** — the
GUI needs to match on error kinds to satisfy C-6 (toast *and* log, never swallow).

**Allocation budget.** Pass image bytes as slices, not owned `Vec<u8>`. NFR-3 gives a lossless save
of a 10MB file 50ms, which is roughly a file copy — there is no room for incidental copies. If a copy
is genuinely required, say why in a comment directly above it.

**Checked arithmetic on anything the file told you.** PNG chunk lengths, EXIF offsets and IFD counts
come from untrusted input. Use `checked_add` / `checked_mul` / `try_into()`, never `as`. A truncating
cast here produces a plausible-looking wrong length, which is the worst failure mode for a parser.

**Tests are round-trip assertions, not happy paths.** The primary assertion is byte equality of the
pixel data before and after a save (see section A of [docs/verification.md](docs/verification.md)).
When you fix a parsing bug, **add a fixture that reproduces it** — that is how the corpus grows.

**Do not abstract before the second implementation exists.** Per-tool adapters will eventually need a
trait; one adapter does not. No generic parameters or `Box<dyn _>` indirection until something
actually varies.

**Dependencies need a reason.** Apache-2.0 compatible only, GPL-family is incompatible
([ADR-0005](docs/adr/0005-license-apache-2.0.md)), and NFR-8 caps the binary at 20MB. State the
reason in the PR body or an ADR.

**`#[allow]` always carries a reason.** Put the comment directly above it. `indexing_slicing` has
legitimate uses right after a length check — that is fine, as long as the comment says so. Never put
a blanket `#![allow]` at crate or module level to silence a class of findings.

## Things that are easy to get wrong here

**Splitting the A1111 parameter line on `,` is the bug that breaks every competing tool.**
`Lora hashes: "name: hash, name: hash, ..."` holds dozens of commas inside one quoted value. A
quote-aware tokenizer is mandatory (FR-4).

**Write-back is span-based diff removal, not reserialization.** Preserve the original whitespace, key
order and quoting. Rebuilding the string makes other tools unable to read the file.

**There are no universally present keys.** Only semantic roles are shared across tools, which is why
the canonical field model exists (FR-3) and why presets reference canonical fields, never tool-specific
key names (C-8).

**`Size` is the base generation size, not the final dimensions.** Hires upscale changes the output.

**C2PA is preserved on purpose** ([ADR-0011](docs/adr/0011-c2pa-not-removed.md)). Hiding *how* an
image was made is supported; hiding *that* it is AI-generated is not. Editing still invalidates the
C2PA signature — that is a documented limitation, not a bug to fix.

## Conventions

**Commit messages** — [Conventional Commits](https://www.conventionalcommits.org/) prefix, subject and
body **in Japanese**, matching existing history. The body explains *why*, and references the ADR or FR
it implements. Look at `git log` before writing one.

**Public documents** — English is canonical, with a `*.ja.md` counterpart (`README.md` / `README.ja.md`,
`CONTRIBUTING.md` / `CONTRIBUTING.ja.md`). Keep both in sync.

**ADRs and research notes are written in Japanese.** Do not translate them; they are the owner's
working record.

**New source files** carry `// SPDX-License-Identifier: Apache-2.0`.

**Dependencies** — Apache-2.0 compatible only; GPL-family licences are incompatible
([ADR-0005](docs/adr/0005-license-apache-2.0.md)). Adding `cargo-deny` is a planned task.

## Where to look

| Question | File |
|---|---|
| What is being built, and to what numbers | [docs/requirements.md](docs/requirements.md) |
| Why a decision was made, and what was rejected | [docs/adr/README.md](docs/adr/README.md) |
| Where each tool stores metadata, and parsing pitfalls | [docs/research/metadata-formats.md](docs/research/metadata-formats.md) |
| How to verify a change on three OSes | [docs/verification.md](docs/verification.md) |
| Fixture policy | [tests/fixtures/README.md](tests/fixtures/README.md) |
| Branch, PR and merge rules | [CONTRIBUTING.md](CONTRIBUTING.md) §6 |
| Implementation order, phase by phase | [docs/roadmap.md](docs/roadmap.md) |
| Why the order is what it is | [docs/requirements.md](docs/requirements.md) §8 |

## Build targets

| Target | Built on | Supported version |
|---|---|---|
| macOS (Apple Silicon, arm64 only) | the owner's Mac | 14.6+ |
| Windows | real Windows hardware | 11 only |
| Linux (x86_64) | real Ubuntu hardware | 24.04 LTS |

A Mac cannot produce Linux or Windows bundles. Intel Macs and universal builds are out of scope
([ADR-0008](docs/adr/0008-apple-silicon-only.md)). Releases are unsigned and published only on
GitHub Releases ([ADR-0010](docs/adr/0010-no-code-signing.md), [ADR-0012](docs/adr/0012-github-only-distribution.md)).

## If you are starting implementation

[docs/roadmap.md](docs/roadmap.md) breaks the work into phases P0–P8 and 58 issue-sized tasks, each
with its own acceptance criteria and dependencies. **Work the phase you are in, in dependency order**
— a phase does not start until the previous one meets its exit criteria.

Seven tasks are marked 🧑 in the roadmap — P0-6, P2-3, P3-10 and all of P8. They need real hardware,
a running generator, or a publishing action, so **you cannot complete them.** Each one has a
ready-to-paste checklist in the roadmap's appendix. Say that they need a human rather than closing
them, and never mark one done because the code side looks finished.

The first test to write is the quote-aware tokenizer against
`tests/fixtures/text/forge-neo-params.txt`.
