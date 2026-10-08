//! Deterministic upstream API inventory-mapping support.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

/// One generated native operation.
#[derive(Debug, Deserialize)]
struct NativeFunction {
    /// Stable native operation identity.
    operation: String,
}

/// Generated native metadata fields used by the parity inventory.
#[derive(Debug, Deserialize)]
struct NativeMetadata {
    /// Generated package functions.
    functions: Vec<NativeFunction>,
}

/// Result of comparing an upstream method census with generated operations.
///
/// This report proves name coverage only. An explicit mapping still needs
/// separate behavioral evidence before it can be called API parity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiParityReport {
    /// Number of distinct upstream method identities.
    pub upstream_count: usize,
    /// Upstream identities with a direct or explicit reshaped mapping.
    pub mapped_count: usize,
    /// Upstream identities whose name exactly matches a generated operation.
    pub exact_count: usize,
    /// Upstream identities covered only by an explicit mapping entry.
    pub explicit_mapping_count: usize,
    /// Upstream identities that still require a mapping or implementation.
    pub pending: Vec<String>,
}

/// Explicit Series method reshaping where names differ across APIs.
const SERIES_ALIASES: &[(&str, &str)] = &[
    ("_try_from_arrow_unchecked", "from_arrow_ipc"),
    ("_try_from_arrow_unchecked_with_md", "from_arrow_ipc"),
    ("append_owned", "append"),
    ("array", "leaf_values"),
    ("array_ref", "leaf_values"),
    ("as_ref", "clone"),
    ("as_single_ptr", "to_arrow_ipc"),
    ("binary", "to_arrow_ipc"),
    ("binary_offset", "to_arrow_ipc"),
    ("bitand", "bit_and"),
    ("bitor", "bit_or"),
    ("bitxor", "bit_xor"),
    ("bool", "boolean_values"),
    ("cat", "values"),
    ("cat16", "values"),
    ("cat32", "values"),
    ("cat8", "values"),
    ("cast_with_options", "cast_with_options"),
    ("cast_unchecked", "overflowing_cast"),
    ("chunks_mut", "rechunk"),
    ("compute_len", "len"),
    ("date", "to_physical"),
    ("datetime", "to_physical"),
    ("decimal", "to_physical"),
    ("default", "empty"),
    ("deref", "clone"),
    ("deserialize_from_reader", "deserialize"),
    ("div", "divide"),
    ("duration", "to_physical"),
    ("eq", "equal_values"),
    ("equals", "equal"),
    ("equals_missing", "equal_missing"),
    ("ext", "to_storage"),
    ("extend_constant", "extend"),
    ("f16", "float_values"),
    ("f32", "float_values"),
    ("f64", "float_values"),
    ("fmt", "values"),
    ("fmt_list", "values"),
    ("from", "from_strings"),
    ("from_any_values", "from_strings"),
    ("from_any_values_and_dtype", "from_strings"),
    ("from_array", "from_arrow_ipc"),
    ("from_arrow", "from_arrow_ipc"),
    ("from_arrow_chunks", "from_arrow_ipc"),
    ("from_chunk_and_dtype", "from_arrow_ipc"),
    ("from_chunks_and_dtype_unchecked", "from_arrow_ipc"),
    ("from_iter", "from_strings"),
    ("from_physical_unchecked", "cast"),
    ("from_vec", "from_strings"),
    ("get_flags", "sorted_flag"),
    ("gt", "greater_than"),
    ("gt_eq", "greater_than_or_equal"),
    ("get_leaf_array", "leaf_values"),
    ("i16", "integer_values"),
    ("i32", "integer_values"),
    ("i64", "integer_values"),
    ("i8", "integer_values"),
    ("i128", "values"),
    ("idx", "integer_values"),
    ("into_chunks", "select_chunk"),
    ("into_date", "cast"),
    ("into_datetime", "cast"),
    ("into_decimal", "cast"),
    ("into_duration", "cast"),
    ("into_extension", "to_storage"),
    ("into_frame", "to_frame"),
    ("into_series", "clone"),
    ("into_time", "cast"),
    ("is_series", "clone"),
    ("is_sorted_flag", "sorted_flag"),
    ("iter_chunks", "select_chunk"),
    ("iter", "values"),
    ("list", "leaf_values"),
    ("list_offsets_and_validities_recursive", "to_arrow_ipc"),
    ("lt", "less_than"),
    ("lt_eq", "less_than_or_equal"),
    ("mean_reduce", "mean"),
    ("mul", "multiply"),
    ("n_chunks", "chunk_count"),
    ("new_empty", "empty"),
    ("new", "from_strings"),
    ("null", "new_null"),
    ("not_equal", "not_equal_values"),
    ("sample_frac", "sample_fraction"),
    ("phys_iter", "values"),
    ("row_encode_ordered", "row_encode"),
    ("row_encode_unordered", "row_encode"),
    ("serialize_into_writer", "serialize"),
    ("serialize_to_bytes", "serialize"),
    ("split_at", "split_before"),
    ("str_value", "values"),
    ("struct_", "leaf_values"),
    ("sub", "subtract"),
    ("sum_reduce", "sum"),
    ("to_arrow", "to_arrow_ipc"),
    ("to_arrow_with_field", "to_arrow_ipc"),
    ("to_physical_repr", "to_physical"),
    ("take_inner", "clone"),
    ("try_add_owned", "add"),
    ("try_bool", "boolean_values"),
    ("try_array", "leaf_values"),
    ("try_binary", "to_arrow_ipc"),
    ("try_binary_offset", "to_arrow_ipc"),
    ("try_cat", "values"),
    ("try_cat16", "values"),
    ("try_cat32", "values"),
    ("try_cat8", "values"),
    ("try_date", "to_physical"),
    ("try_datetime", "to_physical"),
    ("try_decimal", "to_physical"),
    ("try_duration", "to_physical"),
    ("try_ext", "to_storage"),
    ("try_from", "from_strings"),
    ("try_f16", "float_values"),
    ("try_f32", "float_values"),
    ("try_f64", "float_values"),
    ("try_i16", "integer_values"),
    ("try_i32", "integer_values"),
    ("try_i64", "integer_values"),
    ("try_i8", "integer_values"),
    ("try_i128", "values"),
    ("try_idx", "integer_values"),
    ("try_mul_owned", "multiply"),
    ("try_list", "leaf_values"),
    ("try_new", "from_strings"),
    ("try_null", "new_null"),
    ("try_str", "string_values"),
    ("try_struct", "leaf_values"),
    ("try_sub_owned", "subtract"),
    ("try_u16", "integer_values"),
    ("try_u32", "integer_values"),
    ("try_u8", "integer_values"),
    ("try_time", "to_physical"),
    ("try_u128", "values"),
    ("try_u64", "values"),
    ("u16", "integer_values"),
    ("u32", "integer_values"),
    ("u8", "integer_values"),
    ("u128", "values"),
    ("u64", "values"),
    ("rem", "remainder"),
    ("str", "string_values"),
    ("time", "to_physical"),
    ("with_name", "rename"),
    ("wrapping_trunc_div_scalar", "divide"),
];

