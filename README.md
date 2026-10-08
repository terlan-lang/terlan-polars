# terlan-polars

Polars DataFrame integration for Terlan, and the first proof package for
Terlan's third-party module system.

See [ROADMAP.md](ROADMAP.md) for the implementation plan.

## Status

This package currently provides:

- the public `polars.DataFrame` Terlan API;
- package metadata for Git-first Terlan dependency resolution;
- a Rust native adapter crate;
- native package metadata that advertises the helper executable and
  environment variable consumed by Terlan runtime builds;
- default stub native behavior for compiler/package validation;
- a package-owned NativeBoundary helper executable for runtime calls;
- explicit `DataFrame.dispose()` lifecycle control with stale and forged
  handle rejection;
- correlated helper replies with an explicit synchronous request-credit
  window;
- native helper Cargo feature metadata so `terlc run` can build the helper
  when needed;
- real CSV, Parquet, JSON/NDJSON, and IPC I/O, including lazy streaming sinks
  for CSV, Parquet, NDJSON, and IPC; DataFrame and Series observers and exact
  equality with ordinary or null-equal semantics;
  typed and nullable Series construction, populated temporal Series, and
  scalar plus recursive List/Array/Decimal/Struct, Categorical/Enum, and
  timezone-aware Datetime `DataType` operations, including Int128/UInt128 and
  Float16 storage; typed schema inspection; Series
  literal-expression conversion; expression pipelines; stable grouped
  aggregation;
  configurable equi joins with typed cardinality validation and deterministic
  row-order policies, as-of and expression-driven non-equi joins; vertical,
  horizontal, and diagonal concatenation; subset-aware row deduplication and
  null removal; bounded head/tail, positive/negative-offset slicing, and
  bounds-safe indexed row gathering with optional null rows; seeded and
  unseeded fixed-count/fractional row sampling with replacement, shuffle, and
  generated-output safety controls; per-row uniqueness/duplication masks and
  default or reproducibly seeded UInt64 row hashes;
  configurable multi-column ordering and top/bottom-row selection;
  ordered dtype inspection, Polars-compatible unit-aware DataFrame/Series size
  reporting, typed DataFrame construction from named Series, non-strict column
  removal, multi-predicate and named-constraint filtering, chained conditional
  expressions, and strided row gathering;
  schema-preserving row clearing and chunk consolidation; immutable column
  renaming/removal, row reversal, row shifting with optional expression fills,
  and row-index insertion; whole-frame
  null/NaN filling, NaN-row removal, and null-count summaries;
  whole-frame sum/product/mean/median/minimum/maximum, variance, standard
  deviation, quantile, non-null/total count, and exact/approximate cardinality
  reductions; reshape operations;
  and composable lazy scans/query plans with schema
  resolution and explicit caching behind
  `real-polars`;
