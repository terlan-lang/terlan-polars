# Polars API inventory and behavioral coverage

This document separates name-level inventory coverage from behavioral parity
with the official Python Polars 2.0 source revision and its Rust 0.55.1 crate
graph. The inventory gate proves that every identity has a native name match or
an explicit mapping. It does not prove that a differently shaped mapping
preserves behavior. Executable examples and family tests provide behavioral
evidence, and the Python cheat-sheet audit records intentional boundary
differences and optional ecosystem integrations outside the Rust inventory.

## Compatibility target

Terlan Polars may claim behavioral API parity only when every user-visible
public operation enabled by the pinned Polars feature profile has one of these
verified implementations:

- a direct typed Terlan operation with equivalent behavior;
- an idiomatic Terlan operation that combines or reshapes Rust overloads; or
- a safe Terlan composition that preserves the operation's observable result.

Different spelling, immutable ownership, and combined option records are
appropriate reshaping. Missing behavior is not. Unsafe Rust entry points must
map to safe validated Terlan behavior rather than becoming public unsafe
operations.

| Area | Current state | Completion gate |
| --- | --- | --- |
| Python Polars 1.43 cheat sheet | Core behavior complete | Every printed operation is classified by observable behavior in [POLARS_CHEATSHEET_COVERAGE.md](POLARS_CHEATSHEET_COVERAGE.md); optional formats and Python/service integrations are identified separately |
| Python Polars 2.0 migration | Complete | The adapter pins the official `py-2.0.0` source revision; Map, the replacement `bin_*` family, `datetime`/`repeat` naming, Struct arithmetic/casts, Decimal widening, deferred/exact SQL, Parquet ENUM decoding, streaming, concat, explode, membership, removed operations, hashes, and scan-backed CSV behavior have executable coverage in [POLARS_2_ALIGNMENT.md](POLARS_2_ALIGNMENT.md) |
| Documentation examples | Complete | All 90 executable Rust expression-guide regions and every getting-started DataFrame region execute |
| DataFrame | Inventory mapped for pinned profile | 165/165 upstream identities have native-name matches or explicit mappings |
| Series | Inventory mapped for pinned profile | 193/193 upstream identities have native-name matches or explicit mappings |
| LazyFrame | Inventory mapped for pinned profile | 115/115 upstream identities have native-name matches or explicit mappings |
| Expressions | Inventory mapped and cheat-sheet gaps covered | 251/251 identities are mapped; option-bearing casts, null filling, selectors, sorting and windows plus range constructors, `sign`, `rolling_map`, typed tree traversal/rewrite, `arg_true`, list construction, duration construction, and temporal/timezone operations have executable behavioral tests |
| Data types | Portable inventory mapped | 52/52 portable identities mapped; Rust/Arrow pointer and disabled-feature identities are explicitly scoped out |
| I/O | Enabled-format inventory mapped | 126/126 identities in the enabled CSV, Parquet, JSON/NDJSON, and IPC profile are mapped; CSV replacement names and bounded multi-file schema inference are exposed; cloud scans accept per-call storage options and streaming sinks support keyed partitions |
| Joins/concat | Implemented for enabled profile | Join coalescing, update semantics, strict/relaxed vertical concat, strict-by-default horizontal concat, explicit null-extending horizontal concat, and diagonal concat are exposed |
| Grouping/reshape | Implemented with safe callback reshaping | Ordinary, dynamic, and rolling groups, declarative materialized-group projections, upsampling, pivot, unpivot, option-bearing explode, unnest, partitioning, and empty-frame transpose are exposed |
| Extensibility | Safely reshaped | Serializable declarative expression UDFs replace process-local Rust callbacks and drive ordinary application plus per-window `rolling_map` and per-group projections |
| SQL | Inventory mapped for pinned profile | 17/17 SQL identities have explicit mappings across context lifecycle, queries, tables, and function registration |
| Interchange | Implemented by reshaping | Owned Arrow IPC bytes and VM-owned tensor packets replace process-local pointers |
| Execution | Implemented for enabled profile | Default and `auto` collection use streaming; explicit legacy in-memory, concurrent, and batched collection plus enabled source/sink behavior is mapped |

## Inventory gate

The canonical upstream inventory is extracted from rustdoc JSON using
`cargo-public-api` and the exact Cargo feature graph resolved by this package.
The facade's dependency globs are expanded through the `polars-core`,
`polars-lazy`, `polars-ops`, `polars-io`, `polars-time`, and `polars-sql`
component inventories.