/// Explicit DataFrame method reshaping where names differ across APIs.
const DATAFRAME_ALIASES: &[(&str, &str)] = &[
    ("_new_unchecked_impl", "with_column"),
    ("_slice_and_realloc", "slice_rows"),
    ("_to_metadata", "schema"),
    ("align_chunks", "rechunk_frame"),
    ("align_chunks_par", "rechunk_frame"),
    ("append_record_batch", "from_arrow_ipc"),
    ("apply", "with_columns_exprs"),
    ("apply_at_idx", "with_columns_exprs"),
    ("apply_columns", "with_columns_exprs"),
    ("apply_columns_par", "with_columns_exprs"),
    ("cached_schema", "schema"),
    ("clear", "clear_rows"),
    ("column", "column_series"),
    ("columns_mut", "with_columns_exprs"),
    ("columns_mut_retain_schema", "with_columns_exprs"),
    ("default", "empty_with_height"),
    ("deserialize_from_reader", "deserialize"),
    ("drop", "drop_columns"),
    ("drop_in_place", "drop_columns"),
    ("drop_many", "drop_columns"),
    ("drop_many_amortized", "drop_columns"),
    ("drop_nulls", "drop_null_rows"),
    ("dtypes", "schema"),
    ("empty", "empty_with_height"),
    ("empty_with_arc_schema", "empty_with_schema"),
    ("ensure_matches_schema", "select_exprs"),
    ("explode_impl", "explode"),
    ("extend", "concat_vertical"),
    ("eq", "frames_equal"),
    ("equals", "frames_equal"),
    ("equals_missing", "frames_equal_missing"),
    ("fields", "schema"),
    ("fill_null", "fill_null"),
    ("filter", "filter_expr"),
    ("filter_seq", "filter_expr"),
    ("first_col_n_chunks", "first_col_n_chunks"),
    ("fmt", "rows"),
    ("from", "from_rows"),
    ("from_rows_and_schema", "from_rows"),
    ("from_rows_iter_and_schema", "from_rows"),
    ("gather_group_unchecked", "gather_rows"),
    ("get_column_names", "columns"),
    ("get_column_names_owned", "columns"),
    ("get_column_index", "column_index"),
    ("get", "column_at_index"),
    ("get_row", "row"),
    ("get_row_amortized", "row"),
    ("get_row_amortized_unchecked", "row"),
    ("get_supertype", "schema"),
    ("hash_rows", "row_hashes"),
    ("group_by", "group_agg"),
    ("group_by_stable", "group_agg"),
    ("group_by_with_series", "group_agg"),
    ("hstack", "concat_horizontal"),
    ("hstack_mut", "concat_horizontal"),
    ("hstack_mut_unchecked", "concat_horizontal"),
    ("index", "column_at_index"),
    ("into_columns", "column_at_index"),
    ("into_struct", "to_struct"),
    ("is_duplicated", "row_is_duplicated"),
    ("is_unique", "row_is_unique"),
    ("len", "height"),
    ("materialized_column_iter", "column_at_index"),
    ("max_n_chunks", "max_chunk_count"),
    ("new", "with_column"),
    ("new_from_index", "gather_rows"),
    ("new_infer_broadcast", "with_column"),
    ("new_infer_height", "with_column"),
    ("new_unchecked", "with_column"),
    ("new_unchecked_infer_broadcast", "with_column"),
    ("new_unchecked_infer_height", "with_column"),
    ("new_unchecked_with_broadcast", "with_column"),
    ("new_with_broadcast", "with_column"),
    ("null_count", "null_counts"),
    ("partition_by_stable", "partition_by"),
    ("pipe", "with_columns_exprs"),
    ("pipe_mut", "with_columns_exprs"),
    ("pipe_with_args", "with_columns_exprs"),
    ("push_column_unchecked", "with_column"),
    ("n_chunks", "first_col_n_chunks"),
    ("rename", "rename_columns"),
    ("rename_many", "rename_columns"),
    ("replace", "replace_column"),
    ("reverse", "reverse_rows"),
    ("sample_frac", "sample_rows_fraction"),
    ("sample_n", "sample_rows_n"),
    ("sample_n_literal", "sample_rows_n"),
    ("select_at_idx", "column_at_index"),
    ("select_to_vec", "select"),
    ("select_unchecked", "select"),
    ("schema_equal", "schema_equal"),
    ("serialize_into_writer", "serialize"),
    ("serialize_to_bytes", "serialize"),
    ("set_column_names", "rename_columns"),
    ("try_get_column_index", "column_index"),
    ("shift", "shift_rows"),
    ("shape", "shape"),
    ("shape_has_zero", "shape_has_zero"),
    ("slice", "slice_rows"),
    ("slice_par", "slice_rows"),
    ("sort", "sort_rows_by"),
    ("sort_in_place", "sort_rows_by"),
    ("split_at", "split_before"),
    ("transpose_impl", "transpose"),
    ("take", "gather_rows"),
    ("take_slice_unchecked", "gather_rows"),
    ("take_slice_unchecked_impl", "gather_rows"),
    ("take_unchecked", "gather_rows"),
    ("take_unchecked_impl", "gather_rows"),
    ("try_extend", "concat_vertical"),
    ("unique", "unique_rows"),
    ("unique_impl", "unique_rows"),
    ("unique_stable", "unique_rows"),
    ("vstack", "concat_vertical"),
    ("vstack_mut", "concat_vertical"),
    ("vstack_mut_owned", "concat_vertical"),
    ("vstack_mut_owned_unchecked", "concat_vertical"),
    ("vstack_mut_unchecked", "concat_vertical"),
    ("rechunk_mut", "rechunk_frame"),
    ("rechunk_mut_par", "rechunk_frame"),
    ("rechunk_into_arrow", "to_arrow_ipc"),
    ("rechunk_to_arrow", "to_arrow_ipc"),
    ("set_height", "slice_rows"),
    ("set_opt_schema", "select_exprs"),
    ("set_schema", "select_exprs"),
    ("set_schema_from", "select_exprs"),
    ("try_apply", "with_columns_exprs"),
    ("try_apply_at_idx", "with_columns_exprs"),
    ("try_apply_columns", "with_columns_exprs"),
    ("try_apply_columns_par", "with_columns_exprs"),
    ("try_from", "from_rows"),
    ("try_from_rows_iter_and_schema", "from_rows"),
    ("with_columns_mut", "with_columns_exprs"),
    ("with_row_index_mut", "with_row_index"),
    ("with_schema", "select_exprs"),
    ("with_schema_from", "select_exprs"),
];

