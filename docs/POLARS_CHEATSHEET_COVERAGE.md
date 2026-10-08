# Polars 1.43 cheat-sheet coverage

This audit maps every API operation printed in Posit's
[Python Polars cheat sheet](https://opensource.posit.co/resources/cheatsheets/polars/polars-cheatsheet.pdf)
(Polars 1.43.0, updated 2026-08) to the public `terlan-polars` API. The native
package pins the official Python Polars 2.0 source revision, so this remains a
Python-facing compatibility audit in addition to the published-Rust inventory.

The status labels have deliberately different meanings:

- **Direct**: a public Terlan operation exposes the behavior and executes
  through the native adapter.
- **Composition**: the same result is available by combining the named public
  Terlan operations. There is no identically shaped convenience call.
- **Partial**: useful behavior exists, but an option or result from the printed
  Python operation cannot be preserved.
- **Missing**: the package has no exact operation or composition.
- **External**: the operation belongs to another Python package, visualization
  system, or hosted service rather than the Polars dataframe engine.
- **Optional ecosystem**: the operation is a format/service integration rather
  than core dataframe behavior and is intentionally left to a separate adapter.

Passing the existing upstream parity inventory is not evidence for every row
below. In particular, the inventory records name-level reshaping for some Rust
callbacks and convenience methods. This audit checks whether the observable
cheat-sheet behavior is actually expressible from the public Terlan API.

## Validation snapshot

Validated on 2026-10-08:

- the complete `make release-check` component set passed, including 163 native-library tests, 34 native
  helper tests, compiler checks, script linting, and all 76 Terlan package tests
  against the real helper, plus optional Plotly HTML and PNG rendering tests;
- `make parity-check` passed as an inventory-mapping check for 919 upstream
  identities;
- the hardened report separates 353 exact native-name matches from 566
  explicit mappings, so a green inventory cannot be read as behavioral proof;
- the source audit checked the exported Terlan declarations, generated native
  metadata, native dispatch, pinned feature profile, and executable test
  families before assigning the statuses below.

## Getting started, structures, and execution

| Cheat-sheet operation | Status | Terlan mapping and limits |
| --- | --- | --- |
| `uv pip install "polars[all]"`, `import polars` | External | Python installation/import operations do not apply to a Terlan package. |
| `pl.show_versions()` | Direct | `show_versions` reports the Terlan package, pinned Polars, and visualization backend versions. |
| `pl.Series(name, values)` | Direct | Typed `series_from_*` and nullable variants; wide integers use lossless decimal text. |
| `pl.DataFrame({...})` | Direct | `dataframe_from_series` constructs a typed frame from an ordered list of named Series; `from_rows` covers string rows. |
| `df.with_row_index(name, offset)` | Direct | `with_row_index`; `lazy_with_row_index` for a lazy plan. |
| `df.lazy()` / `pl.scan_*()` | Direct | `lazy` and the supported `scan_*` calls. |
| `lf.collect()` | Direct | `collect`. |
| `lf.collect(engine="streaming")` | Direct | `collect_streaming`. |
| `lf.explain()` | Direct | `describe_plan` and `describe_plan_tree`. |
| `lf.show_graph()` | Direct | `describe_plan_dot` returns Graphviz DOT for logical or optimized plans; `describe_plan_tree` provides a terminal form. |
| `lf.profile()` | Compatibility | Python Polars 2.0 removed this call. Terlan retains `profile` and `profile_plan` for existing programs; new code should use plan inspection and ordinary collection. |

## Data types and inspection

| Cheat-sheet operation | Status | Terlan mapping and limits |
| --- | --- | --- |
| `Decimal` | Direct | `decimal_type`. |
| `Float32`, `Float64` | Direct | Public scalar data types. |
| `Int8`, `Int16`, `Int32`, `Int64`, `Int128` | Direct | Public scalar data types; populated Int128 Series cross the VM boundary as checked decimal text. |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | Direct | Public scalar data types. UInt128 is also supported. |
| `Date`, `Datetime`, `Duration`, `Time` | Direct | `DateType`, `datetime_type`, duration unit types, and `TimeType`. |
| `Array`, `List`, `Struct` | Direct | `array_type`, `list_type`, and `struct_type`. |
| `String`, `Categorical`, `Enum` | Direct | `Utf8`, `categorical_type`, and `enum_type`. |
| `Boolean`, `Binary`, `Null` | Direct | `Boolean`, `BinaryType`, and `NullType`. |
| `df.schema` | Direct | `schema`. |
| `df.dtypes` | Direct | `dtypes` returns ordered Polars data-type names. |
| `df.glimpse()` | Direct | `glimpse` returns a bounded column-oriented rendering with data types and sample values. |
| `df.describe()` | Direct | `describe` returns count, null count, mean, standard deviation, min, quartiles, and max, preserving mixed numeric/string summary columns. |
| `df.estimated_size("mb")` | Direct | The unit overload of `estimated_size` supports Polars-compatible b/kb/mb/gb/tb scaling and full plural names. |
| `e.cast(dtype)` (strict) | Direct | `strict_cast_to` / `strict_cast_nested`; `cast_with_options(..., "strict")` exposes the Rust option spelling directly. |
| `e.cast(dtype, strict=False)` | Direct | `cast_to` / `cast_nested` perform non-strict casts and produce null for failures; `cast_with_options` also supports non-strict and overflowing modes. |

## Reading and writing data

| Format or option | Status | Terlan mapping and limits |
| --- | --- | --- |
| Avro `read_*` / `write_*` | Optional ecosystem | No Avro adapter is bundled in the core package. |
| Clipboard `read_*` / `write_*` | External | Clipboard access is host/UI integration and is not part of the native dataframe boundary. |
| CSV `read_*`, `scan_*`, `write_*`, `sink_*` | Direct | `read_csv[_with]`, `scan_csv`, `write_csv[_with]`, and `lazy_write_csv`. |
| Database `read_*` | Direct | `read_database_uri` and `read_database_env` support PostgreSQL and SQLite through ConnectorX. Database scan/write/sink calls are not exported. |
| Delta Lake `read_*`, `scan_*`, `write_*` | Optional ecosystem | Delta transaction-log support belongs in a separate storage adapter. |
| Excel / ODS `read_*`, `write_*` | Optional ecosystem | Spreadsheet parsing belongs in a separate format adapter. |
| Iceberg `scan_*` | Optional ecosystem | Iceberg catalog integration belongs in a separate storage adapter. |
| IPC / Feather `read_*`, `scan_*`, `write_*`, `sink_*` | Direct | `read_ipc[_with]`, `scan_ipc`, `write_ipc[_with]`, and `lazy_write_ipc`. |
| JSON `read_*`, `write_*` | Direct | `read_json[_with]` and `write_json`. The cheat sheet does not mark JSON scan/sink support. |
| NDJSON `read_*`, `scan_*`, `write_*`, `sink_*` | Direct | `read_ndjson`, `scan_ndjson`, `write_ndjson`, and `lazy_write_ndjson`. |
| Parquet `read_*`, `scan_*`, `write_*`, `sink_*` | Direct | `read_parquet[_with]`, `scan_parquet`, `write_parquet[_with]`, and `lazy_write_parquet`. |
| PyArrow Dataset `scan_*` | External | Python objects cannot cross the native boundary; owned Arrow IPC byte interchange is available instead. |
| `schema_overrides`, `n_rows`, `row_index_name` | Direct | Detailed CSV options expose all three; applicable Parquet and IPC option descriptors also expose row limits and/or row-index configuration. |
| `compression` | Direct | Detailed Parquet and IPC write options expose compression; CSV has its applicable write options. |
| `storage_options` | Direct | `scan_*_with_storage_options` accepts parallel key/value lists for CSV, Parquet, NDJSON, and IPC per-call cloud configuration. |
| `scan_parquet("s3://...")` | Direct | `scan_parquet` accepts cloud URLs; credentials/configuration come from the environment. |
| `sink_parquet(PartitionBy(...))` | Direct | `lazy_write_parquet_partitioned` accepts key expressions, key inclusion, and row/approximate-byte file limits; equivalent CSV, NDJSON, and IPC sinks are also exported. |

## Selecting, creating, filtering, slicing, and sorting

| Cheat-sheet operation | Status | Terlan mapping and limits |
| --- | --- | --- |
| `df.select("a", "b")` | Direct | `select`. |
| `df.select(pl.col("x") * 2)` | Direct | `select_exprs` with `col` and `multiply`. |
| Named/aliased select (`doubled=...`) | Direct | `alias`. |
| Regex select (`pl.col("^.*_color$")`) | Direct | `col` preserves Polars regex expansion. |
| `df.select(pl.all())`, `pl.col("*")` | Direct | `all_columns`; wildcard column expansion is also accepted by Polars expressions. |
| Selector `numeric()` / `string()` | Direct | `dtype_cols_typed` with the required numeric or `Utf8` data types. |
| Selector `starts_with()` / `contains()` | Direct | `selector_starts_with` and `selector_contains` produce first-class selector expressions. |
| Selector `first()` | Direct | `selector_first`. |
| Selector set operators `|`, `&`, `-`, `^`, `~` | Direct | `selector_union`, `selector_intersection`, `selector_difference`, `selector_exclusive_or`, and `selector_complement`. |
| `df.drop(..., strict=False)` | Direct | The Boolean overload of `drop_columns` ignores absent names when strict mode is false. |
| `df.with_columns(...)` | Direct | `with_columns` and `with_columns_sequential`. |
| `e.fill_null(value)` | Direct | `fill_null`. |
| `pl.lit(value)` | Direct | `lit` for String/Int/Float/Bool and `null_lit`; a Series literal is available through `series_to_expr`. |
| `df.with_row_index(...)` | Direct | `with_row_index`. |
| `df.filter("valid")` | Direct | `filter(col("valid"))`. |
| `df.filter(e)` | Direct | `filter`. |
| `df.filter(e1, e2)` | Direct | `filter_all` combines a non-empty expression list with conjunction. |
| Boolean filter `&` / `|` | Direct | `and_predicate` / `or_predicate`. |
| Named filter constraints | Direct | `filter_constraints` pairs column names with equality-value expressions; use `lit` for mixed scalar values. |
| `df.drop_nulls()` / `df.drop_nulls(subset)` | Direct | `drop_null_rows`. |
| `df.unique(subset, keep)` | Direct | `unique_rows` with typed keep policy. |
| `df.head`, `df.tail`, `df.slice` | Direct | `head`, `tail`, and `slice_rows`. |
| `df.gather_every` | Direct | DataFrame `gather_every` validates a positive step and non-negative offset. |
| `df.sample(n)`, replacement, fraction | Direct | `sample_rows_n` / `sample_rows_fraction`, including replacement, shuffle, and seeded variants. |
| Single/multi-column `df.sort` | Direct | `sort_by` / `sort_rows_by`. |
| `nulls_last`, scalar/mixed `descending` | Direct | `sort_rows_by` accepts one flag per expression and null placement. |
| Sort by arithmetic or `list.len()` expression | Direct | `sort_rows_by` accepts expressions; `list_len` is exported. |
| `df.top_k`, `df.bottom_k` | Direct | `top_rows_by` / `bottom_rows_by`. |

## Reshaping, aggregation, joins, and concatenation

| Cheat-sheet operation | Status | Terlan mapping and limits |
| --- | --- | --- |
| `df.unpivot` | Direct | `unpivot`; `lazy_unpivot` for plans. |
| `df.pivot` and aggregated pivot | Direct | `pivot`; `lazy_pivot` for plans. |
| `df.explode` | Direct | `explode_columns`; `lazy_explode`. |
| `df.unnest` | Direct | `unnest_columns`; `lazy_unnest`. |
| `df.transpose(include_header=True)` | Direct | `transpose` with a non-empty header-column name. |
| `df.partition_by` | Direct | `partition_by`, returning `DataFrameSet`. |
| `df.group_by` with one/many keys | Direct | One-shot `group_agg` / `lazy_group_agg` accept expression key lists. |
| Group `len`, `head`, `mean`, list aggregation, named aggregation | Direct | `len_expr`, `head_expression`, `mean`, `implode`/aggregation, and `alias` supplied to `group_agg`. |
| `GroupBy.map_groups(...)` | Direct | `map_groups` materializes each group and applies a serializable declarative expression projection, the safe Terlan equivalent of a process-local Python callback. |
| Window aggregation `.over(...)` | Direct | `over`, plus ordered/explode/join window variants. |
| `group_by_dynamic(...)` | Direct | `dynamic_group_agg`. |
| `df.rolling(...).agg(...)` | Direct | `rolling_group_agg`. |
| `df.upsample(...)` | Direct | `upsample` supports a temporal/index column, interval, optional groups, and stable ordering. |
| `sum_horizontal`, `any_horizontal` | Direct | Same-named operations; selectors are supplied as expansion expressions. |
| Inner/left/full/semi/anti/cross joins | Direct | `join` / `lazy_join`; independently named keys are supported. |
| Full join `coalesce=True` | Direct | `join_with_coalesce` and `lazy_join_with_coalesce` expose key-column coalescing. |
| `join_asof` | Direct | `asof_join` / `lazy_asof_join`. |
| `join_where` | Direct | `join_where` / `lazy_join_where`. |
| Join `left_on`, `right_on`, `join_nulls`, `suffix` | Direct | Public join arguments; `join_nulls` is named `nulls_equal`. |
| Join validation `m:m`, `m:1`, `1:m`, `1:1` | Direct | Typed `JoinValidation` passed to `join_with_options`. |
| Vertical, horizontal, diagonal `concat` | Direct | `concat_vertical`, strict `concat_horizontal`, explicit null-extending `concat_horizontal_extend`, and `concat_diagonal` (pairwise and composable for lists). |
| `concat(..., how="vertical_relaxed")` | Direct | `concat_vertical_relaxed` coerces columns to common supertypes. |
| `df.update(other, on, how)` | Direct | `update` supports key or row-index matching, left/inner/full modes, and optional null overwrites. |

## Beginning and combining expressions

| Cheat-sheet operation | Status | Terlan mapping and limits |
| --- | --- | --- |
| `pl.col`, `pl.col("*")`, `pl.all`, `pl.lit` | Direct | `col`, `all_columns`, and `lit`. |
| `pl.arange` / integer `*_range(s)` | Direct | `int_range` and `int_ranges`. |
| `pl.date_range(s)` | Direct | `date_range` and `date_ranges`. |
| Time/datetime `*_range(s)` | Direct | `time_range`/`time_ranges` and `datetime_range`/`datetime_ranges`. |
| `+`, `-`, `*`, `/`, `//`, `**`, `%` | Direct | `add`, `subtract`, `multiply`, `divide`/`true_div`, `floor_divide`, `pow`, and `modulo`. `true_div` guarantees floating-point division for integer inputs. |
| `dot` | Direct | `dot`. |
| `<`, `<=`, `==`, `>=`, `>`, `!=` | Direct | `lt`, `lte`, `equal`, `gte`, `gt`, and `not_equal`. |
| `&`, `|`, `~`, `^` | Direct | `and_predicate`, `or_predicate`, `not_expr`, and `xor`. |
| Chained `when` / `then` / `otherwise` | Direct | `when_chain` pairs predicate and value expression lists with an `otherwise` expression; the first matching predicate wins. |

## Numeric, missing-value, shape, shift, and rolling expressions

| Cheat-sheet operation | Status | Terlan mapping and limits |
| --- | --- | --- |
| `abs`, `exp` | Direct | Same-named operations. |
| `sign` | Direct | `sign` preserves Polars sign behavior. |
| `cbrt`, `sqrt` | Direct | Same-named operations preserve Polars behavior, including real cube roots for negative values. |
| `log`, `log1p` | Direct | Same-named operations. |
| `log10` | Direct | Same-named base-10 logarithm expression. |
| `cos`, `sin`, `tan`; hyperbolic forms | Direct | Same-named operations. |
| Inverse trig and inverse hyperbolic forms | Direct | `arccos`, `arcsin`, `arctan`, `arccosh`, `arcsinh`, `arctanh`. |
| `degrees`, `radians` | Direct | Same-named operations. |
| `ceil`, `floor`, `round` | Direct | `ceil`, `floor`, and `round_to`/`round_with_mode`. |
| `clip`, `cut`, `qcut`; Polars 2 `bin_intervals`, `bin_quantiles`, `bin_ranks` | Direct | `clip` is direct. Expr and eager Series `bin_*` replacements support explicit/uniform specs, labels or indices, closure control, and interval Structs; typed/count helpers cover every breakpoint form; deprecated `cut`/`qcut` remain compatibility shims. |
| `fill_nan`, `fill_null` | Direct | Same-named operations. |
| `is_finite`, `is_infinite`, `is_nan`, `is_not_nan`, `is_null`, `is_not_null` | Direct | Same-named operations. |
| `drop_nans`, `drop_nulls` | Direct | Same-named expression operations. |
| `flatten`, `explode`, `implode` | Direct | `explode` covers flatten/explode semantics; `implode` is direct. |
| `reshape` | Direct | `reshape_expression`. |
| `backward_fill`, `forward_fill` | Direct | `fill_null_backward` and `fill_null_forward` cover defaults; `fill_null_with_strategy` preserves directional limits and also exposes min, max, mean, zero, and one strategies. |
| `interpolate`, `shift` | Direct | `interpolate` and `shift`/`shift_and_fill`. |
| `cum_count`, `cum_sum`, `cum_max`, `cum_min` | Direct | Same-named operations with forward/reverse control. |
| `diff`, `pct_change` | Direct | Same-named operations. |
| `ewm_mean`, `ewm_std`, `ewm_var` | Direct | Same-named operations. |
| `rolling_max`, `rolling_min`, `rolling_mean`, `rolling_median`, `rolling_std`, `rolling_var` | Direct | Same-named operations, with weighted and time/index-window variants where applicable. |
| `rolling_map` | Direct | `rolling_map` invokes a checked one-argument declarative `ExprUdf` against each materialized rolling Series. |

## Sorting, summaries, counting, and selection expressions

| Cheat-sheet operation | Status | Terlan mapping and limits |
| --- | --- | --- |
| `sort`, `sort_by`, `arg_sort` | Direct | `sort_with`, full `sort_by` with per-key descending/null placement and stable-order control, ascending/descending conveniences, and `arg_sort`. |
| `shuffle`, `reverse`, `rank` | Direct | Same-named operations, including seeded shuffle and rank policies. |
| `is_duplicated`, `is_unique`, `is_first_distinct`, `is_last_distinct` | Direct | Same-named operations. |
| `all`, `any` | Direct | Same-named reductions with null handling. |
| `max`, `min`, `mean`, `nan_max`, `nan_min`, `median`, `std`, `var` | Direct | Same-named reductions. |
| `entropy`, `kurtosis`, `skew`, `product`, `quantile`, `sum` | Direct | Same-named reductions. |
| `arg_max`, `arg_min`, `first`, `last`, `get`, `mode` | Direct | Same-named or `get_expression` operations. |
| `len`, `count`, `null_count` | Direct | `len_values`, `count`, and `null_count`. |
| `n_unique`, `approx_n_unique`, `arg_unique`, `unique` | Direct | Same-named operations. |
| `unique_counts`, `value_counts` | Direct | Same-named operations. |
| `head`, `tail`, `limit` | Direct | `head_expression` and `tail_expression`; `limit` is the head operation. |
| `bottom_k`, `top_k` | Direct | Same-named operations and multi-key variants. |
| `gather`, `gather_every` | Direct | Same-named operations. |
| `sample`, `slice` | Direct | Fixed-count/fractional seeded/unseeded sample operations and `slice_expression`. |
| `arg_true` | Direct | `arg_true` returns indices of true values. |
| `replace` via mapping | Direct | `replace`, `replace_strict`, and `replace_or_default`; Series literals allow multi-value old/new mappings. |
| `search_sorted` | Direct | Same-named operation. |

## Array, list, categorical, temporal, string, struct, binary, name, and meta namespaces

| Cheat-sheet operation | Status | Terlan mapping and limits |
| --- | --- | --- |
| Cast to `Array(dtype, width)` | Direct | `array_type` plus `cast_nested` / `strict_cast_nested`. |
| `arr.max`, `arr.sort` | Direct | `array_max` and `array_sort[_with]`. |
| `pl.list("a", "b")` | Direct | `concat_list` constructs a list from horizontal input expressions. |
| `list.len`, `list.get`, `list.sort`, `list.join`, `list.contains` | Direct | Same-named `list_*` operations. |
| Cast to `Categorical` / `Enum` | Direct | `categorical_type` / `enum_type` with typed cast operations. |
| `cat.get_categories` | Direct | `categorical_categories`. |
| `pl.date(year, month, day)` | Direct | The three-expression `date` overload constructs Date values and preserves the year expression's output name; the ISO literal overload remains available. |
| `pl.datetime(...)` | Direct | `datetime` accepts year/month/day/hour/minute/second/microsecond/ambiguity expressions plus time unit and timezone, and preserves the year expression's output name. |
| `pl.duration(...)` | Direct | `duration` accepts week/day/hour/minute/second/millisecond/microsecond/nanosecond expressions and a time unit. |
| `dt.month` | Direct | `month`. |
| `dt.replace(...)` | Direct | `temporal_replace` exposes component replacement. |
| `dt.strftime(...)` | Direct | `date_format`. |
| `dt.convert_time_zone(...)` | Direct | `convert_time_zone`; `replace_time_zone` also exposes localization/replacement. |
| `dt.total_seconds()` | Direct | `total_seconds`. |
| `str.contains`, `str.split`, `str.to_uppercase` | Direct | `string_contains`, `string_split`, and `string_uppercase`. |
| `str.to_datetime` | Direct | `parse_datetime`. |
| `str.extract`, `str.strip_chars` | Direct | `string_extract` and `string_strip_chars`. |
| `pl.struct`, `struct.field`, `struct.rename_fields`, `struct.with_fields` | Direct | `as_struct`, `struct_field`, `struct_rename_fields`, and `struct_with_fields`. |
| `bin.base64_decode`, `bin.hex_encode` | Direct | `binary_base64_decode` and `binary_hex_encode`. |
| `name.prefix`, `name.to_lowercase` | Direct | `prefix` and `lowercase_names`. |
| `meta.output_name`, `meta.is_regex`, `meta.has_multiple_outputs` | Direct | `meta_output_name`, `meta_is_regex_projection`, and `meta_has_multiple_outputs`. |

## Styling, visualization, and cloud execution

| Cheat-sheet operation | Status | Terlan mapping and limits |
| --- | --- | --- |
| Great Tables `GT`, `tab_stub`, `cols_label`, `tab_header`, `fmt_number`, `fmt_nanoplot`, `data_color` | Direct | A declarative Rust styling layer exports the same operations and bounded HTML with escaped cell/header content. It does not expose Python Great Tables objects. |
| `df.plot.point(...)` / `df.plot.scatter(...)` | Direct | `plot_point_svg` is the canonical Polars spelling and `plot_scatter_svg` is its alias. Their color column maps to Terlan's grouping argument. Line, bar, histogram, and box SVG renderers are also available through Plotlars/Plotters. SVG is the lightweight default. |
| Plotnine `ggplot`, `aes`, `geom_point`; Plotly, hvPlot, Seaborn, Matplotlib | External | Python plotting-library objects and Plotnine's general grammar are external. The five covered chart forms have native SVG plus optional Plotly HTML and PNG outputs. |
| `df.to_pandas()` | External | A Python pandas object cannot cross the native boundary. Data can be exported as owned Arrow IPC bytes for a Python adapter to consume. |
| `polars_cloud.ComputeContext`, `lf.remote(ctx).execute().await_result()` | External | Polars Cloud is a separate hosted Python service and is not integrated. |

## Result

`terlan-polars` now covers every core dataframe and expression behavior printed
on the sheet. Its plotting calls cover the chart operations printed on the
sheet, while arbitrary Altair marks, encoding objects, and chained chart
configuration remain external visualization-library behavior. `map_groups` is
deliberately constrained to safe declarative group projections, and
Python-object interchange remains outside the native boundary. Avro, Delta
Lake, spreadsheets, and Iceberg are classified as separate optional adapters;
clipboard, PyArrow objects, Python plotting object models, and the hosted
Polars Cloud service remain external integrations.