- immutable opaque `Expr` values with typed builders for column selectors,
  literals, dates, arithmetic, predicates, aliases, name transforms,
  aggregation, calendar/subsecond temporal components and projections,
  month boundaries, epoch timestamps, calendar bucketing/offsets, complete
  Duration totals, string splitting, List statistics,
  literal and regex String search/counting, Unicode-aware padding, zfill,
  normalization and reversal, hexadecimal/Base64 codecs, and regex escaping;
  slicing, shifting, joining, sampling, gathering, differences, null removal,
  match counting, set algebra, element evaluation/aggregation, configurable
  null/order behavior, and List-to-Array/Struct conversion; plus
  complete seeded/unseeded random shuffle and sampling expressions, and
  complete integer bit-count, leading/trailing-bit, and bitwise-reduction
  expressions; complete scalar/group statistics including NaN-propagating
  extrema, min/max-by, and automatic/count/explicit-bin histograms; complete
  circular and hyperbolic trigonometry, inverse functions, quadrant-aware
  `arctan2`, and degree/radian conversion; complete Boolean validity,
  finiteness, NaN, distinctness, membership, interval, proximity, logical,
  horizontal, and reduction expressions; complete forward/reverse cumulative
  count, sum, product, minimum, and maximum expressions; configurable
  numeric rounding modes, significant-figure rounding, truncation,
  floor/ceiling, absolute value, clipping, logarithms, exponentials, entropy,
  skewness, kurtosis, product aggregation, pi, and data-type bounds;
  scalar count/length, null-aware endpoints, singleton/List aggregation,
  quantiles, modes, unique values/indices, extrema indices, dot products, and
  null/NaN removal;
  generic slice/append/rechunk, head/tail, configurable sorting and argument
  sorting, index lookup/search, gather/get, reverse/shift/fill, repeat-by,
  discrete and percentage change, strided gather, and constant extension;
  configurable deterministic and seeded-random ranking plus direct and
  multi-key top/bottom-k selection; value replacement with strict/default
  handling; Polars 2 Expr and eager Series interval, quantile, and rank
  binning with explicit or uniform specs, labels or indices, and optional
  interval Structs, plus compiler-safe typed/count breakpoint helpers; legacy
  cut/qcut compatibility; component-based Date/Datetime construction, direct
  base-10 logarithms, and repeat;
  coordinate interpolation, local peak masks, run-length encoding/IDs, and
  validated fixed-width expression reshape; configurable exponentially
  weighted mean, variance, standard deviation, and coordinate-based mean;
  cumulative expression evaluation, group-index aggregation, List-valued
  join windows, and ordered windows with explicit mapping behavior;
  configurable
  fixed-window rolling minimum, maximum, mean, sum, median, variance, and
  standard-deviation expressions plus quantiles, ranks, skewness, and
  kurtosis; complete index/time-based rolling-by minimum, maximum, mean, sum,
  median, variance, standard deviation, quantile, and rank expressions;
  callback-free expression-name retention, affixes, pattern replacement,
  casing, and Struct field affixes; plus
  fixed-width Array statistics, indexing, slicing, shifting, joining,
  exploding, match counting, element evaluation/aggregation, configurable
  null/order behavior, and List/Struct conversion,
  Categorical/Enum inspection, predicates, and slicing; and Binary byte,
  slicing, codec, and typed reinterpretation operations; plus complete Struct
  field selection, renaming, JSON encoding, and field updates; and read-only
  expression metadata for root/output names, projection/literal
  classification, expansion detection, and textual/DOT tree formatting;
- direct string-valued construction through `from_rows`, including empty-frame
  support and stable ragged-row validation; and
- bounded row materialization through `rows(limit)`, with stable null
  rendering and explicit invalid-limit errors;
- SQL contexts with named DataFrame registration and lazy query execution;
- PostgreSQL and SQLite query ingestion, plus deferred HTTP, S3, Google Cloud
  Storage, and Azure scans;
- declarative expression UDFs with positional parameters and required output
  types, plus typed Polars extension descriptors; and
- binary Arrow IPC round trips and VM-brokered numeric projection for ndarray
  and PyTorch consumers.

## Checks

```bash
terlc check src
terlc run examples/consumer_project
terlc run examples/loaded_helper_project
terlc run examples/polars_getting_started
terlc run examples/polars_expressions
terlc run examples/iris_dataset_audit
terlc run examples/polars_series
terlc run examples/polars_io_formats
terlc run examples/polars_lazy_full
terlc run examples/polars_relational
terlc run examples/polars_reshape
terlc run examples/polars_advanced_relational
terlc run examples/polars_expression_namespaces
cargo test --manifest-path native/Cargo.toml
cargo test --manifest-path native/Cargo.toml --features real-polars
```

From the Terlan compiler repository, the full package boundary gate also runs
both Cargo feature profiles, both Terlan consumers, and every package test
against the real helper:

```bash
make terlan-polars-package-check
```

Before publishing a revision, run the package's complete local release gate:

```bash
make release-check
```

The feature-resolved Rust API inventory gate is complemented by the
[Polars 2.0 alignment record](docs/POLARS_2_ALIGNMENT.md) and the
[Polars 1.43 cheat-sheet audit](docs/POLARS_CHEATSHEET_COVERAGE.md). The
[graphics contract](docs/POLARS_GRAPHICS.md) records the supported chart and
backend matrix, output bounds, and the boundary with Altair configuration. The
audit tracks Python conveniences, I/O formats, and ecosystem operations that
are not part of the pinned Rust API denominator, checks observable behavior
beyond name mappings, and records the remaining gaps.

This additionally requires zero unmapped identities in the published Polars 0.55.2 Rust inventory baseline,
strict missing-documentation checks for the Rust adapter, and canonical Terlan
formatting. Numerical-stack integration is checked separately with
`polars-pytorch-interop-check`, `polars-ndarray-pytorch-interop-check`, and
`polars-immutable-ml-interop-check` when LibTorch is available.

The integration runners live under `scripts/` and are registered in
`terlan.toml`. Run `make scripts-check` to recursively type-check and lint
the `.terls` runners, lint their Terlan fixtures, and check formatting without
building the native numerical packages. This validation also runs as part of
`package-check` and `release-check`.