Every inventory identity must have an exact generated operation name or an
explicit mapping to one. The gate fails on an unmapped identity, an operation
removed from generated native metadata, or an upstream feature-profile change.
It reports exact matches separately from explicit mappings so mapping coverage
cannot be presented as behavioral proof. Extraction details and the initial
counts are recorded in
[POLARS_UPSTREAM_API_BASELINE.md](POLARS_UPSTREAM_API_BASELINE.md).

Related executable evidence is grouped by the same family used by the strict
inventory. These suites exercise broad behavior, but they are not a one-to-one
proof for every explicit alias:

| Family | Inventory/mapping | Executable evidence |
| --- | --- | --- |
| DataFrame | `DATAFRAME_ALIASES`, 165 identities | `dataframe_extended_test`, native adapter tests, and `test/DataFrame*Test.terl` |
| Series | `SERIES_ALIASES`, 193 identities | `series_test` and `test/SeriesOperationsTest.terl`, `NullableSeriesTest.terl`, `WideIntegerSeriesTest.terl` |
| LazyFrame | `LAZY_FRAME_ALIASES`, 115 identities | `lazyframe_extended_test`, native adapter tests, and `test/LazyFrame*Test.terl` |
| Expr | `EXPR_ALIASES`, 251 identities | native expression tests and the executable expression namespace examples |
| DataType | direct portable inventory, 52 identities | `data_type_test`, `DataTypeTest.terl`, and `ExtensionDataTypeTest.terl` |
| I/O | `IO_MAPPINGS`, 126 identities | `io_options_test`, `IoOptionsTest.terl`, database tests, and I/O examples |
| SQL | `SQL_MAPPINGS`, 17 identities | native SQL tests and `SqlContextTest.terl` |

## Implemented compatibility slices

- Generic scalar predicates specialize `String`, `Int`, `Float`, and `Bool`
  arguments at compile time and preserve that concrete type across the native
  boundary. This keeps one public operation name without erasing scalar
  semantics into a runtime union.
- `DataFrame` additionally exposes column-free height and typed empty/all-null
  construction; first-column chunk lengths, maximum chunk count, rechunk
  readiness and zero-dimension inspection; immutable buffer shrinking;
  ordered-schema equality; checked column and row indexing; signed split
  variants; immutable Series insertion, replacement and named addition;
  row-to-Struct packing; and pinned Polars binary serialization.
- `Series` is a public helper-owned resource with extraction, typed
  string/integer/float/boolean construction, ordinary `Option`-based nullable
  construction, populated Date/Datetime construction, inspection, typed cast,
  empty typed construction, DataFrame conversion, immutable
  literal-expression conversion, ordinary and null-equal value comparison,
  and explicit release. The direct Series surface additionally covers
  immutable rename and clear operations; estimated size, chunk metadata and
  sorted-state inspection; head, tail, signed slicing and split variants;
  rechunking, storage shrinking, reversing and stable or unstable sorting;
  stable uniqueness, explode, null filling, strided gathering, seeded and
  unseeded shuffle/sampling, append and extend; dtype-preserving reductions;
  arithmetic, bitwise and value-wise comparison; and floating-point
  classification masks. Typed all-null construction, strict/non-strict/
  overflowing cast modes, immutable sorted-state mutation, physical and
  extension-storage conversion, implode and singleton-list packing, List and
  Array reshape, typed scalar/null extension, Boolean-mask zipping, and pinned
  binary serialization are also available. The 0.55 surface adds
  `to_unit_list` and checked reconstruction of Categorical/Enum values from
  physical category IDs. DataFrame comparison additionally
  requires matching column names and shape.
- Direct cheat-sheet conveniences expose typed DataFrame construction from
  named Series, ordered `dtypes`, unit-aware DataFrame and Series
  `estimated_size`, non-strict column removal, conjunction and named-constraint
  filtering, chained conditional expressions, DataFrame `gather_every`,
  component-based Date construction, and `log10`.
- Local CSV, Parquet, JSON, NDJSON, and IPC reads/writes are implemented;
  matching lazy scans and streaming sinks are available for CSV, Parquet,
  NDJSON, and IPC. The four lazy scan formats accept explicit per-call cloud
  option key/value lists, and their streaming sinks support keyed partitions
  with bounded file sizing. PostgreSQL and SQLite queries are ingested through
  ConnectorX and Arrow. HTTP, S3, Google Cloud Storage, and Azure paths are
  accepted by deferred cloud-capable scans.
- SQL contexts register and unregister named DataFrames, report tables in
  deterministic order, and compile SQL statements into ordinary `LazyFrame`
  plans.