/// Explicit LazyFrame method reshaping where names differ across APIs.
const LAZY_FRAME_ALIASES: &[(&str, &str)] = &[
    ("_collect_post_opt", "collect"),
    ("_profile_post_opt", "profile_plan"),
    ("_with_eager", "collect"),
    ("anti_join", "join"),
    ("anonymous_scan", "scan_parquet"),
    ("bottom_k", "bottom_rows_by"),
    ("clear", "clear_rows"),
    ("count", "column_non_null_counts"),
    ("collect_schema", "schema"),
    ("collect_all_with_engine", "collect_all_with_engine"),
    ("collect_batches", "collect"),
    ("collect_concurrently", "collect_all"),
    ("collect_with_engine", "collect_with_engine"),
    ("cross_join", "join"),
    ("describe_optimized_plan", "describe_plan"),
    ("describe_optimized_plan_tree", "describe_plan_tree"),
    ("explain", "describe_plan"),
    ("explain_all", "describe_plan"),
    ("drop", "drop_columns"),
    ("drop_nans", "drop_nan_rows"),
    ("drop_nulls", "drop_null_rows"),
    ("fill_nan", "fill_nan_values"),
    ("fill_null", "fill_null_values"),
    ("filter", "filter_expr"),
    ("first", "first_row"),
    ("from", "collect"),
    ("from_logical_plan", "describe_plan"),
    ("full_join", "join"),
    ("gather", "gather_rows"),
    ("group_by_dynamic", "dynamic_group_agg"),
    ("group_by", "group_agg"),
    ("group_by_stable", "group_agg"),
    ("inner_join", "join"),
    ("hint", "set_optimization"),
    ("join_builder", "join"),
    ("last", "last_row"),
    ("lazy", "collect"),
    ("map", "with_columns"),
    ("match_to_schema", "cast"),
    ("max", "column_maxima"),
    ("mean", "column_means"),
    ("median", "column_medians"),
    ("min", "column_minima"),
    ("null_count", "null_counts"),
    ("optimize", "without_optimizations"),
    ("pipe_with_schema", "with_columns"),
    ("pipe_with_schemas", "with_columns"),
    ("remove", "remove_where"),
    ("profile", "profile"),
    ("quantile", "column_quantiles"),
    ("rename", "rename_columns"),
    ("reverse", "reverse_rows"),
    ("rolling", "rolling_group_agg"),
    ("scan_ipc_sources", "scan_ipc"),
    ("scan_parquet_files", "scan_parquet"),
    ("scan_parquet_sources", "scan_parquet"),
    ("schema_with_arenas", "schema"),
    ("select_seq", "select_exprs_sequential"),
    ("semi_join", "join"),
    ("set_cached_arena", "cache"),
    ("shift", "shift_rows"),
    ("shift_and_fill", "shift_and_fill_rows"),
    ("slice", "slice_rows"),
    ("sort", "sort"),
    ("sort_by_exprs", "sort_rows_by"),
    ("std", "column_stddevs"),
    ("sink", "write_parquet"),
    ("sink_batches", "write_parquet"),
    ("sum", "column_sums"),
    ("top_k", "top_rows_by"),
    ("to_alp", "describe_plan"),
    ("to_alp_optimized", "describe_plan"),
    ("unique", "unique_rows"),
    ("unique_generic", "unique_rows"),
    ("unique_stable", "unique_rows"),
    ("unique_stable_generic", "unique_rows"),
    ("var", "column_variances"),
    ("with_column", "with_columns"),
    ("with_columns_seq", "with_columns_sequential"),
    ("with_cluster_with_columns", "set_optimization"),
    ("with_check_order", "set_optimization"),
    ("with_comm_subexpr_elim", "set_optimization"),
    ("with_comm_subplan_elim", "set_optimization"),
    ("with_optimizations", "set_optimization"),
    ("with_predicate_pushdown", "set_optimization"),
    ("with_projection_pushdown", "set_optimization"),
    ("with_gpu", "set_optimization"),
    ("with_row_estimate", "set_optimization"),
    ("with_simplify_expr", "set_optimization"),
    ("with_slice_pushdown", "set_optimization"),
    ("with_streaming", "collect_streaming"),
    ("with_type_check", "set_optimization"),
    ("with_type_coercion", "set_optimization"),
    ("get_current_optimizations", "current_optimizations"),
];

