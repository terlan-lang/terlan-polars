# Polars 2.0 alignment

`terlan-polars` targets the stable Python Polars 2.0 API model. The native
adapter pins the official `py-2.0.0` source revision
`22a147de3d2bb2e44b97338a2510816c7105c9f2`. That source identifies its Rust
crates as 0.55.1; the earlier crates.io 0.55.2 pin did not contain the complete
Python 2.0 engine behavior.

The same revision is used by the adapter and all vendored Plotlars crates, so
Polars values never cross between duplicate crate versions. `dtype-map` is part
of the resolved feature profile. The source revision also requires Polars'
hashbrown prefetch fork. A vendored `polars-utils` copy replaces the unstable
`isolate_lowest_one` call with the equivalent stable expression
`y & y.wrapping_neg()`, allowing the package to keep using the stable Rust
compiler.

## Implemented behavior

The adapter now executes the Python 2.0 engine behavior for:

- streaming as the default lazy engine, with a contained fallback for plans the
  Rust streaming engine cannot execute;
- scan-backed CSV/IPC behavior represented by Terlan's typed readers and scans;
- strict horizontal concatenation and explicit null-extending concatenation;
- `explode(empty_as_null=false)` defaults;
- exact `Int128` supertypes for signed integers combined with `UInt64`;
- strict membership coercion and temporal/Enum lookup rules;
- leftmost-argument output names for `datetime` and `repeat`;
- outer-null preservation for List/Array `to_struct`;
- empty DataFrame transpose and height-preserving zero-width frames;
- the single-seed hash API and the new CSV inference/name rules;
- removed casts and operations, including String-to-temporal casts,
  integer/categorical casts, scalar-to-List casts, mixed Boolean/integer bitwise
  operations, Duration variance/deviation, flat `list.gather` indices,
  mismatched `struct.rename_fields`, and nonnumeric logarithms/exponentials;
- the `Map` dtype plus key/value inspection, `list_to_map`, and Map
  entries/keys/values/len/contains/get expressions;
- per-field Struct arithmetic supertypes and rejection of nonnumeric fields;
- widened Decimal precision for rounding, sign, and sums;
- deferred SQL resolution, exact SQL Decimal literals, and the 2.0 SQL numeric
  rules;
- Parquet `ENUM` logical and converted types decoding as String; and
- `bin_intervals`, `bin_quantiles`, and `bin_ranks`, including explicit and
  uniform specifications, labels or indices, interval structs, closure
  direction, primitive or arbitrary orderable Series breakpoints, and
  tied-rank behavior, on both expressions and eager Series. Canonical methods
  accept floating breakpoints or fractions; concrete `_count`, `_ints`,
  `_strings`, `_bools`, and `_series` helpers cover the remaining input forms
  without relying on compiler-native collection unions.

Executable regression tests in `native/src/polars_2_semantics_test.rs` cover all
of the engine changes above, including a generated Parquet ENUM footer fixture.

## Complete upgrade-guide classification