- DataFrames round-trip through binary Arrow IPC. Numeric projections can be
  exported through the VM-owned tensor broker for ndarray and PyTorch
  consumers with explicit dtype and null policy.
- Declarative expression UDFs define bounded positional parameters, a required
  output data type, and an expression body. Application performs structural
  substitution and rejects malformed payloads, unresolved parameters, and
  arity mismatches before execution. The same descriptors execute against
  materialized rolling windows and safe per-group projections.
- Lazy plans support expression selection/mutation/filtering, including
  null-preserving inverse filtering, grouping,
  sorting, limits, typed subset-aware deduplication, null-row removal, bounded
  head/tail endpoint selection, schema-preserving clearing, row slicing,
  whole-frame shifting, schema resolution without row
  materialization, explicit cache nodes, generic joins, explode/unpivot/unnest,
  textual/tree/Graphviz plan descriptions, result-plus-timing execution
  profiling as a legacy compatibility shim, default streaming collection,
  explicit in-memory collection, an all-disabled optimizer baseline, and
  named control of ten pinned optimizer passes. Eager and lazy expression
  projection/mutation can use either parallel-default or sequential execution.
- Eager and lazy row-set operations expose stable or optimizer-friendly
  deduplication with first/last/any/remove-all retention, optional key subsets,
  subset-aware null-row removal, head/tail endpoints, schema-preserving row
  clearing, positive or negative-offset row slicing, and bounds-safe indexed
  gathering with optional null rows. Eager frames additionally expose seeded
  and unseeded fixed-count/fractional row sampling with replacement and output
  shuffle controls, validated numeric inputs, and a generated-output safety
  limit. Per-row identity inspection exposes uniqueness/duplication Boolean
  masks and default or seeded UInt64 hashes without materializing row values;
  empty frames are handled at the adapter boundary. Eager frames can also
  consolidate column storage into one chunk.
- Eager and lazy schema/layout operations expose simultaneous strict or
  non-strict renaming, validated column removal, row reversal, and unsigned
  row-index insertion with an explicit starting offset.
- Eager and lazy whole-frame shifts accept signed periods and either null-fill
  the resulting gap or fill it from an expression.
- Eager and lazy whole-frame reductions expose per-column sum, product, mean,
  median, minimum, maximum, variance, standard deviation, and configurable
  quantile results while retaining column names. They also distinguish
  non-null counts from total lengths and expose exact or approximate distinct
  cardinality.
- Eager and lazy missing-data operations expose expression-driven whole-frame
  null and NaN filling, optional-subset NaN-row removal, and one-row per-column
  null-count summaries.
- Relational operations cover independently named keys, suffix and null-match
  options, typed one-to-one/one-to-many/many-to-one validation, deterministic
  left/right row-order policies, inner/left/right/full/semi/anti/cross/as-of
  modes, and expression-driven non-equi joins, plus
  vertical concatenation, strict-by-default horizontal concatenation, explicit
  null-extending horizontal concatenation, and diagonal concatenation.
- Grouping and reshape cover ordinary, dynamic, and rolling groups; pivot;
  eager and lazy explode, unpivot, and unnest; plus eager transpose.
- Temporal expressions expose millennium/century/year/ISO-year, leap-year and
  month metadata, quarter/week/day/ordinal-day, local date/time/datetime
  projection, component-based Date/Datetime construction,
  hour/minute/second/subsecond components, month boundaries, epoch
  timestamps at every Polars time unit, duration-string truncation/rounding and
  calendar offsets, plus fractional or integral Duration totals from days
  through nanoseconds.
- String expressions expose byte/character lengths, prefix/suffix and regex
  predicates, extraction/replacement, casing and trimming, slicing, literal
  and regex search/counting, character-aware padding and signed zfill, all
  Unicode normalization forms, Unicode reversal, hexadecimal/Base64 codecs,
  and regex escaping.
  List expressions expose splitting, length, indexing, membership, sorting,
  numeric aggregation and statistics, first/last and arg-min/max, joining,
  slicing/head/tail, shifting, List-to-Array conversion, element
  evaluation/aggregation, configurable sort/get/null behavior, null removal,
  seeded and unseeded sampling, gathering, differences, match counting, set
  algebra, and named List-to-Struct conversion. This covers the complete
  public List expression namespace in the pinned Polars 2.0 source revision.
  Fixed-width Array expressions expose length,
  indexing, membership and match counts, sorting, numeric aggregation and
  statistics, List/Array-preserving slicing, head/tail extraction, shifting,
  exploding, String joining, element evaluation/aggregation, configurable
  sorting/indexing/null equality, and conversion to Lists or Structs. This
  covers the complete public Array expression namespace in pinned Polars
  the pinned Polars 2.0 source revision.
  Categorical/Enum expressions expose category values,
  byte/character lengths, prefix/suffix predicates, and slicing. Binary
  expressions expose containment, prefix/suffix predicates, byte sizes,
  indexing, head/tail/slice extraction, hex and Base64 codecs, and typed
  endian-explicit reinterpretation. This covers the complete public
  Categorical and Binary expression namespaces in the pinned Polars 2.0 source revision.
  Struct expressions cover named, positional, wildcard/regex-capable
  multi-field selection, field renaming, JSON encoding, and adding or replacing
  fields. This covers the complete public Struct expression namespace in the
  pinned release.
