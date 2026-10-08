# Polars upstream API baseline

This baseline defines the denominator for the Terlan Polars Rust API inventory.
The stable inventory denominator is the last published Rust release, Polars
0.55.2. Runtime behavior is pinned separately to the official Python Polars
2.0 source revision; see `POLARS_2_ALIGNMENT.md`.

## Extraction

The inventory is generated from rustdoc JSON with `cargo-public-api` 0.52.0
and `nightly-2026-07-16`. Enabling every Polars Cargo feature is not the
baseline: Polars contains mutually toolchain-sensitive features. The package's
resolved feature graph is.

```bash
cargo metadata \
  --format-version 1 \
  --manifest-path native/Cargo.toml \
  --features real-polars

cargo +nightly-2026-07-16 public-api \
  --manifest-path <cargo-registry>/polars-core-0.55.2/Cargo.toml \
  --no-default-features \
  --features <resolved-polars-core-features> \
  -sss
```

The same extraction applies to `polars-lazy`, `polars-ops`, `polars-io`,
`polars-time`, and `polars-sql`. The top-level `polars` facade cannot be used
alone because rustdoc represents dependency glob re-exports as opaque
`<<crate::prelude::*>>` entries.

## Initial inventory

The feature-resolved public API output contains:

| Component | Public API lines |
| --- | ---: |
| `polars-core` | 13,993 |
| `polars-lazy` | 711 |
| `polars-plan` | 10,882 |
| `polars-io` | 2,357 |
| `polars-sql` | 35 |

`polars-ops` produces valid rustdoc JSON, but `cargo-public-api` 0.52.0 is
killed while rendering its expanded implementation graph on the current
machine. Its inventory must therefore be read directly from the same rustdoc
JSON until the renderer can process it within the memory budget.

The high-level callable census contains 165 unique `DataFrame` method names,
193 unique `Series` method names, and 115 unique `LazyFrame` method names.
Terlan currently generates 1,029 distinct native operations, including 143
`polars.dataframe.*`, 149 `polars.series.*`, and 92
`polars.lazy_frame.*` operations. These counts are not a parity percentage: Rust
overloads may combine into one Terlan operation, and one Rust behavior may be
implemented by a differently named Terlan composition.

The committed high-level census and reviewed operation aliases are checked by:

```bash
cargo run --manifest-path native/Cargo.toml \
  --features real-polars \
  --bin polars_api_parity
```

The report currently inventories all 473 high-level `DataFrame`, `Series`, and
`LazyFrame` method names. Passing `--require-complete` makes any remaining
unmapped identity fail the command. The report separates exact native-name
matches from explicit mappings. This is an inventory gate; behavioral parity
requires separate executable evidence for the mapped behavior.

## Mapping rules

Every user-visible upstream identity is classified for inventory purposes as:

- `direct`: one Terlan operation preserves the behavior;
- `reshaped`: a documented Terlan API combines overloads, options, ownership,
  or callback composition while preserving behavior; or
- `pending`: behavior is not yet available.

The checker verifies that direct and explicit mapped operation names still
exist in generated native metadata. It does not inspect the reshaping rationale
or prove observable equivalence. Unsafe, mutable, pointer-based, or
callback-based Rust signatures still require a safe Terlan design and
behavioral tests before parity can be claimed.

The package must not claim full API parity solely because this inventory has no
unmapped identities.