/// Explicit expression reshaping for namespaces, callbacks, and Rust traits.
const EXPR_ALIASES: &[(&str, &str)] = &[
    ("agg_with_fmt_str", "apply_udf"),
    ("and", "and_predicate"),
    ("apply", "apply_udf"),
    ("apply_children", "map_children"),
    ("apply_many", "apply_udf"),
    ("apply_with_fmt_str", "apply_udf"),
    ("arr", "array_agg"),
    ("binary", "binary_size_bytes"),
    ("cast_with_options", "cast_with_options"),
    ("cat", "categorical_categories"),
    ("cbrt", "cbrt"),
    ("cum_prod", "cum_product"),
    ("default", "null"),
    ("div", "divide"),
    ("dt", "date"),
    ("eq", "equal"),
    ("eq_missing", "equal_missing"),
    ("ext", "cast"),
    ("extract_i64", "extract_i64"),
    ("extract_usize", "extract_usize"),
    ("fill_null_with_strategy", "fill_null_with_strategy"),
    ("filter", "filter_expression"),
    ("floor_div", "floor_divide"),
    ("fmt", "meta_format_tree"),
    ("from", "lit"),
    ("get", "get_expression"),
    ("gt_eq", "gte"),
    ("head", "head_expression"),
    ("hist", "histogram_auto"),
    ("into_selector", "into_selector"),
    ("is_between", "between"),
    ("list", "list_agg"),
    ("lt_eq", "lte"),
    ("map", "apply_udf"),
    ("map_binary", "apply_udf"),
    ("map_children", "map_children"),
    ("map_expr", "rewrite"),
    ("map_many", "apply_udf"),
    ("map_n_ary", "apply_udf"),
    ("map_ternary", "apply_udf"),
    ("map_unary", "apply_udf"),
    ("map_with_fmt_str", "apply_udf"),
    ("meta", "meta_format_tree"),
    ("mul", "multiply"),
    ("n_ary", "apply_udf"),
    ("name", "keep_name"),
    ("neg", "neg"),
    ("neq", "not_equal"),
    ("neq_missing", "not_equal_missing"),
    ("nodes", "meta_children"),
    ("nodes_owned", "meta_children"),
    ("not", "not_expr"),
    ("or", "or_predicate"),
    ("over_with_options", "over_with_options"),
    ("qcut_uniform", "qcut_equal_frequency"),
    ("rem", "modulo"),
    ("reshape", "reshape_expression"),
    ("rewrite", "rewrite"),
    ("rolling", "rolling"),
    ("rolling_map", "rolling_map"),
    ("sample_frac", "sample_fraction"),
    ("set_sorted_flag", "set_sorted_flag"),
    ("sign", "sign"),
    ("slice", "slice_expression"),
    ("sort", "sort_with"),
    ("sort_by", "sort_by"),
    ("sqrt", "sqrt"),
    ("str", "string_len_chars"),
    ("struct_", "struct_fields"),
    ("sub", "subtract"),
    ("tail", "tail_expression"),
    ("to_field", "to_field"),
    ("to_physical", "to_physical"),
    ("true_div", "true_div"),
    ("try_into_selector", "try_into_selector"),
    ("try_map_expr", "rewrite"),
    ("try_map_n_ary", "apply_udf"),
    ("visit", "meta_nodes"),
];