- Random expressions expose seeded and unseeded shuffle, fixed-count sampling,
  and fractional sampling with replacement and output-shuffle controls.
  Bitwise expressions expose one/zero counts, leading/trailing one/zero counts,
  and AND/OR/XOR reductions. These cover the complete public Random and Bitwise
  expression namespaces in the pinned Polars 2.0 source revision.
- Statistics expressions expose standard deviation, variance, minimum, maximum,
  mean, median, sum, min/max-by ordering, NaN-propagating extrema, and
  histograms using automatic, fixed-count, or explicit breakpoints. This
  covers the complete public Statistics expression namespace in pinned Polars
  the pinned Polars 2.0 source revision.
- Trigonometry expressions expose sine, cosine, tangent, cotangent, their
  inverse forms, the complete hyperbolic/inverse-hyperbolic family,
  quadrant-aware `arctan2`, and degree/radian conversion. This covers the
  complete public Trigonometry expression namespace in the pinned Polars 2.0 source revision.
- Boolean expressions expose null, finite/infinite and NaN masks;
  first/last/unique/duplicated masks; null-aware equality; logical operators;
  configurable interval, membership, and floating-point proximity predicates;
  horizontal disjunction; and any/all/empty/null reductions with explicit
  null behavior. Together with the existing negation and horizontal
  conjunction operations, this covers the complete public Boolean expression
  namespace in the pinned Polars 2.0 source revision.
- Cumulative expressions expose count, sum, product, minimum, and maximum with
  both forward and reverse traversal. This covers the complete public
  cumulative expression family in the pinned Polars 2.0 source revision.
- Advanced aggregation and window expressions expose cumulative evaluation,
  group-index aggregation, List-valued join mapping, and ordered windows with
  explicit partition, sort direction, null placement, and mapping behavior.
- Numeric transformations expose all three pinned rounding modes,
  significant-figure rounding, decimal truncation, floor/ceiling, absolute
  value, inclusive clipping, logarithms/exponentials, product aggregation,
  entropy, skewness, kurtosis, pi, and data-type bounds. Invalid mode, digit,
  decimal, and entropy-base arguments fail at the adapter boundary.
- Scalar aggregation and reduction expose non-null count versus total length,
  first/last non-null values, singleton validation, ordered List aggregation,
  all pinned quantile interpolation methods, ordered modes, unique values and
  first indices, extrema indices, dot products, and null/NaN removal.
- Generic sequence expressions expose expression-valued slicing, append and
  rechunk, head/tail, configurable direction/null sorting and argument-sort,
  index lookup and sorted insertion search, gather/get, reverse and shift/fill,
  repeat-by, discrete and percentage change, strided gather, and constant
  extension. Stable generic sorting uses an adapter-owned materialized-series
  path because pinned Polars can assert in its lazy ordering optimizer when
  `maintain_order` is enabled.
- Ranking and selection expose average/min/max/dense/ordinal tie methods,
  explicitly seeded random ties, direct top/bottom-k, and multi-key
  top/bottom-k with validated one-direction-per-key ordering.
- Replacement and binning expose ordinary, strict, and default-backed
  replacement plus Polars 2 `bin_intervals`, `bin_quantiles`, and `bin_ranks`.
  Expr and eager Series each support explicit or uniform specifications,
  UInt32 indices or caller labels, and optional interval Structs. Typed/count
  helper names cover non-floating breakpoint forms without native collection
  unions. Legacy `cut`/`qcut` calls remain available for source compatibility.
- Shape and sequence analysis expose coordinate-aware linear interpolation,
  local minimum/maximum masks, run-length Structs, zero-based run IDs, and
  validated fixed-width Array reshaping with one optional inferred dimension.
- Exponentially weighted expressions expose direct-alpha sum, mean, variance,
  and standard deviation with adjustment, bias, readiness, and null controls,
  plus coordinate-based sum and mean with validated positive constant
  half-life durations.
