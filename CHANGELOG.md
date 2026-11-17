# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

Nothing has been released yet. The project is in its design phase; everything below is groundwork.

### Added
- Cargo workspace skeleton (`sdmeta-core`, `sdmeta-cli`) with no implementation yet
- Requirements document covering FR-1 to FR-12 and NFR-1 to NFR-8
- Metadata format research notes, with a synthetic test fixture reproducing the parser pitfalls
- Architecture Decision Records ADR-0001 through ADR-0013
- A 3-OS verification checklist, the only quality gate now that CI is gone
- Apache-2.0 `LICENSE` and `NOTICE`, contribution guide, issue and pull request templates

### Removed
- GitHub Actions CI. Builds and checks run on local hardware instead — see
  [ADR-0007](docs/adr/0007-ci-abolished-local-builds.md)

<!--
Add a section here on release.
## [0.1.0] - YYYY-MM-DD
-->
