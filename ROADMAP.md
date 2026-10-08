# terlan-polars Implementation Roadmap

`terlan-polars` is the external Polars DataFrame package for Terlan. Its first
job is to validate Terlan's third-party package model: a user should be able to
add a dependency, import `polars.DataFrame`, typecheck source, and receive clear
target diagnostics without this package living in the compiler or standard
library.

The Polars adapter is the motivating payload, but packaging is the concept
being proven first. Real DataFrame execution comes after the package can behave
like an ordinary external module.

## Target Shape

Package identity:

- GitHub repository: `terlan-lang/terlan-polars`
- Terlan package name: `terlan-polars`
- Terlan namespace: `polars`
- Rust adapter crate: `terlan_polars_native`

Expected repository layout:

```text
terlan-polars/
  README.md
  ROADMAP.md
  LICENSE
  terlan.toml
  src/
    polars/
      DataFrame.terl
  bindings/
    polars.mapping.toml
  native/
    Cargo.toml
    src/
      lib.rs
      bridge.rs
  test/
    fixtures/
      people.csv
    read_csv_test.terl
    select_test.terl
  examples/
    consumer_project/
      terlan.toml
      src/
        Main.terl
```

Public Terlan imports should stay short:

```terlan
import polars.DataFrame.{read_csv}.

read_csv("people.csv").
```

## Bootstrap API

The package bootstrap exposed one opaque DataFrame type and a narrow set of
operations:

```terlan
module polars.DataFrame.

pub opaque type DataFrame.

pub read_csv(path: String): Result[DataFrame, Error].
pub from_rows(columns: List[String], rows: List[List[String]]): Result[DataFrame, Error].

pub (df: DataFrame) height(): Int.
pub (df: DataFrame) width(): Int.
pub (df: DataFrame) columns(): List[String].
pub (df: DataFrame) schema(): List[ColumnSchema].
pub (df: DataFrame) rows(limit: Int): Result[List[List[String]], Error].
pub (df: DataFrame) filter_eq[T](column: String, value: T): Result[DataFrame, Error].
pub (df: DataFrame) sort_by(column: String, descending: Bool): Result[DataFrame, Error].
pub (df: DataFrame) group_count(keys: List[String]): Result[DataFrame, Error].
pub (df: DataFrame) lazy(): LazyFrame.
pub (plan: LazyFrame) where_eq[T](column: String, value: T): LazyFrame.
pub (plan: LazyFrame) project(columns: List[String]): LazyFrame.
pub (plan: LazyFrame) collect(): Result[DataFrame, Error].
pub (plan: LazyFrame) release(): Unit.
pub (df: DataFrame) select(columns: List[String]): Result[DataFrame, Error].
pub (df: DataFrame) dispose(): Unit.
```

This deliberately small bootstrap API validated package resolution, native
adapter linkage, opaque resource ownership, stable error conversion, receiver
methods, and basic DataFrame behavior. It is not the final API scope; POL3.0
requires full behavioral parity with the pinned Polars feature profile.

## Packaging Validation Goals

Before real Polars execution is required, this package must prove the external
module path:

- A separate consumer project can declare `terlan-polars` as a dependency.
- The compiler resolves package metadata from outside the Terlan repository.
- Imports use `polars.DataFrame`, with no `std.native` exposure to consumers.
- Package source typechecks independently from core `std`.
- Native adapter requirements are declared as target capabilities, not hidden
  compiler assumptions.
- Unsupported targets fail during package validation with stable diagnostics.
- Consumers can depend on this package by local path and Git URL. A future
  Terlan registry/static index may improve discovery, but Hex is not the
  package model.

## Implementation Gates

### POL0.0 - Third-Party Package Contract

Status: Complete

Goal: define the minimum contract a Terlan third-party module must satisfy,
using `terlan-polars` as the first concrete package.

Tasks:

- Specify which fields in `terlan.toml` are required for external package
  identity, namespace, source roots, native capabilities, and version
  constraints.
- Define the consumer dependency shape for local path and Git dependencies.
- Define the lockfile pinning model for Git dependencies.
- Document how package namespaces avoid collisions with `std` and user modules.
- Document the diagnostic shape for unsupported native adapter targets.

Checks:

```bash
make -C ../terlan terlan-polars-package-check
```

The contract is enforced by ordinary `terlc check` package loading, explicit
`terlc package fetch` lockfile generation, and the compiler-owned package
quality gate. A separate `terlc package validate` command is not required.

Exit criteria:

- The package contract is documented in this repository.
- Validation can run without compiling real Polars.
- Failures distinguish package metadata errors from source type errors and
  native target capability errors.

### POL0.1 - Package Scaffold

Status: Complete

Goal: create the minimal third-party package skeleton with no real Polars
dependency.

Tasks:

- Add `terlan.toml` for Terlan package metadata and source roots.
- Add `src/polars/DataFrame.terl` with the public API and stub behavior.
- Add `bindings/polars.mapping.toml` using the public `polars` namespace.
- Add `native/Cargo.toml` and `native/src/lib.rs` with stub Rust functions.
- Add `test/DataFrameReadCsvTest.terl` for import/typecheck coverage.
- Add `examples/consumer_project` as a separate package that depends on this
  repository by local path.

Checks:

```bash
terlc check src/polars/DataFrame.terl
terlc --cache-dir _build/.terlan check test/DataFrameReadCsvTest.terl
cargo test --manifest-path native/Cargo.toml
```

Exit criteria:

- The repository has all required package files.
- The package validates as an external dependency, not a core source tree.
- The Terlan API typechecks with the current compiler.
- The Rust adapter compiles and returns stable unavailable-native errors.
- No real `polars` crate dependency is linked yet.

### POL0.2 - External Namespace Alignment

Status: Complete

Goal: ensure all generated and hand-written package surfaces use
`polars.DataFrame` instead of a core-standard-library namespace.

Tasks:

- Port the scratch package skeleton into this repository with namespace updates.
- Update mapping metadata from the old scratch namespace to `polars`.
- Keep native machinery private to package internals.
- Verify compiler diagnostics remain target-neutral when native adapters are
  unsupported.

Checks:

```bash
terlc check src/polars/DataFrame.terl
rg "old scratch namespace" .
```

Exit criteria:

- User-facing Terlan source and docs mention only `polars`.
- Any remaining old scratch namespace references are historical notes or
  removed.
- Binding metadata maps public Terlan names to private Rust adapter functions.

### POL0.3 - Package Typecheck Tests

Status: Complete

Goal: prove package-local tests can import and typecheck the first API.

Tasks:

- Add `test/DataFrameReadCsvTest.terl`.
- Add `test/DataFrameHeadTest.terl`.
- Add `test/DataFrameSelectTest.terl`.
- Cover `read_csv`, `height`, `width`, `columns`, and `select` at source/type
  level.
- Add small CSV fixtures for later runtime tests.

Checks:

```bash
terlc check src/polars/DataFrame.terl
terlc --cache-dir _build/.terlan check test/DataFrameReadCsvTest.terl
terlc --cache-dir _build/.terlan check test/DataFrameHeadTest.terl
terlc --cache-dir _build/.terlan check test/DataFrameSelectTest.terl
```

Exit criteria:

- Tests import `polars.DataFrame`.
- Tests exercise the intended API shape.
- Tests do not require real Polars execution yet.

### POL0.4 - Consumer Project Proof

Status: Complete

Goal: prove `terlan-polars` functions as Terlan's first third-party module from
outside the package repository.

Tasks:

- Add `examples/consumer_project/terlan.toml` with a local path dependency on
  `terlan-polars`.
- Add `examples/consumer_project/src/Main.terl` that imports
  `polars.DataFrame`.
- Typecheck the consumer without adding package files to core `std` or compiler
  fixtures.
- Verify the compiler reports a target-capability diagnostic if the selected
  target cannot load Rust native adapters.
- Verify source-only operations can still typecheck when native execution is
  unavailable.

Checks:

```bash
terlc run examples/consumer_project
terlc build examples/consumer_project --target js.shared
```

The second command is an intentional failure check. It must report
`error[package_native_target_unsupported]`, name the `terlan-polars` local
dependency and `native-process-helper` capability, and identify `terlan-vm` as
the supported target before backend emission.

Exit criteria:

- The consumer resolves `terlan-polars` through dependency metadata.
- `polars.DataFrame` is importable from outside this repository.
- The compiler treats this package as third-party code.
- Native adapter unavailability is reported as a package capability issue, not
  as a missing core module.

### POL0.5 - Rust Adapter Stub

Status: Complete

Goal: lock the Rust adapter boundary before linking the real Polars crate.

Required Rust API:

```rust
pub struct TerlanPolarsDataFrame;
pub struct TerlanPolarsError;

pub fn read_csv(path: &str) -> Result<TerlanPolarsDataFrame, TerlanPolarsError>;
pub fn height(df: &TerlanPolarsDataFrame) -> usize;
pub fn width(df: &TerlanPolarsDataFrame) -> usize;
pub fn columns(df: &TerlanPolarsDataFrame) -> Vec<String>;
pub fn select(
    df: &TerlanPolarsDataFrame,
    columns: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError>;
```

Tasks:

- Add stable error fields for code and message.
- Add unit tests for unavailable-native behavior.
- Add stub observer tests for `height`, `width`, and `columns`.
- Add a bridge module that models handle allocation, disposal, stale-handle
  rejection, request ids, and credit reporting.

Checks:

```bash
cargo test --manifest-path native/Cargo.toml
cargo clippy --manifest-path native/Cargo.toml --all-targets -- -D warnings
```

Exit criteria:

- The adapter boundary is documented and tested.
- Stale-handle behavior is represented before real DataFrames exist.
- Clippy passes with warnings denied.

### POL1.0 - Real Polars Linkage

Status: Complete

Goal: link the Rust `polars` crate inside the package adapter and execute the
first DataFrame operations.

Tasks:

- Add a gated `real-polars` feature in `native/Cargo.toml`.
- Add the Polars dependency with `lazy`, `csv`, and `strings` features.
- Implement `read_csv` against a committed CSV fixture.
- Implement `height`, `width`, `columns`, typed `schema`, scalar equality
  filtering, and `select` against real Polars
  DataFrames.
- Convert Polars failures into stable `TerlanPolarsError` values.

Checks:

```bash
cargo test --manifest-path native/Cargo.toml --features real-polars
terlc run examples/loaded_helper_project
```

Exit criteria:

- `read_csv` reads a real fixture.
- `height` and `width` return real dimensions.
- `columns` returns real column names in order.
- `schema` returns ordered `ColumnSchema` records with stable dtype names.
- `filter_eq` preserves String, Int, Float, and Bool literal types and reports
  missing-column or incompatible-type failures through `Result`.
- `sort_by` orders by one column in ascending or descending order and reports
  missing columns through `Result`.
- `group_count` performs stable grouping over one or more keys and materializes
  a `count` column; empty and missing keys fail explicitly.
- `LazyFrame` supports deferred equality filters and projections, explicit
  collection, deferred schema errors, and deterministic plan disposal.
- `from_rows` constructs string-valued and empty DataFrames directly and
  rejects ragged input with `invalid_row_width`.
- `rows(limit)` materializes bounded display values, preserves null
  observability, and rejects negative or excessive limits.
- `select` returns a projected DataFrame.
- Polars errors expose stable Terlan error codes and messages.

### POL1.1 - NativeBoundary Resource Lifecycle

Status: Complete

Goal: execute real operations through NativeBoundary resource handles rather
than direct Rust values.

Tasks:

- Store DataFrames behind owned resource handles.
- Reject stale or forged handles.
- Add explicit release/dispose behavior.
- Route blocking CSV and selection work through the worker path.
- Preserve request correlation and credit behavior in replies.

Checks:

```bash
cargo test --manifest-path native/Cargo.toml --features real-polars
terlc run examples/loaded_helper_project
```

Exit criteria:

- Native resources have deterministic ownership and cleanup.
- Handle lifecycle tests cover live, disposed, and stale paths.
- Scheduler-sensitive calls do not do long-running work inline.

The production helper now owns real DataFrames behind id/generation/type
handles, exposes explicit disposal, rejects reuse, duplicate disposal, forged
generations, and forged types, and releases all remaining resources when the
helper exits. Each synchronous subprocess request carries a request id and each
reply echoes that id with one available credit, keeping blocking CSV and
selection work outside the VM process.

### POL1.2 - Compiler Package Integration

Status: Complete

Goal: make the package usable from a Terlan project through normal dependency
metadata.

Tasks:

- Validate `terlan.toml` dependency metadata.
- Ensure package source roots are discovered.
- Ensure package summaries can be generated or consumed when compiler support
  lands.
- Validate target capability checks for Rust native adapters.
- Promote the consumer-project proof from local path dependency to the same
  resolution path used for published packages.

Checks:

```bash
make -C ../terlan terlan-polars-package-check
```

Exit criteria:

- A consumer can depend on `terlan-polars`.
- The `polars` namespace is available to source imports.
- Unsupported targets fail with stable diagnostics before runtime execution.
- The package model is reusable for a second third-party module without
  Polars-specific compiler special cases.