`scripts/CheckSources.terls` owns source discovery and compiler invocation for
`scripts-check`, `terlan-grouped-binding-check`, and
`terlan-function-reference-check`. It reports the failing command and stops
on the first failure; each compiler invocation has a ten-minute timeout and
a 16 MiB output limit.

The package metadata declares the helper binary, helper environment variable,
and Cargo features, so `terlc run` can build
`native/target/debug/terlan-polars-native-boundary` and set the NativeBoundary
helper path for the launched Terlan runtime.

Both qualified calls and selective imports are executable. The consumer
example uses `import polars.DataFrame.{dispose, height, read_csv}.` and calls
the functions without a module qualifier.

`DataFrame` values are opaque aliases of helper-owned resources. Call
`dispose(df)` when ownership is complete. Disposal invalidates every copied
alias of that handle; reuse and duplicate disposal fail with the stable
`stale_native_handle` code.

`LazyFrame` values are also helper-owned. Plans support local-file scans,
expression projections and mutations, ordinary filters and null-preserving
inverse filters, groups, single- and
multi-column sorting, top/bottom-row selection, limits,
generic and non-equi joins, reshape operations, plan descriptions, and
per-node execution profiling, named optimizer-pass controls, sequential
projection/mutation execution, and ordinary or streaming collection. Call
`release(plan)` for every plan handle; query/schema
failures are intentionally deferred until collection.

`from_rows(columns, rows)` constructs a DataFrame without filesystem I/O. Each
row must match the declared column count; otherwise it returns
`invalid_row_width`. Missing and malformed CSV inputs are covered by executable
package tests.

`rows(df, limit)` materializes at most `limit` rows as display strings so
results can be inspected without exposing Polars values through the native
boundary. Nulls render as `"null"`; negative limits and limits above 10,000
return stable errors.

## Official Polars getting-started examples

`examples/polars_getting_started` ports every DataFrame example on the
official Polars getting-started page using the same people, family, and
additional-people data:

```bash
terlc run examples/polars_getting_started
```

The executable covers CSV write/read with dates; ordinary and expanded
`select`; `with_columns`; simple and combined filtering; grouped counts and
aggregations; the chained first-name/decade query; a left join; and vertical
concatenation. Expressions are immutable values, so they require no native
handle or explicit release. Only DataFrame and LazyFrame resources use
`dispose`/`release`.

## Official Polars Rust expression examples

`examples/polars_expressions` executes every non-placeholder tagged example
in Polars' Rust expression-guide source at the pinned upstream revision:

```bash
terlc run examples/polars_expressions
```

That source contains 90 executable regions across operations, selectors,
expansion, casting, strings, missing data, aggregation, windows, folds,
structs, and the UDF introduction. The remaining 45 tagged regions are
upstream Rust placeholders and are recorded separately in
`docs/EXPRESSIONS_GUIDE_COVERAGE.md`.

## Iris dataset audit

The Polars-only machine-learning data preparation experiment vendors the UCI
Iris dataset and verifies its shape, typed feature schema, balanced classes,
numeric feature projection, lazy species partition, bounded inspection, and
native resource lifecycle:

```bash
terlc run examples/iris_dataset_audit
```

The example intentionally stops at dataset preparation. DataFrames can now
export selected numeric columns through the VM-owned ndarray or PyTorch tensor
broker when a consumer needs the next stage. Dataset attribution and
normalization details are recorded in
`examples/iris_dataset_audit/DATASET.md`.

The native helper currently supports the `terlan-vm` target. Building a
consumer for JS, Wasm Core, Android, or iOS fails during package planning with
`error[package_native_target_unsupported]`; it does not fall through to a
backend emitter or attempt native execution.

## Dependency metadata

Local development uses a path dependency:

```toml
[dependencies]
terlan-polars = { path = "../terlan-polars" }
```

Published consumers will use an immutable Git commit:

```toml
[dependencies]
terlan-polars = { git = "https://github.com/terlan-lang/terlan-polars", rev = "<full-40-or-64-character-commit-id>" }
```

Terlan rejects tags, branch names, and abbreviated revisions in this field.
Resolve the immutable revision and generate `terlan.lock` explicitly before
building:

```bash
terlc package fetch .
terlc run .
```

Normal builds use only the verified package cache and never perform implicit
network access. Until the current package sources are committed and published,
repository development continues to use the checked-out path dependency.
