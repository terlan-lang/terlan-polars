# Polars expressions guide coverage

This inventory tracks executable Terlan ports of the official Polars
[expressions guide](https://docs.pola.rs/user-guide/expressions/).

The baseline is upstream Polars commit
`9c223540d8c6fffec1c9cdf532d11fca619c6e98`. The canonical scope is the Rust
example directory at `docs/source/src/rust/user-guide/expressions`, not the
Python snippets rendered alongside it. At that revision it contains 135 tagged
regions: 90 executable Rust examples and 45 placeholder regions that only ask
for a future Rust translation. Only the 90 real examples are implementation
targets; placeholders remain recorded so they cannot be mistaken for missing
Terlan work.

An example is complete only when its Terlan port executes through the real
native helper and asserts its result. Merely exposing the referenced method is
not sufficient.

| Rust source | Executable | Upstream placeholders | Status |
| --- | ---: | ---: | --- |
| `operations.rs` | 9 | 0 | Complete |
| `column-selections.rs` | 6 | 7 | Complete |
| `expression-expansion.rs` | 15 | 7 | Complete |
| `casting.rs` | 10 | 0 | Complete |
| `strings.rs` | 9 | 0 | Complete |
| `lists.rs` | 0 | 16 | Not applicable |
| `structs.rs` | 7 | 6 | Complete |
| `missing-data.rs` | 11 | 0 | Complete |
| `aggregation.rs` | 9 | 0 | Complete |
| `window.rs` | 7 | 6 | Complete |
| `folds.rs` | 6 | 0 | Complete |
| `user-defined-functions.rs` | 1 | 5 | Complete |
| **Total** | **90** | **45** | **Complete** |

## Porting rules

- Preserve the documented transformation and result, while expressing it
  through typed Terlan APIs.
- Use the same source data where it is deterministic. Replace Python/NumPy
  random generation with committed deterministic fixtures.
- Keep expressions immutable values. Adding guide coverage must not turn
  `Expr` into a helper-owned resource.
- Add reusable typed expression operations; do not add tutorial-specific
  native calls.
- Prefer the public `Series` resource for standalone Series demonstrations;
  use one-column DataFrames where the operation is only exposed as an
  expression pipeline.
- Regions containing only the upstream comment “Contribute the Rust
  translation…” are not Terlan implementation targets.

## Tagged example inventory

### Operations

`dataframe`, `arithmetic`, `comparison`, `boolean`, `bitwise`, `count`,
`value_counts`, `unique_counts`, `collatz`.

### Column selections

Executable: `selectors_df`, `all`, `exclude`, `expansion_by_names`,
`expansion_by_regex`, `expansion_by_dtype`.

Upstream placeholders: `selectors_intro`, `selectors_diff`, `selectors_union`,
`selectors_by_name`, `selectors_to_expr`, `selectors_is_selector_utility`,
`selectors_colnames_utility`.

### Expression expansion

Executable: `df`, `col-with-names`, `expression-list`, `col-with-dtype`,
`col-with-dtypes`, `col-with-regex`, `all`, `all-exclude`, `col-exclude`,
`duplicate-error`, `alias`, `prefix-suffix`, `name-map`, `for-with_columns`,
`yield-expressions`.

Upstream placeholders: `selectors`, `selectors-set-operations`,
`selectors-expressions`, `selector-ambiguity`, `as_expr`, `is_selector`,
`expand_selector`.

### Casting

`dfnum`, `castnum`, `downcast`, `overflow`, `overflow2`, `strings`, `strings2`,
`bool`, `dates`, `dates2`.

### Strings

`df`, `existence`, `extract`, `extract_all`, `replace`, `concat`, `casing`,
`strip`, `slice`.

### Lists

All sixteen tagged regions are upstream Rust placeholders: `list-example`,
`array-example`, `numpy-array-inference`, `weather`, `split`,
`explode`, `list-slicing`, `element-wise-casting`, `element-wise-regex`,
`children`, `list-sorting`, `list-aggregation`, `list-entropy`,
`weather_by_day`, `rank_pct`, `array-overview`.

### Structs

Executable: `ratings_df`, `state_value_counts`, `struct_unnest`,
`series_struct`, `series_struct_extract`, `struct_ranking`,
`multi_column_apply`.

Upstream placeholders: `series_struct_error`, `series_struct_rename`,
`struct-rename-check`, `struct_duplicates`, `ack`, `struct-ack`.

### Missing data

`dataframe`, `count`, `isnull`, `dataframe2`, `fill`, `fillexpr`,
`fillstrategy`, `fillinterpolate`, `nan`, `nan-computed`, `nanfill`.

### Aggregation

`dataframe`, `basic`, `conditional`, `filter`, `nested`, `filter-nested`,
`sort`, `sort2`, `sort3`.

### Window functions

Executable: `pokemon`, `rank`, `pokemon-mean`, `group_by`, `operations`, `sort`,
`examples`.

Upstream placeholders: `rank-multiple`, `rank-explode`, `athletes`,
`athletes-sort-over-country`, `athletes-explode`, `athletes-join`.

### Folds

`mansum`, `mansum-explicit`, `manprod`, `manprod-fixed`, `conditional`,
`string`.

### User-defined functions

Executable: `dataframe`.

Upstream placeholders: `individual_log`, `diff_from_mean`, `np_log`,
`diff_from_mean_numba`, `combine`.
