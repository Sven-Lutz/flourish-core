# Flourish Core

**Public source:** Stand-alone, MIT-licensed Rust library for Flourish's domain kernel. This is the first public open-source component of Flourish; **it is not a complete desktop/mobile Personal OS or a tagged production release**. The broader Flourish application is developed separately and has not been publicly released under this license.

`flourish-core` provides storage-independent domain contracts for user authorization, capture, personal context, user-owned Paths, identity and synchronization, evidence metadata, and Method/Protocol rules. It contains no desktop UI, hosted service, model, database, or personal journal records.

## Build and test

The initial source release was validated with **Rust 1.94.1** (126 tests) and was also tested with Rust 1.98. These are verified toolchains, **not a formally guaranteed minimum supported Rust version (MSRV)**.

```sh
cargo +1.94.1 test --locked
```

Use `cargo test --locked` with a compatible local toolchain if Rust 1.94.1 is not installed, and report the actual version. `Cargo.lock` pins dependency versions; direct dependency `url = "=2.5.8"` is declared in `Cargo.toml`.

The crate's manifest currently declares `publish = false`: **there is no crates.io publication or binary release**. The package version `0.1.0` reflects its development stage and is not a published release tag or a stability promise.

## Domain and security boundaries

- Domain authorization and user-owned decisions must be enforced by adapters and applications embedding the Core; the library is not itself a storage or access-control service.
- Operational evidence, scientific claims and personal observations have distinct authority. Method metadata must not be interpreted as medical advice or proof that an intervention works.
- Never commit actual journal, health, identity or other private records; use synthetic fixtures only.
- This is research-informed domain infrastructure, **not a medical device or clinically validated intervention**.

## Community and security

Contributions and local testing guidance: [CONTRIBUTING.md](CONTRIBUTING.md).

For suspected vulnerabilities, **do not use public issues**. Use [private vulnerability reporting](https://github.com/Sven-Lutz/flourish-core/security/advisories/new); see the [Security Policy](.github/SECURITY.md).

There are currently **no tagged releases or support guarantees**. Changes to source contracts may be incompatible during the development phase.

## License

This stand-alone Core source is licensed under [MIT](LICENSE), Copyright (c) 2026 Sven Lutz. Please retain the copyright and license notices when redistributing all or substantial portions.

**Scope:** This license does not automatically apply to the full Flourish application, external publications, third-party dependencies, Flourish brand marks or visual identity. The trademark and visual branding are not granted by the Core's MIT license.