/// Explicit immutable reshaping of Polars format readers, writers, and options.
const IO_MAPPINGS: &[(&str, &str)] = &[
    ("CsvParseOptions.default", "polars.io.csv_read_options"),
    (
        "CsvParseOptions.with_comment_prefix",
        "polars.io.csv_read_options",
    ),
    (
        "CsvParseOptions.with_decimal_comma",
        "polars.io.csv_read_options",
    ),
    (
        "CsvParseOptions.with_encoding",
        "polars.io.csv_read_options",
    ),
    (
        "CsvParseOptions.with_eol_char",
        "polars.io.csv_read_options",
    ),
    (
        "CsvParseOptions.with_missing_is_null",
        "polars.io.csv_read_options",
    ),
    (
        "CsvParseOptions.with_null_values",
        "polars.io.csv_read_options",
    ),
    (
        "CsvParseOptions.with_quote_char",
        "polars.io.csv_read_options",
    ),
    (
        "CsvParseOptions.with_separator",
        "polars.io.csv_read_options",
    ),
    (
        "CsvParseOptions.with_truncate_ragged_lines",
        "polars.io.csv_read_options",
    ),
    (
        "CsvParseOptions.with_try_parse_dates",
        "polars.io.csv_read_options",
    ),
    ("CsvReadOptions.default", "polars.io.csv_read_options"),
    (
        "CsvReadOptions.get_parse_options",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.into_reader_with_file_handle",
        "polars.dataframe.read_csv_with_options",
    ),
    (
        "CsvReadOptions.map_parse_options",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.try_into_reader_with_file_path",
        "polars.dataframe.read_csv_with_options",
    ),
    (
        "CsvReadOptions.with_chunk_size",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_column_names_overwrite",
        "polars.io.csv_read_options_with_column_names",
    ),
    ("CsvReadOptions.with_columns", "polars.io.csv_read_options"),
    (
        "CsvReadOptions.with_dtype_overwrite",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_has_header",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_ignore_errors",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_infer_schema_files",
        "polars.io.csv_read_options_with_infer_schema_files",
    ),
    (
        "CsvReadOptions.with_infer_schema_length",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_low_memory",
        "polars.io.csv_read_options",
    ),
    ("CsvReadOptions.with_n_rows", "polars.io.csv_read_options"),
    (
        "CsvReadOptions.with_n_threads",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_parse_options",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_path",
        "polars.dataframe.read_csv_with_options",
    ),
    (
        "CsvReadOptions.with_projection",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_raise_if_empty",
        "polars.io.csv_read_options",
    ),
    ("CsvReadOptions.with_rechunk", "polars.io.csv_read_options"),
    (
        "CsvReadOptions.with_row_index",
        "polars.io.csv_read_options",
    ),
    ("CsvReadOptions.with_schema", "polars.io.csv_read_options"),
    (
        "CsvReadOptions.with_schema_overwrite",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_skip_lines",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_skip_rows",
        "polars.io.csv_read_options",
    ),
    (
        "CsvReadOptions.with_skip_rows_after_header",
        "polars.io.csv_read_options",
    ),
    ("CsvReader._with_predicate", "polars.lazy_frame.scan_csv"),
    ("CsvReader.finish", "polars.dataframe.read_csv_with_options"),
    ("CsvReader.new", "polars.dataframe.read_csv_with_options"),
    (
        "CsvReader.set_rechunk",
        "polars.dataframe.read_csv_with_options",
    ),
    (
        "CsvReader.with_options",
        "polars.dataframe.read_csv_with_options",
    ),
    (
        "CsvWriter.batched",
        "polars.dataframe.write_csv_with_options",
    ),
    (
        "CsvWriter.finish",
        "polars.dataframe.write_csv_with_options",
    ),
    ("CsvWriter.include_bom", "polars.io.csv_write_options"),
    ("CsvWriter.include_header", "polars.io.csv_write_options"),
    ("CsvWriter.n_threads", "polars.io.csv_write_options"),
    ("CsvWriter.new", "polars.io.csv_write_options"),
    ("CsvWriter.with_batch_size", "polars.io.csv_write_options"),
    ("CsvWriter.with_date_format", "polars.io.csv_write_options"),
    (
        "CsvWriter.with_datetime_format",
        "polars.io.csv_write_options",
    ),
    (
        "CsvWriter.with_decimal_comma",
        "polars.io.csv_write_options",
    ),
    (
        "CsvWriter.with_float_precision",
        "polars.io.csv_write_options",
    ),
    (
        "CsvWriter.with_float_scientific",
        "polars.io.csv_write_options",
    ),
    (
        "CsvWriter.with_line_terminator",
        "polars.io.csv_write_options",
    ),
    ("CsvWriter.with_null_value", "polars.io.csv_write_options"),
    ("CsvWriter.with_quote_char", "polars.io.csv_write_options"),
    ("CsvWriter.with_quote_style", "polars.io.csv_write_options"),
    ("CsvWriter.with_separator", "polars.io.csv_write_options"),
    ("CsvWriter.with_time_format", "polars.io.csv_write_options"),
    ("IpcReader.custom_metadata", "polars.io.ipc_custom_metadata"),
    ("IpcReader.finish", "polars.dataframe.read_ipc_with_options"),
    (
        "IpcReader.finish_with_scan_ops",
        "polars.lazy_frame.scan_ipc",
    ),
    (
        "IpcReader.memory_mapped",
        "polars.dataframe.read_ipc_with_options",
    ),
    ("IpcReader.new", "polars.dataframe.read_ipc_with_options"),
    ("IpcReader.schema", "polars.dataframe.schema"),
    (
        "IpcReader.set_rechunk",
        "polars.dataframe.read_ipc_with_options",
    ),
    (
        "IpcReader.with_columns",
        "polars.dataframe.read_ipc_with_options",
    ),
    (
        "IpcReader.with_hive_partition_columns",
        "polars.lazy_frame.scan_ipc",
    ),
    (
        "IpcReader.with_include_file_path",
        "polars.io.ipc_read_options",
    ),
    (
        "IpcReader.with_n_rows",
        "polars.dataframe.read_ipc_with_options",
    ),
    (
        "IpcReader.with_projection",
        "polars.dataframe.read_ipc_with_options",
    ),
    (
        "IpcReader.with_row_index",
        "polars.dataframe.read_ipc_with_options",
    ),
    (
        "IpcWriter.batched",
        "polars.dataframe.write_ipc_with_options",
    ),
    (
        "IpcWriter.finish",
        "polars.dataframe.write_ipc_with_options",
    ),
    ("IpcWriter.new", "polars.io.ipc_write_options"),
    (
        "IpcWriter.set_custom_schema_metadata",
        "polars.io.ipc_write_options",
    ),
    ("IpcWriter.with_compat_level", "polars.io.ipc_write_options"),
    ("IpcWriter.with_compression", "polars.io.ipc_write_options"),
    ("IpcWriter.with_parallel", "polars.io.ipc_write_options"),
    (
        "IpcWriter.with_record_batch_size",
        "polars.io.ipc_write_options",
    ),
    (
        "IpcWriter.with_record_batch_statistics",
        "polars.io.ipc_write_options",
    ),
    ("IpcWriterOptions.default", "polars.io.ipc_write_options"),
    (
        "IpcWriterOptions.to_writer",
        "polars.dataframe.write_ipc_with_options",
    ),
    (
        "JsonReader.finish",
        "polars.dataframe.read_json_with_options",
    ),
    ("JsonReader.infer_schema_len", "polars.io.json_read_options"),
    ("JsonReader.new", "polars.io.json_read_options"),
    ("JsonReader.set_rechunk", "polars.io.json_read_options"),
    ("JsonReader.with_batch_size", "polars.io.json_read_options"),
    (
        "JsonReader.with_ignore_errors",
        "polars.io.json_read_options",
    ),
    ("JsonReader.with_json_format", "polars.io.json_read_options"),
    ("JsonReader.with_projection", "polars.io.json_read_options"),
    ("JsonReader.with_schema", "polars.io.json_read_options"),
    (
        "JsonReader.with_schema_overwrite",
        "polars.io.json_read_options",
    ),
    ("JsonWriter.finish", "polars.dataframe.write_json"),
    ("JsonWriter.new", "polars.dataframe.write_json"),
    (
        "JsonWriter.with_json_format",
        "polars.dataframe.write_ndjson",
    ),
    (
        "ParquetReader.finish",
        "polars.dataframe.read_parquet_with_options",
    ),
    ("ParquetReader.get_metadata", "polars.io.parquet_metadata"),
    (
        "ParquetReader.new",
        "polars.dataframe.read_parquet_with_options",
    ),
    ("ParquetReader.num_rows", "polars.io.parquet_row_count"),
    (
        "ParquetReader.read_parallel",
        "polars.dataframe.read_parquet_with_options",
    ),
    ("ParquetReader.schema", "polars.dataframe.schema"),
    (
        "ParquetReader.set_low_memory",
        "polars.dataframe.read_parquet_with_options",
    ),
    ("ParquetReader.set_metadata", "polars.io.parquet_metadata"),
    (
        "ParquetReader.set_rechunk",
        "polars.dataframe.read_parquet_with_options",
    ),
    (
        "ParquetReader.with_arrow_schema_projection",
        "polars.dataframe.read_parquet_with_options",
    ),
    (
        "ParquetReader.with_columns",
        "polars.dataframe.read_parquet_with_options",
    ),
    (
        "ParquetReader.with_hive_partition_columns",
        "polars.lazy_frame.scan_parquet",
    ),
    (
        "ParquetReader.with_include_file_path",
        "polars.io.parquet_read_options",
    ),
    (
        "ParquetReader.with_projection",
        "polars.dataframe.read_parquet_with_options",
    ),
    (
        "ParquetReader.with_row_index",
        "polars.dataframe.read_parquet_with_options",
    ),
    (
        "ParquetReader.with_slice",
        "polars.dataframe.read_parquet_with_options",
    ),
    (
        "ParquetWriteOptions.compat_level",
        "polars.io.parquet_write_options",
    ),
    (
        "ParquetWriteOptions.to_writer",
        "polars.dataframe.write_parquet_with_options",
    ),
    (
        "ParquetWriter.batched",
        "polars.dataframe.write_parquet_with_options",
    ),
    (
        "ParquetWriter.finish",
        "polars.dataframe.write_parquet_with_options",
    ),
    ("ParquetWriter.new", "polars.io.parquet_write_options"),
    (
        "ParquetWriter.set_parallel",
        "polars.io.parquet_write_options",
    ),
    (
        "ParquetWriter.with_compression",
        "polars.io.parquet_write_options",
    ),
    (
        "ParquetWriter.with_context_info",
        "polars.io.parquet_write_options",
    ),
    (
        "ParquetWriter.with_data_page_size",
        "polars.io.parquet_write_options",
    ),
    (
        "ParquetWriter.with_key_value_metadata",
        "polars.io.parquet_write_options",
    ),
    (
        "ParquetWriter.with_row_group_size",
        "polars.io.parquet_write_options",
    ),
    (
        "ParquetWriter.with_statistics",
        "polars.io.parquet_write_options",
    ),
];