| Polars 2.0 change | Terlan status |
| --- | --- |
| Streaming lazy default | Implemented by `collect` and the `auto` engine |
| `read_csv`/`read_ipc` scan-backed wrappers | Reshaped into typed eager readers and lazy scans; unsafe IPC memory mapping is rejected |
| Physical `show_graph` default | Separate logical, optimized, tree, and Graphviz plan operations avoid an ambiguous stage default |
| Remove `LazyFrame.profile` | Retained as a compatibility shim with optimization/query elapsed intervals |
| Strict horizontal concat | Implemented; null extension is a separate operation |
| `explode(empty_as_null=false)` | Implemented |
| Signed integer plus `UInt64` uses `Int128` | Inherited from the 2.0 engine and tested for Series and expressions |
| Strict `is_in` coercion | Implemented for `is_in`, `list.contains`, and `arr.contains` |
| Selector/column operator distinction | Inherent in distinct `Expr` and `Selector` APIs |
| `datetime`/`repeat` use the leftmost input name | Implemented and tested for a named year expression and a literal repeat value |
| Preserve outer nulls in `to_struct` | Implemented for List and Array expressions |
| Preserve file-object seek position | Not applicable: scans accept paths and URIs |
| Preserve zero-width frame height | Implemented for eager and lazy frame operations |
| Arrow stream imports always return Series | Reshaped into typed Series/DataFrame Arrow IPC entry points; return type is never inferred from a Python protocol object |
| Remove Python DataFrame interchange protocol | Owned Arrow IPC bytes and VM-owned tensor packets are used |
| Empty transpose | Implemented |
| `shift(n=None)` raises | Inherent: shift counts have type `Int`/`Expr`, so `None` is not admitted |
| Unknown Arrow extensions | Extension name, metadata, and storage are preserved |
| New `Map` dtype and namespace | Implemented |
| Struct arithmetic per-field supertypes | Inherited from the pinned 2.0 engine and tested |
| Decimal precision widening | Inherited from the pinned 2.0 engine and tested |
| Deferred SQL query resolution | Inherited from the pinned 2.0 SQL engine and tested |
| SQL Decimal result scales for `*` and `/` | Inherited from the pinned 2.0 SQL engine |
| Exact SQL numeric literals and truncating `%`/`DIV` | Inherited from the pinned 2.0 SQL engine; exact literal typing is tested |
| Parquet `ENUM` loads as String | Inherited from the pinned 2.0 Parquet reader and tested |
| Integer/Categorical casts removed | Enforced by strict cast validation and tested |
| String-to-temporal casts removed | Enforced by strict cast validation and tested |
| Flat-to-List casts removed | Enforced by strict cast validation and tested |
| Boolean/integer bitwise operations removed | Enforced for eager Series and expressions and tested |
| Duration `std`/`var` removed | Inherited from the 2.0 engine and tested |
| Invalid strict Struct casts removed | Enforced before execution and tested |
| Flat dtypes where List is required | Rejected by the pinned engine and tested with `list.gather` |
| Membership converts temporal units | Inherited from the 2.0 engine and tested across nanoseconds/microseconds |
| Decimal/float membership removed | Enforced before execution and tested |
| Naive/aware datetime membership removed | Enforced before execution and tested |
| Unknown Enum membership labels are absent | Inherited from the 2.0 engine and tested |
| Nonnumeric `log`/`log1p`/`exp` removed | Enforced before execution and tested |
| Required List-to-Struct fields | Inherent: Terlan requires explicit field names |
| Keyword-only Python arguments | Not applicable to named, typed Terlan functions |
| Single-seed hash | Implemented |
| Python-list `Series.search_sorted` input removed | Inherent: Terlan accepts an explicit expression, so scalar List literals and Series-like targets are unambiguous |
| Exact `struct.rename_fields` width | Implemented |
| Removed Categorical ordering parameter | Inherent: Terlan has no ordering parameter |
| CSV `infer_schema_files=10` default | Implemented in descriptors and tested |
| Headerless CSV names start at `column_0` | Inherited from the 2.0 scan reader and tested |
| CSV schema matches by name and respects file columns | Implemented by routing descriptor reads through `LazyCsvReader.collect`; tested with reversed schema order |
| CSV extra/missing column policies | Implemented as immutable `with_column_policies` options; `ignore`/`insert`, default raising, and automatic ragged-line truncation are tested |
| Headerless empty CSV with schema | Explicit in Terlan: callers pass `raise_if_empty=false`; the empty typed frame behavior is tested |
| Removed deprecated Python APIs and arguments | The typed Terlan surface exposes their 2.0 replacements; intentional source-compatibility shims are listed below |
| Avro marked unstable | Not exposed by `terlan-polars` |
| Redundant strict horizontal concat deprecated | Inherent: horizontal concat has no redundant `strict` parameter |
| `cut`/`qcut` deprecated for `bin_*` | All three replacement families are implemented and tested for Expr and Series, including compiler-safe typed/count helpers; legacy calls remain compatibility shims |

## Compatibility shims

Existing Terlan programs can still call `profile`, `profile_plan`,
`with_context`, the Boolean form of `collect_all`, reader `rechunk` fields, and
Series `as_list`. `with_context` is implemented with the 2.0 horizontal lazy
concatenation model. Expression `rechunk` preserves the expression and callers
can rechunk the collected DataFrame. The removed `agg_groups` expression
returns a typed construction error directing callers to add a row index before
aggregation.

Expression sorting retains the portable `maintain_order` descriptor field, but
compiles through the supported 2.0 expression sort API, which removed that
option. Equal values are indistinguishable in a single sorted expression;
multi-column stable ordering remains available through `sort_by`. `update`
explicitly requests left input order, matching the Python 2.0 default despite
the new streaming engine's unordered ordinary joins. Expression `top_k` and
`bottom_k` expose the selected values without promising an order that upstream
does not guarantee.

## Inventory boundary

The 919-identity name inventory remains the last published Rust release
baseline, Polars 0.55.2. It provides a stable review denominator for the Rust
surface. The source-pinned Python 2.0 delta is covered by the behavioral tests
and public Map operations described above. This distinction avoids presenting
the Python release tag's internal Rust crate version (0.55.1) as an older API
model.

## References

- [Python Polars 2.0 release](https://github.com/pola-rs/polars/releases/tag/py-2.0.0)
- [Polars 2.0 upgrade guide](https://docs.pola.rs/releases/upgrade/2/)
- [Pinned Polars source revision](https://github.com/pola-rs/polars/commit/22a147de3d2bb2e44b97338a2510816c7105c9f2)
- [Rust Polars 0.55.2 release baseline](https://github.com/pola-rs/polars/releases/tag/rs-0.55.2)
