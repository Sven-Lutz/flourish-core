# Contributing to Flourish Core

Thank you for helping improve the Flourish domain kernel. This repository is intentionally small: it publishes the **stand-alone MIT-licensed Rust Core**, not the private Flourish desktop application, server infrastructure, AI models, personal journals, proprietary third-party assets or internal project tooling.

## Before proposing changes

Start with a small GitHub issue or focused pull request explaining the user-visible or domain-level problem. Avoid changing public domain contracts purely to make the API more complex; document behavior, safety boundaries and migration consequences where relevant. Maintainers may decline changes outside the Core's scope.

**Never submit** actual personal/journal/health records, identifying screenshots, authentication material, secrets, proprietary code, internal Flourish repository history, or unpublished scientific documents. Use synthetic test data.

For security vulnerabilities use the confidential process in [SECURITY.md](.github/SECURITY.md), **not** a public issue.

## Local validation

The repository includes `Cargo.lock` for reproducible dependency resolution. The initial source candidate was tested with **Rust 1.94.1** (126 tests passed) and also with Rust 1.98. This is **not yet a formal minimum-supported-Rust-version (MSRV) commitment**.

```sh
cargo +1.94.1 test --locked
```

If Rust 1.94.1 is not installed, use an available compatible toolchain and state which version you tested. Do not infer compatibility with untested versions. Tests are local; this project makes no promise of a hosted CI service.

## Pull request expectations

- Explain motivation, scope, risks, and what changed.
- Add regression tests for material behavior changes and note commands that actually ran.
- Respect user-owned decisions, explicit authorization, privacy boundaries, evidence provenance and uncertainty; keep domain rules separate from storage or transport.
- Keep the patch focused. No formatting-only churn or bundled unrelated features.
- For external facts or scientific claims, include stable sources and limitations. A domain type or method metadata must not imply an intervention is clinically validated.
- State remaining issues and any platform/toolchain checks that are **UNVERIFIED**.

## Licensing and rights

The files in this repository are available under the [MIT License](LICENSE), with the existing copyright notice preserved. By submitting changes, contributors must have the right to contribute the material and be willing for the accepted contribution to be distributed under this same repository license. Do not copy third-party text or code without verifying its redistribution and attribution requirements. No contributor copyright assignment is requested by this document.

An open pull request is not a promise of acceptance or a support arrangement.