/// SQL callables mapped to their complete generated operation identities.
const SQL_MAPPINGS: &[(&str, &str)] = &[
    ("context.default", "polars.sql_context.new"),
    ("context.execute", "polars.sql_context.execute"),
    ("context.get_tables", "polars.sql_context.tables"),
    ("context.new", "polars.sql_context.new"),
    ("context.new_from_table_map", "polars.sql_context.register"),
    ("context.register", "polars.sql_context.register"),
    ("context.registry", "polars.sql.functions"),
    ("context.registry_mut", "polars.expr.define_udf"),
    ("context.unregister", "polars.sql_context.unregister"),
    ("context.with_function_registry", "polars.expr.define_udf"),
    ("extract_table_identifiers", "polars.sql.table_identifiers"),
    ("keywords.all_functions", "polars.sql.functions"),
    ("keywords.all_keywords", "polars.sql.keywords"),
    ("registry.contains", "polars.sql.functions"),
    ("registry.get_udf", "polars.expr.define_udf"),
    ("registry.register", "polars.expr.define_udf"),
    ("sql_expr", "polars.expr.sql"),
];

/// Builds the current DataFrame parity report from committed upstream metadata.
pub fn dataframe_api_parity_report() -> Result<ApiParityReport, String> {
    let upstream = include_str!("../../bindings/upstream/polars-dataframe-methods-0.55.2.txt");
    let metadata = include_str!("../generated/polars.DataFrame.native_boundary.json");
    api_parity_report(upstream, metadata, "polars.dataframe.", DATAFRAME_ALIASES)
}