- Fixed-window rolling expressions expose minimum, maximum, mean, sum, median,
  variance, and standard deviation with configurable window size, minimum
  sample readiness, and centered or trailing labels. Quantile interpolation,
  seeded rank methods, skewness bias, and kurtosis Fisher/bias options are also
  exposed. Weighted minimum, maximum, mean, sum, median, variance, standard
  deviation, and quantile variants validate weight shape and finite values.
  Declarative UDFs cover expression-defined transformations. Query collection
  contains the pinned Polars nullable-weight panic path and returns a typed
  `native_execution_panic` error without terminating the native worker.
- Rolling-by expressions expose the complete pinned family of minimum, maximum,
  mean, sum, median, variance, standard deviation, quantile, and rank operations
  over duration or integer-index windows, with minimum-sample and closed-side
  configuration.
- Expression-name operations cover root-name retention, prefix/suffix,
  literal or regex replacement, lowercase/uppercase conversion, and Struct
  field prefix/suffix. Serializable `map_name` and `map_field_names` operations
  replace callback-shaped name transforms with prefix, suffix, literal
  replacement, lowercase, and uppercase modes.
- Expression metadata exposes root and output names, multi-output, column,
  projection, literal, and regex-expansion classification, plus text and
  Graphviz DOT formatting. Typed traversal returns the full root-first node
  sequence or immediate child expressions. Unary declarative UDFs can rewrite
  the full tree bottom-up or only its immediate children, with bounded encoded
  output. Constant integer extraction is checked, and `to_field` resolves a
  portable named field against explicit schema name/type lists.
- Option-bearing expression operations preserve the behavior that a simple
  alias cannot represent: strict/non-strict/overflowing casts, every Polars
  null-fill strategy and directional limit, checked selector conversion,
  per-key sort direction and null placement with stable-order control, ordered
  window mapping, sorted-state flags, true division, and physical conversion.
- `DataType` is a public scalar union used by expression casts, dtype column
  selectors, Series casts, and empty Series construction; the legacy
  string-taking operations remain available for compatibility. It covers
  the package-supported signed/unsigned integer widths and floats, strings,
  binary storage, nulls, Date/Time, all Datetime resolutions, and all Duration
  resolutions. Opaque typed descriptors add recursively composable List,
  fixed-width Array, Decimal, and named-field Struct data types to those same
  expression and Series operations. They also cover dynamically populated
  Categorical values, ordered fixed-category Enum values, and validated
  timezone-aware Datetime values at every supported time resolution.
  Extension descriptors preserve a validated extension name, optional
  metadata, and recursively typed storage. The
  scalar union includes signed and unsigned 128-bit integers and 16-bit,
  32-bit, and 64-bit floating-point storage for typed casts and schemas.
  Populated Int128 and UInt128 Series use validated decimal strings so values
  cross the current Int64 VM scalar boundary without truncation. Generic
  `Option[T]` construction is specialized by the compiler and nullable
  primitive Series execute end to end.

## Intentional reshaping

- The pinned Cargo profile does not enable Polars `object`; Object storage is
  therefore not part of this compatibility denominator. `is_object` and
  `contains_objects` still provide stable false-valued inspection.
- Rust callbacks are process-local code pointers. Terlan uses serializable,
  typed expression UDF descriptors with structural substitution instead.
- Rust expression namespace accessors (`arr`, `binary`, `cat`, `dt`, `ext`,
  `list`, `meta`, `name`, `str`, and `struct_`) have no observable result by
  themselves. Their methods are flattened into typed Terlan operations. Rust
  operator-trait spellings delegate to the corresponding named operations.
  Callback constructors delegate to declarative `ExprUdf` application, while
  tree visitor and rewriter traits use bounded typed traversal, `map_children`,
  and bottom-up `rewrite`. These are the remaining explicit expression
  inventory mappings after direct option-bearing operations are accounted for.
- Rust Arrow arrays and fields are process-local ownership objects. The package
  uses owned Arrow IPC bytes and typed tensor packets instead.
- Mutable builders and batched writers are represented by immutable validated
  option descriptors followed by one terminal read or write operation.
- Unsafe and unchecked Rust methods map to validated immutable operations; no
  unsafe package entry point is exposed.
- Static scatter, line, bar, histogram, and box plots render as bounded SVG by
  default through Plotlars/Plotters. Interactive Plotly HTML and PNG rendering
  are optional Cargo features. Declarative table styling escapes data and
  headers and enforces bounded HTML output.