Local and locked Git dependency discovery, cached interface consumption, VM
execution, and generic package-native target validation are complete. Git
resolution is explicit through `terlc package fetch`; subsequent builds verify
the lockfile, origin, revision, clean checkout, and tree checksum without
network access.

### POL1.3 - Git Package Publish Preflight

Status: Release pending

Goal: make the package consumable as a Git-first Terlan package.

Tasks:

- Include license, repository, links, compiler constraints, and native adapter
- requirements in `terlan.toml`.
- Document installation and first usage in `README.md`.
- Add release preflight commands.
- Verify local path and Git dependency declarations.
- Verify the generated lockfile pins the exact Git commit.

Checks:

```bash
terlc check src/polars/DataFrame.terl
cargo test --manifest-path native/Cargo.toml
make -C ../terlan terlan-polars-package-check
```

Exit criteria:

- Terlan package metadata validates.
- Local path and Git dependency metadata validate.
- Terlan source checks pass.
- Native adapter tests pass.
- README explains experimental status, requirements, and first API.

Publication metadata is now accepted directly in `terlan.toml`, including
description, license, repository, compiler constraint, and project links.
Dependency manifests require a full 40- or 64-character hexadecimal Git commit
and reject branches, tags, and abbreviated revisions. The compiler now checks
out immutable Git sources explicitly, generates `terlan.lock`, and resolves
offline builds from a provenance- and checksum-verified cache. This milestone
remains release-pending until the current `terlan-polars` sources are
committed, published, and exercised by the acceptance consumer at that
published commit.

### POL2.0 - Runtime Smoke Tests

Status: Complete

Goal: run an end-to-end Terlan program that reads a CSV through
`terlan-polars`.

Tasks:

- Add the package acceptance project under `examples/loaded_helper_project`.
- Add `examples/iris_dataset_audit` as a Polars-only ML data preparation
  experiment using the attributed UCI Iris dataset.
- Run a Terlan program that imports `polars.DataFrame`.
- Exercise `read_csv`, `height`, typed `schema`, scalar filtering,
  single-column sorting, `select`, and `head` through the real helper
  together with grouped row-count aggregation
  and lazy query execution
  handles; keep `width` and `columns` covered by the real adapter/helper tests.
- Capture expected output or test assertions.

Checks:

```bash
terlc run examples/loaded_helper_project
terlc run examples/iris_dataset_audit
```

Exit criteria:

- The example runs without manual native setup beyond documented prerequisites.
- The Iris audit proves 150 rows, four numeric features, three balanced species
  classes, eager feature projection, and lazy class partitioning.
- Failures report stable Terlan errors, not Rust panics or transport details.

### POL2.1 - Expression Contexts and Getting-Started Parity

Status: Complete

Goal: port every DataFrame example from the official Polars getting-started
guide through a reusable Terlan expression API.

Completed work:

- Added immutable opaque `Expr` values backed by private serialized syntax,
  without native handles or lifecycle calls.
- Added column and expansion selectors, scalar/date literals, arithmetic,
  aliases and name transforms, date extraction, comparison/range predicates,
  string splitting, and aggregation expressions.
- Added expression contexts for `select`, `with_columns`, `filter`, and stable
  grouped aggregation.
- Added CSV writing/date inference, left joins, and vertical concatenation.
- Added `examples/polars_getting_started` with the guide's exact fixture data
  and executable assertions for all examples.
- Expanded generated NativeBoundary metadata and the compiler package gate to
  all 49 public operations.

Checks:

```bash
terlc run examples/polars_getting_started
make -C ../terlan terlan-polars-package-check
```

### POL3.0 - Full Polars API Parity

Status: Complete

Goal: provide behavioral parity with every user-visible public API enabled by
the pinned Polars 0.55.2 feature profile. Terlan may combine Rust overloads,
replace mutation with immutable results, and represent options with typed
values, but it may not omit observable behavior.

Tasks:

- [x] Extract the feature-resolved upstream API through rustdoc JSON and
  `cargo-public-api`.
- [x] Expand facade dependency globs through the `polars-core`, `polars-lazy`,
  `polars-ops`, `polars-io`, `polars-time`, and `polars-sql` inventories.
- [x] Map every identity to a direct Terlan operation, an explicitly reshaped
  equivalent, or `pending`.
- [x] Expand the direct Series compatibility surface to 124 generated
  operations covering metadata, shape, storage, ordering, sampling,
  reductions, arithmetic, comparison, floating-point masks, typed null
  construction, cast modes, reshaping, conditional selection, and binary
  serialization.