/// Builds the current LazyFrame parity report from committed upstream metadata.
pub fn lazyframe_api_parity_report() -> Result<ApiParityReport, String> {
    let upstream = include_str!("../../bindings/upstream/polars-lazyframe-methods-0.55.2.txt");
    let metadata = include_str!("../generated/polars.DataFrame.native_boundary.json");
    api_parity_report(upstream, metadata, "polars.lazy_frame.", LAZY_FRAME_ALIASES)
}

/// Builds the current expression parity report from committed upstream metadata.
pub fn expression_api_parity_report() -> Result<ApiParityReport, String> {
    let upstream = include_str!("../../bindings/upstream/polars-expr-methods-0.55.2.txt");
    let metadata = include_str!("../generated/polars.DataFrame.native_boundary.json");
    api_parity_report(upstream, metadata, "polars.expr.", EXPR_ALIASES)
}

/// Builds the portable DataType parity report.
pub fn data_type_api_parity_report() -> Result<ApiParityReport, String> {
    let upstream =
        include_str!("../../bindings/upstream/polars-datatype-portable-methods-0.55.2.txt");
    let metadata = include_str!("../generated/polars.DataFrame.native_boundary.json");
    api_parity_report(upstream, metadata, "polars.dtype.", &[])
}

/// Builds the format I/O parity report across immutable option and terminal operations.
pub fn io_api_parity_report() -> Result<ApiParityReport, String> {
    let upstream = include_str!("../../bindings/upstream/polars-io-format-callables-0.55.2.txt");
    let metadata = include_str!("../generated/polars.DataFrame.native_boundary.json");
    mapped_api_parity_report(upstream, metadata, IO_MAPPINGS)
}

/// Builds the SQL parity report across SQL-context and expression operations.
pub fn sql_api_parity_report() -> Result<ApiParityReport, String> {
    let upstream = include_str!("../../bindings/upstream/polars-sql-callables-0.55.2.txt");
    let metadata = include_str!("../generated/polars.DataFrame.native_boundary.json");
    mapped_api_parity_report(upstream, metadata, SQL_MAPPINGS)
}

/// Builds the current Series parity report from committed upstream and native metadata.
///
/// Exact operation suffixes count as direct mappings. `SERIES_ALIASES`
/// contains reviewed API reshaping; all other identities remain pending.
pub fn series_api_parity_report() -> Result<ApiParityReport, String> {
    let upstream = include_str!("../../bindings/upstream/polars-series-methods-0.55.2.txt");
    let metadata = include_str!("../generated/polars.DataFrame.native_boundary.json");
    api_parity_report(upstream, metadata, "polars.series.", SERIES_ALIASES)
}

/// Compares newline-delimited upstream identities with generated metadata.
fn api_parity_report(
    upstream: &str,
    metadata: &str,
    prefix: &str,
    aliases: &[(&str, &str)],
) -> Result<ApiParityReport, String> {
    let upstream = upstream
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let metadata: NativeMetadata = serde_json::from_str(metadata)
        .map_err(|error| format!("invalid generated native metadata: {error}"))?;
    let operations = metadata
        .functions
        .into_iter()
        .filter_map(|function| function.operation.strip_prefix(prefix).map(str::to_string))
        .collect::<BTreeSet<_>>();
    let aliases = aliases.iter().copied().collect::<BTreeMap<_, _>>();

    let exact_count = upstream
        .iter()
        .filter(|method| operations.contains(method.as_str()))
        .count();
    let pending = upstream
        .iter()
        .filter(|method| {
            !operations.contains(method.as_str())
                && aliases
                    .get(method.as_str())
                    .is_none_or(|operation| !operations.contains(*operation))
        })
        .cloned()
        .collect::<Vec<_>>();

    let mapped_count = upstream.len() - pending.len();
    Ok(ApiParityReport {
        upstream_count: upstream.len(),
        mapped_count,
        exact_count,
        explicit_mapping_count: mapped_count - exact_count,
        pending,
    })
}

