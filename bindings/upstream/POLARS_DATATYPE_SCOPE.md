# Polars DataType parity scope

The inventories in this directory are derived from the public `DataType`
implementations in `polars-core` 0.55.2 under the feature set pinned by
`native/Cargo.toml`.

`polars-datatype-methods-0.55.2.txt` is the complete Rust method census.
`polars-datatype-portable-methods-0.55.2.txt` is the cross-language behavioral
denominator enforced by the strict parity gate.

The following Rust methods are intentionally outside that denominator:

| Methods | Reason |
| --- | --- |
| `_month_days_ns_struct_type` | Private-style Arrow interval construction helper, not a user data-type operation |
| `boxed` | Rust allocation/ownership conversion; Terlan data-type values already have value semantics |
| `cat_mapping`, `cat_physical` | Expose Polars process-local categorical mapping internals |
| `from_arrow`, `from_arrow_dtype`, `from_arrow_field` | Accept Rust Arrow objects; package interchange uses owned Arrow IPC bytes |
| `from_categories`, `from_frozen_categories` | Accept Rust-owned category registries; `categorical_type` and `enum_type` are the portable constructors |
| `into_inner_dtype`, `try_into_inner_dtype` | Rust ownership variants of the exposed `inner_data_type` behavior |
| `map_leaves`, `try_mutate_with`, `try_visit_with`, `visit_with` | Accept Rust closures; Terlan exposes deterministic leaf replacement and structural inspection |
| `materialize_unknown` | Operates on compiler-internal unknown types, which cannot cross the package boundary |
| `to_arrow`, `try_to_arrow`, `to_arrow_field`, `to_arrow_field_metadata` | Return Rust Arrow objects or metadata; package interchange uses Arrow IPC |
| `value_within_range` | Accepts Polars `AnyValue`; Terlan validates values through typed Series construction and casts |

These are API reshapes or host-language implementation details, not untracked
gaps. Any future public Terlan operation that admits unknown types, Arrow
objects, category mappings, callbacks, or `AnyValue` must first define a
portable owned type and then move the corresponding method into the enforced
inventory.
