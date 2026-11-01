# Flourish Core

**Status:** Stand-alone Rust source package for the Flourish domain kernel. This package is being prepared for a future public source release; the surrounding Flourish application remains private until separately reviewed and authorized.

`flourish-core` contains **storage-independent domain contracts**, including user authorization, capture, personal context, user-owned Paths, identity/synchronization contracts, scientific evidence metadata, and Method/Protocol rules. It does not ship a desktop UI, a web service, a hosted AI model, a database, personal journal records, or a working clinical intervention.

## Development

Requirements: Rust toolchain compatible with this checkout. The private monorepo pins a toolchain; the exact released Core version should be tested against its documented minimum supported Rust version before public release.

From the monorepo root:

```sh
cargo test --locked --manifest-path crates/flourish-core/Cargo.toml
```

From a stand-alone copy of this directory:

```sh
cargo test --locked
```

The only direct external dependency declared in this package is `url = "=2.5.8"`; transitive dependencies are recorded in `Cargo.lock`. The manifest currently declares `publish = false` to prevent accidental publication to a package registry. Opening the source on GitHub does **not** automatically authorize publication to crates.io. Tests verify library behavior; they do **not** constitute evidence that Flourish improves real-world outcomes or is suitable for diagnosis or treatment.

## Architecture and boundaries

- The core validates and represents typed domain rules. Storage, network integrations, rendering, API keys and user data must remain in separate adapters/products.
- Changes to important user-owned state require the authorization and confirmation contracts defined by the product. Other applications embedding this crate must correctly enforce those contracts in their own integration layer.
- Evidence metadata and observed personal behavior have different authority. A library type or successful test is not an endorsement of any scientific intervention.
- Avoid logging or committing real journal, health, identity or personal-context records when adding test fixtures.

## License and contribution status

**Core source code only:** MIT — see [LICENSE](LICENSE), Copyright (c) 2026 Sven Lutz.

This scoped license does **not** automatically cover the rest of the Flourish application, its brand assets, external research publications, or any separately licensed dependencies. Copies and substantial portions of licensed Core software must preserve the MIT notice and permission text.

The Flourish name and visual identity are not licensed as trademarks by this software license. The separate public source repository and contributor/security intake process will be announced only after the final security and release review.

## Limitations

This library is not a complete Personal OS on its own. Platform clients, persistent storage, synchronization infrastructure, integrations, scientific Method curation and any optional hosted services are separate components. No API stability or backwards compatibility promise is implied before a tagged public release.