/// Compares upstream identities with explicit complete native operation names.
fn mapped_api_parity_report(
    upstream: &str,
    metadata: &str,
    mappings: &[(&str, &str)],
) -> Result<ApiParityReport, String> {
    let upstream = upstream
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let metadata: NativeMetadata = serde_json::from_str(metadata)
        .map_err(|error| format!("invalid generated native metadata: {error}"))?;
    let operations = metadata
        .functions
        .into_iter()
        .map(|function| function.operation)
        .collect::<BTreeSet<_>>();
    let mappings = mappings.iter().copied().collect::<BTreeMap<_, _>>();
    let pending = upstream
        .iter()
        .filter(|identity| {
            mappings
                .get(identity.as_str())
                .is_none_or(|operation| !operations.contains(*operation))
        })
        .cloned()
        .collect::<Vec<_>>();
    let mapped_count = upstream.len() - pending.len();
    Ok(ApiParityReport {
        upstream_count: upstream.len(),
        mapped_count,
        exact_count: 0,
        explicit_mapping_count: mapped_count,
        pending,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    /// Produces minimal generated metadata for inventory tests.
    fn metadata(operations: &[&str]) -> String {
        serde_json::json!({
            "functions": operations
                .iter()
                .map(|operation| serde_json::json!({"operation": operation}))
                .collect::<Vec<_>>()
        })
        .to_string()
    }

    #[test]
    fn report_counts_exact_alias_and_pending_identities() {
        let report = api_parity_report(
            "sum\nappend_owned\nmissing\nsum\n",
            &metadata(&["polars.series.sum", "polars.series.append"]),
            "polars.series.",
            &[("append_owned", "append")],
        )
        .expect("parity report");
        assert_eq!(report.upstream_count, 3);
        assert_eq!(report.mapped_count, 2);
        assert_eq!(report.exact_count, 1);
        assert_eq!(report.explicit_mapping_count, 1);
        assert_eq!(report.pending, ["missing"]);
    }

    #[test]
    fn report_rejects_invalid_metadata() {
        let error =
            api_parity_report("sum", "{", "polars.series.", &[]).expect_err("invalid metadata");
        assert!(error.starts_with("invalid generated native metadata:"));
    }

    #[test]
    fn committed_series_inventory_is_complete_and_sorted() {
        let upstream = include_str!("../../bindings/upstream/polars-series-methods-0.55.2.txt")
            .lines()
            .collect::<Vec<_>>();
        assert_eq!(upstream.len(), 193);
        assert!(upstream.windows(2).all(|pair| pair[0] < pair[1]));

        let report = series_api_parity_report().expect("committed parity report");
        assert_eq!(report.upstream_count, 193);
        assert_eq!(report.mapped_count, report.upstream_count);
        assert!(report.pending.is_empty());
    }

    #[test]
    fn committed_dataframe_and_lazyframe_inventories_are_complete_and_sorted() {
        let dataframe = include_str!("../../bindings/upstream/polars-dataframe-methods-0.55.2.txt")
            .lines()
            .collect::<Vec<_>>();
        let lazyframe = include_str!("../../bindings/upstream/polars-lazyframe-methods-0.55.2.txt")
            .lines()
            .collect::<Vec<_>>();
        assert_eq!(dataframe.len(), 165);
        assert_eq!(lazyframe.len(), 115);
        assert!(dataframe.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(lazyframe.windows(2).all(|pair| pair[0] < pair[1]));

        let dataframe = dataframe_api_parity_report().expect("DataFrame parity report");
        let lazyframe = lazyframe_api_parity_report().expect("LazyFrame parity report");
        assert_eq!(dataframe.mapped_count, dataframe.upstream_count);
        assert_eq!(lazyframe.mapped_count, lazyframe.upstream_count);
        assert!(dataframe.pending.is_empty());
        assert!(lazyframe.pending.is_empty());
    }

    #[test]
    fn committed_expression_and_sql_inventories_are_complete_and_sorted() {
        let expressions = include_str!("../../bindings/upstream/polars-expr-methods-0.55.2.txt")
            .lines()
            .collect::<Vec<_>>();
        let sql = include_str!("../../bindings/upstream/polars-sql-callables-0.55.2.txt")
            .lines()
            .collect::<Vec<_>>();
        assert_eq!(expressions.len(), 251);
        assert_eq!(sql.len(), 17);
        assert!(expressions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(sql.windows(2).all(|pair| pair[0] < pair[1]));

        let expressions = expression_api_parity_report().expect("Expr parity report");
        let sql = sql_api_parity_report().expect("SQL parity report");
        assert_eq!(expressions.mapped_count, expressions.upstream_count);
        assert_eq!(sql.mapped_count, sql.upstream_count);
        assert!(expressions.pending.is_empty());
        assert!(sql.pending.is_empty());
    }

    #[test]
    fn committed_data_type_inventory_is_complete_scoped_and_sorted() {
        let complete = include_str!("../../bindings/upstream/polars-datatype-methods-0.55.2.txt")
            .lines()
            .collect::<Vec<_>>();
        let portable =
            include_str!("../../bindings/upstream/polars-datatype-portable-methods-0.55.2.txt")
                .lines()
                .collect::<Vec<_>>();
        assert_eq!(complete.len(), 73);
        assert_eq!(portable.len(), 52);
        assert!(complete.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(portable.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(portable.iter().all(|method| complete.contains(method)));

        let report = data_type_api_parity_report().expect("DataType parity report");
        assert_eq!(report.upstream_count, 52);
        assert_eq!(report.mapped_count, report.upstream_count);
        assert!(report.pending.is_empty());
    }

    #[test]
    fn committed_io_inventory_is_complete_and_sorted() {
        let upstream =
            include_str!("../../bindings/upstream/polars-io-format-callables-0.55.2.txt")
                .lines()
                .collect::<Vec<_>>();
        assert_eq!(upstream.len(), 126);
        assert!(upstream.windows(2).all(|pair| pair[0] < pair[1]));

        let report = io_api_parity_report().expect("I/O parity report");
        assert_eq!(report.upstream_count, 126);
        assert_eq!(report.mapped_count, report.upstream_count);
        assert!(report.pending.is_empty());
    }
}