- [x] Expand the direct DataFrame compatibility surface to 125 generated
  operations, including schema/null construction, chunk and shape metadata,
  checked indexing, immutable column mutation, Struct packing, and pinned
  binary serialization.
- [x] Commit executable high-level DataFrame, Series, and LazyFrame API
  denominators for pinned Polars 0.55.2.
- [x] Add compiler specialization and native layouts for generic constructors
  so direct AOT `Option[T]` values can reach nullable package operations.
- [x] Require executable evidence for every direct or reshaped mapping.
- [x] Implement all pending DataFrame, Series, LazyFrame, expression, data type,
  I/O, SQL, interchange, callback-composition, and execution behavior.
- [x] Document every public Rust item and keep strict `missing_docs` rustdoc
  green.
- [x] Fail the package gate on unmapped upstream identities or feature-profile
  drift.

Checks:

```bash
make -C ../terlan terlan-polars-package-check
RUSTDOCFLAGS='--document-private-items -D missing_docs' \
  cargo doc --no-deps --manifest-path native/Cargo.toml --features real-polars
```

Exit criteria:

- No upstream identity is `pending`.
- Every reshaping records why the Terlan API preserves behavior.
- Every mapping names permanent native and Terlan test evidence.
- The parity inventory regenerates deterministically for Polars 0.55.2.
- Strict Rust documentation completes without missing-item diagnostics.

### POL4.0 - Numerical Stack Convergence

Status: Complete

Goal: make Polars the tabular producer in one directional numerical pipeline:

```text
                  +-> ndarray -> transformed packet -+
Polars -> TNXP ---+                                +-> PyTorch
                  +--------------------------------+
                  +-> CUDA when explicitly selected
```

Tasks:

- [x] Export selected primitive numeric columns as a checked pointer-free TNXP
  packet with explicit Float64 casting and null rejection.
- [x] Preserve requested column order and emit exact `[rows, columns]`
  row-major shape and strides.
- [x] Rechunk selected cast columns explicitly before encoding while leaving
  the source DataFrame unchanged; reject temporal, nested, and other
  non-primitive physical layouts through a closed dtype whitelist.
- [x] Export a primitive numeric Series through the same one-column encoder.
- [x] Keep `terlan-polars` independent of PyTorch and CUDA.
- [x] Expose `DataFrame.to_pytorch_packet` for direct packet consumption while
  keeping tensor construction in `terlan-pytorch`.
- [x] Validate all four Iris feature columns as an exact `[150, 4]` packet with
  known endpoint values and bounded contents.
- [x] Execute the PyTorch-targeted Iris packet directly through
  `Tensor.from_packet`, verify shape `[150, 4]`, and reduce to `7.9` without an
  ndarray relay.
- [x] Execute the Iris packet through `terlan-ndarray`, transpose the owned
  array from `[150, 4]` to `[4, 150]`, export a new PyTorch-targeted packet,
  and reduce the resulting tensor to `7.9` with independent cleanup.
- [x] Execute the Iris packet through an immutable `terlan-ndarray` consumer.
- [x] Execute the resulting array through an immutable `terlan-pytorch`
  consumer and verify one deterministic reduction or model result.
- [x] Publish immutable revisions and run the converged pipeline without
  sibling checkout access.

Arrow C Data and CUDA IPC are not baseline requirements. They may be admitted
later only when equivalent benchmarks prove that the owned TNXP copy is a
material bottleneck and maintained wrappers satisfy the ownership contract.

The immutable gate snapshots all three packages at exact Git revisions,
fetches them through the compiler package resolver, deletes the source
repositories, and executes the transformed Iris test solely from the consumer
package cache with Cargo network access disabled.

Gates:

```bash
make polars-pytorch-interop-check
make polars-ndarray-pytorch-interop-check
make polars-immutable-ml-interop-check
```

`polars-ndarray-interop-check` remains as a compatibility alias for the
ndarray-to-PyTorch gate.

## Non-Goals

- Do not make Polars a core standard-library module.
- Do not expose a core-standard-library namespace as the public package
  namespace.
- Do not link Polars into the Terlan compiler.
- Do not special-case `terlan-polars` in dependency resolution; it is the first
  proof of a general third-party module mechanism.
- Do not build a general Rust crate binding generator before this curated
  package works.

## Open Decisions

- Exact compiler version constraint for the first Git-distributed package.
- Whether package summaries are generated during publish or checked in after
  review.
- Final native adapter build UX for consumers without a Rust toolchain.
- Whether long-running operations use a dedicated worker per package instance
  or a shared NativeBoundary worker pool.
