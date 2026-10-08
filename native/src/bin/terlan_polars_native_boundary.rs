#![forbid(unsafe_code)]
//! Line-delimited JSON worker that executes typed Terlan Polars native operations.
//!
//! The process owns all Polars resources, validates opaque handle generations,
//! and returns stable protocol replies without exposing Rust or Polars pointers
//! to generated Terlan applications.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use terlan_polars_native::dataframe_extended::*;
use terlan_polars_native::expressions::*;
use terlan_polars_native::lazyframe_extended::*;
use terlan_polars_native::series::*;
use terlan_polars_native::series_extended::*;
use terlan_polars_native::{
    apply_expression_udf, asof_join, bottom_rows_by, clear_rows, collect_lazy,
    collect_lazy_streaming, cols_label, column_approx_unique_counts, column_lengths, column_maxima,
    column_means, column_medians, column_minima, column_non_null_counts, column_products,
    column_quantiles, column_series, column_stddevs, column_sums, column_unique_counts,
    column_variances, columns, concat_diagonal, concat_horizontal, concat_horizontal_extend,
    concat_vertical, concat_vertical_relaxed, csv_read_options, csv_read_options_with_column_names,
    csv_read_options_with_column_policies, csv_read_options_with_infer_schema_files,
    csv_write_options, data_color, data_types, dataframe_from_arrow_ipc, dataframe_tensor_packet,
    dataframe_to_arrow_ipc, define_expression_udf, describe, drop_columns,
    drop_columns_with_options, drop_nan_rows, drop_null_rows, estimated_size,
    estimated_size_with_unit, explode_columns, explode_columns_with_options,
    expression_udf_parameter, fill_nan_values, fill_null_values, fill_null_with_strategy,
    filter_all, filter_constraints, filter_eq, filter_expr, fmt_nanoplot, fmt_number, frames_equal,
    frames_equal_missing, from_rows, gather_rows, gather_rows_every, glimpse, group_agg,
    group_count, gt, head, height, ipc_custom_metadata, ipc_read_options, ipc_write_options,
    join as join_frames, join_where, join_with_coalesce, join_with_options, json_read_options,
    lazy_asof_join, lazy_bottom_rows_by, lazy_cache, lazy_clear_rows,
    lazy_column_approx_unique_counts, lazy_column_lengths, lazy_column_maxima, lazy_column_means,
    lazy_column_medians, lazy_column_minima, lazy_column_non_null_counts, lazy_column_products,
    lazy_column_quantiles, lazy_column_stddevs, lazy_column_sums, lazy_column_unique_counts,
    lazy_column_variances, lazy_describe_plan, lazy_describe_plan_dot, lazy_describe_plan_tree,
    lazy_drop_columns, lazy_drop_nan_rows, lazy_drop_null_rows, lazy_dynamic_group_agg,
    lazy_explode, lazy_explode_with_options, lazy_fill_nan_values, lazy_fill_null_values,
    lazy_filter_eq, lazy_filter_expr, lazy_first_row, lazy_gather_rows, lazy_group_agg, lazy_join,
    lazy_join_where, lazy_join_with_coalesce, lazy_join_with_options, lazy_last_row,
    lazy_left_join, lazy_limit, lazy_null_counts, lazy_profile, lazy_profile_plan,
    lazy_remove_where, lazy_rename_columns, lazy_reverse_rows, lazy_rolling_group_agg,
    lazy_scan_csv, lazy_scan_csv_with_storage_options, lazy_scan_ipc,
    lazy_scan_ipc_with_storage_options, lazy_scan_ndjson, lazy_scan_ndjson_with_storage_options,
    lazy_scan_parquet, lazy_scan_parquet_with_storage_options, lazy_schema, lazy_select,
    lazy_select_exprs, lazy_select_exprs_sequential, lazy_set_optimization,
    lazy_shift_and_fill_rows, lazy_shift_rows, lazy_slice_rows, lazy_sort, lazy_sort_rows_by,
    lazy_tail, lazy_top_rows_by, lazy_unique_rows, lazy_unnest, lazy_unpivot, lazy_with_columns,
    lazy_with_columns_sequential, lazy_with_row_index, lazy_without_optimizations, lazy_write_csv,
    lazy_write_csv_partitioned, lazy_write_ipc, lazy_write_ipc_partitioned, lazy_write_ndjson,
    lazy_write_ndjson_partitioned, lazy_write_parquet, lazy_write_parquet_partitioned, left_join,
    map_groups, null_counts, parquet_metadata, parquet_read_options, parquet_row_count,
    parquet_write_options, pivot, plot_bar_html, plot_bar_png, plot_bar_svg, plot_box_html,
    plot_box_png, plot_box_svg, plot_histogram_html, plot_histogram_png, plot_histogram_svg,
    plot_line_html, plot_line_png, plot_line_svg, plot_options, plot_scatter_html,
    plot_scatter_png, plot_scatter_svg, plot_with_html, plot_with_png, plot_with_svg, read_csv,
    read_csv_dates, read_csv_with_options, read_database_env, read_database_uri, read_ipc,
    read_ipc_with_options, read_json, read_json_with_options, read_ndjson, read_parquet,
    read_parquet_with_options, rechunk, remove_where, rename_columns, reverse_rows, row_hashes,
    row_hashes_seeded, row_is_duplicated, row_is_unique, rows, sample_rows_fraction,
    sample_rows_fraction_seeded, sample_rows_n, sample_rows_n_seeded, schema, select, select_exprs,
    select_exprs_sequential, series_cast, series_data_type, series_empty, series_equal,
    series_equal_missing, series_from_bools, series_from_dates, series_from_datetimes,
    series_from_floats, series_from_ints, series_from_nullable_bools, series_from_nullable_dates,
    series_from_nullable_datetimes, series_from_nullable_floats, series_from_nullable_ints,
    series_from_nullable_strings, series_from_strings, series_len, series_name, series_null_count,
    series_tensor_packet, series_to_frame, series_values, shift_and_fill_rows, shift_rows,
    show_versions, slice_rows, sort_by, sort_rows_by, sql_context_execute, sql_context_new,
    sql_context_register, sql_context_tables, sql_context_unregister, tab_header, tab_stub,
    table_html, tail, to_lazy, top_rows_by, transpose, unique_rows, unnest_columns, unpivot,
    update, upsample, width, with_columns_exprs, with_columns_sequential, with_plot_color,
    with_plot_dimensions, with_plot_facet, with_plot_title, with_row_index, write_csv,
    write_csv_with_options, write_ipc, write_ipc_with_options, write_json, write_ndjson,
    write_parquet, write_parquet_with_options, TerlanPolarsColumnSchema, TerlanPolarsDataFrame,
    TerlanPolarsError, TerlanPolarsFrameSet, TerlanPolarsLazyFrame, TerlanPolarsLazyFrameSet,
    TerlanPolarsScalar, TerlanPolarsSeries, TerlanPolarsSqlContext,
};

const DATAFRAME_TYPE: &str = "polars.DataFrame.DataFrame";
const DATAFRAME_SET_TYPE: &str = "polars.DataFrame.DataFrameSet";
const LAZY_FRAME_TYPE: &str = "polars.DataFrame.LazyFrame";
const LAZY_FRAME_SET_TYPE: &str = "polars.DataFrame.LazyFrameSet";
const SERIES_TYPE: &str = "polars.DataFrame.Series";
const SQL_CONTEXT_TYPE: &str = "polars.DataFrame.SqlContext";
const CREDIT_WINDOW: usize = 1;

#[path = "terlan_polars_native_boundary/data_type_worker.rs"]
mod data_type_worker;
#[path = "terlan_polars_native_boundary/dataframe_worker.rs"]
mod dataframe_worker;
#[path = "terlan_polars_native_boundary/io_worker.rs"]
mod io_worker;
#[path = "terlan_polars_native_boundary/lazyframe_worker.rs"]
mod lazyframe_worker;
#[path = "terlan_polars_native_boundary/series_worker.rs"]
mod series_worker;

fn main() -> ExitCode {
    run_loop(io::stdin().lock(), io::stdout())
}

fn run_loop(input: impl BufRead, mut output: impl Write) -> ExitCode {
    let mut worker = Worker::default();
    for line in input.lines() {
        let reply = match line {
            Ok(line) => worker.execute_line(line.trim_end()),
            Err(error) => protocol_error("native_boundary_read_error", &error.to_string()),
        };
        if writeln!(output, "{reply}").is_err() {
            return ExitCode::SUCCESS;
        }
        let _ = output.flush();
    }
    ExitCode::SUCCESS
}

struct Worker {
    owner: String,
    next_id: u64,
    frames: HashMap<u64, FrameResource>,
    frame_sets: HashMap<u64, FrameSetResource>,
    lazy_frames: HashMap<u64, LazyFrameResource>,
    lazy_frame_sets: HashMap<u64, LazyFrameSetResource>,
    series: HashMap<u64, SeriesResource>,
    sql_contexts: HashMap<u64, SqlContextResource>,
}

struct FrameResource {
    generation: u64,
    dataframe: TerlanPolarsDataFrame,
}

struct FrameSetResource {
    generation: u64,
    set: TerlanPolarsFrameSet,
}

struct LazyFrameResource {
    generation: u64,
    lazy_frame: TerlanPolarsLazyFrame,
}

struct LazyFrameSetResource {
    generation: u64,
    set: TerlanPolarsLazyFrameSet,
}

struct SeriesResource {
    generation: u64,
    series: TerlanPolarsSeries,
}

struct SqlContextResource {
    generation: u64,
    context: TerlanPolarsSqlContext,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeHandle {
    owner: String,
    id: u64,
    generation: u64,
    type_name: String,
}

impl Default for Worker {
    fn default() -> Self {
        static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
        let sequence = NEXT_OWNER.fetch_add(1, Ordering::Relaxed);
        Self {
            owner: format!("polars:{}:{sequence}", std::process::id()),
            next_id: 0,
            frames: HashMap::new(),
            frame_sets: HashMap::new(),
            lazy_frames: HashMap::new(),
            lazy_frame_sets: HashMap::new(),
            series: HashMap::new(),
            sql_contexts: HashMap::new(),
        }
    }
}

#[derive(Debug)]
enum NativeArg {
    Text(String),
    Bytes(Vec<u8>),
    Int(i64),
    Float(f64),
    Bool(bool),
    Strings(Vec<String>),
    Ints(Vec<i64>),
    Floats(Vec<f64>),
    Bools(Vec<bool>),
    NullableStrings(Vec<Option<String>>),
    NullableInts(Vec<Option<i64>>),
    NullableFloats(Vec<Option<f64>>),
    NullableBools(Vec<Option<bool>>),
    Nulls(usize),
    StringRows(Vec<Vec<String>>),
    Handle(NativeHandle),
}

impl Worker {
    fn execute_line(&mut self, line: &str) -> String {
        match parse_request(line) {
            Ok(request) => {
                let request_id = request.request_id;
                let payload = self.execute(request);
                format!("reply {request_id} {CREDIT_WINDOW} {payload}")
            }
            Err(error) => error,
        }
    }

    fn execute(&mut self, request: Request) -> String {
        match request.operation.as_str() {
            "polars.show_versions" => {
                if request.args.is_empty() {
                    format!("ok_string {}", STANDARD.encode(show_versions()))
                } else {
                    protocol_error("native_bad_args", "show_versions expects no arguments")
                }
            }
            "polars.dataframe.read_csv" => self.read_csv(request.args),
            "polars.dataframe.read_csv_dates" => self.read_csv_dates(request.args),
            "polars.dataframe.write_csv" => self.write_csv(request.args),
            "polars.dataframe.read_parquet" => {
                self.read_dataframe_path(request.args, "read_parquet", read_parquet)
            }
            "polars.dataframe.write_parquet" => {
                self.write_dataframe_path(request.args, "write_parquet", write_parquet)
            }
            "polars.dataframe.read_json" => {
                self.read_dataframe_path(request.args, "read_json", read_json)
            }
            "polars.dataframe.write_json" => {
                self.write_dataframe_path(request.args, "write_json", write_json)
            }
            "polars.dataframe.read_ndjson" => {
                self.read_dataframe_path(request.args, "read_ndjson", read_ndjson)
            }
            "polars.dataframe.write_ndjson" => {
                self.write_dataframe_path(request.args, "write_ndjson", write_ndjson)
            }
            "polars.dataframe.read_ipc" => {
                self.read_dataframe_path(request.args, "read_ipc", read_ipc)
            }
            "polars.dataframe.write_ipc" => {
                self.write_dataframe_path(request.args, "write_ipc", write_ipc)
            }
            "polars.io.csv_read_options" => self.csv_read_options(request.args),
            "polars.io.csv_read_options_with_column_names" => {
                self.csv_read_options_with_column_names(request.args)
            }
            "polars.io.csv_read_options_with_infer_schema_files" => {
                self.csv_read_options_with_infer_schema_files(request.args)
            }
            "polars.io.csv_read_options_with_column_policies" => {
                self.csv_read_options_with_column_policies(request.args)
            }
            "polars.io.csv_write_options" => self.csv_write_options(request.args),
            "polars.io.json_read_options" => self.json_read_options(request.args),
            "polars.io.parquet_read_options" => self.parquet_read_options(request.args),
            "polars.io.parquet_write_options" => self.parquet_write_options(request.args),
            "polars.io.ipc_read_options" => self.ipc_read_options(request.args),
            "polars.io.ipc_write_options" => self.ipc_write_options(request.args),
            "polars.io.ipc_custom_metadata" => {
                io_path_string_result(request.args, "ipc_custom_metadata", ipc_custom_metadata)
            }
            "polars.io.parquet_metadata" => {
                io_path_string_result(request.args, "parquet_metadata", parquet_metadata)
            }
            "polars.io.parquet_row_count" => {
                io_path_int_result(request.args, "parquet_row_count", parquet_row_count)
            }
            "polars.dataframe.read_csv_with_options" => {
                self.read_with_options(request.args, "read_csv_with_options", read_csv_with_options)
            }
            "polars.dataframe.write_csv_with_options" => self.write_with_options(
                request.args,
                "write_csv_with_options",
                write_csv_with_options,
            ),
            "polars.dataframe.read_json_with_options" => self.read_with_options(
                request.args,
                "read_json_with_options",
                read_json_with_options,
            ),
            "polars.dataframe.read_parquet_with_options" => self.read_with_options(
                request.args,
                "read_parquet_with_options",
                read_parquet_with_options,
            ),
            "polars.dataframe.write_parquet_with_options" => self.write_with_options(
                request.args,
                "write_parquet_with_options",
                write_parquet_with_options,
            ),
            "polars.dataframe.read_ipc_with_options" => {
                self.read_with_options(request.args, "read_ipc_with_options", read_ipc_with_options)
            }
            "polars.dataframe.write_ipc_with_options" => self.write_with_options(
                request.args,
                "write_ipc_with_options",
                write_ipc_with_options,
            ),
            "polars.dataframe.read_database_uri" => {
                self.read_database(request.args, "read_database_uri", read_database_uri)
            }
            "polars.dataframe.read_database_env" => {
                self.read_database(request.args, "read_database_env", read_database_env)
            }
            "polars.dataframe.from_arrow_ipc" => self.from_arrow_ipc(request.args),
            "polars.dataframe.to_arrow_ipc" => self.to_arrow_ipc(request.args),
            "polars.visualization.scatter_svg" => {
                self.plot_grouped_xy_svg(request.args, "plot_scatter_svg", plot_scatter_svg)
            }
            "polars.visualization.line_svg" => self.plot_line_svg(request.args),
            "polars.visualization.bar_svg" => {
                self.plot_grouped_xy_svg(request.args, "plot_bar_svg", plot_bar_svg)
            }
            "polars.visualization.histogram_svg" => self.plot_histogram_svg(request.args),
            "polars.visualization.box_svg" => {
                self.plot_grouped_xy_svg(request.args, "plot_box_svg", plot_box_svg)
            }
            "polars.visualization.scatter_html" => {
                self.plot_grouped_xy_svg(request.args, "plot_scatter_html", plot_scatter_html)
            }
            "polars.visualization.line_html" => self.plot_line_html(request.args),
            "polars.visualization.bar_html" => {
                self.plot_grouped_xy_svg(request.args, "plot_bar_html", plot_bar_html)
            }
            "polars.visualization.histogram_html" => self.plot_histogram_html(request.args),
            "polars.visualization.box_html" => {
                self.plot_grouped_xy_svg(request.args, "plot_box_html", plot_box_html)
            }
            "polars.visualization.scatter_png" => {
                self.plot_grouped_xy_svg(request.args, "plot_scatter_png", plot_scatter_png)
            }
            "polars.visualization.line_png" => self.plot_line_png(request.args),
            "polars.visualization.bar_png" => {
                self.plot_grouped_xy_svg(request.args, "plot_bar_png", plot_bar_png)
            }
            "polars.visualization.histogram_png" => self.plot_histogram_png(request.args),
            "polars.visualization.box_png" => {
                self.plot_grouped_xy_svg(request.args, "plot_box_png", plot_box_png)
            }
            "polars.visualization.options" => plot_options_new(request.args),
            "polars.visualization.options.title" => {
                plot_options_text(request.args, "with_plot_title", with_plot_title)
            }
            "polars.visualization.options.color" => {
                plot_options_text(request.args, "with_plot_color", with_plot_color)
            }
            "polars.visualization.options.facet" => plot_options_facet(request.args),
            "polars.visualization.options.dimensions" => plot_options_dimensions(request.args),
            "polars.visualization.render_svg" => {
                self.plot_with_options(request.args, "plot_with_svg", plot_with_svg)
            }
            "polars.visualization.render_html" => {
                self.plot_with_options(request.args, "plot_with_html", plot_with_html)
            }
            "polars.visualization.render_png" => {
                self.plot_with_options(request.args, "plot_with_png", plot_with_png)
            }
            "polars.table.gt" => table_style_new(request.args),
            "polars.table.tab_stub" => table_style_two_text(request.args, "tab_stub", tab_stub),
            "polars.table.cols_label" => table_style_labels(request.args),
            "polars.table.tab_header" => {
                table_style_three_text(request.args, "tab_header", tab_header)
            }
            "polars.table.fmt_number" => table_style_number(request.args),
            "polars.table.fmt_nanoplot" => {
                table_style_columns(request.args, "fmt_nanoplot", fmt_nanoplot)
            }
            "polars.table.data_color" => table_style_color(request.args),
            "polars.table.to_html" => self.table_html(request.args),
            "polars.dataframe.from_rows" => self.from_rows(request.args),
            "polars.dataframe.height" => self.height(request.args),
            "polars.dataframe.width" => self.width(request.args),
            "polars.dataframe.estimated_size" => self.estimated_size(request.args),
            "polars.dataframe.estimated_size_with_unit" => {
                self.estimated_size_with_unit(request.args)
            }
            "polars.dataframe.glimpse" => self.glimpse(request.args),
            "polars.dataframe.describe" => self.describe(request.args),
            "polars.dataframe.frames_equal" => self.frames_equal(request.args, false),
            "polars.dataframe.frames_equal_missing" => self.frames_equal(request.args, true),
            "polars.dataframe.sample_rows_n" => self.sample_rows_n(request.args, false),
            "polars.dataframe.sample_rows_n_seeded" => self.sample_rows_n(request.args, true),
            "polars.dataframe.sample_rows_fraction" => {
                self.sample_rows_fraction(request.args, false)
            }
            "polars.dataframe.sample_rows_fraction_seeded" => {
                self.sample_rows_fraction(request.args, true)
            }
            "polars.dataframe.row_is_unique" => self.row_mask(request.args, true),
            "polars.dataframe.row_is_duplicated" => self.row_mask(request.args, false),
            "polars.dataframe.row_hashes" => self.row_hashes(request.args, false),
            "polars.dataframe.row_hashes_seeded" => self.row_hashes(request.args, true),
            "polars.dataframe.columns" => self.columns(request.args),
            "polars.dataframe.rows" => self.rows(request.args),
            "polars.dataframe.schema" => self.schema(request.args),
            "polars.dataframe.dtypes" => self.data_types(request.args),
            "polars.dataframe.column_series" => self.column_series(request.args),
            "polars.dataframe.export_tensor_packet" => self.export_tensor_packet(request.args),
            "polars.series.export_tensor_packet" => self.export_series_tensor_packet(request.args),
            "polars.sql_context.new" => self.sql_context_new(request.args),
            "polars.sql_context.register" => self.sql_context_register(request.args),
            "polars.sql_context.unregister" => self.sql_context_unregister(request.args),
            "polars.sql_context.tables" => self.sql_context_tables(request.args),
            "polars.sql_context.execute" => self.sql_context_execute(request.args),
            "polars.sql.keywords" => self.sql_keywords(request.args),
            "polars.sql.functions" => self.sql_functions(request.args),
            "polars.sql.table_identifiers" => self.sql_table_identifiers(request.args),
            "polars.expr.sql" => expr_text_result_arg(request.args, "sql_expr", expr_sql),
            "polars.sql_context.dispose" => self.dispose_sql_context(request.args),
            "polars.series.from_strings" => self.series_from_strings(request.args),
            "polars.series.from_ints" => self.series_from_ints(request.args),
            "polars.series.from_floats" => self.series_from_floats(request.args),
            "polars.series.from_bools" => self.series_from_bools(request.args),
            "polars.series.from_int128_strings" => {
                self.series_from_wide_integer_strings(request.args, true)
            }
            "polars.series.from_uint128_strings" => {
                self.series_from_wide_integer_strings(request.args, false)
            }
            "polars.series.from_nullable_strings" => {
                self.series_from_nullable_strings(request.args)
            }
            "polars.series.from_nullable_ints" => self.series_from_nullable_ints(request.args),
            "polars.series.from_nullable_floats" => self.series_from_nullable_floats(request.args),
            "polars.series.from_nullable_bools" => self.series_from_nullable_bools(request.args),
            "polars.series.from_dates" => self.series_from_dates(request.args),
            "polars.series.from_nullable_dates" => self.series_from_nullable_dates(request.args),
            "polars.series.from_datetimes" => self.series_from_datetimes(request.args),
            "polars.series.from_nullable_datetimes" => {
                self.series_from_nullable_datetimes(request.args)
            }
            "polars.series.empty" => self.series_empty(request.args),
            "polars.series.empty_typed" => self.series_empty(request.args),
            "polars.series.empty_nested" => self.series_empty(request.args),
            "polars.series.name" => self.series_name(request.args),
            "polars.series.len" => self.series_len(request.args),
            "polars.series.null_count" => self.series_null_count(request.args),
            "polars.series.equal" => self.series_equal(request.args, false),
            "polars.series.equal_missing" => self.series_equal(request.args, true),
            "polars.series.data_type" => self.series_data_type(request.args),
            "polars.series.values" => self.series_values(request.args),
            "polars.series.clone" => self.series_clone(request.args),
            "polars.series.row_encode" => self.series_row_encode(request.args),
            "polars.series.to_arrow_ipc" => self.series_to_arrow_ipc(request.args),
            "polars.series.from_arrow_ipc" => self.series_from_arrow_ipc(request.args),
            "polars.series.int128_strings" => self.series_wide_integer_strings(request.args, true),
            "polars.series.uint128_strings" => {
                self.series_wide_integer_strings(request.args, false)
            }
            "polars.series.integer_values" => self.series_typed_values(request.args, "integer"),
            "polars.series.float_values" => self.series_typed_values(request.args, "float"),
            "polars.series.boolean_values" => self.series_typed_values(request.args, "boolean"),
            "polars.series.string_values" => self.series_typed_values(request.args, "string"),
            "polars.series.cast" => self.series_cast(request.args),
            "polars.series.cast_typed" => self.series_cast(request.args),
            "polars.series.cast_nested" => self.series_cast(request.args),
            "polars.series.to_frame" => self.series_to_frame(request.args),
            "polars.series.to_expr" => self.series_to_expr(request.args),
            "polars.series.bin_intervals"
            | "polars.series.bin_intervals_count"
            | "polars.series.bin_intervals_series"
            | "polars.series.bin_intervals_ints"
            | "polars.series.bin_intervals_strings"
            | "polars.series.bin_intervals_bools" => self.series_bin_intervals(request.args, false),
            "polars.series.bin_intervals_labeled"
            | "polars.series.bin_intervals_labeled_count"
            | "polars.series.bin_intervals_labeled_series"
            | "polars.series.bin_intervals_labeled_ints"
            | "polars.series.bin_intervals_labeled_strings"
            | "polars.series.bin_intervals_labeled_bools" => {
                self.series_bin_intervals(request.args, true)
            }
            "polars.series.bin_quantiles" | "polars.series.bin_quantiles_count" => {
                self.series_bin_quantiles(request.args, false)
            }
            "polars.series.bin_quantiles_labeled" | "polars.series.bin_quantiles_labeled_count" => {
                self.series_bin_quantiles(request.args, true)
            }
            "polars.series.bin_ranks" | "polars.series.bin_ranks_count" => {
                self.series_bin_ranks(request.args, false)
            }
            "polars.series.bin_ranks_labeled" | "polars.series.bin_ranks_labeled_count" => {
                self.series_bin_ranks(request.args, true)
            }
            "polars.series.dispose" => self.dispose_series(request.args),
            "polars.dataframe.filter_eq" => self.filter_eq(request.args),
            "polars.dataframe.sort_by" => self.sort_by(request.args),
            "polars.dataframe.sort_rows_by" => self.sort_rows_by(request.args),
            "polars.dataframe.top_rows_by" => self.top_rows_by(request.args),
            "polars.dataframe.bottom_rows_by" => self.bottom_rows_by(request.args),
            "polars.dataframe.group_count" => self.group_count(request.args),
            "polars.dataframe.lazy" => self.to_lazy(request.args),
            "polars.lazy_frame.scan_csv" => self.lazy_scan_csv(request.args),
            "polars.lazy_frame.scan_csv_with_storage_options" => {
                self.lazy_scan_csv_with_storage_options(request.args)
            }
            "polars.lazy_frame.scan_parquet" => {
                self.lazy_scan_path(request.args, "scan_parquet", lazy_scan_parquet)
            }
            "polars.lazy_frame.scan_parquet_with_storage_options" => self
                .lazy_scan_path_with_storage_options(
                    request.args,
                    "scan_parquet_with_storage_options",
                    lazy_scan_parquet_with_storage_options,
                ),
            "polars.lazy_frame.scan_ndjson" => {
                self.lazy_scan_path(request.args, "scan_ndjson", lazy_scan_ndjson)
            }
            "polars.lazy_frame.scan_ndjson_with_storage_options" => self
                .lazy_scan_path_with_storage_options(
                    request.args,
                    "scan_ndjson_with_storage_options",
                    lazy_scan_ndjson_with_storage_options,
                ),
            "polars.lazy_frame.scan_ipc" => {
                self.lazy_scan_path(request.args, "scan_ipc", lazy_scan_ipc)
            }
            "polars.lazy_frame.scan_ipc_with_storage_options" => self
                .lazy_scan_path_with_storage_options(
                    request.args,
                    "scan_ipc_with_storage_options",
                    lazy_scan_ipc_with_storage_options,
                ),
            "polars.lazy_frame.write_csv" => {
                self.lazy_write_path(request.args, "lazy_write_csv", lazy_write_csv)
            }
            "polars.lazy_frame.write_parquet" => {
                self.lazy_write_path(request.args, "lazy_write_parquet", lazy_write_parquet)
            }
            "polars.lazy_frame.write_ndjson" => {
                self.lazy_write_path(request.args, "lazy_write_ndjson", lazy_write_ndjson)
            }
            "polars.lazy_frame.write_ipc" => {
                self.lazy_write_path(request.args, "lazy_write_ipc", lazy_write_ipc)
            }
            "polars.lazy_frame.write_csv_partitioned" => self.lazy_write_partitioned(
                request.args,
                "lazy_write_csv_partitioned",
                lazy_write_csv_partitioned,
            ),
            "polars.lazy_frame.write_parquet_partitioned" => self.lazy_write_partitioned(
                request.args,
                "lazy_write_parquet_partitioned",
                lazy_write_parquet_partitioned,
            ),
            "polars.lazy_frame.write_ndjson_partitioned" => self.lazy_write_partitioned(
                request.args,
                "lazy_write_ndjson_partitioned",
                lazy_write_ndjson_partitioned,
            ),
            "polars.lazy_frame.write_ipc_partitioned" => self.lazy_write_partitioned(
                request.args,
                "lazy_write_ipc_partitioned",
                lazy_write_ipc_partitioned,
            ),
            "polars.lazy_frame.filter_eq" => self.lazy_filter_eq(request.args),
            "polars.lazy_frame.select" => self.lazy_select(request.args),
            "polars.lazy_frame.select_exprs" => self.lazy_select_exprs(request.args),
            "polars.lazy_frame.select_exprs_sequential" => {
                self.lazy_select_exprs_sequential(request.args)
            }
            "polars.lazy_frame.with_columns" => self.lazy_with_columns(request.args),
            "polars.lazy_frame.with_columns_sequential" => {
                self.lazy_with_columns_sequential(request.args)
            }
            "polars.lazy_frame.filter_expr" => self.lazy_filter_expr(request.args),
            "polars.lazy_frame.remove_where" => self.lazy_remove_where(request.args),
            "polars.lazy_frame.group_agg" => self.lazy_group_agg(request.args),
            "polars.lazy_frame.sort" => self.lazy_sort(request.args),
            "polars.lazy_frame.sort_rows_by" => self.lazy_sort_rows_by(request.args),
            "polars.lazy_frame.top_rows_by" => self.lazy_top_rows_by(request.args),
            "polars.lazy_frame.bottom_rows_by" => self.lazy_bottom_rows_by(request.args),
            "polars.lazy_frame.limit" => self.lazy_limit(request.args),
            "polars.lazy_frame.tail" => self.lazy_tail(request.args),
            "polars.lazy_frame.unique_rows" => self.lazy_unique_rows(request.args),
            "polars.lazy_frame.drop_null_rows" => self.lazy_drop_null_rows(request.args),
            "polars.lazy_frame.slice_rows" => self.lazy_slice_rows(request.args),
            "polars.lazy_frame.gather_rows" => self.lazy_gather_rows(request.args),
            "polars.lazy_frame.clear_rows" => self.lazy_clear_rows(request.args),
            "polars.lazy_frame.first_row" => self.lazy_first_row(request.args),
            "polars.lazy_frame.last_row" => self.lazy_last_row(request.args),
            "polars.lazy_frame.column_sums" => self.lazy_column_sums(request.args),
            "polars.lazy_frame.column_means" => self.lazy_column_means(request.args),
            "polars.lazy_frame.column_medians" => self.lazy_column_medians(request.args),
            "polars.lazy_frame.column_minima" => self.lazy_column_minima(request.args),
            "polars.lazy_frame.column_maxima" => self.lazy_column_maxima(request.args),
            "polars.lazy_frame.column_products" => self.lazy_column_products(request.args),
            "polars.lazy_frame.column_variances" => self.lazy_column_variances(request.args),
            "polars.lazy_frame.column_stddevs" => self.lazy_column_stddevs(request.args),
            "polars.lazy_frame.column_quantiles" => self.lazy_column_quantiles(request.args),
            "polars.lazy_frame.column_non_null_counts" => {
                self.lazy_column_non_null_counts(request.args)
            }
            "polars.lazy_frame.column_lengths" => self.lazy_column_lengths(request.args),
            "polars.lazy_frame.column_unique_counts" => {
                self.lazy_column_unique_counts(request.args)
            }
            "polars.lazy_frame.column_approx_unique_counts" => {
                self.lazy_column_approx_unique_counts(request.args)
            }
            "polars.lazy_frame.rename_columns" => self.lazy_rename_columns(request.args),
            "polars.lazy_frame.drop_columns" => self.lazy_drop_columns(request.args),
            "polars.lazy_frame.reverse_rows" => self.lazy_reverse_rows(request.args),
            "polars.lazy_frame.with_row_index" => self.lazy_with_row_index(request.args),
            "polars.lazy_frame.fill_null_values" => self.lazy_fill_null_values(request.args),
            "polars.lazy_frame.fill_nan_values" => self.lazy_fill_nan_values(request.args),
            "polars.lazy_frame.drop_nan_rows" => self.lazy_drop_nan_rows(request.args),
            "polars.lazy_frame.null_counts" => self.lazy_null_counts(request.args),
            "polars.lazy_frame.schema" => self.lazy_schema(request.args),
            "polars.lazy_frame.cast" => self.lazy_cast(request.args, false),
            "polars.lazy_frame.cast_all" => self.lazy_cast(request.args, true),
            "polars.lazy_frame.current_optimizations" => {
                self.lazy_current_optimizations(request.args)
            }
            "polars.lazy_frame.pivot" => self.lazy_pivot(request.args),
            "polars.lazy_frame.with_context" => self.lazy_with_context(request.args),
            "polars.lazy_frame.collect_all" => self.lazy_collect_all(request.args),
            "polars.lazy_frame.collect_all_with_engine" => {
                self.lazy_collect_all_with_engine(request.args)
            }
            "polars.lazy_frame_set.new" => self.lazyframe_set_new(request.args),
            "polars.lazy_frame_set.append" => self.lazyframe_set_append(request.args),
            "polars.lazy_frame_set.len" => self.lazyframe_set_len(request.args),
            "polars.lazy_frame_set.dispose" => self.dispose_lazy_frame_set(request.args),
            "polars.lazy_frame.cache" => self.lazy_cache(request.args),
            "polars.lazy_frame.without_optimizations" => {
                self.lazy_without_optimizations(request.args)
            }
            "polars.lazy_frame.set_optimization" => self.lazy_set_optimization(request.args),
            "polars.lazy_frame.shift_rows" => self.lazy_shift_rows(request.args),
            "polars.lazy_frame.shift_and_fill_rows" => self.lazy_shift_and_fill_rows(request.args),
            "polars.lazy_frame.left_join" => self.lazy_left_join(request.args),
            "polars.lazy_frame.join" => self.lazy_join(request.args),
            "polars.lazy_frame.join_with_options" => self.lazy_join_with_options(request.args),
            "polars.lazy_frame.join_with_coalesce" => self.lazy_join_with_coalesce(request.args),
            "polars.lazy_frame.join_where" => self.lazy_join_where(request.args),
            "polars.lazy_frame.asof_join" => self.lazy_asof_join(request.args),
            "polars.lazy_frame.dynamic_group_agg" => self.lazy_dynamic_group_agg(request.args),
            "polars.lazy_frame.rolling_group_agg" => self.lazy_rolling_group_agg(request.args),
            "polars.lazy_frame.explode" => self.lazy_explode(request.args),
            "polars.lazy_frame.unpivot" => self.lazy_unpivot(request.args),
            "polars.lazy_frame.unnest" => self.lazy_unnest(request.args),
            "polars.lazy_frame.describe_plan" => self.lazy_describe_plan(request.args),
            "polars.lazy_frame.describe_plan_tree" => self.lazy_describe_plan_tree(request.args),
            "polars.lazy_frame.describe_plan_dot" => self.lazy_describe_plan_dot(request.args),
            "polars.lazy_frame.profile_plan" => self.lazy_profile_plan(request.args),
            "polars.lazy_frame.profile" => self.lazy_profile(request.args),
            "polars.lazy_frame.collect" => self.collect_lazy(request.args),
            "polars.lazy_frame.collect_streaming" => self.collect_lazy_streaming(request.args),
            "polars.lazy_frame.collect_with_engine" => self.collect_lazy_with_engine(request.args),
            "polars.lazy_frame.dispose" => self.dispose_lazy(request.args),
            "polars.dataframe.select" => self.select(request.args),
            "polars.dataframe.head" => self.head(request.args),
            "polars.dataframe.tail" => self.tail(request.args),
            "polars.dataframe.clear_rows" => self.clear_rows(request.args),
            "polars.dataframe.rechunk_frame" => self.rechunk(request.args),
            "polars.dataframe.column_sums" => self.column_sums(request.args),
            "polars.dataframe.column_means" => self.column_means(request.args),
            "polars.dataframe.column_medians" => self.column_medians(request.args),
            "polars.dataframe.column_minima" => self.column_minima(request.args),
            "polars.dataframe.column_maxima" => self.column_maxima(request.args),
            "polars.dataframe.column_products" => self.column_products(request.args),
            "polars.dataframe.column_variances" => self.column_variances(request.args),
            "polars.dataframe.column_stddevs" => self.column_stddevs(request.args),
            "polars.dataframe.column_quantiles" => self.column_quantiles(request.args),
            "polars.dataframe.column_non_null_counts" => self.column_non_null_counts(request.args),
            "polars.dataframe.column_lengths" => self.column_lengths(request.args),
            "polars.dataframe.column_unique_counts" => self.column_unique_counts(request.args),
            "polars.dataframe.column_approx_unique_counts" => {
                self.column_approx_unique_counts(request.args)
            }
            "polars.dataframe.unique_rows" => self.unique_rows(request.args),
            "polars.dataframe.drop_null_rows" => self.drop_null_rows(request.args),
            "polars.dataframe.slice_rows" => self.slice_rows(request.args),
            "polars.dataframe.gather_rows" => self.gather_rows(request.args),
            "polars.dataframe.gather_every" => self.gather_rows_every(request.args),
            "polars.dataframe.rename_columns" => self.rename_columns(request.args),
            "polars.dataframe.drop_columns" => self.drop_columns(request.args),
            "polars.dataframe.drop_columns_with_options" => {
                self.drop_columns_with_options(request.args)
            }
            "polars.dataframe.reverse_rows" => self.reverse_rows(request.args),
            "polars.dataframe.with_row_index" => self.with_row_index(request.args),
            "polars.dataframe.fill_null_values" => self.fill_null_values(request.args),
            "polars.dataframe.fill_null" => self.fill_null_with_strategy(request.args),
            "polars.dataframe.fill_nan_values" => self.fill_nan_values(request.args),
            "polars.dataframe.drop_nan_rows" => self.drop_nan_rows(request.args),
            "polars.dataframe.null_counts" => self.null_counts(request.args),
            "polars.dataframe.shift_rows" => self.shift_rows(request.args),
            "polars.dataframe.shift_and_fill_rows" => self.shift_and_fill_rows(request.args),
            "polars.dataframe.dispose" => self.dispose(request.args),
            "polars.dataframe.empty_with_height" => self.dataframe_empty_with_height(request.args),
            "polars.dataframe.empty_with_schema" => self.dataframe_from_schema(request.args, false),
            "polars.dataframe.full_null" => self.dataframe_from_schema(request.args, true),
            "polars.dataframe.chunk_lengths" => self.dataframe_chunk_lengths(request.args),
            "polars.dataframe.shape" => self.dataframe_shape(request.args),
            "polars.dataframe.first_col_n_chunks" => self.dataframe_metadata_int(
                request.args,
                "dataframe_first_col_n_chunks",
                dataframe_first_col_n_chunks,
            ),
            "polars.dataframe.max_chunk_count" => self.dataframe_metadata_int(
                request.args,
                "dataframe_max_chunk_count",
                dataframe_max_chunk_count,
            ),
            "polars.dataframe.should_rechunk" => self.dataframe_metadata_bool(
                request.args,
                "dataframe_should_rechunk",
                dataframe_should_rechunk,
            ),
            "polars.dataframe.shape_has_zero" => self.dataframe_metadata_bool(
                request.args,
                "dataframe_shape_has_zero",
                dataframe_shape_has_zero,
            ),
            "polars.dataframe.shrink_to_fit" => self.dataframe_unary(
                request.args,
                "dataframe_shrink_to_fit",
                dataframe_shrink_to_fit,
            ),
            "polars.dataframe.schema_equal" => self.dataframe_schema_equal(request.args),
            "polars.dataframe.column_index" => self.dataframe_column_index(request.args),
            "polars.dataframe.column_at_index" => self.dataframe_column_at_index(request.args),
            "polars.dataframe.row" => self.dataframe_row(request.args),
            "polars.dataframe.split_before" => self.dataframe_split(request.args, true),
            "polars.dataframe.split_after" => self.dataframe_split(request.args, false),
            "polars.dataframe.insert_column" => {
                self.dataframe_column_mutation(request.args, "insert")
            }
            "polars.dataframe.replace_column" => {
                self.dataframe_column_mutation(request.args, "replace")
            }
            "polars.dataframe.with_column" => self.dataframe_column_mutation(request.args, "with"),
            "polars.dataframe.to_struct" => self.dataframe_to_struct(request.args),
            "polars.dataframe.serialize" => self.dataframe_serialize(request.args),
            "polars.dataframe.deserialize" => self.dataframe_deserialize(request.args),
            "polars.dataframe.partition_by" => self.dataframe_partition_by(request.args),
            "polars.dataframe.split_chunks" => self.dataframe_split_chunks(request.args, false),
            "polars.dataframe.iter_chunks" => self.dataframe_split_chunks(request.args, false),
            "polars.dataframe.iter_chunks_physical" => {
                self.dataframe_split_chunks(request.args, true)
            }
            "polars.dataframe.split_chunks_by_n" => self.dataframe_split_chunks_by_n(request.args),
            "polars.dataframe_set.len" => self.dataframe_set_len(request.args),
            "polars.dataframe_set.get" => self.dataframe_set_get(request.args),
            "polars.dataframe_set.dispose" => self.dispose_dataframe_set(request.args),
            "polars.series.rename" => self.series_rename(request.args),
            "polars.series.estimated_size" => self.series_size(request.args, false),
            "polars.series.estimated_size_with_unit" => {
                self.series_estimated_size_with_unit(request.args)
            }
            "polars.series.chunk_count" => self.series_size(request.args, true),
            "polars.series.chunk_lengths" => self.series_chunk_lengths(request.args),
            "polars.series.sorted_flag" => self.series_sorted_flag(request.args),
            "polars.series.clear" => self.series_unary(request.args, "series_clear", series_clear),
            "polars.series.head" => {
                self.series_length_transform(request.args, "series_head", series_head)
            }
            "polars.series.tail" => {
                self.series_length_transform(request.args, "series_tail", series_tail)
            }
            "polars.series.slice" => self.series_slice(request.args),
            "polars.series.split_before" => self.series_split(request.args, true),
            "polars.series.split_after" => self.series_split(request.args, false),
            "polars.series.rechunk" => {
                self.series_unary(request.args, "series_rechunk", series_rechunk)
            }
            "polars.series.shrink_to_fit" => {
                self.series_unary(request.args, "series_shrink_to_fit", series_shrink_to_fit)
            }
            "polars.series.reverse" => {
                self.series_unary(request.args, "series_reverse", series_reverse)
            }
            "polars.series.sort" => self.series_sort(request.args),
            "polars.series.unique_stable" => {
                self.series_unary(request.args, "series_unique_stable", series_unique_stable)
            }
            "polars.series.explode" => self.series_explode(request.args),
            "polars.series.fill_null" => self.series_fill_null(request.args, false),
            "polars.series.fill_null_limited" => self.series_fill_null(request.args, true),
            "polars.series.gather_every" => self.series_gather_every(request.args),
            "polars.series.shuffle" => self.series_shuffle(request.args, false),
            "polars.series.shuffle_seeded" => self.series_shuffle(request.args, true),
            "polars.series.sample_n" => self.series_sample_n(request.args, false),
            "polars.series.sample_n_seeded" => self.series_sample_n(request.args, true),
            "polars.series.sample_fraction" => self.series_sample_fraction(request.args, false),
            "polars.series.sample_fraction_seeded" => {
                self.series_sample_fraction(request.args, true)
            }
            "polars.series.append" => {
                self.series_binary(request.args, "series_append", series_append)
            }
            "polars.series.extend" => {
                self.series_binary(request.args, "series_extend", series_extend)
            }
            "polars.series.sum" => self.series_unary(request.args, "series_sum", series_sum),
            "polars.series.min" => self.series_unary(request.args, "series_min", series_min),
            "polars.series.max" => self.series_unary(request.args, "series_max", series_max),
            "polars.series.product" => {
                self.series_unary(request.args, "series_product", series_product)
            }
            "polars.series.mean" => self.series_unary(request.args, "series_mean", series_mean),
            "polars.series.add" => self.series_binary(request.args, "series_add", series_add),
            "polars.series.subtract" => {
                self.series_binary(request.args, "series_subtract", series_subtract)
            }
            "polars.series.multiply" => {
                self.series_binary(request.args, "series_multiply", series_multiply)
            }
            "polars.series.divide" => {
                self.series_binary(request.args, "series_divide", series_divide)
            }
            "polars.series.remainder" => {
                self.series_binary(request.args, "series_remainder", series_remainder)
            }
            "polars.series.bit_and" => {
                self.series_binary(request.args, "series_bit_and", series_bit_and)
            }
            "polars.series.bit_or" => {
                self.series_binary(request.args, "series_bit_or", series_bit_or)
            }
            "polars.series.bit_xor" => {
                self.series_binary(request.args, "series_bit_xor", series_bit_xor)
            }
            "polars.series.equal_values" => {
                self.series_binary(request.args, "series_equal_values", series_equal_values)
            }
            "polars.series.not_equal_values" => self.series_binary(
                request.args,
                "series_not_equal_values",
                series_not_equal_values,
            ),
            "polars.series.less_than" => {
                self.series_binary(request.args, "series_less_than", series_less_than)
            }
            "polars.series.less_than_or_equal" => self.series_binary(
                request.args,
                "series_less_than_or_equal",
                series_less_than_or_equal,
            ),
            "polars.series.greater_than" => {
                self.series_binary(request.args, "series_greater_than", series_greater_than)
            }
            "polars.series.greater_than_or_equal" => self.series_binary(
                request.args,
                "series_greater_than_or_equal",
                series_greater_than_or_equal,
            ),
            "polars.series.is_nan" => {
                self.series_unary(request.args, "series_is_nan", series_is_nan)
            }
            "polars.series.is_not_nan" => {
                self.series_unary(request.args, "series_is_not_nan", series_is_not_nan)
            }
            "polars.series.is_finite" => {
                self.series_unary(request.args, "series_is_finite", series_is_finite)
            }
            "polars.series.is_infinite" => {
                self.series_unary(request.args, "series_is_infinite", series_is_infinite)
            }
            "polars.series.full_null"
            | "polars.series.full_null_typed"
            | "polars.series.full_null_nested" => self.series_full_null(request.args, false),
            "polars.series.new_null" => self.series_full_null(request.args, true),
            "polars.series.set_sorted_flag" => self.series_set_sorted_flag(request.args),
            "polars.series.should_rechunk" => self.series_should_rechunk(request.args),
            "polars.series.strict_cast"
            | "polars.series.strict_cast_typed"
            | "polars.series.strict_cast_nested" => {
                self.series_cast_mode(request.args, "series_strict_cast", series_strict_cast)
            }
            "polars.series.non_strict_cast"
            | "polars.series.non_strict_cast_typed"
            | "polars.series.non_strict_cast_nested" => self.series_cast_mode(
                request.args,
                "series_non_strict_cast",
                series_non_strict_cast,
            ),
            "polars.series.overflowing_cast"
            | "polars.series.overflowing_cast_typed"
            | "polars.series.overflowing_cast_nested" => self.series_cast_mode(
                request.args,
                "series_overflowing_cast",
                series_overflowing_cast,
            ),
            "polars.series.cast_with_options" => self.series_cast_with_options(request.args),
            "polars.series.to_physical" => {
                self.series_unary(request.args, "series_to_physical", series_to_physical)
            }
            "polars.series.to_storage" => {
                self.series_unary(request.args, "series_to_storage", series_to_storage)
            }
            "polars.series.implode" => {
                self.series_unary(request.args, "series_implode", series_implode)
            }
            "polars.series.as_list" => {
                self.series_unary(request.args, "series_as_list", series_as_list)
            }
            "polars.series.to_unit_list" => {
                self.series_unary(request.args, "series_to_unit_list", series_to_unit_list)
            }
            "polars.series.from_cats_and_dtype" => self.series_from_cats_and_dtype(request.args),
            "polars.series.reshape_list" => self.series_reshape(request.args, false),
            "polars.series.reshape_array" => self.series_reshape(request.args, true),
            "polars.series.extend_string" => self.series_extend_constant(request.args, "string"),
            "polars.series.extend_int" => self.series_extend_constant(request.args, "int"),
            "polars.series.extend_float" => self.series_extend_constant(request.args, "float"),
            "polars.series.extend_bool" => self.series_extend_constant(request.args, "bool"),
            "polars.series.extend_null" => self.series_extend_constant(request.args, "null"),
            "polars.series.zip_with" => self.series_zip_with(request.args),
            "polars.series.serialize" => self.series_serialize(request.args),
            "polars.series.deserialize" => self.series_deserialize(request.args),
            "polars.series.not_equal_missing" => self.series_binary(
                request.args,
                "series_not_equal_missing",
                series_not_equal_missing,
            ),
            "polars.series.select_chunk" => self.series_select_chunk(request.args),
            "polars.series.leaf_values" => {
                self.series_unary(request.args, "series_leaf_values", series_leaf_values)
            }
            "polars.series.to_float" => {
                self.series_unary(request.args, "series_to_float", series_to_float)
            }
            "polars.expr.col" => expr_text_arg(request.args, "col", expr_col),
            "polars.expr.udf_parameter" => expression_udf_parameter_args(request.args),
            "polars.expr.define_udf" | "polars.expr.define_nested_udf" => {
                define_expression_udf_args(request.args)
            }
            "polars.expr.apply_udf" => apply_expression_udf_args(request.args),
            "polars.dtype.list" => expr_text_arg(request.args, "list_type", data_type_list),
            "polars.dtype.map" => expr_two_text_args(request.args, "map_type", data_type_map),
            "polars.dtype.nested_list" => expr_text_arg(request.args, "list_type", data_type_list),
            "polars.dtype.array" => expr_text_int_args(request.args, "array_type", data_type_array),
            "polars.dtype.nested_array" => {
                expr_text_int_args(request.args, "array_type", data_type_array)
            }
            "polars.dtype.decimal" => {
                expr_two_int_args(request.args, "decimal_type", data_type_decimal)
            }
            "polars.dtype.categorical" => expression_reply(data_type_categorical()),
            "polars.dtype.enum_type" => expr_strings_arg(request.args, "enum_type", data_type_enum),
            "polars.dtype.datetime_tz" => {
                expr_two_text_args(request.args, "datetime_type", data_type_datetime)
            }
            "polars.dtype.field" => {
                expr_two_text_args(request.args, "data_type_field", data_type_field)
            }
            "polars.dtype.nested_field" => {
                expr_two_text_args(request.args, "data_type_field", data_type_field)
            }
            "polars.dtype.struct_type" => {
                expr_strings_arg(request.args, "struct_type", data_type_struct)
            }
            "polars.dtype.extension" | "polars.dtype.nested_extension" => {
                expr_two_text_args(request.args, "extension_type", data_type_extension)
            }
            "polars.dtype.extension_metadata" | "polars.dtype.nested_extension_metadata" => {
                expr_three_text_args(
                    request.args,
                    "extension_type_with_metadata",
                    data_type_extension_metadata,
                )
            }
            operation if operation.starts_with("polars.dtype.") => {
                self.data_type_operation(operation, request.args)
            }
            "polars.expr.cols" => expr_strings_arg(request.args, "cols", expr_cols),
            "polars.expr.dtype_cols" => {
                expr_strings_arg(request.args, "dtype_cols", expr_dtype_cols)
            }
            "polars.expr.dtype_cols_typed" => {
                expr_strings_arg(request.args, "dtype_cols_typed", expr_dtype_cols)
            }
            "polars.expr.dtype_cols_nested" => {
                expr_strings_arg(request.args, "dtype_cols_typed", expr_dtype_cols)
            }
            "polars.expr.all" => expression_reply(expr_all()),
            "polars.expr.selector_all" => expression_reply(expr_selector_all()),
            "polars.expr.selector_names" => {
                expr_strings_arg(request.args, "selector_names", expr_selector_names)
            }
            "polars.expr.selector_numeric" => expression_reply(expr_selector_numeric()),
            "polars.expr.selector_string" => expression_reply(expr_selector_string()),
            "polars.expr.selector_starts_with" => expr_text_arg(
                request.args,
                "selector_starts_with",
                expr_selector_starts_with,
            ),
            "polars.expr.selector_contains" => {
                expr_text_arg(request.args, "selector_contains", expr_selector_contains)
            }
            "polars.expr.selector_first" => expression_reply(expr_selector_first()),
            "polars.expr.selector_union" => {
                expr_two_text_args(request.args, "selector_union", expr_selector_union)
            }
            "polars.expr.selector_intersection" => expr_two_text_args(
                request.args,
                "selector_intersection",
                expr_selector_intersection,
            ),
            "polars.expr.selector_difference" => expr_two_text_args(
                request.args,
                "selector_difference",
                expr_selector_difference,
            ),
            "polars.expr.selector_exclusive_or" => expr_two_text_args(
                request.args,
                "selector_exclusive_or",
                expr_selector_exclusive_or,
            ),
            "polars.expr.selector_complement" => expr_text_arg(
                request.args,
                "selector_complement",
                expr_selector_complement,
            ),
            "polars.expr.into_selector" => {
                expr_text_result_arg(request.args, "into_selector", expr_into_selector)
            }
            "polars.expr.try_into_selector" => {
                expr_text_result_arg(request.args, "try_into_selector", expr_into_selector)
            }
            "polars.expr.concat_list" => {
                expr_strings_arg(request.args, "concat_list", expr_concat_list)
            }
            "polars.expr.int_range" | "polars.expr.int_range_nested" => {
                expr_int_range_args(request.args)
            }
            "polars.expr.int_ranges" | "polars.expr.int_ranges_nested" => {
                expr_four_text_args(request.args, "int_ranges", expr_int_ranges)
            }
            "polars.expr.lit" => expr_scalar_arg(request.args),
            "polars.expr.extract_i64" => {
                expr_int_result_arg(request.args, "extract_i64", expr_extract_i64)
            }
            "polars.expr.extract_usize" => {
                expr_int_result_arg(request.args, "extract_usize", expr_extract_usize)
            }
            "polars.expr.null" => expression_reply(expr_null()),
            "polars.expr.date" => expr_text_arg(request.args, "date", expr_date),
            "polars.expr.date_components" => {
                expr_three_text_args(request.args, "date", expr_date_components)
            }
            "polars.expr.len" => expression_reply(expr_len()),
            "polars.expr.pi" => expression_reply(expr_pi()),
            "polars.expr.element" => expression_reply(expr_element()),
            "polars.expr.millennium" => expr_text_arg(request.args, "millennium", expr_millennium),
            "polars.expr.century" => expr_text_arg(request.args, "century", expr_century),
            "polars.expr.year" => expr_text_arg(request.args, "year", expr_year),
            "polars.expr.is_leap_year" => {
                expr_text_arg(request.args, "is_leap_year", expr_is_leap_year)
            }
            "polars.expr.iso_year" => expr_text_arg(request.args, "iso_year", expr_iso_year),
            "polars.expr.month" => expr_text_arg(request.args, "month", expr_month),
            "polars.expr.days_in_month" => {
                expr_text_arg(request.args, "days_in_month", expr_days_in_month)
            }
            "polars.expr.quarter" => expr_text_arg(request.args, "quarter", expr_quarter),
            "polars.expr.week" => expr_text_arg(request.args, "week", expr_week),
            "polars.expr.day" => expr_text_arg(request.args, "day", expr_day),
            "polars.expr.weekday" => expr_text_arg(request.args, "weekday", expr_weekday),
            "polars.expr.ordinal_day" => {
                expr_text_arg(request.args, "ordinal_day", expr_ordinal_day)
            }
            "polars.expr.time_of_day" => {
                expr_text_arg(request.args, "time_of_day", expr_time_of_day)
            }
            "polars.expr.calendar_date" => {
                expr_text_arg(request.args, "calendar_date", expr_calendar_date)
            }
            "polars.expr.local_datetime" => {
                expr_text_arg(request.args, "local_datetime", expr_local_datetime)
            }
            "polars.expr.hour" => expr_text_arg(request.args, "hour", expr_hour),
            "polars.expr.minute" => expr_text_arg(request.args, "minute", expr_minute),
            "polars.expr.second" => expr_text_arg(request.args, "second", expr_second),
            "polars.expr.millisecond" => {
                expr_text_arg(request.args, "millisecond", expr_millisecond)
            }
            "polars.expr.microsecond" => {
                expr_text_arg(request.args, "microsecond", expr_microsecond)
            }
            "polars.expr.nanosecond" => expr_text_arg(request.args, "nanosecond", expr_nanosecond),
            "polars.expr.month_start" => {
                expr_text_arg(request.args, "month_start", expr_month_start)
            }
            "polars.expr.month_end" => expr_text_arg(request.args, "month_end", expr_month_end),
            "polars.expr.date_range" => {
                expr_four_text_args(request.args, "date_range", expr_date_range)
            }
            "polars.expr.date_ranges" => {
                expr_four_text_args(request.args, "date_ranges", expr_date_ranges)
            }
            "polars.expr.datetime_range" => expr_datetime_range_args(request.args, false),
            "polars.expr.datetime_ranges" => expr_datetime_range_args(request.args, true),
            "polars.expr.time_range" => {
                expr_four_text_args(request.args, "time_range", expr_time_range)
            }
            "polars.expr.time_ranges" => {
                expr_four_text_args(request.args, "time_ranges", expr_time_ranges)
            }
            "polars.expr.duration" => expr_duration_args(request.args),
            "polars.expr.datetime" => expr_datetime_args(request.args),
            "polars.expr.repeat" => expr_two_text_args(request.args, "repeat", expr_repeat),
            "polars.expr.temporal_replace" => {
                expr_text_strings_args(request.args, "temporal_replace", expr_temporal_replace)
            }
            "polars.expr.convert_time_zone" => {
                expr_two_text_args(request.args, "convert_time_zone", expr_convert_time_zone)
            }
            "polars.expr.replace_time_zone" => {
                expr_four_text_args(request.args, "replace_time_zone", expr_replace_time_zone)
            }
            "polars.expr.timestamp_milliseconds" => expr_text_arg(
                request.args,
                "timestamp_milliseconds",
                expr_timestamp_milliseconds,
            ),
            "polars.expr.timestamp_microseconds" => expr_text_arg(
                request.args,
                "timestamp_microseconds",
                expr_timestamp_microseconds,
            ),
            "polars.expr.timestamp_nanoseconds" => expr_text_arg(
                request.args,
                "timestamp_nanoseconds",
                expr_timestamp_nanoseconds,
            ),
            "polars.expr.temporal_truncate" => {
                expr_two_text_args(request.args, "temporal_truncate", expr_temporal_truncate)
            }
            "polars.expr.temporal_round" => {
                expr_two_text_args(request.args, "temporal_round", expr_temporal_round)
            }
            "polars.expr.temporal_offset_by" => {
                expr_two_text_args(request.args, "temporal_offset_by", expr_temporal_offset_by)
            }
            "polars.expr.total_days" => {
                expr_text_bool_args(request.args, "total_days", expr_total_days)
            }
            "polars.expr.total_hours" => {
                expr_text_bool_args(request.args, "total_hours", expr_total_hours)
            }
            "polars.expr.total_minutes" => {
                expr_text_bool_args(request.args, "total_minutes", expr_total_minutes)
            }
            "polars.expr.total_seconds" => {
                expr_text_bool_args(request.args, "total_seconds", expr_total_seconds)
            }
            "polars.expr.total_milliseconds" => {
                expr_text_bool_args(request.args, "total_milliseconds", expr_total_milliseconds)
            }
            "polars.expr.total_microseconds" => {
                expr_text_bool_args(request.args, "total_microseconds", expr_total_microseconds)
            }
            "polars.expr.total_nanoseconds" => {
                expr_text_bool_args(request.args, "total_nanoseconds", expr_total_nanoseconds)
            }
            "polars.expr.mean" => expr_text_arg(request.args, "mean", expr_mean),
            "polars.expr.max" => expr_text_arg(request.args, "max", expr_max),
            "polars.expr.not_expr" => expr_text_arg(request.args, "not_expr", expr_not),
            "polars.expr.is_null" => expr_text_arg(request.args, "is_null", expr_is_null),
            "polars.expr.is_not_null" => {
                expr_text_arg(request.args, "is_not_null", expr_is_not_null)
            }
            "polars.expr.is_finite" => expr_text_arg(request.args, "is_finite", expr_is_finite),
            "polars.expr.is_infinite" => {
                expr_text_arg(request.args, "is_infinite", expr_is_infinite)
            }
            "polars.expr.is_nan" => expr_text_arg(request.args, "is_nan", expr_is_nan),
            "polars.expr.is_not_nan" => expr_text_arg(request.args, "is_not_nan", expr_is_not_nan),
            "polars.expr.is_first_distinct" => {
                expr_text_arg(request.args, "is_first_distinct", expr_is_first_distinct)
            }
            "polars.expr.is_last_distinct" => {
                expr_text_arg(request.args, "is_last_distinct", expr_is_last_distinct)
            }
            "polars.expr.is_unique" => expr_text_arg(request.args, "is_unique", expr_is_unique),
            "polars.expr.is_duplicated" => {
                expr_text_arg(request.args, "is_duplicated", expr_is_duplicated)
            }
            "polars.expr.has_nulls" => expr_text_arg(request.args, "has_nulls", expr_has_nulls),
            "polars.expr.any" => expr_text_bool_args(request.args, "any", expr_any),
            "polars.expr.all_true" => expr_text_bool_args(request.args, "all", expr_all_true),
            "polars.expr.is_empty" => expr_text_bool_args(request.args, "is_empty", expr_is_empty),
            "polars.expr.n_unique" => expr_text_arg(request.args, "n_unique", expr_n_unique),
            "polars.expr.approx_n_unique" => {
                expr_text_arg(request.args, "approx_n_unique", expr_approx_n_unique)
            }
            "polars.expr.value_counts" => {
                expr_text_arg(request.args, "value_counts", expr_value_counts)
            }
            "polars.expr.unique_stable" => {
                expr_text_arg(request.args, "unique_stable", expr_unique_stable)
            }
            "polars.expr.unique_counts" => {
                expr_text_arg(request.args, "unique_counts", expr_unique_counts)
            }
            "polars.expr.cos" => expr_text_arg(request.args, "cos", expr_cos),
            "polars.expr.cot" => expr_text_arg(request.args, "cot", expr_cot),
            "polars.expr.sin" => expr_text_arg(request.args, "sin", expr_sin),
            "polars.expr.tan" => expr_text_arg(request.args, "tan", expr_tan),
            "polars.expr.arccos" => expr_text_arg(request.args, "arccos", expr_arccos),
            "polars.expr.arcsin" => expr_text_arg(request.args, "arcsin", expr_arcsin),
            "polars.expr.arctan" => expr_text_arg(request.args, "arctan", expr_arctan),
            "polars.expr.arctan2" => expr_two_text_args(request.args, "arctan2", expr_arctan2),
            "polars.expr.cosh" => expr_text_arg(request.args, "cosh", expr_cosh),
            "polars.expr.sinh" => expr_text_arg(request.args, "sinh", expr_sinh),
            "polars.expr.tanh" => expr_text_arg(request.args, "tanh", expr_tanh),
            "polars.expr.arccosh" => expr_text_arg(request.args, "arccosh", expr_arccosh),
            "polars.expr.arcsinh" => expr_text_arg(request.args, "arcsinh", expr_arcsinh),
            "polars.expr.arctanh" => expr_text_arg(request.args, "arctanh", expr_arctanh),
            "polars.expr.degrees" => expr_text_arg(request.args, "degrees", expr_degrees),
            "polars.expr.radians" => expr_text_arg(request.args, "radians", expr_radians),
            "polars.expr.add" => expr_two_text_args(request.args, "add", expr_add),
            "polars.expr.subtract" => expr_two_text_args(request.args, "subtract", expr_subtract),
            "polars.expr.multiply" => expr_two_text_args(request.args, "multiply", expr_multiply),
            "polars.expr.divide" => expr_two_text_args(request.args, "divide", expr_divide),
            "polars.expr.true_div" => {
                expr_two_text_args(request.args, "true_div", expr_true_divide)
            }
            "polars.expr.floor_divide" => {
                expr_two_text_args(request.args, "floor_divide", expr_floor_divide)
            }
            "polars.expr.modulo" => expr_two_text_args(request.args, "modulo", expr_modulo),
            "polars.expr.pow" => expr_two_text_args(request.args, "pow", expr_pow),
            "polars.expr.neg" => expr_text_arg(request.args, "neg", expr_neg),
            "polars.expr.sqrt" => expr_text_arg(request.args, "sqrt", expr_sqrt),
            "polars.expr.cbrt" => expr_text_arg(request.args, "cbrt", expr_cbrt),
            "polars.expr.equal" => expr_two_text_args(request.args, "equal", expr_eq),
            "polars.expr.not_equal" => expr_two_text_args(request.args, "not_equal", expr_neq),
            "polars.expr.equal_missing" => {
                expr_two_text_args(request.args, "equal_missing", expr_eq_missing)
            }
            "polars.expr.not_equal_missing" => {
                expr_two_text_args(request.args, "not_equal_missing", expr_neq_missing)
            }
            "polars.expr.lt" => expr_two_text_args(request.args, "lt", expr_lt),
            "polars.expr.lte" => expr_two_text_args(request.args, "lte", expr_lte),
            "polars.expr.gt" => expr_two_text_args(request.args, "gt", expr_gt),
            "polars.expr.gte" => expr_two_text_args(request.args, "gte", expr_gte),
            "polars.expr.and_predicate" => {
                expr_two_text_args(request.args, "and_predicate", expr_and)
            }
            "polars.expr.or_predicate" => expr_two_text_args(request.args, "or_predicate", expr_or),
            "polars.expr.xor" => expr_two_text_args(request.args, "xor", expr_xor),
            "polars.expr.logical_and" => {
                expr_two_text_args(request.args, "logical_and", expr_logical_and)
            }
            "polars.expr.logical_or" => {
                expr_two_text_args(request.args, "logical_or", expr_logical_or)
            }
            "polars.expr.when_then_otherwise" => expr_three_text_args(
                request.args,
                "when_then_otherwise",
                expr_when_then_otherwise,
            ),
            "polars.expr.when_chain" => {
                expr_two_strings_text_args(request.args, "when_chain", expr_when_chain)
            }
            "polars.expr.alias" => expr_two_text_args(request.args, "alias", expr_alias),
            "polars.expr.suffix" => expr_two_text_args(request.args, "suffix", expr_suffix),
            "polars.expr.prefix" => expr_two_text_args(request.args, "prefix", expr_prefix),
            "polars.expr.keep_name" => expr_text_arg(request.args, "keep_name", expr_keep_name),
            "polars.expr.replace_name" => {
                expr_three_text_bool_args(request.args, "replace_name", expr_replace_name)
            }
            "polars.expr.lowercase_names" => {
                expr_text_arg(request.args, "lowercase_names", expr_lowercase_names)
            }
            "polars.expr.map_name" => expr_four_text_args(request.args, "map_name", expr_map_name),
            "polars.expr.map_field_names" => {
                expr_four_text_args(request.args, "map_field_names", expr_map_field_names)
            }
            "polars.expr.prefix_field_names" => {
                expr_two_text_args(request.args, "prefix_field_names", expr_prefix_field_names)
            }
            "polars.expr.suffix_field_names" => {
                expr_two_text_args(request.args, "suffix_field_names", expr_suffix_field_names)
            }
            "polars.expr.exclude" => expr_text_strings_args(request.args, "exclude", expr_exclude),
            "polars.expr.round" => expr_text_int_args(request.args, "round_to", expr_round),
            "polars.expr.round_with_mode" => {
                expr_text_int_text_args(request.args, "round_with_mode", expr_round_with_mode)
            }
            "polars.expr.round_sig_figs" => {
                expr_text_int_args(request.args, "round_sig_figs", expr_round_sig_figs)
            }
            "polars.expr.truncate" => expr_text_int_args(request.args, "truncate", expr_truncate),
            "polars.expr.floor" => expr_text_arg(request.args, "floor", expr_floor),
            "polars.expr.ceil" => expr_text_arg(request.args, "ceil", expr_ceil),
            "polars.expr.abs" => expr_text_arg(request.args, "abs", expr_abs),
            "polars.expr.sign" => expr_text_arg(request.args, "sign", expr_sign),
            "polars.expr.to_physical" => {
                expr_text_arg(request.args, "to_physical", expr_to_physical)
            }
            "polars.expr.set_sorted_flag" => {
                expr_three_text_args(request.args, "set_sorted_flag", expr_set_sorted_flag)
            }
            "polars.expr.is_sorted" => {
                expr_three_text_args(request.args, "is_sorted", expr_is_sorted)
            }
            "polars.expr.clip" => expr_three_text_args(request.args, "clip", expr_clip),
            "polars.expr.clip_min" => expr_two_text_args(request.args, "clip_min", expr_clip_min),
            "polars.expr.clip_max" => expr_two_text_args(request.args, "clip_max", expr_clip_max),
            "polars.expr.product" => expr_text_arg(request.args, "product", expr_product),
            "polars.expr.log" => expr_two_text_args(request.args, "log", expr_log),
            "polars.expr.log10" => expr_text_arg(request.args, "log10", expr_log10),
            "polars.expr.log1p" => expr_text_arg(request.args, "log1p", expr_log1p),
            "polars.expr.exp" => expr_text_arg(request.args, "exp", expr_exp),
            "polars.expr.entropy" => {
                expr_text_float_bool_args(request.args, "entropy", expr_entropy)
            }
            "polars.expr.skew" => expr_text_bool_args(request.args, "skew", expr_skew),
            "polars.expr.kurtosis" => {
                expr_text_two_bool_args(request.args, "kurtosis", expr_kurtosis)
            }
            "polars.expr.upper_bound" => {
                expr_text_arg(request.args, "upper_bound", expr_upper_bound)
            }
            "polars.expr.lower_bound" => {
                expr_text_arg(request.args, "lower_bound", expr_lower_bound)
            }
            "polars.expr.between" => expr_three_text_args(request.args, "between", expr_between),
            "polars.expr.between_closed" => {
                expr_four_text_args(request.args, "between_closed", expr_between_closed)
            }
            "polars.expr.is_in" => expr_two_text_bool_args(request.args, "is_in", expr_is_in),
            "polars.expr.is_close" => {
                expr_two_text_two_float_bool_args(request.args, "is_close", expr_is_close)
            }
            "polars.expr.split_first" => {
                expr_two_text_args(request.args, "split_first", expr_split_first)
            }
            "polars.expr.string_split" => {
                expr_two_text_args(request.args, "string_split", expr_string_split)
            }
            "polars.expr.date_format" => {
                expr_two_text_args(request.args, "date_format", expr_date_format)
            }
            "polars.expr.uppercase_names" => {
                expr_text_arg(request.args, "uppercase_names", expr_uppercase_names)
            }
            "polars.expr.cast" => expr_two_text_args(request.args, "cast", expr_cast),
            "polars.expr.cast_with_options" => {
                expr_three_text_args(request.args, "cast_with_options", expr_cast_with_options)
            }
            "polars.expr.cast_typed" => expr_two_text_args(request.args, "cast_to", expr_cast),
            "polars.expr.cast_nested" => expr_two_text_args(request.args, "cast_to", expr_cast),
            "polars.expr.strict_cast" => {
                expr_two_text_args(request.args, "strict_cast", expr_strict_cast)
            }
            "polars.expr.strict_cast_typed" => {
                expr_two_text_args(request.args, "strict_cast_to", expr_strict_cast)
            }
            "polars.expr.strict_cast_nested" => {
                expr_two_text_args(request.args, "strict_cast_to", expr_strict_cast)
            }
            "polars.expr.parse_datetime" => {
                expr_text_arg(request.args, "parse_datetime", expr_parse_datetime)
            }
            "polars.expr.string_len_bytes" => {
                expr_text_arg(request.args, "string_len_bytes", expr_string_len_bytes)
            }
            "polars.expr.string_len_chars" => {
                expr_text_arg(request.args, "string_len_chars", expr_string_len_chars)
            }
            "polars.expr.string_starts_with" => {
                expr_two_text_args(request.args, "string_starts_with", expr_string_starts_with)
            }
            "polars.expr.string_contains" => {
                expr_two_text_args(request.args, "string_contains", expr_string_contains)
            }
            "polars.expr.string_ends_with" => {
                expr_two_text_args(request.args, "string_ends_with", expr_string_ends_with)
            }
            "polars.expr.string_extract" => {
                expr_two_text_int_args(request.args, "string_extract", expr_string_extract)
            }
            "polars.expr.string_extract_all" => {
                expr_two_text_args(request.args, "string_extract_all", expr_string_extract_all)
            }
            "polars.expr.string_replace" => {
                expr_three_text_args(request.args, "string_replace", expr_string_replace)
            }
            "polars.expr.string_replace_all" => {
                expr_three_text_args(request.args, "string_replace_all", expr_string_replace_all)
            }
            "polars.expr.string_titlecase" => {
                expr_text_arg(request.args, "string_titlecase", expr_string_titlecase)
            }
            "polars.expr.string_lowercase" => {
                expr_text_arg(request.args, "string_lowercase", expr_string_lowercase)
            }
            "polars.expr.string_uppercase" => {
                expr_text_arg(request.args, "string_uppercase", expr_string_uppercase)
            }
            "polars.expr.string_strip_chars" => {
                expr_two_text_args(request.args, "string_strip_chars", expr_string_strip_chars)
            }
            "polars.expr.string_strip_chars_start" => expr_two_text_args(
                request.args,
                "string_strip_chars_start",
                expr_string_strip_chars_start,
            ),
            "polars.expr.string_strip_chars_end" => expr_two_text_args(
                request.args,
                "string_strip_chars_end",
                expr_string_strip_chars_end,
            ),
            "polars.expr.string_strip_prefix" => expr_two_text_args(
                request.args,
                "string_strip_prefix",
                expr_string_strip_prefix,
            ),
            "polars.expr.string_strip_suffix" => expr_two_text_args(
                request.args,
                "string_strip_suffix",
                expr_string_strip_suffix,
            ),
            "polars.expr.string_slice" => {
                expr_two_text_args(request.args, "string_slice", expr_string_slice)
            }
            "polars.expr.string_head" => {
                expr_two_text_args(request.args, "string_head", expr_string_head)
            }
            "polars.expr.string_tail" => {
                expr_two_text_args(request.args, "string_tail", expr_string_tail)
            }
            "polars.expr.string_contains_literal" => expr_two_text_args(
                request.args,
                "string_contains_literal",
                expr_string_contains_literal,
            ),
            "polars.expr.string_find_literal" => expr_two_text_args(
                request.args,
                "string_find_literal",
                expr_string_find_literal,
            ),
            "polars.expr.string_find" => {
                expr_two_text_bool_args(request.args, "string_find", expr_string_find)
            }
            "polars.expr.string_count_matches" => expr_two_text_bool_args(
                request.args,
                "string_count_matches",
                expr_string_count_matches,
            ),
            "polars.expr.string_pad_start" => {
                expr_three_text_args(request.args, "string_pad_start", expr_string_pad_start)
            }
            "polars.expr.string_pad_end" => {
                expr_three_text_args(request.args, "string_pad_end", expr_string_pad_end)
            }
            "polars.expr.string_zfill" => {
                expr_two_text_args(request.args, "string_zfill", expr_string_zfill)
            }
            "polars.expr.string_hex_encode" => {
                expr_text_arg(request.args, "string_hex_encode", expr_string_hex_encode)
            }
            "polars.expr.string_hex_decode" => {
                expr_text_bool_args(request.args, "string_hex_decode", expr_string_hex_decode)
            }
            "polars.expr.string_base64_encode" => expr_text_arg(
                request.args,
                "string_base64_encode",
                expr_string_base64_encode,
            ),
            "polars.expr.string_base64_decode" => expr_text_bool_args(
                request.args,
                "string_base64_decode",
                expr_string_base64_decode,
            ),
            "polars.expr.string_normalize" => {
                expr_two_text_args(request.args, "string_normalize", expr_string_normalize)
            }
            "polars.expr.string_reverse" => {
                expr_text_arg(request.args, "string_reverse", expr_string_reverse)
            }
            "polars.expr.string_escape_regex" => expr_text_arg(
                request.args,
                "string_escape_regex",
                expr_string_escape_regex,
            ),
            "polars.expr.null_count" => expr_text_arg(request.args, "null_count", expr_null_count),
            "polars.expr.drop_nulls" => expr_text_arg(request.args, "drop_nulls", expr_drop_nulls),
            "polars.expr.drop_nans" => expr_text_arg(request.args, "drop_nans", expr_drop_nans),
            "polars.expr.fill_null" => {
                expr_two_text_args(request.args, "fill_null", expr_fill_null)
            }
            "polars.expr.fill_null_forward" => {
                expr_text_arg(request.args, "fill_null_forward", expr_fill_null_forward)
            }
            "polars.expr.fill_null_backward" => {
                expr_text_arg(request.args, "fill_null_backward", expr_fill_null_backward)
            }
            "polars.expr.fill_null_with_strategy" => expr_two_text_int_args(
                request.args,
                "fill_null_with_strategy",
                expr_fill_null_with_strategy,
            ),
            "polars.expr.interpolate" => {
                expr_text_arg(request.args, "interpolate", expr_interpolate)
            }
            "polars.expr.interpolate_by" => {
                expr_two_text_args(request.args, "interpolate_by", expr_interpolate_by)
            }
            "polars.expr.peak_min" => expr_text_arg(request.args, "peak_min", expr_peak_min),
            "polars.expr.peak_max" => expr_text_arg(request.args, "peak_max", expr_peak_max),
            "polars.expr.rle" => expr_text_arg(request.args, "rle", expr_rle),
            "polars.expr.rle_id" => expr_text_arg(request.args, "rle_id", expr_rle_id),
            "polars.expr.reshape_expression" => {
                expr_text_ints_args(request.args, "reshape_expression", expr_reshape)
            }
            "polars.expr.ewm_sum" => expr_ewm_sum_or_mean_args(request.args, true),
            "polars.expr.ewm_mean" => expr_ewm_sum_or_mean_args(request.args, false),
            "polars.expr.ewm_std" => expr_ewm_statistic_args(request.args, false),
            "polars.expr.ewm_var" => expr_ewm_statistic_args(request.args, true),
            "polars.expr.ewm_mean_by" => {
                expr_three_text_args(request.args, "ewm_mean_by", expr_ewm_mean_by)
            }
            "polars.expr.ewm_sum_by" => {
                expr_three_text_args(request.args, "ewm_sum_by", expr_ewm_sum_by)
            }
            "polars.expr.fill_nan" => expr_two_text_args(request.args, "fill_nan", expr_fill_nan),
            "polars.expr.sum" => expr_text_arg(request.args, "sum", expr_sum),
            "polars.expr.count" => expr_text_arg(request.args, "count", expr_count),
            "polars.expr.len_values" => expr_text_arg(request.args, "len_values", expr_len_values),
            "polars.expr.first_non_null" => {
                expr_text_arg(request.args, "first_non_null", expr_first_non_null)
            }
            "polars.expr.last_non_null" => {
                expr_text_arg(request.args, "last_non_null", expr_last_non_null)
            }
            "polars.expr.item" => expr_text_bool_args(request.args, "item", expr_item),
            "polars.expr.implode" => expr_text_bool_args(request.args, "implode", expr_implode),
            "polars.expr.quantile" => expr_three_text_args(request.args, "quantile", expr_quantile),
            "polars.expr.mode" => expr_text_bool_args(request.args, "mode", expr_mode),
            "polars.expr.unique" => expr_text_arg(request.args, "unique", expr_unique),
            "polars.expr.arg_unique" => expr_text_arg(request.args, "arg_unique", expr_arg_unique),
            "polars.expr.arg_true" => expr_text_arg(request.args, "arg_true", expr_arg_true),
            "polars.expr.arg_min" => expr_text_arg(request.args, "arg_min", expr_arg_min),
            "polars.expr.arg_max" => expr_text_arg(request.args, "arg_max", expr_arg_max),
            "polars.expr.dot" => expr_two_text_args(request.args, "dot", expr_dot),
            "polars.expr.cum_count" => {
                expr_text_bool_args(request.args, "cum_count", expr_cum_count)
            }
            "polars.expr.cum_sum" => expr_text_bool_args(request.args, "cum_sum", expr_cum_sum),
            "polars.expr.cum_product" => {
                expr_text_bool_args(request.args, "cum_product", expr_cum_product)
            }
            "polars.expr.cum_min" => expr_text_bool_args(request.args, "cum_min", expr_cum_min),
            "polars.expr.cum_max" => expr_text_bool_args(request.args, "cum_max", expr_cum_max),
            "polars.expr.rolling_min" => {
                expr_text_two_int_bool_args(request.args, "rolling_min", expr_rolling_min)
            }
            "polars.expr.rolling_max" => {
                expr_text_two_int_bool_args(request.args, "rolling_max", expr_rolling_max)
            }
            "polars.expr.rolling_mean" => {
                expr_text_two_int_bool_args(request.args, "rolling_mean", expr_rolling_mean)
            }
            "polars.expr.rolling_sum" => {
                expr_text_two_int_bool_args(request.args, "rolling_sum", expr_rolling_sum)
            }
            "polars.expr.rolling_median" => {
                expr_text_two_int_bool_args(request.args, "rolling_median", expr_rolling_median)
            }
            "polars.expr.rolling_var" => {
                expr_text_two_int_bool_args(request.args, "rolling_var", expr_rolling_var)
            }
            "polars.expr.rolling_std" => {
                expr_text_two_int_bool_args(request.args, "rolling_std", expr_rolling_std)
            }
            "polars.expr.rolling_min_weighted" => expr_rolling_weighted_args(
                request.args,
                "rolling_min_weighted",
                expr_rolling_min_weighted,
            ),
            "polars.expr.rolling_max_weighted" => expr_rolling_weighted_args(
                request.args,
                "rolling_max_weighted",
                expr_rolling_max_weighted,
            ),
            "polars.expr.rolling_mean_weighted" => expr_rolling_weighted_args(
                request.args,
                "rolling_mean_weighted",
                expr_rolling_mean_weighted,
            ),
            "polars.expr.rolling_sum_weighted" => expr_rolling_weighted_args(
                request.args,
                "rolling_sum_weighted",
                expr_rolling_sum_weighted,
            ),
            "polars.expr.rolling_median_weighted" => expr_rolling_weighted_args(
                request.args,
                "rolling_median_weighted",
                expr_rolling_median_weighted,
            ),
            "polars.expr.rolling_var_weighted" => expr_rolling_weighted_args(
                request.args,
                "rolling_var_weighted",
                expr_rolling_var_weighted,
            ),
            "polars.expr.rolling_std_weighted" => expr_rolling_weighted_args(
                request.args,
                "rolling_std_weighted",
                expr_rolling_std_weighted,
            ),
            "polars.expr.rolling_quantile" => expr_rolling_quantile_args(request.args),
            "polars.expr.rolling_quantile_weighted" => {
                expr_rolling_quantile_weighted_args(request.args)
            }
            "polars.expr.rolling_rank" => expr_rolling_rank_args(request.args),
            "polars.expr.rolling_skew" => expr_rolling_skew_args(request.args),
            "polars.expr.rolling_kurtosis" => expr_rolling_kurtosis_args(request.args),
            "polars.expr.rolling_map" => expr_rolling_map_args(request.args),
            "polars.expr.rolling" => expr_five_text_args(request.args, "rolling", expr_rolling),
            "polars.expr.rolling_min_by" => {
                expr_rolling_by_args(request.args, "rolling_min_by", expr_rolling_min_by)
            }
            "polars.expr.rolling_max_by" => {
                expr_rolling_by_args(request.args, "rolling_max_by", expr_rolling_max_by)
            }
            "polars.expr.rolling_mean_by" => {
                expr_rolling_by_args(request.args, "rolling_mean_by", expr_rolling_mean_by)
            }
            "polars.expr.rolling_sum_by" => {
                expr_rolling_by_args(request.args, "rolling_sum_by", expr_rolling_sum_by)
            }
            "polars.expr.rolling_median_by" => {
                expr_rolling_by_args(request.args, "rolling_median_by", expr_rolling_median_by)
            }
            "polars.expr.rolling_var_by" => {
                expr_rolling_by_args(request.args, "rolling_var_by", expr_rolling_var_by)
            }
            "polars.expr.rolling_std_by" => {
                expr_rolling_by_args(request.args, "rolling_std_by", expr_rolling_std_by)
            }
            "polars.expr.rolling_quantile_by" => expr_rolling_quantile_by_args(request.args),
            "polars.expr.rolling_rank_by" => expr_rolling_rank_by_args(request.args),
            "polars.expr.std" => expr_text_int_args(request.args, "std", expr_std),
            "polars.expr.var" => expr_text_int_args(request.args, "var", expr_var),
            "polars.expr.min" => expr_text_arg(request.args, "min", expr_min),
            "polars.expr.median" => expr_text_arg(request.args, "median", expr_median),
            "polars.expr.min_by" => expr_two_text_args(request.args, "min_by", expr_min_by),
            "polars.expr.max_by" => expr_two_text_args(request.args, "max_by", expr_max_by),
            "polars.expr.nan_min" => expr_text_arg(request.args, "nan_min", expr_nan_min),
            "polars.expr.nan_max" => expr_text_arg(request.args, "nan_max", expr_nan_max),
            "polars.expr.histogram_auto" => {
                expr_text_two_bool_args(request.args, "histogram_auto", expr_histogram_auto)
            }
            "polars.expr.histogram_count" => {
                expr_text_int_two_bool_args(request.args, "histogram_count", expr_histogram_count)
            }
            "polars.expr.histogram_bins" => {
                expr_two_text_two_bool_args(request.args, "histogram_bins", expr_histogram_bins)
            }
            "polars.expr.fold_sum" => {
                expr_text_strings_args(request.args, "fold_sum", expr_fold_sum)
            }
            "polars.expr.fold_product" => {
                expr_text_strings_args(request.args, "fold_product", expr_fold_product)
            }
            "polars.expr.sum_horizontal" => {
                expr_strings_arg(request.args, "sum_horizontal", expr_sum_horizontal)
            }
            "polars.expr.all_horizontal" => {
                expr_strings_arg(request.args, "all_horizontal", expr_all_horizontal)
            }
            "polars.expr.any_horizontal" => {
                expr_strings_arg(request.args, "any_horizontal", expr_any_horizontal)
            }
            "polars.expr.concat_string" => {
                expr_strings_text_args(request.args, "concat_string", expr_concat_string)
            }
            "polars.expr.first" => expr_text_arg(request.args, "first", expr_first),
            "polars.expr.last" => expr_text_arg(request.args, "last", expr_last),
            "polars.expr.sort_ascending" => {
                expr_text_arg(request.args, "sort_ascending", expr_sort_ascending)
            }
            "polars.expr.rank_dense_descending" => expr_text_arg(
                request.args,
                "rank_dense_descending",
                expr_rank_dense_descending,
            ),
            "polars.expr.rank" => expr_two_text_bool_args(request.args, "rank", expr_rank),
            "polars.expr.rank_random" => expr_rank_random_args(request.args),
            "polars.expr.top_k" => expr_two_text_args(request.args, "top_k", expr_top_k),
            "polars.expr.bottom_k" => expr_two_text_args(request.args, "bottom_k", expr_bottom_k),
            "polars.expr.top_k_by" => expr_top_k_by_args(request.args, false),
            "polars.expr.bottom_k_by" => expr_top_k_by_args(request.args, true),
            "polars.expr.replace" => expr_three_text_args(request.args, "replace", expr_replace),
            "polars.expr.replace_strict" => {
                expr_three_text_args(request.args, "replace_strict", expr_replace_strict)
            }
            "polars.expr.replace_or_default" => {
                expr_four_text_args(request.args, "replace_or_default", expr_replace_or_default)
            }
            "polars.expr.cut" => expr_cut_args(request.args, false),
            "polars.expr.cut_labeled" => expr_cut_labeled_args(request.args, false),
            "polars.expr.qcut" => expr_cut_args(request.args, true),
            "polars.expr.qcut_labeled" => expr_cut_labeled_args(request.args, true),
            "polars.expr.qcut_equal_frequency" => {
                expr_qcut_equal_frequency_args(request.args, false)
            }
            "polars.expr.qcut_equal_frequency_labeled" => {
                expr_qcut_equal_frequency_args(request.args, true)
            }
            "polars.expr.bin_intervals"
            | "polars.expr.bin_intervals_count"
            | "polars.expr.bin_intervals_series"
            | "polars.expr.bin_intervals_ints"
            | "polars.expr.bin_intervals_strings"
            | "polars.expr.bin_intervals_bools" => self.expr_bin_intervals(request.args, false),
            "polars.expr.bin_intervals_labeled"
            | "polars.expr.bin_intervals_labeled_count"
            | "polars.expr.bin_intervals_labeled_series"
            | "polars.expr.bin_intervals_labeled_ints"
            | "polars.expr.bin_intervals_labeled_strings"
            | "polars.expr.bin_intervals_labeled_bools" => {
                self.expr_bin_intervals(request.args, true)
            }
            "polars.expr.bin_quantiles" | "polars.expr.bin_quantiles_count" => {
                expr_bin_quantiles_args(request.args, false)
            }
            "polars.expr.bin_quantiles_labeled" | "polars.expr.bin_quantiles_labeled_count" => {
                expr_bin_quantiles_args(request.args, true)
            }
            "polars.expr.bin_ranks" | "polars.expr.bin_ranks_count" => {
                expr_bin_ranks_args(request.args, false)
            }
            "polars.expr.bin_ranks_labeled" | "polars.expr.bin_ranks_labeled_count" => {
                expr_bin_ranks_args(request.args, true)
            }
            "polars.expr.explode" => expr_text_arg(request.args, "explode", expr_explode),
            "polars.expr.shuffle" => expr_text_arg(request.args, "shuffle", expr_shuffle),
            "polars.expr.shuffle_seeded" => {
                expr_text_int_args(request.args, "shuffle_seeded", expr_shuffle_seeded)
            }
            "polars.expr.sample_n" => {
                expr_two_text_two_bool_args(request.args, "sample_n", expr_sample_n)
            }
            "polars.expr.sample_n_seeded" => expr_two_text_two_bool_int_args(
                request.args,
                "sample_n_seeded",
                expr_sample_n_seeded,
            ),
            "polars.expr.sample_fraction" => {
                expr_two_text_two_bool_args(request.args, "sample_fraction", expr_sample_fraction)
            }
            "polars.expr.sample_fraction_seeded" => expr_two_text_two_bool_int_args(
                request.args,
                "sample_fraction_seeded",
                expr_sample_fraction_seeded,
            ),
            "polars.expr.hash" => expr_text_int_args(request.args, "hash", expr_hash),
            "polars.expr.bitwise_count_ones" => {
                expr_text_arg(request.args, "bitwise_count_ones", expr_bitwise_count_ones)
            }
            "polars.expr.bitwise_count_zeros" => expr_text_arg(
                request.args,
                "bitwise_count_zeros",
                expr_bitwise_count_zeros,
            ),
            "polars.expr.bitwise_leading_ones" => expr_text_arg(
                request.args,
                "bitwise_leading_ones",
                expr_bitwise_leading_ones,
            ),
            "polars.expr.bitwise_leading_zeros" => expr_text_arg(
                request.args,
                "bitwise_leading_zeros",
                expr_bitwise_leading_zeros,
            ),
            "polars.expr.bitwise_trailing_ones" => expr_text_arg(
                request.args,
                "bitwise_trailing_ones",
                expr_bitwise_trailing_ones,
            ),
            "polars.expr.bitwise_trailing_zeros" => expr_text_arg(
                request.args,
                "bitwise_trailing_zeros",
                expr_bitwise_trailing_zeros,
            ),
            "polars.expr.bitwise_and" => {
                expr_text_arg(request.args, "bitwise_and", expr_bitwise_and)
            }
            "polars.expr.bitwise_or" => expr_text_arg(request.args, "bitwise_or", expr_bitwise_or),
            "polars.expr.bitwise_xor" => {
                expr_text_arg(request.args, "bitwise_xor", expr_bitwise_xor)
            }
            "polars.expr.list_len" => expr_text_arg(request.args, "list_len", expr_list_len),
            "polars.expr.list_to_map" => {
                expr_text_arg(request.args, "list_to_map", expr_list_to_map)
            }
            "polars.expr.map_entries" => {
                expr_text_arg(request.args, "map_entries", expr_map_entries)
            }
            "polars.expr.map_keys" => expr_text_arg(request.args, "map_keys", expr_map_keys),
            "polars.expr.map_values" => expr_text_arg(request.args, "map_values", expr_map_values),
            "polars.expr.map_len" => expr_text_arg(request.args, "map_len", expr_map_len),
            "polars.expr.map_contains_key" => {
                expr_two_text_args(request.args, "map_contains_key", expr_map_contains_key)
            }
            "polars.expr.map_get" => expr_two_text_args(request.args, "map_get", expr_map_get),
            "polars.expr.list_sum" => expr_text_arg(request.args, "list_sum", expr_list_sum),
            "polars.expr.list_mean" => expr_text_arg(request.args, "list_mean", expr_list_mean),
            "polars.expr.list_min" => expr_text_arg(request.args, "list_min", expr_list_min),
            "polars.expr.list_max" => expr_text_arg(request.args, "list_max", expr_list_max),
            "polars.expr.list_sort" => expr_text_arg(request.args, "list_sort", expr_list_sort),
            "polars.expr.list_get" => expr_two_text_args(request.args, "list_get", expr_list_get),
            "polars.expr.list_contains" => {
                expr_two_text_args(request.args, "list_contains", expr_list_contains)
            }
            "polars.expr.list_std" => expr_text_int_args(request.args, "list_std", expr_list_std),
            "polars.expr.list_var" => expr_text_int_args(request.args, "list_var", expr_list_var),
            "polars.expr.list_median" => {
                expr_text_arg(request.args, "list_median", expr_list_median)
            }
            "polars.expr.list_first" => expr_text_arg(request.args, "list_first", expr_list_first),
            "polars.expr.list_last" => expr_text_arg(request.args, "list_last", expr_list_last),
            "polars.expr.list_arg_min" => {
                expr_text_arg(request.args, "list_arg_min", expr_list_arg_min)
            }
            "polars.expr.list_arg_max" => {
                expr_text_arg(request.args, "list_arg_max", expr_list_arg_max)
            }
            "polars.expr.list_join" => {
                expr_two_text_bool_args(request.args, "list_join", expr_list_join)
            }
            "polars.expr.list_shift" => {
                expr_two_text_args(request.args, "list_shift", expr_list_shift)
            }
            "polars.expr.list_slice" => {
                expr_three_text_args(request.args, "list_slice", expr_list_slice)
            }
            "polars.expr.list_head" => {
                expr_two_text_args(request.args, "list_head", expr_list_head)
            }
            "polars.expr.list_tail" => {
                expr_two_text_args(request.args, "list_tail", expr_list_tail)
            }
            "polars.expr.list_to_array" => {
                expr_text_int_args(request.args, "list_to_array", expr_list_to_array)
            }
            "polars.expr.list_eval" => {
                expr_two_text_args(request.args, "list_eval", expr_list_eval)
            }
            "polars.expr.list_agg" => expr_two_text_args(request.args, "list_agg", expr_list_agg),
            "polars.expr.list_sort_with" => {
                expr_text_three_bool_args(request.args, "list_sort_with", expr_list_sort_with)
            }
            "polars.expr.list_get_with" => {
                expr_two_text_bool_args(request.args, "list_get_with", expr_list_get_with)
            }
            "polars.expr.list_contains_with" => {
                expr_two_text_bool_args(request.args, "list_contains_with", expr_list_contains_with)
            }
            "polars.expr.list_drop_nulls" => {
                expr_text_arg(request.args, "list_drop_nulls", expr_list_drop_nulls)
            }
            "polars.expr.list_sample_n" => {
                expr_two_text_two_bool_args(request.args, "list_sample_n", expr_list_sample_n)
            }
            "polars.expr.list_sample_n_seeded" => expr_two_text_two_bool_int_args(
                request.args,
                "list_sample_n_seeded",
                expr_list_sample_n_seeded,
            ),
            "polars.expr.list_sample_fraction" => expr_two_text_two_bool_args(
                request.args,
                "list_sample_fraction",
                expr_list_sample_fraction,
            ),
            "polars.expr.list_sample_fraction_seeded" => expr_two_text_two_bool_int_args(
                request.args,
                "list_sample_fraction_seeded",
                expr_list_sample_fraction_seeded,
            ),
            "polars.expr.list_gather" => {
                expr_two_text_bool_args(request.args, "list_gather", expr_list_gather)
            }
            "polars.expr.list_gather_every" => {
                expr_three_text_args(request.args, "list_gather_every", expr_list_gather_every)
            }
            "polars.expr.list_diff_drop" => {
                expr_text_int_args(request.args, "list_diff_drop", expr_list_diff_drop)
            }
            "polars.expr.list_diff_ignore" => {
                expr_text_int_args(request.args, "list_diff_ignore", expr_list_diff_ignore)
            }
            "polars.expr.list_to_struct" => {
                expr_text_strings_args(request.args, "list_to_struct", expr_list_to_struct)
            }
            "polars.expr.list_count_matches" => {
                expr_two_text_args(request.args, "list_count_matches", expr_list_count_matches)
            }
            "polars.expr.list_set_union" => {
                expr_two_text_args(request.args, "list_set_union", expr_list_set_union)
            }
            "polars.expr.list_set_difference" => expr_two_text_args(
                request.args,
                "list_set_difference",
                expr_list_set_difference,
            ),
            "polars.expr.list_set_intersection" => expr_two_text_args(
                request.args,
                "list_set_intersection",
                expr_list_set_intersection,
            ),
            "polars.expr.list_set_symmetric_difference" => expr_two_text_args(
                request.args,
                "list_set_symmetric_difference",
                expr_list_set_symmetric_difference,
            ),
            "polars.expr.array_len" => expr_text_arg(request.args, "array_len", expr_array_len),
            "polars.expr.array_sum" => expr_text_arg(request.args, "array_sum", expr_array_sum),
            "polars.expr.array_mean" => expr_text_arg(request.args, "array_mean", expr_array_mean),
            "polars.expr.array_min" => expr_text_arg(request.args, "array_min", expr_array_min),
            "polars.expr.array_max" => expr_text_arg(request.args, "array_max", expr_array_max),
            "polars.expr.array_sort" => expr_text_arg(request.args, "array_sort", expr_array_sort),
            "polars.expr.array_to_list" => {
                expr_text_arg(request.args, "array_to_list", expr_array_to_list)
            }
            "polars.expr.array_get" => {
                expr_two_text_args(request.args, "array_get", expr_array_get)
            }
            "polars.expr.array_contains" => {
                expr_two_text_args(request.args, "array_contains", expr_array_contains)
            }
            "polars.expr.array_std" => {
                expr_text_int_args(request.args, "array_std", expr_array_std)
            }
            "polars.expr.array_var" => {
                expr_text_int_args(request.args, "array_var", expr_array_var)
            }
            "polars.expr.array_median" => {
                expr_text_arg(request.args, "array_median", expr_array_median)
            }
            "polars.expr.array_arg_min" => {
                expr_text_arg(request.args, "array_arg_min", expr_array_arg_min)
            }
            "polars.expr.array_arg_max" => {
                expr_text_arg(request.args, "array_arg_max", expr_array_arg_max)
            }
            "polars.expr.array_join" => {
                expr_two_text_bool_args(request.args, "array_join", expr_array_join)
            }
            "polars.expr.array_count_matches" => expr_two_text_args(
                request.args,
                "array_count_matches",
                expr_array_count_matches,
            ),
            "polars.expr.array_to_struct" => {
                expr_text_arg(request.args, "array_to_struct", expr_array_to_struct)
            }
            "polars.expr.array_slice" => {
                expr_three_text_bool_args(request.args, "array_slice", expr_array_slice)
            }
            "polars.expr.array_head" => {
                expr_two_text_bool_args(request.args, "array_head", expr_array_head)
            }
            "polars.expr.array_tail" => {
                expr_two_text_bool_args(request.args, "array_tail", expr_array_tail)
            }
            "polars.expr.array_shift" => {
                expr_two_text_args(request.args, "array_shift", expr_array_shift)
            }
            "polars.expr.array_explode" => {
                expr_text_two_bool_args(request.args, "array_explode", expr_array_explode)
            }
            "polars.expr.array_eval" => {
                expr_two_text_bool_args(request.args, "array_eval", expr_array_eval)
            }
            "polars.expr.array_agg" => {
                expr_two_text_args(request.args, "array_agg", expr_array_agg)
            }
            "polars.expr.array_sort_with" => {
                expr_text_three_bool_args(request.args, "array_sort_with", expr_array_sort_with)
            }
            "polars.expr.array_get_with" => {
                expr_two_text_bool_args(request.args, "array_get_with", expr_array_get_with)
            }
            "polars.expr.array_contains_with" => expr_two_text_bool_args(
                request.args,
                "array_contains_with",
                expr_array_contains_with,
            ),
            "polars.expr.categorical_categories" => expr_text_arg(
                request.args,
                "categorical_categories",
                expr_categorical_categories,
            ),
            "polars.expr.categorical_len_bytes" => expr_text_arg(
                request.args,
                "categorical_len_bytes",
                expr_categorical_len_bytes,
            ),
            "polars.expr.categorical_len_chars" => expr_text_arg(
                request.args,
                "categorical_len_chars",
                expr_categorical_len_chars,
            ),
            "polars.expr.categorical_starts_with" => expr_two_text_args(
                request.args,
                "categorical_starts_with",
                expr_categorical_starts_with,
            ),
            "polars.expr.categorical_ends_with" => expr_two_text_args(
                request.args,
                "categorical_ends_with",
                expr_categorical_ends_with,
            ),
            "polars.expr.categorical_slice" => {
                expr_text_two_int_args(request.args, "categorical_slice", expr_categorical_slice)
            }
            "polars.expr.categorical_slice_to_end" => expr_text_int_args(
                request.args,
                "categorical_slice_to_end",
                expr_categorical_slice_to_end,
            ),
            "polars.expr.binary_contains" => {
                expr_two_text_args(request.args, "binary_contains", expr_binary_contains)
            }
            "polars.expr.binary_starts_with" => {
                expr_two_text_args(request.args, "binary_starts_with", expr_binary_starts_with)
            }
            "polars.expr.binary_ends_with" => {
                expr_two_text_args(request.args, "binary_ends_with", expr_binary_ends_with)
            }
            "polars.expr.binary_size_bytes" => {
                expr_text_arg(request.args, "binary_size_bytes", expr_binary_size_bytes)
            }
            "polars.expr.binary_get" => {
                expr_two_text_args(request.args, "binary_get", expr_binary_get)
            }
            "polars.expr.binary_head" => {
                expr_two_text_args(request.args, "binary_head", expr_binary_head)
            }
            "polars.expr.binary_tail" => {
                expr_two_text_args(request.args, "binary_tail", expr_binary_tail)
            }
            "polars.expr.binary_slice" => {
                expr_three_text_args(request.args, "binary_slice", expr_binary_slice)
            }
            "polars.expr.binary_hex_decode" => {
                expr_text_bool_args(request.args, "binary_hex_decode", expr_binary_hex_decode)
            }
            "polars.expr.binary_hex_encode" => {
                expr_text_arg(request.args, "binary_hex_encode", expr_binary_hex_encode)
            }
            "polars.expr.binary_base64_decode" => expr_text_bool_args(
                request.args,
                "binary_base64_decode",
                expr_binary_base64_decode,
            ),
            "polars.expr.binary_base64_encode" => expr_text_arg(
                request.args,
                "binary_base64_encode",
                expr_binary_base64_encode,
            ),
            "polars.expr.binary_reinterpret_typed" | "polars.expr.binary_reinterpret_nested" => {
                expr_two_text_bool_args(request.args, "binary_reinterpret", expr_binary_reinterpret)
            }
            "polars.expr.filter_expression" => {
                expr_two_text_args(request.args, "filter_expression", expr_filter_expression)
            }
            "polars.expr.slice_expression" => {
                expr_three_text_args(request.args, "slice_expression", expr_slice_expression)
            }
            "polars.expr.append" => {
                expr_two_text_bool_args(request.args, "append_expression", expr_append)
            }
            "polars.expr.rechunk" => expr_text_arg(request.args, "rechunk", expr_rechunk),
            "polars.expr.arg_sort" => {
                expr_text_two_bool_args(request.args, "arg_sort", expr_arg_sort)
            }
            "polars.expr.index_of" => expr_two_text_args(request.args, "index_of", expr_index_of),
            "polars.expr.search_sorted" => {
                expr_three_text_bool_args(request.args, "search_sorted", expr_search_sorted)
            }
            "polars.expr.gather" => expr_two_text_bool_args(request.args, "gather", expr_gather),
            "polars.expr.get_expression" => {
                expr_two_text_bool_args(request.args, "get_expression", expr_get_expression)
            }
            "polars.expr.sort_with" => {
                expr_text_two_bool_args(request.args, "sort_with", expr_sort_with)
            }
            "polars.expr.sort_with_stability" => {
                expr_text_three_bool_args(request.args, "sort_with", expr_sort_with_stability)
            }
            "polars.expr.reverse" => expr_text_arg(request.args, "reverse", expr_reverse),
            "polars.expr.shift" => expr_two_text_args(request.args, "shift", expr_shift),
            "polars.expr.shift_and_fill" => {
                expr_three_text_args(request.args, "shift_and_fill", expr_shift_and_fill)
            }
            "polars.expr.repeat_by" => {
                expr_two_text_args(request.args, "repeat_by", expr_repeat_by)
            }
            "polars.expr.diff" => expr_three_text_args(request.args, "diff", expr_diff),
            "polars.expr.pct_change" => {
                expr_two_text_args(request.args, "pct_change", expr_pct_change)
            }
            "polars.expr.gather_every" => {
                expr_text_two_int_args(request.args, "gather_every", expr_gather_every)
            }
            "polars.expr.extend_constant" => {
                expr_three_text_args(request.args, "extend_constant", expr_extend_constant)
            }
            "polars.expr.sort_by_ascending" => {
                expr_text_strings_args(request.args, "sort_by_ascending", expr_sort_by_ascending)
            }
            "polars.expr.sort_by_descending" => {
                expr_text_strings_args(request.args, "sort_by_descending", expr_sort_by_descending)
            }
            "polars.expr.sort_by" => expr_sort_by_args(request.args),
            "polars.expr.over" => expr_text_strings_args(request.args, "over", expr_over),
            "polars.expr.over_explode" => {
                expr_text_strings_args(request.args, "over_explode", expr_over_explode)
            }
            "polars.expr.over_join" => {
                expr_text_strings_args(request.args, "over_join", expr_over_join)
            }
            "polars.expr.over_ordered" => {
                expr_over_options_args(request.args, "over_ordered", expr_over_ordered)
            }
            "polars.expr.over_with_options" => {
                expr_over_options_args(request.args, "over_with_options", expr_over_with_options)
            }
            "polars.expr.agg_groups" => expr_text_arg(request.args, "agg_groups", expr_agg_groups),
            "polars.expr.cumulative_eval" => {
                expr_two_text_int_args(request.args, "cumulative_eval", expr_cumulative_eval)
            }
            "polars.expr.head_expression" => {
                expr_text_int_args(request.args, "head_expression", expr_head_expression)
            }
            "polars.expr.tail_expression" => {
                expr_text_int_args(request.args, "tail_expression", expr_tail_expression)
            }
            "polars.expr.as_struct" => expr_strings_arg(request.args, "as_struct", expr_as_struct),
            "polars.expr.struct_field" => {
                expr_two_text_args(request.args, "struct_field", expr_struct_field)
            }
            "polars.expr.struct_field_at" => {
                expr_text_int_args(request.args, "struct_field_at", expr_struct_field_at)
            }
            "polars.expr.struct_fields" => {
                expr_text_strings_args(request.args, "struct_fields", expr_struct_fields)
            }
            "polars.expr.struct_rename_fields" => expr_text_strings_args(
                request.args,
                "struct_rename_fields",
                expr_struct_rename_fields,
            ),
            "polars.expr.struct_json_encode" => {
                expr_text_arg(request.args, "struct_json_encode", expr_struct_json_encode)
            }
            "polars.expr.struct_with_fields" => {
                expr_text_strings_args(request.args, "struct_with_fields", expr_struct_with_fields)
            }
            "polars.expr.to_field" => {
                expr_text_two_strings_result_args(request.args, "to_field", expr_to_field)
            }
            "polars.expr.meta_root_names" => expr_metadata_root_names(request.args),
            "polars.expr.meta_nodes" => {
                expr_metadata_list_arg(request.args, "meta_nodes", expr_meta_nodes)
            }
            "polars.expr.meta_children" => {
                expr_metadata_list_arg(request.args, "meta_children", expr_meta_children)
            }
            "polars.expr.rewrite" => {
                expr_two_text_result_args(request.args, "rewrite", expr_rewrite)
            }
            "polars.expr.map_children" => {
                expr_two_text_result_args(request.args, "map_children", expr_map_children)
            }
            "polars.expr.meta_output_name" => {
                expr_text_arg(request.args, "meta_output_name", expr_meta_output_name)
            }
            "polars.expr.meta_has_multiple_outputs" => expr_metadata_bool_arg(
                request.args,
                "meta_has_multiple_outputs",
                expr_meta_has_multiple_outputs,
            ),
            "polars.expr.meta_is_column" => {
                expr_metadata_bool_arg(request.args, "meta_is_column", expr_meta_is_column)
            }
            "polars.expr.meta_is_simple_projection" => expr_metadata_bool_arg(
                request.args,
                "meta_is_simple_projection",
                expr_meta_is_simple_projection,
            ),
            "polars.expr.meta_is_column_selection" => expr_metadata_text_bool_arg(
                request.args,
                "meta_is_column_selection",
                expr_meta_is_column_selection,
            ),
            "polars.expr.meta_is_literal" => {
                expr_metadata_text_bool_arg(request.args, "meta_is_literal", expr_meta_is_literal)
            }
            "polars.expr.meta_is_regex_projection" => expr_metadata_bool_arg(
                request.args,
                "meta_is_regex_projection",
                expr_meta_is_regex_projection,
            ),
            "polars.expr.meta_format_tree" => {
                expr_text_bool_args(request.args, "meta_format_tree", expr_meta_format_tree)
            }
            "polars.dataframe.select_exprs" => self.select_exprs(request.args),
            "polars.dataframe.select_exprs_sequential" => {
                self.select_exprs_sequential(request.args)
            }
            "polars.dataframe.with_columns_exprs" => self.with_columns_exprs(request.args),
            "polars.dataframe.with_columns_sequential" => {
                self.with_columns_sequential(request.args)
            }
            "polars.dataframe.filter_expr" => self.filter_expr(request.args),
            "polars.dataframe.filter_all" => self.filter_all(request.args),
            "polars.dataframe.filter_constraints" => self.filter_constraints(request.args),
            "polars.dataframe.remove_where" => self.remove_where(request.args),
            "polars.dataframe.group_agg" => self.group_agg(request.args),
            "polars.dataframe.map_groups" => self.map_groups(request.args),
            "polars.dataframe.left_join" => self.left_join(request.args),
            "polars.dataframe.join" => self.join(request.args),
            "polars.dataframe.join_with_options" => self.join_with_options(request.args),
            "polars.dataframe.join_with_coalesce" => self.join_with_coalesce(request.args),
            "polars.dataframe.join_where" => self.join_where(request.args),
            "polars.dataframe.asof_join" => self.asof_join(request.args),
            "polars.dataframe.concat_vertical" => self.concat_vertical(request.args),
            "polars.dataframe.concat_vertical_relaxed" => {
                self.concat_vertical_relaxed(request.args)
            }
            "polars.dataframe.concat_horizontal" => self.concat_horizontal(request.args),
            "polars.dataframe.concat_horizontal_extend" => {
                self.concat_horizontal_extend(request.args)
            }
            "polars.dataframe.concat_diagonal" => self.concat_diagonal(request.args),
            "polars.dataframe.explode" => self.explode_columns(request.args),
            "polars.dataframe.unpivot" => self.unpivot(request.args),
            "polars.dataframe.pivot" => self.pivot(request.args),
            "polars.dataframe.unnest" => self.unnest_columns(request.args),
            "polars.dataframe.transpose" => self.transpose(request.args),
            "polars.dataframe.upsample" => self.upsample(request.args),
            "polars.dataframe.update" => self.update(request.args),
            operation => protocol_error(
                "native_operation_unknown",
                &format!("unknown native operation `{operation}`"),
            ),
        }
    }

    fn read_csv(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(path)] = args.as_slice() else {
            return protocol_error("native_bad_args", "read_csv expects one string path");
        };
        match read_csv(path) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn read_csv_dates(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(path)] = args.as_slice() else {
            return protocol_error("native_bad_args", "read_csv_dates expects one string path");
        };
        match read_csv_dates(path) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn write_csv(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(path)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "write_csv expects a DataFrame handle and path",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match write_csv(dataframe, path) {
            Ok(()) => "result_ok_unit".to_string(),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn read_dataframe_path(
        &mut self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &str,
        )
            -> Result<TerlanPolarsDataFrame, terlan_polars_native::TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Text(path)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects one string path"),
            );
        };
        match function(path) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn read_database(
        &mut self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &str,
            &str,
        )
            -> Result<TerlanPolarsDataFrame, terlan_polars_native::TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Text(connection), NativeArg::Text(query)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects a connection value and SQL query"),
            );
        };
        match function(connection, query) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn from_arrow_ipc(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Bytes(bytes)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "from_arrow_ipc expects one Bytes payload",
            );
        };
        match dataframe_from_arrow_ipc(bytes) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn to_arrow_ipc(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "to_arrow_ipc expects one DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        encode_bytes_result(dataframe_to_arrow_ipc(dataframe))
    }

    fn plot_grouped_xy_svg(
        &self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &TerlanPolarsDataFrame,
            &str,
            &str,
            &str,
            &str,
        ) -> Result<Vec<u8>, TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(x), NativeArg::Text(y), NativeArg::Text(group), NativeArg::Text(title)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects a DataFrame handle, two columns, group, and title"),
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        encode_bytes_result(function(dataframe, x, y, group, title))
    }

    fn plot_line_svg(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(x), NativeArg::Text(y), NativeArg::Text(title)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "plot_line_svg expects a DataFrame handle, x column, y column, and title",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        encode_bytes_result(plot_line_svg(dataframe, x, y, title))
    }

    fn plot_histogram_svg(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(x), NativeArg::Text(group), NativeArg::Text(title)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "plot_histogram_svg expects a DataFrame handle, value column, group, and title",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        encode_bytes_result(plot_histogram_svg(dataframe, x, group, title))
    }

    fn plot_line_html(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(x), NativeArg::Text(y), NativeArg::Text(title)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "plot_line_html expects a DataFrame handle, x column, y column, and title",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        encode_bytes_result(plot_line_html(dataframe, x, y, title))
    }

    fn plot_histogram_html(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(x), NativeArg::Text(group), NativeArg::Text(title)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "plot_histogram_html expects a DataFrame handle, value column, group, and title",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        encode_bytes_result(plot_histogram_html(dataframe, x, group, title))
    }

    fn plot_line_png(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(x), NativeArg::Text(y), NativeArg::Text(title)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "plot_line_png expects a DataFrame handle, x column, y column, and title",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        encode_bytes_result(plot_line_png(dataframe, x, y, title))
    }

    fn plot_histogram_png(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(x), NativeArg::Text(group), NativeArg::Text(title)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "plot_histogram_png expects a DataFrame handle, value column, group, and title",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        encode_bytes_result(plot_histogram_png(dataframe, x, group, title))
    }

    fn plot_with_options(
        &self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &TerlanPolarsDataFrame,
            &str,
            &str,
            &str,
            &str,
        ) -> Result<Vec<u8>, TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(kind), NativeArg::Text(x), NativeArg::Text(y), NativeArg::Text(options)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                &format!(
                    "{name} expects a DataFrame handle, plot kind, two columns, and PlotOptions"
                ),
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        encode_bytes_result(function(dataframe, kind, x, y, options))
    }

    fn table_html(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(style), NativeArg::Int(max_rows)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "table_html expects a DataFrame handle, TableStyle, and maximum row count",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        encode_bytes_result(table_html(dataframe, style, *max_rows))
    }

    fn write_dataframe_path(
        &self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &TerlanPolarsDataFrame,
            &str,
        ) -> Result<(), terlan_polars_native::TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(path)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects a DataFrame handle and path"),
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match function(dataframe, path) {
            Ok(()) => "result_ok_unit".to_string(),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn from_rows(&mut self, args: Vec<NativeArg>) -> String {
        let (columns, rows) = match args.as_slice() {
            [NativeArg::Strings(columns), NativeArg::StringRows(rows)] => {
                (columns.as_slice(), rows.as_slice())
            }
            [NativeArg::Strings(columns), NativeArg::Strings(rows)] if rows.is_empty() => {
                (columns.as_slice(), &[][..])
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "from_rows expects column names and nested string rows",
                )
            }
        };
        match from_rows(columns, rows) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn height(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "height expects one DataFrame handle");
        };
        match self.frame(handle) {
            Ok(dataframe) => format!("ok_int {}", height(dataframe)),
            Err(error) => error,
        }
    }

    fn width(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "width expects one DataFrame handle");
        };
        match self.frame(handle) {
            Ok(dataframe) => format!("ok_int {}", width(dataframe)),
            Err(error) => error,
        }
    }

    fn estimated_size(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "estimated_size expects one DataFrame handle",
            );
        };
        match self.frame(handle) {
            Ok(dataframe) => format!("ok_int {}", estimated_size(dataframe)),
            Err(error) => error,
        }
    }

    fn estimated_size_with_unit(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(unit)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "estimated_size expects a DataFrame handle and size unit",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match estimated_size_with_unit(dataframe, unit) {
            Ok(value) => format!("result_ok_float {value}"),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn glimpse(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "glimpse expects one DataFrame handle");
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match glimpse(dataframe) {
            Ok(value) => format!("result_ok_string {}", STANDARD.encode(value)),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn describe(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "describe expects one DataFrame handle");
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match describe(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn export_tensor_packet(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Text(dtype), NativeArg::Text(null_policy), NativeArg::Text(consumer)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "export_tensor_packet expects a DataFrame handle, column names, dtype, null policy, and consumer namespace",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match dataframe_tensor_packet(dataframe, columns, dtype, null_policy, consumer) {
            Ok(packet) if packet.is_empty() => "ok_bytes".to_string(),
            Ok(packet) => format!("ok_bytes {}", STANDARD.encode(packet)),
            Err(error) => protocol_error(error.code(), error.message()),
        }
    }

    fn export_series_tensor_packet(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(dtype), NativeArg::Text(null_policy), NativeArg::Text(consumer)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "export_series_tensor_packet expects a Series handle, dtype, null policy, and consumer namespace",
            );
        };
        let series = match self.series(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match series_tensor_packet(series, dtype, null_policy, consumer) {
            Ok(packet) if packet.is_empty() => "ok_bytes".to_string(),
            Ok(packet) => format!("ok_bytes {}", STANDARD.encode(packet)),
            Err(error) => protocol_error(error.code(), error.message()),
        }
    }

    fn sql_context_new(&mut self, args: Vec<NativeArg>) -> String {
        if !args.is_empty() {
            return protocol_error("native_bad_args", "sql_context expects no arguments");
        }
        self.store_sql_context(sql_context_new())
    }

    fn sql_context_register(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(context_handle), NativeArg::Text(name), NativeArg::Handle(frame_handle)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "register_table expects a SqlContext handle, table name, and DataFrame handle",
            );
        };
        let context = match self.sql_context(context_handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let dataframe = match self.frame(frame_handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match sql_context_register(context, name, dataframe) {
            Ok(()) => "result_ok_unit".to_string(),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn sql_context_unregister(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(name)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "unregister_table expects a SqlContext handle and table name",
            );
        };
        let context = match self.sql_context(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match sql_context_unregister(context, name) {
            Ok(()) => "result_ok_unit".to_string(),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn sql_context_tables(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "sql_tables expects one SqlContext handle",
            );
        };
        match self.sql_context(handle) {
            Ok(context) => {
                let tables = sql_context_tables(context);
                if tables.is_empty() {
                    "ok_strings".to_string()
                } else {
                    format!("ok_strings {}", encode_string_list(&tables))
                }
            }
            Err(error) => error,
        }
    }

    /// Returns the pinned Polars SQL keyword inventory.
    fn sql_keywords(&self, args: Vec<NativeArg>) -> String {
        if !args.is_empty() {
            return protocol_error("native_bad_args", "sql_keywords expects no arguments");
        }
        match terlan_polars_native::sql_keywords() {
            Ok(values) if values.is_empty() => "result_ok_strings".to_string(),
            Ok(values) => format!("result_ok_strings {}", encode_string_list(&values)),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    /// Returns the pinned Polars SQL function inventory.
    fn sql_functions(&self, args: Vec<NativeArg>) -> String {
        if !args.is_empty() {
            return protocol_error("native_bad_args", "sql_functions expects no arguments");
        }
        match terlan_polars_native::sql_functions() {
            Ok(values) if values.is_empty() => "result_ok_strings".to_string(),
            Ok(values) => format!("result_ok_strings {}", encode_string_list(&values)),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    /// Extracts table identifiers from one SQL statement.
    fn sql_table_identifiers(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(query), NativeArg::Bool(include_schema), NativeArg::Bool(unique)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "sql_table_identifiers expects a query and two boolean options",
            );
        };
        match terlan_polars_native::sql_table_identifiers(query, *include_schema, *unique) {
            Ok(values) if values.is_empty() => "result_ok_strings".to_string(),
            Ok(values) => format!("result_ok_strings {}", encode_string_list(&values)),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn sql_context_execute(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(query)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "sql_query expects a SqlContext handle and SQL query",
            );
        };
        let result = {
            let context = match self.sql_context_mut(handle) {
                Ok(value) => value,
                Err(error) => return error,
            };
            sql_context_execute(context, query)
        };
        match result {
            Ok(lazy_frame) => self.store_result_lazy_frame_result(lazy_frame),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn frames_equal(&self, args: Vec<NativeArg>, missing_equal: bool) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "frames_equal expects two DataFrame handles",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let equal = if missing_equal {
            frames_equal_missing(left, right)
        } else {
            frames_equal(left, right)
        };
        format!("ok_bool {equal}")
    }

    fn sample_rows_n(&mut self, args: Vec<NativeArg>, seeded: bool) -> String {
        let (handle, count, with_replacement, shuffle, seed) = match args.as_slice() {
            [NativeArg::Handle(handle), NativeArg::Int(count), NativeArg::Bool(with_replacement), NativeArg::Bool(shuffle)]
                if !seeded =>
            {
                (handle, *count, *with_replacement, *shuffle, None)
            }
            [NativeArg::Handle(handle), NativeArg::Int(count), NativeArg::Bool(with_replacement), NativeArg::Bool(shuffle), NativeArg::Int(seed)]
                if seeded =>
            {
                (handle, *count, *with_replacement, *shuffle, Some(*seed))
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "sample_rows_n expects a DataFrame handle, count, replacement flag, shuffle flag, and optional seed",
                );
            }
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let result = match seed {
            Some(seed) => sample_rows_n_seeded(dataframe, count, with_replacement, shuffle, seed),
            None => sample_rows_n(dataframe, count, with_replacement, shuffle),
        };
        match result {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn sample_rows_fraction(&mut self, args: Vec<NativeArg>, seeded: bool) -> String {
        let (handle, fraction, with_replacement, shuffle, seed) = match args.as_slice() {
            [NativeArg::Handle(handle), NativeArg::Float(fraction), NativeArg::Bool(with_replacement), NativeArg::Bool(shuffle)]
                if !seeded =>
            {
                (handle, *fraction, *with_replacement, *shuffle, None)
            }
            [NativeArg::Handle(handle), NativeArg::Float(fraction), NativeArg::Bool(with_replacement), NativeArg::Bool(shuffle), NativeArg::Int(seed)]
                if seeded =>
            {
                (handle, *fraction, *with_replacement, *shuffle, Some(*seed))
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "sample_rows_fraction expects a DataFrame handle, fraction, replacement flag, shuffle flag, and optional seed",
                );
            }
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let result = match seed {
            Some(seed) => {
                sample_rows_fraction_seeded(dataframe, fraction, with_replacement, shuffle, seed)
            }
            None => sample_rows_fraction(dataframe, fraction, with_replacement, shuffle),
        };
        match result {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn row_mask(&mut self, args: Vec<NativeArg>, unique: bool) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "row identity masks expect one DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let result = if unique {
            row_is_unique(dataframe)
        } else {
            row_is_duplicated(dataframe)
        };
        match result {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn row_hashes(&mut self, args: Vec<NativeArg>, seeded: bool) -> String {
        let (handle, seed) = match args.as_slice() {
            [NativeArg::Handle(handle)] if !seeded => (handle, None),
            [NativeArg::Handle(handle), NativeArg::Int(seed)] if seeded => (handle, Some(*seed)),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "row_hashes expects one DataFrame handle and an optional seed",
                );
            }
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let result = match seed {
            Some(seed) => row_hashes_seeded(dataframe, seed),
            None => row_hashes(dataframe),
        };
        match result {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn columns(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "columns expects one DataFrame handle");
        };
        match self.frame(handle) {
            Ok(dataframe) => format!("ok_strings {}", encode_string_list(&columns(dataframe))),
            Err(error) => error,
        }
    }

    fn rows(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(limit)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "rows expects one DataFrame handle and one integer limit",
            );
        };
        let Ok(limit) = usize::try_from(*limit) else {
            return adapter_result_error("invalid_limit", "rows limit cannot be negative");
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match rows(dataframe, limit) {
            Ok(rows) => format!("result_ok_string_rows {}", encode_string_rows(&rows)),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn schema(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "schema expects one DataFrame handle");
        };
        match self.frame(handle) {
            Ok(dataframe) => format!("ok_schema {}", encode_schema(&schema(dataframe))),
            Err(error) => error,
        }
    }

    fn data_types(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "dtypes expects one DataFrame handle");
        };
        match self.frame(handle) {
            Ok(dataframe) => format!("ok_strings {}", encode_string_list(&data_types(dataframe))),
            Err(error) => error,
        }
    }

    fn column_series(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(name)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_series expects a DataFrame handle and column name",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_series(dataframe, name) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_strings(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(name), NativeArg::Strings(values)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_from_strings expects a name and string list",
            );
        };
        match series_from_strings(name, values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_ints(&mut self, args: Vec<NativeArg>) -> String {
        let (name, values): (&str, &[i64]) = match args.as_slice() {
            [NativeArg::Text(name), NativeArg::Ints(values)] => (name, values),
            [NativeArg::Text(name), NativeArg::Strings(values)] if values.is_empty() => (name, &[]),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_from_ints expects a name and integer list",
                )
            }
        };
        match series_from_ints(name, values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_floats(&mut self, args: Vec<NativeArg>) -> String {
        let (name, values): (&str, &[f64]) = match args.as_slice() {
            [NativeArg::Text(name), NativeArg::Floats(values)] => (name, values),
            [NativeArg::Text(name), NativeArg::Strings(values)] if values.is_empty() => (name, &[]),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_from_floats expects a name and float list",
                )
            }
        };
        match series_from_floats(name, values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_bools(&mut self, args: Vec<NativeArg>) -> String {
        let (name, values): (&str, &[bool]) = match args.as_slice() {
            [NativeArg::Text(name), NativeArg::Bools(values)] => (name, values),
            [NativeArg::Text(name), NativeArg::Strings(values)] if values.is_empty() => (name, &[]),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_from_bools expects a name and boolean list",
                )
            }
        };
        match series_from_bools(name, values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_nullable_strings(&mut self, args: Vec<NativeArg>) -> String {
        let (name, values): (&str, Vec<Option<String>>) = match args.as_slice() {
            [NativeArg::Text(name), NativeArg::NullableStrings(values)] => (name, values.clone()),
            [NativeArg::Text(name), NativeArg::Nulls(count)] => (name, vec![None; *count]),
            [NativeArg::Text(name), NativeArg::Strings(values)] if values.is_empty() => {
                (name, Vec::new())
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_from_nullable_strings expects a name and nullable string list",
                )
            }
        };
        match series_from_nullable_strings(name, &values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_nullable_ints(&mut self, args: Vec<NativeArg>) -> String {
        let (name, values): (&str, Vec<Option<i64>>) = match args.as_slice() {
            [NativeArg::Text(name), NativeArg::NullableInts(values)] => (name, values.clone()),
            [NativeArg::Text(name), NativeArg::Nulls(count)] => (name, vec![None; *count]),
            [NativeArg::Text(name), NativeArg::Strings(values)] if values.is_empty() => {
                (name, Vec::new())
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_from_nullable_ints expects a name and nullable integer list",
                )
            }
        };
        match series_from_nullable_ints(name, &values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_nullable_floats(&mut self, args: Vec<NativeArg>) -> String {
        let (name, values): (&str, Vec<Option<f64>>) = match args.as_slice() {
            [NativeArg::Text(name), NativeArg::NullableFloats(values)] => (name, values.clone()),
            [NativeArg::Text(name), NativeArg::Nulls(count)] => (name, vec![None; *count]),
            [NativeArg::Text(name), NativeArg::Strings(values)] if values.is_empty() => {
                (name, Vec::new())
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_from_nullable_floats expects a name and nullable float list",
                )
            }
        };
        match series_from_nullable_floats(name, &values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_nullable_bools(&mut self, args: Vec<NativeArg>) -> String {
        let (name, values): (&str, Vec<Option<bool>>) = match args.as_slice() {
            [NativeArg::Text(name), NativeArg::NullableBools(values)] => (name, values.clone()),
            [NativeArg::Text(name), NativeArg::Nulls(count)] => (name, vec![None; *count]),
            [NativeArg::Text(name), NativeArg::Strings(values)] if values.is_empty() => {
                (name, Vec::new())
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_from_nullable_bools expects a name and nullable boolean list",
                )
            }
        };
        match series_from_nullable_bools(name, &values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_dates(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(name), NativeArg::Strings(values)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_from_dates expects a name and ISO date string list",
            );
        };
        match series_from_dates(name, values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_nullable_dates(&mut self, args: Vec<NativeArg>) -> String {
        let (name, values): (&str, Vec<Option<String>>) = match args.as_slice() {
            [NativeArg::Text(name), NativeArg::NullableStrings(values)] => (name, values.clone()),
            [NativeArg::Text(name), NativeArg::Nulls(count)] => (name, vec![None; *count]),
            [NativeArg::Text(name), NativeArg::Strings(values)] if values.is_empty() => {
                (name, Vec::new())
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_from_nullable_dates expects a name and nullable ISO date list",
                )
            }
        };
        match series_from_nullable_dates(name, &values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_datetimes(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(name), NativeArg::Strings(values)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_from_datetimes expects a name and ISO local datetime string list",
            );
        };
        match series_from_datetimes(name, values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_from_nullable_datetimes(&mut self, args: Vec<NativeArg>) -> String {
        let (name, values): (&str, Vec<Option<String>>) = match args.as_slice() {
            [NativeArg::Text(name), NativeArg::NullableStrings(values)] => {
                (name, values.clone())
            }
            [NativeArg::Text(name), NativeArg::Nulls(count)] => (name, vec![None; *count]),
            [NativeArg::Text(name), NativeArg::Strings(values)] if values.is_empty() => {
                (name, Vec::new())
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_from_nullable_datetimes expects a name and nullable ISO local datetime list",
                )
            }
        };
        match series_from_nullable_datetimes(name, &values) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_empty(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(name), NativeArg::Text(data_type)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_empty expects a name and data type",
            );
        };
        match series_empty(name, data_type) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_name(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "series_name expects a Series handle");
        };
        match self.series(handle) {
            Ok(value) => format!("ok_string {}", STANDARD.encode(series_name(value))),
            Err(error) => error,
        }
    }

    fn series_len(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "series_len expects a Series handle");
        };
        match self.series(handle) {
            Ok(value) => format!("ok_int {}", series_len(value)),
            Err(error) => error,
        }
    }

    fn series_null_count(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_null_count expects a Series handle",
            );
        };
        match self.series(handle) {
            Ok(value) => format!("ok_int {}", series_null_count(value)),
            Err(error) => error,
        }
    }

    fn series_equal(&self, args: Vec<NativeArg>, missing_equal: bool) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right)] = args.as_slice() else {
            return protocol_error("native_bad_args", "series_equal expects two Series handles");
        };
        let left = match self.series(left) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let right = match self.series(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let equal = if missing_equal {
            series_equal_missing(left, right)
        } else {
            series_equal(left, right)
        };
        format!("ok_bool {equal}")
    }

    fn series_data_type(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_data_type expects a Series handle",
            );
        };
        match self.series(handle) {
            Ok(value) => format!("ok_string {}", STANDARD.encode(series_data_type(value))),
            Err(error) => error,
        }
    }

    fn series_values(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(limit)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_values expects a Series handle and integer limit",
            );
        };
        let Ok(limit) = usize::try_from(*limit) else {
            return protocol_error("invalid_limit", "series values limit cannot be negative");
        };
        let value = match self.series(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match series_values(value, limit) {
            Ok(values) => format!("ok_strings {}", encode_string_list(&values)),
            Err(error) => protocol_error(error.code(), error.message()),
        }
    }

    fn series_cast(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(data_type)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_cast expects a Series handle and data type",
            );
        };
        let value = match self.series(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match series_cast(value, data_type) {
            Ok(value) => self.store_result_series(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_to_frame(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "series_to_frame expects a Series handle");
        };
        let value = match self.series(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match series_to_frame(value) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn series_to_expr(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_to_expr expects one Series handle",
            );
        };
        let series = match self.series(handle) {
            Ok(series) => series,
            Err(error) => return error,
        };
        expression_reply(expr_series(series))
    }

    #[cfg(feature = "real-polars")]
    fn series_bin_intervals(&mut self, args: Vec<NativeArg>, labeled: bool) -> String {
        let result = if labeled {
            let [NativeArg::Handle(input_handle), spec, NativeArg::Strings(labels), NativeArg::Bool(right_closed), NativeArg::Bool(include_intervals)] =
                args.as_slice()
            else {
                return protocol_error(
                    "native_bad_args",
                    "Series.bin_intervals_labeled expects a Series, breakpoint list/Series or count, labels, and two booleans",
                );
            };
            let input = match self.series(input_handle) {
                Ok(series) => series.clone(),
                Err(error) => return error,
            };
            match spec {
                NativeArg::Floats(values) => series_bin_intervals_labeled(
                    &input,
                    values,
                    labels,
                    *right_closed,
                    *include_intervals,
                ),
                NativeArg::Ints(values) => series_bin_intervals_int_labeled(
                    &input,
                    values,
                    labels,
                    *right_closed,
                    *include_intervals,
                ),
                NativeArg::Strings(values) => series_bin_intervals_string_labeled(
                    &input,
                    values,
                    labels,
                    *right_closed,
                    *include_intervals,
                ),
                NativeArg::Bools(values) => series_bin_intervals_bool_labeled(
                    &input,
                    values,
                    labels,
                    *right_closed,
                    *include_intervals,
                ),
                NativeArg::Int(bins) => series_bin_intervals_uniform_labeled(
                    &input,
                    *bins,
                    labels,
                    *right_closed,
                    *include_intervals,
                ),
                NativeArg::Handle(handle) => {
                    let breaks = match self.series(handle) {
                        Ok(series) => series,
                        Err(error) => return error,
                    };
                    series_bin_intervals_series_labeled(
                        &input,
                        breaks,
                        labels,
                        *right_closed,
                        *include_intervals,
                    )
                }
                _ => {
                    return protocol_error(
                        "native_bad_args",
                        "Series.bin_intervals_labeled received an invalid interval specification",
                    )
                }
            }
        } else {
            let [NativeArg::Handle(input_handle), spec, NativeArg::Bool(right_closed), NativeArg::Bool(include_intervals)] =
                args.as_slice()
            else {
                return protocol_error(
                    "native_bad_args",
                    "Series.bin_intervals expects a Series, breakpoint list/Series or count, and two booleans",
                );
            };
            let input = match self.series(input_handle) {
                Ok(series) => series.clone(),
                Err(error) => return error,
            };
            match spec {
                NativeArg::Floats(values) => {
                    series_bin_intervals(&input, values, *right_closed, *include_intervals)
                }
                NativeArg::Ints(values) => {
                    series_bin_intervals_int(&input, values, *right_closed, *include_intervals)
                }
                NativeArg::Strings(values) => {
                    series_bin_intervals_string(&input, values, *right_closed, *include_intervals)
                }
                NativeArg::Bools(values) => {
                    series_bin_intervals_bool(&input, values, *right_closed, *include_intervals)
                }
                NativeArg::Int(bins) => {
                    series_bin_intervals_uniform(&input, *bins, *right_closed, *include_intervals)
                }
                NativeArg::Handle(handle) => {
                    let breaks = match self.series(handle) {
                        Ok(series) => series,
                        Err(error) => return error,
                    };
                    series_bin_intervals_series(&input, breaks, *right_closed, *include_intervals)
                }
                _ => {
                    return protocol_error(
                        "native_bad_args",
                        "Series.bin_intervals received an invalid interval specification",
                    )
                }
            }
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    #[cfg(feature = "real-polars")]
    fn series_bin_quantiles(&mut self, args: Vec<NativeArg>, labeled: bool) -> String {
        let result = if labeled {
            let [NativeArg::Handle(handle), spec, NativeArg::Strings(labels), NativeArg::Bool(right_closed), NativeArg::Bool(include_intervals)] =
                args.as_slice()
            else {
                return protocol_error(
                    "native_bad_args",
                    "Series.bin_quantiles_labeled expects a Series, fractions/count, labels, and two booleans",
                );
            };
            let input = match self.series(handle) {
                Ok(series) => series,
                Err(error) => return error,
            };
            match spec {
                NativeArg::Floats(values) => series_bin_quantiles_labeled(
                    input,
                    values,
                    labels,
                    *right_closed,
                    *include_intervals,
                ),
                NativeArg::Int(bins) => series_bin_quantiles_uniform_labeled(
                    input,
                    *bins,
                    labels,
                    *right_closed,
                    *include_intervals,
                ),
                _ => {
                    return protocol_error(
                        "native_bad_args",
                        "Series.bin_quantiles_labeled requires fractions or a bin count",
                    )
                }
            }
        } else {
            let [NativeArg::Handle(handle), spec, NativeArg::Bool(right_closed), NativeArg::Bool(include_intervals)] =
                args.as_slice()
            else {
                return protocol_error(
                    "native_bad_args",
                    "Series.bin_quantiles expects a Series, fractions/count, and two booleans",
                );
            };
            let input = match self.series(handle) {
                Ok(series) => series,
                Err(error) => return error,
            };
            match spec {
                NativeArg::Floats(values) => {
                    series_bin_quantiles(input, values, *right_closed, *include_intervals)
                }
                NativeArg::Int(bins) => {
                    series_bin_quantiles_uniform(input, *bins, *right_closed, *include_intervals)
                }
                _ => {
                    return protocol_error(
                        "native_bad_args",
                        "Series.bin_quantiles requires fractions or a bin count",
                    )
                }
            }
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    #[cfg(feature = "real-polars")]
    fn series_bin_ranks(&mut self, args: Vec<NativeArg>, labeled: bool) -> String {
        let result = if labeled {
            let [NativeArg::Handle(handle), spec, NativeArg::Strings(labels), NativeArg::Bool(include_intervals)] =
                args.as_slice()
            else {
                return protocol_error(
                    "native_bad_args",
                    "Series.bin_ranks_labeled expects a Series, fractions/count, labels, and a boolean",
                );
            };
            let input = match self.series(handle) {
                Ok(series) => series,
                Err(error) => return error,
            };
            match spec {
                NativeArg::Floats(values) => {
                    series_bin_ranks_labeled(input, values, labels, false, *include_intervals)
                }
                NativeArg::Int(bins) => series_bin_ranks_uniform_labeled(
                    input,
                    *bins,
                    labels,
                    false,
                    *include_intervals,
                ),
                _ => {
                    return protocol_error(
                        "native_bad_args",
                        "Series.bin_ranks_labeled requires fractions or a bin count",
                    )
                }
            }
        } else {
            let [NativeArg::Handle(handle), spec, NativeArg::Bool(include_intervals)] =
                args.as_slice()
            else {
                return protocol_error(
                    "native_bad_args",
                    "Series.bin_ranks expects a Series, fractions/count, and a boolean",
                );
            };
            let input = match self.series(handle) {
                Ok(series) => series,
                Err(error) => return error,
            };
            match spec {
                NativeArg::Floats(values) => {
                    series_bin_ranks(input, values, false, *include_intervals)
                }
                NativeArg::Int(bins) => {
                    series_bin_ranks_uniform(input, *bins, false, *include_intervals)
                }
                _ => {
                    return protocol_error(
                        "native_bad_args",
                        "Series.bin_ranks requires fractions or a bin count",
                    )
                }
            }
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    #[cfg(not(feature = "real-polars"))]
    fn series_bin_intervals(&mut self, _args: Vec<NativeArg>, _labeled: bool) -> String {
        adapter_result_error(
            "native_unavailable",
            "Series binning requires the real-polars feature",
        )
    }

    #[cfg(not(feature = "real-polars"))]
    fn series_bin_quantiles(&mut self, _args: Vec<NativeArg>, _labeled: bool) -> String {
        adapter_result_error(
            "native_unavailable",
            "Series binning requires the real-polars feature",
        )
    }

    #[cfg(not(feature = "real-polars"))]
    fn series_bin_ranks(&mut self, _args: Vec<NativeArg>, _labeled: bool) -> String {
        adapter_result_error(
            "native_unavailable",
            "Series binning requires the real-polars feature",
        )
    }

    fn filter_eq(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(column), value] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "filter_eq expects a DataFrame handle, column string, and scalar value",
            );
        };
        let Some(value) = native_scalar(value) else {
            return protocol_error(
                "native_bad_args",
                "filter_eq scalar must be String, Int, Float, or Bool",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match filter_eq(dataframe, column, &value) {
            Ok(filtered) => self.store_result_dataframe(filtered),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn sort_by(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(column), NativeArg::Bool(descending)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "sort_by expects a DataFrame handle, column string, and boolean order",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match sort_by(dataframe, column, *descending) {
            Ok(sorted) => self.store_result_dataframe(sorted),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn sort_rows_by(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Bools(descending), NativeArg::Bools(nulls_last), NativeArg::Bool(maintain_order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "sort_rows_by expects a DataFrame handle, columns, descending flags, nulls-last flags, and maintain-order boolean",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match sort_rows_by(dataframe, columns, descending, nulls_last, *maintain_order) {
            Ok(sorted) => self.store_result_dataframe(sorted),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn top_rows_by(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Int(limit), NativeArg::Bools(nulls_last), NativeArg::Bool(maintain_order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "top_rows_by expects a DataFrame handle, columns, integer limit, nulls-last flags, and maintain-order boolean",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match top_rows_by(dataframe, columns, *limit, nulls_last, *maintain_order) {
            Ok(selected) => self.store_result_dataframe(selected),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn bottom_rows_by(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Int(limit), NativeArg::Bools(nulls_last), NativeArg::Bool(maintain_order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "bottom_rows_by expects a DataFrame handle, columns, integer limit, nulls-last flags, and maintain-order boolean",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match bottom_rows_by(dataframe, columns, *limit, nulls_last, *maintain_order) {
            Ok(selected) => self.store_result_dataframe(selected),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn group_count(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(keys)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "group_count expects a DataFrame handle and list of key columns",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match group_count(dataframe, keys) {
            Ok(grouped) => self.store_result_dataframe(grouped),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn to_lazy(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "lazy expects one DataFrame handle");
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(to_lazy(dataframe))
    }

    fn lazy_scan_csv(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(path), NativeArg::Bool(parse_dates)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "scan_csv expects a path and parse_dates boolean",
            );
        };
        match lazy_scan_csv(path, *parse_dates) {
            Ok(plan) => self.store_result_lazy_frame_result(plan),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_scan_csv_with_storage_options(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(path), NativeArg::Bool(parse_dates), NativeArg::Strings(keys), NativeArg::Strings(values)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "scan_csv_with_storage_options expects a path, parse_dates boolean, option keys, and option values",
            );
        };
        match lazy_scan_csv_with_storage_options(path, *parse_dates, keys, values) {
            Ok(plan) => self.store_result_lazy_frame_result(plan),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_scan_path(
        &mut self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &str,
        )
            -> Result<TerlanPolarsLazyFrame, terlan_polars_native::TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Text(path)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects one string path"),
            );
        };
        match function(path) {
            Ok(plan) => self.store_result_lazy_frame_result(plan),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_scan_path_with_storage_options(
        &mut self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &str,
            &[String],
            &[String],
        )
            -> Result<TerlanPolarsLazyFrame, terlan_polars_native::TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Text(path), NativeArg::Strings(keys), NativeArg::Strings(values)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects a path, option keys, and option values"),
            );
        };
        match function(path, keys, values) {
            Ok(plan) => self.store_result_lazy_frame_result(plan),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_write_path(
        &self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &TerlanPolarsLazyFrame,
            &str,
        ) -> Result<(), terlan_polars_native::TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(path)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects a LazyFrame handle and string path"),
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(plan) => plan,
            Err(error) => return error,
        };
        match function(plan, path) {
            Ok(()) => "result_ok_unit".to_string(),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_write_partitioned(
        &self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &TerlanPolarsLazyFrame,
            &str,
            &[String],
            bool,
            i64,
            i64,
        ) -> Result<(), TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(path), NativeArg::Strings(keys), NativeArg::Bool(include_keys), NativeArg::Int(max_rows), NativeArg::Int(approximate_bytes)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects a plan, base path, keys, include-keys flag, row limit, and byte target"),
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(plan) => plan,
            Err(error) => return error,
        };
        match function(
            plan,
            path,
            keys,
            *include_keys,
            *max_rows,
            *approximate_bytes,
        ) {
            Ok(()) => "result_ok_unit".to_string(),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_filter_eq(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(column), value] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy filter_eq expects a LazyFrame handle, column string, and scalar value",
            );
        };
        let Some(value) = native_scalar(value) else {
            return protocol_error(
                "native_bad_args",
                "lazy filter_eq scalar must be String, Int, Float, or Bool",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(plan) => plan,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_filter_eq(plan, column, &value))
    }

    fn lazy_select(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy select expects a LazyFrame handle and list of columns",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(plan) => plan,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_select(plan, columns))
    }

    fn lazy_select_exprs(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(expressions)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_select_exprs expects a LazyFrame handle and expression list",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_select_exprs(plan, expressions) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_select_exprs_sequential(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(expressions)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_select_exprs_sequential expects a LazyFrame handle and expression list",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_select_exprs_sequential(plan, expressions) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_with_columns(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(expressions)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_with_columns expects a LazyFrame handle and expression list",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_with_columns(plan, expressions) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_with_columns_sequential(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(expressions)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_with_columns_sequential expects a LazyFrame handle and expression list",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_with_columns_sequential(plan, expressions) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_filter_expr(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(predicate)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_filter expects a LazyFrame handle and predicate",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_filter_expr(plan, predicate) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_remove_where(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(predicate)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_remove_where expects a LazyFrame handle and predicate",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_remove_where(plan, predicate) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_group_agg(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(keys), NativeArg::Strings(aggregations)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_group_agg expects a LazyFrame handle, keys, and aggregations",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_group_agg(plan, keys, aggregations) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_sort(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(column), NativeArg::Bool(descending)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_sort expects a LazyFrame handle, column, and descending boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_sort(plan, column, *descending))
    }

    fn lazy_sort_rows_by(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Bools(descending), NativeArg::Bools(nulls_last), NativeArg::Bool(maintain_order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_sort_rows_by expects a LazyFrame handle, columns, descending flags, nulls-last flags, and maintain-order boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_sort_rows_by(plan, columns, descending, nulls_last, *maintain_order) {
            Ok(sorted) => self.store_result_lazy_frame_result(sorted),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_top_rows_by(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Int(limit), NativeArg::Bools(nulls_last), NativeArg::Bool(maintain_order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_top_rows_by expects a LazyFrame handle, columns, integer limit, nulls-last flags, and maintain-order boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_top_rows_by(plan, columns, *limit, nulls_last, *maintain_order) {
            Ok(selected) => self.store_result_lazy_frame_result(selected),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_bottom_rows_by(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Int(limit), NativeArg::Bools(nulls_last), NativeArg::Bool(maintain_order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_bottom_rows_by expects a LazyFrame handle, columns, integer limit, nulls-last flags, and maintain-order boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_bottom_rows_by(plan, columns, *limit, nulls_last, *maintain_order) {
            Ok(selected) => self.store_result_lazy_frame_result(selected),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_limit(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(limit)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_limit expects a LazyFrame handle and integer limit",
            );
        };
        let Ok(limit) = usize::try_from(*limit) else {
            return protocol_error("invalid_limit", "lazy limit cannot be negative");
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_limit(plan, limit))
    }

    fn lazy_tail(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(limit)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_tail expects a LazyFrame handle and integer limit",
            );
        };
        let Ok(limit) = usize::try_from(*limit) else {
            return adapter_result_error("invalid_limit", "lazy tail limit cannot be negative");
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame_result(lazy_tail(plan, limit))
    }

    fn lazy_unique_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(subset), NativeArg::Text(keep), NativeArg::Bool(maintain_order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_unique_rows expects a LazyFrame handle, subset columns, keep strategy, and order boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_unique_rows(plan, subset, keep, *maintain_order) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_drop_null_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(subset)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_drop_null_rows expects a LazyFrame handle and subset columns",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_drop_null_rows(plan, subset) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_slice_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(offset), NativeArg::Int(length)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_slice_rows expects a LazyFrame handle, offset, and length",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_slice_rows(plan, *offset, *length) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_gather_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Ints(indices), NativeArg::Bool(null_on_out_of_bounds)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_gather_rows expects a LazyFrame handle, integer indices, and null-on-out-of-bounds boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_gather_rows(plan, indices, *null_on_out_of_bounds) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_clear_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_clear_rows expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_clear_rows(plan))
    }

    fn lazy_first_row(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_first_row expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_first_row(plan))
    }

    fn lazy_last_row(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_last_row expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_last_row(plan))
    }

    fn lazy_column_sums(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_sums expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_column_sums(plan))
    }

    fn lazy_column_means(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_means expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_column_means(plan))
    }

    fn lazy_column_medians(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_medians expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_column_medians(plan))
    }

    fn lazy_column_minima(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_minima expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_column_minima(plan))
    }

    fn lazy_column_maxima(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_maxima expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_column_maxima(plan))
    }

    fn lazy_column_products(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_products expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_column_products(plan))
    }

    fn lazy_column_variances(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(ddof)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_variances expects a LazyFrame handle and ddof",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_column_variances(plan, *ddof) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_column_stddevs(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(ddof)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_stddevs expects a LazyFrame handle and ddof",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_column_stddevs(plan, *ddof) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_column_quantiles(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Float(probability), NativeArg::Text(method)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_quantiles expects a LazyFrame handle, probability, and method",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_column_quantiles(plan, *probability, method) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_column_non_null_counts(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_non_null_counts expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_column_non_null_counts(plan))
    }

    fn lazy_column_lengths(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_lengths expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_column_lengths(plan))
    }

    fn lazy_column_unique_counts(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_unique_counts expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_column_unique_counts(plan))
    }

    fn lazy_column_approx_unique_counts(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_column_approx_unique_counts expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_column_approx_unique_counts(plan))
    }

    fn lazy_rename_columns(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(existing), NativeArg::Strings(new), NativeArg::Bool(strict)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_rename_columns expects a LazyFrame handle, source names, destination names, and strict boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_rename_columns(plan, existing, new, *strict) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_drop_columns(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_drop_columns expects a LazyFrame handle and column names",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_drop_columns(plan, columns) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_reverse_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_reverse_rows expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_reverse_rows(plan))
    }

    fn lazy_with_row_index(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(name), NativeArg::Int(offset)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_with_row_index expects a LazyFrame handle, name, and offset",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_with_row_index(plan, name, *offset) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_fill_null_values(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(value)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_fill_null_values expects a LazyFrame handle and expression",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_fill_null_values(plan, value) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_fill_nan_values(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(value)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_fill_nan_values expects a LazyFrame handle and expression",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_fill_nan_values(plan, value) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_drop_nan_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(subset)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_drop_nan_rows expects a LazyFrame handle and subset columns",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_drop_nan_rows(plan, subset) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_null_counts(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_null_counts expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_null_counts(plan))
    }

    fn lazy_schema(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "lazy_schema expects a LazyFrame handle");
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_schema(plan) {
            Ok(value) => format!("ok_schema {}", encode_schema(&value)),
            Err(error) => protocol_error(error.code(), error.message()),
        }
    }

    fn lazy_cache(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "lazy_cache expects a LazyFrame handle");
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_cache(plan))
    }

    fn lazy_without_optimizations(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_without_optimizations expects a LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_without_optimizations(plan))
    }

    fn lazy_set_optimization(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(name), NativeArg::Bool(enabled)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_set_optimization expects a LazyFrame handle, optimization name, and enabled boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_set_optimization(plan, name, *enabled) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_shift_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(periods)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "lazy_shift_rows expects a LazyFrame handle and periods",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        self.store_result_lazy_frame(lazy_shift_rows(plan, *periods))
    }

    fn lazy_shift_and_fill_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(periods), NativeArg::Text(fill_value)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_shift_and_fill_rows expects a LazyFrame handle, periods, and expression",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_shift_and_fill_rows(plan, *periods, fill_value) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_left_join(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(keys)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_left_join expects two LazyFrame handles and key columns",
            );
        };
        let left = match self.lazy_frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.lazy_frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_left_join(&left, right, keys) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_join(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(left_on), NativeArg::Strings(right_on), NativeArg::Text(kind), NativeArg::Text(suffix), NativeArg::Bool(nulls_equal)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_join expects two LazyFrame handles, left keys, right keys, join type, suffix, and null-equality boolean",
            );
        };
        let left = match self.lazy_frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.lazy_frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_join(&left, right, left_on, right_on, kind, suffix, *nulls_equal) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_join_with_options(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(left_on), NativeArg::Strings(right_on), NativeArg::Text(kind), NativeArg::Text(suffix), NativeArg::Bool(nulls_equal), NativeArg::Text(validation), NativeArg::Text(order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_join_with_options expects two LazyFrame handles, left keys, right keys, join type, suffix, null equality, validation, and row order",
            );
        };
        let left = match self.lazy_frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.lazy_frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_join_with_options(
            &left,
            right,
            left_on,
            right_on,
            kind,
            suffix,
            *nulls_equal,
            validation,
            order,
        ) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_join_with_coalesce(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(left_on), NativeArg::Strings(right_on), NativeArg::Text(kind), NativeArg::Text(suffix), NativeArg::Bool(nulls_equal), NativeArg::Bool(coalesce)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_join_with_coalesce expects two plans, left/right keys, join type, suffix, null equality, and coalesce booleans",
            );
        };
        let left = match self.lazy_frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.lazy_frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_join_with_coalesce(
            &left,
            right,
            left_on,
            right_on,
            kind,
            suffix,
            *nulls_equal,
            *coalesce,
        ) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_join_where(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(predicates), NativeArg::Text(suffix)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_join_where expects two LazyFrame handles, predicates, and a suffix",
            );
        };
        let left = match self.lazy_frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.lazy_frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_join_where(&left, right, predicates, suffix) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_asof_join(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Text(left_on), NativeArg::Text(right_on), NativeArg::Strings(left_by), NativeArg::Strings(right_by), NativeArg::Text(strategy), NativeArg::Text(tolerance), NativeArg::Text(suffix), NativeArg::Bool(allow_equal)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_asof_join expects two plans, join keys, grouping keys, strategy, tolerance, suffix, and equality option",
            );
        };
        let left = match self.lazy_frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.lazy_frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_asof_join(
            &left,
            right,
            left_on,
            right_on,
            left_by,
            right_by,
            strategy,
            tolerance,
            suffix,
            *allow_equal,
        ) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_dynamic_group_agg(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(index), NativeArg::Strings(group_by), NativeArg::Strings(aggregations), NativeArg::Text(every), NativeArg::Text(period), NativeArg::Text(offset), NativeArg::Text(closed), NativeArg::Text(label), NativeArg::Bool(include_boundaries)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "dynamic_group_agg expects a plan, index, grouping expressions, aggregations, window options, and boundary option",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_dynamic_group_agg(
            plan,
            index,
            group_by,
            aggregations,
            every,
            period,
            offset,
            closed,
            label,
            *include_boundaries,
        ) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_rolling_group_agg(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(index), NativeArg::Strings(group_by), NativeArg::Strings(aggregations), NativeArg::Text(period), NativeArg::Text(offset), NativeArg::Text(closed)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "rolling_group_agg expects a plan, index, grouping expressions, aggregations, and window options",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_rolling_group_agg(plan, index, group_by, aggregations, period, offset, closed) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_explode(&mut self, args: Vec<NativeArg>) -> String {
        let (handle, columns, options) = match args.as_slice() {
            [NativeArg::Handle(handle), NativeArg::Strings(columns)] => (handle, columns, None),
            [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Bool(empty_as_null), NativeArg::Bool(keep_nulls)] => {
                (handle, columns, Some((*empty_as_null, *keep_nulls)))
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "lazy_explode expects a LazyFrame handle, columns, and optional explode flags",
                )
            }
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let result = match options {
            Some((empty_as_null, keep_nulls)) => {
                lazy_explode_with_options(plan, columns, empty_as_null, keep_nulls)
            }
            None => lazy_explode(plan, columns),
        };
        match result {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_unpivot(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(on), NativeArg::Strings(index), NativeArg::Text(variable_name), NativeArg::Text(value_name)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_unpivot expects a LazyFrame handle, value columns, index columns, variable name, and value name",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_unpivot(plan, on, index, variable_name, value_name) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_unnest(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Text(separator)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "lazy_unnest expects a LazyFrame handle, columns, and separator",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_unnest(plan, columns, separator) {
            Ok(value) => self.store_result_lazy_frame_result(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_describe_plan(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Bool(optimized)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "describe_plan expects a LazyFrame handle and optimized boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_describe_plan(plan, *optimized) {
            Ok(value) => format!("ok_string {}", STANDARD.encode(value)),
            Err(error) => protocol_error(error.code(), error.message()),
        }
    }

    fn lazy_describe_plan_tree(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Bool(optimized)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "describe_plan_tree expects a LazyFrame handle and optimized boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_describe_plan_tree(plan, *optimized) {
            Ok(value) => format!("ok_string {}", STANDARD.encode(value)),
            Err(error) => protocol_error(error.code(), error.message()),
        }
    }

    fn lazy_describe_plan_dot(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Bool(optimized)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "describe_plan_dot expects a LazyFrame handle and optimized boolean",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_describe_plan_dot(plan, *optimized) {
            Ok(value) => format!("ok_string {}", STANDARD.encode(value)),
            Err(error) => protocol_error(error.code(), error.message()),
        }
    }

    fn lazy_profile_plan(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "profile_plan expects one LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_profile_plan(plan) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn lazy_profile(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "profile expects one LazyFrame handle");
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_profile(plan) {
            Ok(set) => self.store_result_dataframe_set(set),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn collect_lazy(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "collect expects one LazyFrame handle");
        };
        let plan = match self.lazy_frame(handle) {
            Ok(plan) => plan,
            Err(error) => return error,
        };
        match collect_lazy(plan) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn collect_lazy_streaming(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "collect_streaming expects one LazyFrame handle",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match collect_lazy_streaming(plan) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn collect_lazy_with_engine(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(engine)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "collect_with_engine expects a LazyFrame handle and engine name",
            );
        };
        let plan = match self.lazy_frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match lazy_collect_with_engine(plan, engine) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn select(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(names)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "select expects one DataFrame handle and a list of string columns",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match select(dataframe, names) {
            Ok(selected) => self.store_result_dataframe(selected),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn head(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(limit)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "head expects one DataFrame handle and one integer limit",
            );
        };
        let Ok(limit) = usize::try_from(*limit) else {
            return adapter_result_error("invalid_limit", "head limit cannot be negative");
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match head(dataframe, limit) {
            Ok(headed) => self.store_result_dataframe(headed),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn tail(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(limit)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "tail expects one DataFrame handle and one integer limit",
            );
        };
        let Ok(limit) = usize::try_from(*limit) else {
            return adapter_result_error("invalid_limit", "tail limit cannot be negative");
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match tail(dataframe, limit) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn clear_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "clear_rows expects a DataFrame handle");
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match clear_rows(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn rechunk(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "rechunk expects a DataFrame handle");
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match rechunk(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_sums(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "column_sums expects a DataFrame handle");
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_sums(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_means(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "column_means expects a DataFrame handle");
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_means(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_medians(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_medians expects a DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_medians(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_minima(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_minima expects a DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_minima(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_maxima(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_maxima expects a DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_maxima(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_products(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_products expects a DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_products(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_variances(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(ddof)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_variances expects a DataFrame handle and ddof",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_variances(dataframe, *ddof) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_stddevs(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(ddof)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_stddevs expects a DataFrame handle and ddof",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_stddevs(dataframe, *ddof) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_quantiles(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Float(probability), NativeArg::Text(method)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "column_quantiles expects a DataFrame handle, probability, and method",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_quantiles(dataframe, *probability, method) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_non_null_counts(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_non_null_counts expects a DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_non_null_counts(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_lengths(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_lengths expects a DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_lengths(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_unique_counts(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_unique_counts expects a DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_unique_counts(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn column_approx_unique_counts(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "column_approx_unique_counts expects a DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match column_approx_unique_counts(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn unique_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(subset), NativeArg::Text(keep), NativeArg::Bool(maintain_order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "unique_rows expects a DataFrame handle, subset columns, keep strategy, and order boolean",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match unique_rows(dataframe, subset, keep, *maintain_order) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn drop_null_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(subset)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "drop_null_rows expects a DataFrame handle and subset columns",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match drop_null_rows(dataframe, subset) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn slice_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(offset), NativeArg::Int(length)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "slice_rows expects a DataFrame handle, offset, and length",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match slice_rows(dataframe, *offset, *length) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn gather_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Ints(indices), NativeArg::Bool(null_on_out_of_bounds)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "gather_rows expects a DataFrame handle, integer indices, and null-on-out-of-bounds boolean",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match gather_rows(dataframe, indices, *null_on_out_of_bounds) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn gather_rows_every(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(step), NativeArg::Int(offset)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "gather_every expects a DataFrame handle, positive step, and non-negative offset",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match gather_rows_every(dataframe, *step, *offset) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn rename_columns(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(existing), NativeArg::Strings(new), NativeArg::Bool(strict)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "rename_columns expects a DataFrame handle, source names, destination names, and strict boolean",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match rename_columns(dataframe, existing, new, *strict) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn drop_columns(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "drop_columns expects a DataFrame handle and column names",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match drop_columns(dataframe, columns) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn drop_columns_with_options(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Bool(strict)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "drop_columns expects a DataFrame handle, column names, and strict boolean",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match drop_columns_with_options(dataframe, columns, *strict) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn reverse_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "reverse_rows expects a DataFrame handle");
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match reverse_rows(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn with_row_index(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(name), NativeArg::Int(offset)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "with_row_index expects a DataFrame handle, name, and offset",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match with_row_index(dataframe, name, *offset) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn fill_null_values(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(value)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "fill_null_values expects a DataFrame handle and expression",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match fill_null_values(dataframe, value) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn fill_null_with_strategy(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(strategy), NativeArg::Int(limit)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "fill_null_with_strategy expects a DataFrame handle, strategy, and limit",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match fill_null_with_strategy(dataframe, strategy, *limit) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn fill_nan_values(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(value)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "fill_nan_values expects a DataFrame handle and expression",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match fill_nan_values(dataframe, value) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn drop_nan_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(subset)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "drop_nan_rows expects a DataFrame handle and subset columns",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match drop_nan_rows(dataframe, subset) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn null_counts(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "null_counts expects a DataFrame handle");
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match null_counts(dataframe) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn shift_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(periods)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "shift_rows expects a DataFrame handle and periods",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match shift_rows(dataframe, *periods) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn shift_and_fill_rows(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(periods), NativeArg::Text(fill_value)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "shift_and_fill_rows expects a DataFrame handle, periods, and expression",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match shift_and_fill_rows(dataframe, *periods, fill_value) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn select_exprs(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(expressions)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "select_exprs expects a DataFrame handle and expression list",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match select_exprs(dataframe, expressions) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn select_exprs_sequential(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(expressions)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "select_exprs_sequential expects a DataFrame handle and expression list",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match select_exprs_sequential(dataframe, expressions) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn with_columns_exprs(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(expressions)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "with_columns_exprs expects a DataFrame handle and expression list",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match with_columns_exprs(dataframe, expressions) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn with_columns_sequential(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(expressions)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "with_columns_sequential expects a DataFrame handle and expression list",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match with_columns_sequential(dataframe, expressions) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn filter_expr(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(predicate)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "filter_expr expects a DataFrame handle and expression",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match filter_expr(dataframe, predicate) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn filter_all(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(predicates)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "filter_all expects a DataFrame handle and expression list",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match filter_all(dataframe, predicates) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn filter_constraints(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Strings(values)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "filter_constraints expects a DataFrame handle, column list, and expression list",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match filter_constraints(dataframe, columns, values) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn remove_where(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(predicate)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "remove_where expects a DataFrame handle and predicate",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match remove_where(dataframe, predicate) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn group_agg(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(keys), NativeArg::Strings(aggregations)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "group_agg expects a DataFrame handle, keys, and aggregations",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match group_agg(dataframe, keys, aggregations) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn map_groups(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(keys), NativeArg::Strings(expressions)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "map_groups expects a DataFrame handle, key columns, and output expressions",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        match map_groups(&dataframe, keys, expressions) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn left_join(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(keys)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "left_join expects two DataFrame handles and key columns",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match left_join(&left, right, keys) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn join(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(left_on), NativeArg::Strings(right_on), NativeArg::Text(kind), NativeArg::Text(suffix), NativeArg::Bool(nulls_equal)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "join expects two DataFrame handles, left keys, right keys, join type, suffix, and null-equality boolean",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match join_frames(&left, right, left_on, right_on, kind, suffix, *nulls_equal) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn join_with_options(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(left_on), NativeArg::Strings(right_on), NativeArg::Text(kind), NativeArg::Text(suffix), NativeArg::Bool(nulls_equal), NativeArg::Text(validation), NativeArg::Text(order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "join_with_options expects two DataFrame handles, left keys, right keys, join type, suffix, null equality, validation, and row order",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match join_with_options(
            &left,
            right,
            left_on,
            right_on,
            kind,
            suffix,
            *nulls_equal,
            validation,
            order,
        ) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn join_with_coalesce(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(left_on), NativeArg::Strings(right_on), NativeArg::Text(kind), NativeArg::Text(suffix), NativeArg::Bool(nulls_equal), NativeArg::Bool(coalesce)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "join_with_coalesce expects two frames, left/right keys, join type, suffix, null equality, and coalesce booleans",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match join_with_coalesce(
            &left,
            right,
            left_on,
            right_on,
            kind,
            suffix,
            *nulls_equal,
            *coalesce,
        ) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn join_where(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(predicates), NativeArg::Text(suffix)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "join_where expects two DataFrame handles, predicates, and a suffix",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match join_where(&left, right, predicates, suffix) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn asof_join(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Text(left_on), NativeArg::Text(right_on), NativeArg::Strings(left_by), NativeArg::Strings(right_by), NativeArg::Text(strategy), NativeArg::Text(tolerance), NativeArg::Text(suffix), NativeArg::Bool(allow_equal)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "asof_join expects two frames, join keys, grouping keys, strategy, tolerance, suffix, and equality option",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match asof_join(
            &left,
            right,
            left_on,
            right_on,
            left_by,
            right_by,
            strategy,
            tolerance,
            suffix,
            *allow_equal,
        ) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn concat_vertical(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "concat_vertical expects two DataFrame handles",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match concat_vertical(&left, right) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn concat_vertical_relaxed(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "concat_vertical_relaxed expects two DataFrame handles",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match concat_vertical_relaxed(&left, right) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn upsample(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(time_column), NativeArg::Text(every), NativeArg::Strings(group_by), NativeArg::Bool(maintain_order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "upsample expects a DataFrame, time column, interval, grouping columns, and order flag",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        match upsample(&dataframe, time_column, every, group_by, *maintain_order) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn update(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Strings(on), NativeArg::Text(how), NativeArg::Bool(include_nulls)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "update expects two DataFrames, key columns, join type, and include-nulls flag",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match update(&left, right, on, how, *include_nulls) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn concat_horizontal(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right), NativeArg::Bool(strict)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "concat_horizontal expects two DataFrame handles and a strict boolean",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match concat_horizontal(&left, right, *strict) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn concat_horizontal_extend(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "concat_horizontal_extend expects two DataFrame handles",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match concat_horizontal_extend(&left, right) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn concat_diagonal(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "concat_diagonal expects two DataFrame handles",
            );
        };
        let left = match self.frame(left) {
            Ok(value) => value.clone(),
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match concat_diagonal(&left, right) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn explode_columns(&mut self, args: Vec<NativeArg>) -> String {
        let (handle, columns, options) = match args.as_slice() {
            [NativeArg::Handle(handle), NativeArg::Strings(columns)] => (handle, columns, None),
            [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Bool(empty_as_null), NativeArg::Bool(keep_nulls)] => {
                (handle, columns, Some((*empty_as_null, *keep_nulls)))
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "explode expects a DataFrame handle, columns, and optional explode flags",
                )
            }
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        let result = match options {
            Some((empty_as_null, keep_nulls)) => {
                explode_columns_with_options(dataframe, columns, empty_as_null, keep_nulls)
            }
            None => explode_columns(dataframe, columns),
        };
        match result {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn unpivot(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(on), NativeArg::Strings(index), NativeArg::Text(variable_name), NativeArg::Text(value_name)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "unpivot expects a DataFrame handle, value columns, index columns, variable name, and value name",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match unpivot(dataframe, on, index, variable_name, value_name) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn pivot(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(on), NativeArg::Strings(index), NativeArg::Strings(values), NativeArg::Text(aggregation), NativeArg::Text(separator)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "pivot expects a DataFrame handle, on/index/value columns, aggregation, and separator",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match pivot(dataframe, on, index, values, aggregation, separator) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn unnest_columns(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Text(separator)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "unnest expects a DataFrame handle, columns, and separator",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match unnest_columns(dataframe, columns, separator) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn transpose(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(keep_names_as)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "transpose expects a DataFrame handle and retained-name column",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match transpose(dataframe, keep_names_as) {
            Ok(value) => self.store_result_dataframe(value),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    fn dispose(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "dispose expects one DataFrame handle");
        };
        if let Err(error) = self.frame(handle) {
            return error;
        }
        self.frames.remove(&handle.id);
        "ok_unit".to_string()
    }

    fn dispose_dataframe_set(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dispose_dataframe_set expects one DataFrameSet handle",
            );
        };
        if let Err(error) = self.dataframe_set(handle) {
            return error;
        }
        self.frame_sets.remove(&handle.id);
        "ok_unit".to_string()
    }

    fn dispose_lazy(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "dispose expects one LazyFrame handle");
        };
        if let Err(error) = self.lazy_frame(handle) {
            return error;
        }
        self.lazy_frames.remove(&handle.id);
        "ok_unit".to_string()
    }

    fn dispose_lazy_frame_set(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dispose_lazy_frame_set expects one LazyFrameSet handle",
            );
        };
        if let Err(error) = self.lazy_frame_set(handle) {
            return error;
        }
        self.lazy_frame_sets.remove(&handle.id);
        "ok_unit".to_string()
    }

    fn dispose_series(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dispose_series expects one Series handle",
            );
        };
        if let Err(error) = self.series(handle) {
            return error;
        }
        self.series.remove(&handle.id);
        "ok_unit".to_string()
    }

    fn dispose_sql_context(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dispose_sql_context expects one SqlContext handle",
            );
        };
        if let Err(error) = self.sql_context(handle) {
            return error;
        }
        self.sql_contexts.remove(&handle.id);
        "ok_unit".to_string()
    }

    fn store_result_dataframe(&mut self, dataframe: TerlanPolarsDataFrame) -> String {
        self.next_id = self.next_id.saturating_add(1).max(1);
        let id = self.next_id;
        let generation = 1;
        self.frames.insert(
            id,
            FrameResource {
                generation,
                dataframe,
            },
        );
        format!(
            "result_ok_handle {} {id} {generation} {}",
            STANDARD.encode(&self.owner),
            STANDARD.encode(DATAFRAME_TYPE)
        )
    }

    fn store_result_dataframe_set(&mut self, set: TerlanPolarsFrameSet) -> String {
        self.next_id = self.next_id.saturating_add(1).max(1);
        let id = self.next_id;
        let generation = 1;
        self.frame_sets
            .insert(id, FrameSetResource { generation, set });
        format!(
            "result_ok_handle {} {id} {generation} {}",
            STANDARD.encode(&self.owner),
            STANDARD.encode(DATAFRAME_SET_TYPE)
        )
    }

    fn store_result_lazy_frame(&mut self, lazy_frame: TerlanPolarsLazyFrame) -> String {
        self.next_id = self.next_id.saturating_add(1).max(1);
        let id = self.next_id;
        let generation = 1;
        self.lazy_frames.insert(
            id,
            LazyFrameResource {
                generation,
                lazy_frame,
            },
        );
        format!(
            "ok_handle {} {id} {generation} {}",
            STANDARD.encode(&self.owner),
            STANDARD.encode(LAZY_FRAME_TYPE)
        )
    }

    fn store_result_lazy_frame_set(&mut self, set: TerlanPolarsLazyFrameSet) -> String {
        self.next_id = self.next_id.saturating_add(1).max(1);
        let id = self.next_id;
        let generation = 1;
        self.lazy_frame_sets
            .insert(id, LazyFrameSetResource { generation, set });
        format!(
            "result_ok_handle {} {id} {generation} {}",
            STANDARD.encode(&self.owner),
            STANDARD.encode(LAZY_FRAME_SET_TYPE)
        )
    }

    fn store_result_lazy_frame_result(&mut self, lazy_frame: TerlanPolarsLazyFrame) -> String {
        self.next_id = self.next_id.saturating_add(1).max(1);
        let id = self.next_id;
        let generation = 1;
        self.lazy_frames.insert(
            id,
            LazyFrameResource {
                generation,
                lazy_frame,
            },
        );
        format!(
            "result_ok_handle {} {id} {generation} {}",
            STANDARD.encode(&self.owner),
            STANDARD.encode(LAZY_FRAME_TYPE)
        )
    }

    fn store_result_series(&mut self, series: TerlanPolarsSeries) -> String {
        self.next_id = self.next_id.saturating_add(1).max(1);
        let id = self.next_id;
        let generation = 1;
        self.series
            .insert(id, SeriesResource { generation, series });
        format!(
            "result_ok_handle {} {id} {generation} {}",
            STANDARD.encode(&self.owner),
            STANDARD.encode(SERIES_TYPE)
        )
    }

    fn store_sql_context(&mut self, context: TerlanPolarsSqlContext) -> String {
        self.next_id = self.next_id.saturating_add(1).max(1);
        let id = self.next_id;
        let generation = 1;
        self.sql_contexts.insert(
            id,
            SqlContextResource {
                generation,
                context,
            },
        );
        format!(
            "ok_handle {} {id} {generation} {}",
            STANDARD.encode(&self.owner),
            STANDARD.encode(SQL_CONTEXT_TYPE)
        )
    }

    fn frame(&self, handle: &NativeHandle) -> Result<&TerlanPolarsDataFrame, String> {
        if handle.owner != self.owner {
            return Err(protocol_error(
                "stale_native_handle",
                "native handle belongs to another helper process",
            ));
        }
        if handle.type_name != DATAFRAME_TYPE {
            return Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native handle {} generation {} has invalid type `{}`",
                    handle.id, handle.generation, handle.type_name
                ),
            ));
        }
        match self.frames.get(&handle.id) {
            Some(frame) if frame.generation == handle.generation => Ok(&frame.dataframe),
            _ => Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native DataFrame handle {} generation {} is not live",
                    handle.id, handle.generation
                ),
            )),
        }
    }

    fn dataframe_set(&self, handle: &NativeHandle) -> Result<&TerlanPolarsFrameSet, String> {
        if handle.owner != self.owner || handle.type_name != DATAFRAME_SET_TYPE {
            return Err(protocol_error(
                "stale_native_handle",
                "native handle is not a live DataFrameSet owned by this helper",
            ));
        }
        match self.frame_sets.get(&handle.id) {
            Some(set) if set.generation == handle.generation => Ok(&set.set),
            _ => Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native DataFrameSet handle {} generation {} is not live",
                    handle.id, handle.generation
                ),
            )),
        }
    }

    fn lazy_frame(&self, handle: &NativeHandle) -> Result<&TerlanPolarsLazyFrame, String> {
        if handle.owner != self.owner {
            return Err(protocol_error(
                "stale_native_handle",
                "native handle belongs to another helper process",
            ));
        }
        if handle.type_name != LAZY_FRAME_TYPE {
            return Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native handle {} generation {} has invalid type `{}`",
                    handle.id, handle.generation, handle.type_name
                ),
            ));
        }
        match self.lazy_frames.get(&handle.id) {
            Some(plan) if plan.generation == handle.generation => Ok(&plan.lazy_frame),
            _ => Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native LazyFrame handle {} generation {} is not live",
                    handle.id, handle.generation
                ),
            )),
        }
    }

    fn lazy_frame_set(&self, handle: &NativeHandle) -> Result<&TerlanPolarsLazyFrameSet, String> {
        if handle.owner != self.owner || handle.type_name != LAZY_FRAME_SET_TYPE {
            return Err(protocol_error(
                "stale_native_handle",
                "native handle is not a live LazyFrameSet owned by this helper",
            ));
        }
        match self.lazy_frame_sets.get(&handle.id) {
            Some(set) if set.generation == handle.generation => Ok(&set.set),
            _ => Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native LazyFrameSet handle {} generation {} is not live",
                    handle.id, handle.generation
                ),
            )),
        }
    }

    fn series(&self, handle: &NativeHandle) -> Result<&TerlanPolarsSeries, String> {
        if handle.owner != self.owner {
            return Err(protocol_error(
                "stale_native_handle",
                "native handle belongs to another helper process",
            ));
        }
        if handle.type_name != SERIES_TYPE {
            return Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native handle {} generation {} has invalid type `{}`",
                    handle.id, handle.generation, handle.type_name
                ),
            ));
        }
        match self.series.get(&handle.id) {
            Some(series) if series.generation == handle.generation => Ok(&series.series),
            _ => Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native Series handle {} generation {} is not live",
                    handle.id, handle.generation
                ),
            )),
        }
    }

    fn expr_bin_intervals(&self, args: Vec<NativeArg>, labeled: bool) -> String {
        if !matches!(args.get(1), Some(NativeArg::Handle(_))) {
            return expr_bin_intervals_args(args, labeled);
        }
        if labeled {
            let [NativeArg::Text(input), NativeArg::Handle(handle), NativeArg::Strings(labels), NativeArg::Bool(right_closed), NativeArg::Bool(include_intervals)] =
                args.as_slice()
            else {
                return protocol_error(
                    "native_bad_args",
                    "bin_intervals Series overload expects an expression, Series, labels, and two booleans",
                );
            };
            let breaks = match self.series(handle) {
                Ok(series) => series,
                Err(error) => return error,
            };
            expression_reply(
                terlan_polars_native::expressions::expr_bin_intervals_series_labeled(
                    input,
                    breaks,
                    labels,
                    *right_closed,
                    *include_intervals,
                ),
            )
        } else {
            let [NativeArg::Text(input), NativeArg::Handle(handle), NativeArg::Bool(right_closed), NativeArg::Bool(include_intervals)] =
                args.as_slice()
            else {
                return protocol_error(
                    "native_bad_args",
                    "bin_intervals Series overload expects an expression, Series, and two booleans",
                );
            };
            let breaks = match self.series(handle) {
                Ok(series) => series,
                Err(error) => return error,
            };
            expression_reply(
                terlan_polars_native::expressions::expr_bin_intervals_series(
                    input,
                    breaks,
                    *right_closed,
                    *include_intervals,
                ),
            )
        }
    }

    fn sql_context(&self, handle: &NativeHandle) -> Result<&TerlanPolarsSqlContext, String> {
        self.validate_sql_context_handle(handle)?;
        match self.sql_contexts.get(&handle.id) {
            Some(resource) if resource.generation == handle.generation => Ok(&resource.context),
            _ => Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native SqlContext handle {} generation {} is not live",
                    handle.id, handle.generation
                ),
            )),
        }
    }

    fn sql_context_mut(
        &mut self,
        handle: &NativeHandle,
    ) -> Result<&mut TerlanPolarsSqlContext, String> {
        self.validate_sql_context_handle(handle)?;
        match self.sql_contexts.get_mut(&handle.id) {
            Some(resource) if resource.generation == handle.generation => Ok(&mut resource.context),
            _ => Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native SqlContext handle {} generation {} is not live",
                    handle.id, handle.generation
                ),
            )),
        }
    }

    fn validate_sql_context_handle(&self, handle: &NativeHandle) -> Result<(), String> {
        if handle.owner != self.owner {
            return Err(protocol_error(
                "stale_native_handle",
                "native handle belongs to another helper process",
            ));
        }
        if handle.type_name != SQL_CONTEXT_TYPE {
            return Err(protocol_error(
                "stale_native_handle",
                &format!(
                    "native handle {} generation {} has invalid type `{}`",
                    handle.id, handle.generation, handle.type_name
                ),
            ));
        }
        Ok(())
    }
}

fn expression_reply(result: Result<String, terlan_polars_native::TerlanPolarsError>) -> String {
    match result {
        Ok(value) => format!("ok_string {}", STANDARD.encode(value)),
        Err(error) => protocol_error(error.code(), error.message()),
    }
}

fn expression_result_reply(
    result: Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    match result {
        Ok(value) => format!("result_ok_string {}", STANDARD.encode(value)),
        Err(error) => adapter_result_error(error.code(), error.message()),
    }
}

fn table_style_new(args: Vec<NativeArg>) -> String {
    if !args.is_empty() {
        return protocol_error("native_bad_args", "gt expects no arguments");
    }
    format!("ok_string {}", STANDARD.encode(gt()))
}

fn plot_options_new(args: Vec<NativeArg>) -> String {
    if !args.is_empty() {
        return protocol_error("native_bad_args", "plot_options expects no arguments");
    }
    format!("ok_string {}", STANDARD.encode(plot_options()))
}

fn plot_options_text(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str) -> Result<String, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(options), NativeArg::Text(value)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects PlotOptions and a string"),
        );
    };
    expression_result_reply(function(options, value))
}

fn plot_options_facet(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(options), NativeArg::Text(column), NativeArg::Int(columns)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "with_plot_facet expects PlotOptions, a column, and a grid-column count",
        );
    };
    expression_result_reply(with_plot_facet(options, column, *columns))
}

fn plot_options_dimensions(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(options), NativeArg::Int(width), NativeArg::Int(height)] = args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "with_plot_dimensions expects PlotOptions, width, and height",
        );
    };
    expression_result_reply(with_plot_dimensions(options, *width, *height))
}

fn table_style_two_text(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str) -> Result<String, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(style), NativeArg::Text(value)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects a TableStyle and string"),
        );
    };
    expression_result_reply(function(style, value))
}

fn table_style_three_text(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str, &str) -> Result<String, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(style), NativeArg::Text(first), NativeArg::Text(second)] = args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects a TableStyle and two strings"),
        );
    };
    expression_result_reply(function(style, first, second))
}

fn table_style_labels(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(style), NativeArg::Strings(columns), NativeArg::Strings(labels)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "cols_label expects a TableStyle, columns, and labels",
        );
    };
    expression_result_reply(cols_label(style, columns, labels))
}

fn table_style_columns(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &[String]) -> Result<String, TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(style), NativeArg::Strings(columns)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects a TableStyle and columns"),
        );
    };
    expression_result_reply(function(style, columns))
}

fn table_style_number(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(style), NativeArg::Strings(columns), NativeArg::Int(decimals), NativeArg::Bool(separators)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "fmt_number expects a TableStyle, columns, decimals, and separator flag",
        );
    };
    expression_result_reply(fmt_number(style, columns, *decimals, *separators))
}

fn table_style_color(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(style), NativeArg::Strings(columns), NativeArg::Text(low), NativeArg::Text(high)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "data_color expects a TableStyle, columns, low color, and high color",
        );
    };
    expression_result_reply(data_color(style, columns, low, high))
}

/// Dispatches a path-only I/O operation returning a typed string result.
fn io_path_string_result(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(path)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects one string path"),
        );
    };
    expression_result_reply(function(path))
}

/// Dispatches a path-only I/O operation returning a typed integer result.
fn io_path_int_result(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<i64, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(path)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects one string path"),
        );
    };
    match function(path) {
        Ok(value) => format!("result_ok_int {value}"),
        Err(error) => adapter_result_error(error.code(), error.message()),
    }
}

fn expression_bool_reply(result: Result<bool, terlan_polars_native::TerlanPolarsError>) -> String {
    match result {
        Ok(value) => format!("ok_bool {value}"),
        Err(error) => protocol_error(error.code(), error.message()),
    }
}

fn expression_strings_reply(
    result: Result<Vec<String>, terlan_polars_native::TerlanPolarsError>,
) -> String {
    match result {
        Ok(values) if values.is_empty() => "ok_strings".to_string(),
        Ok(values) => format!("ok_strings {}", encode_string_list(&values)),
        Err(error) => protocol_error(error.code(), error.message()),
    }
}

fn expr_int_result_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<i64, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects one expression"));
    };
    match function(input) {
        Ok(value) => format!("result_ok_int {value}"),
        Err(error) => adapter_result_error(error.code(), error.message()),
    }
}

fn expr_metadata_root_names(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input)] = args.as_slice() else {
        return protocol_error("native_bad_args", "meta_root_names expects one expression");
    };
    expression_strings_reply(expr_meta_root_names(input))
}

fn expr_metadata_list_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<Vec<String>, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects one expression"));
    };
    expression_strings_reply(function(input))
}

fn expr_metadata_bool_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<bool, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects one expression"));
    };
    expression_bool_reply(function(input))
}

fn expr_metadata_text_bool_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, bool) -> Result<bool, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Bool(option)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects one expression and one Boolean"),
        );
    };
    expression_bool_reply(function(input, *option))
}

fn expr_text_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(value)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects one string"));
    };
    expression_reply(function(value))
}

/// Dispatches a text-only expression operation with a typed result reply.
fn expr_text_result_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(value)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects one string"));
    };
    expression_result_reply(function(value))
}

fn expression_udf_parameter_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Int(index)] = args.as_slice() else {
        return protocol_error("native_bad_args", "udf_parameter expects one integer index");
    };
    expression_result_reply(expression_udf_parameter(*index))
}

fn define_expression_udf_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Int(arity), NativeArg::Text(output_type), NativeArg::Text(template)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "define_udf expects an integer arity, output type, and expression",
        );
    };
    expression_result_reply(define_expression_udf(*arity, output_type, template))
}

fn apply_expression_udf_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(udf), NativeArg::Strings(arguments)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            "apply_udf expects an expression UDF and expression list",
        );
    };
    expression_result_reply(apply_expression_udf(udf, arguments))
}

fn expr_strings_arg(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&[String]) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Strings(value)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects one string list"),
        );
    };
    expression_reply(function(value))
}

fn expr_scalar_arg(args: Vec<NativeArg>) -> String {
    let [value] = args.as_slice() else {
        return protocol_error("native_bad_args", "lit expects one scalar");
    };
    let Some(value) = native_scalar(value) else {
        return protocol_error("native_bad_args", "lit expects String, Int, Float, or Bool");
    };
    expression_reply(expr_lit(&value))
}

fn expr_int_range_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(start), NativeArg::Text(end), NativeArg::Int(step), NativeArg::Text(data_type)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "int_range expects start and end expressions, an integer step, and a data type",
        );
    };
    expression_reply(expr_int_range(start, end, *step, data_type))
}

fn expr_datetime_range_args(args: Vec<NativeArg>, ranges: bool) -> String {
    let [NativeArg::Text(start), NativeArg::Text(end), NativeArg::Text(interval), NativeArg::Text(closed), NativeArg::Text(time_unit), NativeArg::Text(time_zone)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "datetime range expects start/end expressions, interval, closed mode, time unit, and time zone",
        );
    };
    expression_reply(if ranges {
        expr_datetime_ranges(start, end, interval, closed, time_unit, time_zone)
    } else {
        expr_datetime_range(start, end, interval, closed, time_unit, time_zone)
    })
}

fn expr_duration_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Strings(values), NativeArg::Text(time_unit)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            "duration expects eight component expressions and a time unit",
        );
    };
    expression_reply(expr_duration(values, time_unit))
}

fn expr_datetime_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Strings(values), NativeArg::Text(time_unit), NativeArg::Text(time_zone)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "datetime expects eight component expressions, a time unit, and a time zone",
        );
    };
    expression_reply(expr_datetime(values, time_unit, time_zone))
}

fn expr_two_text_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(left), NativeArg::Text(right)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects two expressions/strings"),
        );
    };
    expression_reply(function(left, right))
}

fn expr_two_text_result_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(left), NativeArg::Text(right)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression and a declarative UDF"),
        );
    };
    expression_result_reply(function(left, right))
}

fn expr_text_strings_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &[String]) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Strings(values)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression and string list"),
        );
    };
    expression_reply(function(input, values))
}

fn expr_text_two_strings_result_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(
        &str,
        &[String],
        &[String],
    ) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Strings(first), NativeArg::Strings(second)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression and two string lists"),
        );
    };
    expression_result_reply(function(input, first, second))
}

fn expr_text_ints_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &[i64]) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Ints(values)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression and integer list"),
        );
    };
    expression_reply(function(input, values))
}

fn expr_sort_by_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input), NativeArg::Strings(by), NativeArg::Bools(descending), NativeArg::Bools(nulls_last), NativeArg::Bool(maintain_order)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "sort_by expects an expression, ordering expressions, descending options, null placement options, and a stability flag",
        );
    };
    expression_reply(expr_sort_by(
        input,
        by,
        descending,
        nulls_last,
        *maintain_order,
    ))
}

fn expr_over_options_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(
        &str,
        &[String],
        &[String],
        bool,
        bool,
        &str,
    ) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Strings(partition_by), NativeArg::Strings(order_by), NativeArg::Bool(descending), NativeArg::Bool(nulls_last), NativeArg::Text(mapping)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!(
                "{name} expects an expression, two expression lists, two booleans, and a mapping"
            ),
        );
    };
    expression_reply(function(
        input,
        partition_by,
        order_by,
        *descending,
        *nulls_last,
        mapping,
    ))
}

fn expr_ewm_sum_or_mean_args(args: Vec<NativeArg>, sum: bool) -> String {
    let [NativeArg::Text(input), NativeArg::Float(alpha), NativeArg::Bool(adjust), NativeArg::Int(min_samples), NativeArg::Bool(ignore_nulls)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            if sum {
                "ewm_sum expects an expression, float, boolean, integer, and boolean"
            } else {
                "ewm_mean expects an expression, float, boolean, integer, and boolean"
            },
        );
    };
    let result = if sum {
        expr_ewm_sum(input, *alpha, *adjust, *min_samples, *ignore_nulls)
    } else {
        expr_ewm_mean(input, *alpha, *adjust, *min_samples, *ignore_nulls)
    };
    expression_reply(result)
}

fn expr_ewm_statistic_args(args: Vec<NativeArg>, variance: bool) -> String {
    let [NativeArg::Text(input), NativeArg::Float(alpha), NativeArg::Bool(adjust), NativeArg::Bool(bias), NativeArg::Int(min_samples), NativeArg::Bool(ignore_nulls)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            if variance {
                "ewm_var expects an expression, float, two booleans, integer, and boolean"
            } else {
                "ewm_std expects an expression, float, two booleans, integer, and boolean"
            },
        );
    };
    let result = if variance {
        expr_ewm_var(input, *alpha, *adjust, *bias, *min_samples, *ignore_nulls)
    } else {
        expr_ewm_std(input, *alpha, *adjust, *bias, *min_samples, *ignore_nulls)
    };
    expression_reply(result)
}

fn expr_strings_text_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&[String], &str) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Strings(values), NativeArg::Text(text)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression list and string"),
        );
    };
    expression_reply(function(values, text))
}

fn expr_two_strings_text_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(
        &[String],
        &[String],
        &str,
    ) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Strings(first), NativeArg::Strings(second), NativeArg::Text(text)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects two expression lists and one expression"),
        );
    };
    expression_reply(function(first, second, text))
}

fn expr_text_int_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, i64) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Int(value)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression and integer"),
        );
    };
    expression_reply(function(input, *value))
}

fn expr_rank_random_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input), NativeArg::Bool(descending), NativeArg::Int(seed)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "rank_random expects an expression, boolean, and integer",
        );
    };
    expression_reply(expr_rank_random(input, *descending, *seed))
}

fn expr_top_k_by_args(args: Vec<NativeArg>, bottom: bool) -> String {
    let [NativeArg::Text(input), NativeArg::Text(count), NativeArg::Strings(by), NativeArg::Bools(descending)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            if bottom {
                "bottom_k_by expects two expressions, an expression list, and a boolean list"
            } else {
                "top_k_by expects two expressions, an expression list, and a boolean list"
            },
        );
    };
    if bottom {
        expression_reply(expr_bottom_k_by(input, count, by, descending))
    } else {
        expression_reply(expr_top_k_by(input, count, by, descending))
    }
}

fn expr_cut_args(args: Vec<NativeArg>, quantiles: bool) -> String {
    if quantiles {
        let [NativeArg::Text(input), NativeArg::Floats(values), NativeArg::Bool(left_closed), NativeArg::Bool(allow_duplicates), NativeArg::Bool(include_breaks)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "qcut expects an expression, float list, and three booleans",
            );
        };
        expression_reply(expr_qcut(
            input,
            values,
            *left_closed,
            *allow_duplicates,
            *include_breaks,
        ))
    } else {
        let [NativeArg::Text(input), NativeArg::Floats(values), NativeArg::Bool(left_closed), NativeArg::Bool(include_breaks)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "cut expects an expression, float list, and two booleans",
            );
        };
        expression_reply(expr_cut(input, values, *left_closed, *include_breaks))
    }
}

fn expr_cut_labeled_args(args: Vec<NativeArg>, quantiles: bool) -> String {
    if quantiles {
        let [NativeArg::Text(input), NativeArg::Floats(values), NativeArg::Strings(labels), NativeArg::Bool(left_closed), NativeArg::Bool(allow_duplicates), NativeArg::Bool(include_breaks)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "qcut_labeled expects an expression, float list, string list, and three booleans",
            );
        };
        expression_reply(expr_qcut_labeled(
            input,
            values,
            labels,
            *left_closed,
            *allow_duplicates,
            *include_breaks,
        ))
    } else {
        let [NativeArg::Text(input), NativeArg::Floats(values), NativeArg::Strings(labels), NativeArg::Bool(left_closed), NativeArg::Bool(include_breaks)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "cut_labeled expects an expression, float list, string list, and two booleans",
            );
        };
        expression_reply(expr_cut_labeled(
            input,
            values,
            labels,
            *left_closed,
            *include_breaks,
        ))
    }
}

fn expr_qcut_equal_frequency_args(args: Vec<NativeArg>, labeled: bool) -> String {
    if labeled {
        let [NativeArg::Text(input), NativeArg::Int(bins), NativeArg::Strings(labels), NativeArg::Bool(left_closed), NativeArg::Bool(allow_duplicates), NativeArg::Bool(include_breaks)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "qcut_equal_frequency_labeled expects an expression, integer, string list, and three booleans",
            );
        };
        expression_reply(expr_qcut_equal_frequency_labeled(
            input,
            *bins,
            labels,
            *left_closed,
            *allow_duplicates,
            *include_breaks,
        ))
    } else {
        let [NativeArg::Text(input), NativeArg::Int(bins), NativeArg::Bool(left_closed), NativeArg::Bool(allow_duplicates), NativeArg::Bool(include_breaks)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "qcut_equal_frequency expects an expression, integer, and three booleans",
            );
        };
        expression_reply(expr_qcut_equal_frequency(
            input,
            *bins,
            *left_closed,
            *allow_duplicates,
            *include_breaks,
        ))
    }
}

fn expr_bin_intervals_args(args: Vec<NativeArg>, labeled: bool) -> String {
    if labeled {
        let [NativeArg::Text(input), spec, NativeArg::Strings(labels), NativeArg::Bool(right_closed), NativeArg::Bool(include_intervals)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "bin_intervals_labeled expects an expression, breakpoint list or bin count, string list, and two booleans",
            );
        };
        let result = match spec {
            NativeArg::Floats(values) => expr_bin_intervals_labeled(
                input,
                values,
                labels,
                *right_closed,
                *include_intervals,
            ),
            NativeArg::Ints(values) => expr_bin_intervals_int_labeled(
                input,
                values,
                labels,
                *right_closed,
                *include_intervals,
            ),
            NativeArg::Strings(values) => expr_bin_intervals_string_labeled(
                input,
                values,
                labels,
                *right_closed,
                *include_intervals,
            ),
            NativeArg::Bools(values) => expr_bin_intervals_bool_labeled(
                input,
                values,
                labels,
                *right_closed,
                *include_intervals,
            ),
            NativeArg::Int(bins) => expr_bin_intervals_uniform_labeled(
                input,
                *bins,
                labels,
                *right_closed,
                *include_intervals,
            ),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "bin_intervals_labeled requires Float, Int, String, or Bool breakpoints, or an integer bin count",
                )
            }
        };
        expression_reply(result)
    } else {
        let [NativeArg::Text(input), spec, NativeArg::Bool(right_closed), NativeArg::Bool(include_intervals)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "bin_intervals expects an expression, breakpoint list or bin count, and two booleans",
            );
        };
        let result = match spec {
            NativeArg::Floats(values) => {
                expr_bin_intervals(input, values, *right_closed, *include_intervals)
            }
            NativeArg::Ints(values) => {
                expr_bin_intervals_int(input, values, *right_closed, *include_intervals)
            }
            NativeArg::Strings(values) => {
                expr_bin_intervals_string(input, values, *right_closed, *include_intervals)
            }
            NativeArg::Bools(values) => {
                expr_bin_intervals_bool(input, values, *right_closed, *include_intervals)
            }
            NativeArg::Int(bins) => {
                expr_bin_intervals_uniform(input, *bins, *right_closed, *include_intervals)
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "bin_intervals requires Float, Int, String, or Bool breakpoints, or an integer bin count",
                )
            }
        };
        expression_reply(result)
    }
}

fn expr_bin_quantiles_args(args: Vec<NativeArg>, labeled: bool) -> String {
    if labeled {
        let [NativeArg::Text(input), spec, NativeArg::Strings(labels), NativeArg::Bool(right_closed), NativeArg::Bool(include_intervals)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "bin_quantiles_labeled expects an expression, fraction list or bin count, string list, and two booleans",
            );
        };
        let result = match spec {
            NativeArg::Floats(values) => {
                expr_bin_quantiles_labeled(input, values, labels, *right_closed, *include_intervals)
            }
            NativeArg::Int(bins) => expr_bin_quantiles_uniform_labeled(
                input,
                *bins,
                labels,
                *right_closed,
                *include_intervals,
            ),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "bin_quantiles_labeled requires a Float list or integer bin count",
                )
            }
        };
        expression_reply(result)
    } else {
        let [NativeArg::Text(input), spec, NativeArg::Bool(right_closed), NativeArg::Bool(include_intervals)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "bin_quantiles expects an expression, fraction list or bin count, and two booleans",
            );
        };
        let result = match spec {
            NativeArg::Floats(values) => {
                expr_bin_quantiles(input, values, *right_closed, *include_intervals)
            }
            NativeArg::Int(bins) => {
                expr_bin_quantiles_uniform(input, *bins, *right_closed, *include_intervals)
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "bin_quantiles requires a Float list or integer bin count",
                )
            }
        };
        expression_reply(result)
    }
}

fn expr_bin_ranks_args(args: Vec<NativeArg>, labeled: bool) -> String {
    if labeled {
        let [NativeArg::Text(input), spec, NativeArg::Strings(labels), NativeArg::Bool(include_intervals)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "bin_ranks_labeled expects an expression, fraction list or bin count, string list, and a boolean",
            );
        };
        let result = match spec {
            NativeArg::Floats(values) => {
                expr_bin_ranks_labeled(input, values, labels, false, *include_intervals)
            }
            NativeArg::Int(bins) => {
                expr_bin_ranks_uniform_labeled(input, *bins, labels, false, *include_intervals)
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "bin_ranks_labeled requires a Float list or integer bin count",
                )
            }
        };
        expression_reply(result)
    } else {
        let [NativeArg::Text(input), spec, NativeArg::Bool(include_intervals)] = args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "bin_ranks expects an expression, fraction list or bin count, and a boolean",
            );
        };
        let result = match spec {
            NativeArg::Floats(values) => expr_bin_ranks(input, values, false, *include_intervals),
            NativeArg::Int(bins) => expr_bin_ranks_uniform(input, *bins, false, *include_intervals),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "bin_ranks requires a Float list or integer bin count",
                )
            }
        };
        expression_reply(result)
    }
}

fn expr_text_int_text_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, i64, &str) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Int(value), NativeArg::Text(option)] = args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression, integer, and string"),
        );
    };
    expression_reply(function(input, *value, option))
}

fn expr_text_float_bool_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, f64, bool) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Float(value), NativeArg::Bool(flag)] = args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression, float, and boolean"),
        );
    };
    expression_reply(function(input, *value, *flag))
}

fn expr_text_two_int_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, i64, i64) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Int(first), NativeArg::Int(second)] = args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression and two integers"),
        );
    };
    expression_reply(function(input, *first, *second))
}

fn expr_text_two_int_bool_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, i64, i64, bool) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Int(first), NativeArg::Int(second), NativeArg::Bool(flag)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression, two integers, and one boolean"),
        );
    };
    expression_reply(function(input, *first, *second, *flag))
}

/// Dispatches a weighted fixed-window expression operation.
fn expr_rolling_weighted_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(
        &str,
        i64,
        i64,
        bool,
        &[f64],
    ) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Int(window_size), NativeArg::Int(min_samples), NativeArg::Bool(center), NativeArg::Floats(weights)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression, two integers, one boolean, and float weights"),
        );
    };
    expression_reply(function(
        input,
        *window_size,
        *min_samples,
        *center,
        weights,
    ))
}

fn expr_rolling_quantile_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input), NativeArg::Int(window_size), NativeArg::Int(min_samples), NativeArg::Bool(center), NativeArg::Float(probability), NativeArg::Text(method)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "rolling_quantile expects an expression, two integers, a boolean, a float, and a method",
        );
    };
    expression_reply(expr_rolling_quantile(
        input,
        *window_size,
        *min_samples,
        *center,
        *probability,
        method,
    ))
}

/// Dispatches a weighted fixed-window quantile expression.
fn expr_rolling_quantile_weighted_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input), NativeArg::Int(window_size), NativeArg::Int(min_samples), NativeArg::Bool(center), NativeArg::Floats(weights), NativeArg::Float(probability), NativeArg::Text(method)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "rolling_quantile_weighted expects an expression, two integers, a boolean, float weights, a probability, and a method",
        );
    };
    expression_reply(expr_rolling_quantile_weighted(
        input,
        *window_size,
        *min_samples,
        *center,
        weights,
        *probability,
        method,
    ))
}

fn expr_rolling_rank_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input), NativeArg::Int(window_size), NativeArg::Int(min_samples), NativeArg::Bool(center), NativeArg::Text(method), NativeArg::Int(seed)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "rolling_rank expects an expression, two window integers, a boolean, a method, and a seed",
        );
    };
    expression_reply(expr_rolling_rank(
        input,
        *window_size,
        *min_samples,
        *center,
        method,
        *seed,
    ))
}

fn expr_rolling_skew_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input), NativeArg::Int(window_size), NativeArg::Int(min_samples), NativeArg::Bool(center), NativeArg::Bool(bias)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "rolling_skew expects an expression, two integers, and two booleans",
        );
    };
    expression_reply(expr_rolling_skew(
        input,
        *window_size,
        *min_samples,
        *center,
        *bias,
    ))
}

fn expr_rolling_kurtosis_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input), NativeArg::Int(window_size), NativeArg::Int(min_samples), NativeArg::Bool(center), NativeArg::Bool(fisher), NativeArg::Bool(bias)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "rolling_kurtosis expects an expression, two integers, and three booleans",
        );
    };
    expression_reply(expr_rolling_kurtosis(
        input,
        *window_size,
        *min_samples,
        *center,
        *fisher,
        *bias,
    ))
}

fn expr_rolling_map_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input), NativeArg::Text(udf), NativeArg::Int(window_size), NativeArg::Int(min_samples), NativeArg::Bool(center)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "rolling_map expects an expression, UDF, window size, minimum samples, and center flag",
        );
    };
    expression_reply(expr_rolling_map(
        input,
        udf,
        *window_size,
        *min_samples,
        *center,
    ))
}

fn expr_rolling_by_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(
        &str,
        &str,
        &str,
        i64,
        &str,
    ) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Text(by), NativeArg::Text(window_size), NativeArg::Int(min_samples), NativeArg::Text(closed)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!(
                "{name} expects two expressions, a duration, an integer, and a closed-window value"
            ),
        );
    };
    expression_reply(function(input, by, window_size, *min_samples, closed))
}

fn expr_rolling_quantile_by_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input), NativeArg::Text(by), NativeArg::Text(window_size), NativeArg::Int(min_samples), NativeArg::Text(closed), NativeArg::Float(probability), NativeArg::Text(method)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "rolling_quantile_by expects two expressions, a duration, an integer, a closed-window value, a probability, and a method",
        );
    };
    expression_reply(expr_rolling_quantile_by(
        input,
        by,
        window_size,
        *min_samples,
        closed,
        *probability,
        method,
    ))
}

fn expr_rolling_rank_by_args(args: Vec<NativeArg>) -> String {
    let [NativeArg::Text(input), NativeArg::Text(by), NativeArg::Text(window_size), NativeArg::Int(min_samples), NativeArg::Text(closed), NativeArg::Text(method), NativeArg::Int(seed)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            "rolling_rank_by expects two expressions, a duration, an integer, a closed-window value, a method, and a seed",
        );
    };
    expression_reply(expr_rolling_rank_by(
        input,
        by,
        window_size,
        *min_samples,
        closed,
        method,
        *seed,
    ))
}

fn expr_text_bool_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, bool) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Bool(value)] = args.as_slice() else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression and boolean"),
        );
    };
    expression_reply(function(input, *value))
}

fn expr_two_text_bool_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str, bool) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Text(value), NativeArg::Bool(flag)] = args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects two strings and one boolean"),
        );
    };
    expression_reply(function(input, value, *flag))
}

fn expr_three_text_bool_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str, &str, bool) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Text(first), NativeArg::Text(second), NativeArg::Bool(flag)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects three expressions and one boolean"),
        );
    };
    expression_reply(function(input, first, second, *flag))
}

fn expr_text_two_bool_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, bool, bool) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Bool(first), NativeArg::Bool(second)] = args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression and two booleans"),
        );
    };
    expression_reply(function(input, *first, *second))
}

fn expr_text_int_two_bool_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, i64, bool, bool) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Int(value), NativeArg::Bool(first), NativeArg::Bool(second)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression, an integer, and two booleans"),
        );
    };
    expression_reply(function(input, *value, *first, *second))
}

fn expr_text_three_bool_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, bool, bool, bool) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Bool(first), NativeArg::Bool(second), NativeArg::Bool(third)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects an expression and three booleans"),
        );
    };
    expression_reply(function(input, *first, *second, *third))
}

fn expr_two_text_two_bool_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str, bool, bool) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Text(value), NativeArg::Bool(first), NativeArg::Bool(second)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects two expressions and two booleans"),
        );
    };
    expression_reply(function(input, value, *first, *second))
}

fn expr_two_text_two_bool_int_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(
        &str,
        &str,
        bool,
        bool,
        i64,
    ) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Text(value), NativeArg::Bool(first), NativeArg::Bool(second), NativeArg::Int(seed)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects two expressions, two booleans, and an integer seed"),
        );
    };
    expression_reply(function(input, value, *first, *second, *seed))
}

fn expr_two_int_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(i64, i64) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Int(first), NativeArg::Int(second)] = args.as_slice() else {
        return protocol_error("native_bad_args", &format!("{name} expects two integers"));
    };
    expression_reply(function(*first, *second))
}

fn expr_two_text_int_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str, i64) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Text(value), NativeArg::Int(number)] = args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects two strings and one integer"),
        );
    };
    expression_reply(function(input, value, *number))
}

fn expr_three_text_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str, &str) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(first), NativeArg::Text(second), NativeArg::Text(third)] = args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects three expressions"),
        );
    };
    expression_reply(function(first, second, third))
}

fn expr_four_text_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(&str, &str, &str, &str) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(first), NativeArg::Text(second), NativeArg::Text(third), NativeArg::Text(fourth)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects four expressions/strings"),
        );
    };
    expression_reply(function(first, second, third, fourth))
}

fn expr_five_text_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(
        &str,
        &str,
        &str,
        &str,
        &str,
    ) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(first), NativeArg::Text(second), NativeArg::Text(third), NativeArg::Text(fourth), NativeArg::Text(fifth)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects five expressions/strings"),
        );
    };
    expression_reply(function(first, second, third, fourth, fifth))
}

fn expr_two_text_two_float_bool_args(
    args: Vec<NativeArg>,
    name: &str,
    function: fn(
        &str,
        &str,
        f64,
        f64,
        bool,
    ) -> Result<String, terlan_polars_native::TerlanPolarsError>,
) -> String {
    let [NativeArg::Text(input), NativeArg::Text(other), NativeArg::Float(absolute_tolerance), NativeArg::Float(relative_tolerance), NativeArg::Bool(flag)] =
        args.as_slice()
    else {
        return protocol_error(
            "native_bad_args",
            &format!("{name} expects two expressions, two floats, and one boolean"),
        );
    };
    expression_reply(function(
        input,
        other,
        *absolute_tolerance,
        *relative_tolerance,
        *flag,
    ))
}

fn native_scalar(value: &NativeArg) -> Option<TerlanPolarsScalar> {
    match value {
        NativeArg::Text(value) => Some(TerlanPolarsScalar::String(value.clone())),
        NativeArg::Int(value) => Some(TerlanPolarsScalar::Int(*value)),
        NativeArg::Float(value) => Some(TerlanPolarsScalar::Float(*value)),
        NativeArg::Bool(value) => Some(TerlanPolarsScalar::Bool(*value)),
        _ => None,
    }
}

struct Request {
    request_id: u64,
    operation: String,
    args: Vec<NativeArg>,
}

fn parse_request(line: &str) -> Result<Request, String> {
    let mut parts = line.split_whitespace();
    match parts.next() {
        Some("call") => {}
        _ => {
            return Err(protocol_error(
                "native_boundary_bad_request",
                "expected call request",
            ))
        }
    }
    let request_id = parts
        .next()
        .ok_or_else(|| protocol_error("native_boundary_bad_request", "missing request id"))?
        .parse::<u64>()
        .map_err(|error| {
            protocol_error(
                "native_boundary_bad_request",
                &format!("invalid request id: {error}"),
            )
        })?;
    let operation = parts
        .next()
        .ok_or_else(|| protocol_error("native_boundary_bad_request", "missing operation"))?;
    let operation = decode_text(operation)
        .map_err(|message| protocol_error("native_boundary_bad_operation", &message))?;
    let args = parts
        .map(parse_arg)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|message| protocol_error("native_boundary_bad_arg", &message))?;
    Ok(Request {
        request_id,
        operation,
        args,
    })
}

fn parse_arg(value: &str) -> Result<NativeArg, String> {
    if let Some(encoded) = value.strip_prefix("s:") {
        return decode_text(encoded).map(NativeArg::Text);
    }
    if let Some(encoded) = value.strip_prefix("x:") {
        return STANDARD
            .decode(encoded)
            .map(NativeArg::Bytes)
            .map_err(|error| format!("invalid Bytes argument: {error}"));
    }
    if let Some(encoded) = value.strip_prefix("a:") {
        return decode_text(encoded).map(NativeArg::Text);
    }
    if let Some(encoded) = value.strip_prefix("i:") {
        return encoded
            .parse()
            .map(NativeArg::Int)
            .map_err(|error| format!("invalid integer argument: {error}"));
    }
    if let Some(encoded) = value.strip_prefix("f:") {
        return encoded
            .parse()
            .map(NativeArg::Float)
            .map_err(|error| format!("invalid float argument: {error}"));
    }
    if let Some(encoded) = value.strip_prefix("b:") {
        return encoded
            .parse()
            .map(NativeArg::Bool)
            .map_err(|error| format!("invalid boolean argument: {error}"));
    }
    if let Some(encoded) = value.strip_prefix("ls:") {
        return decode_string_list(encoded).map(NativeArg::Strings);
    }
    if let Some(encoded) = value.strip_prefix("li:") {
        return decode_primitive_list(encoded, "integer", str::parse).map(NativeArg::Ints);
    }
    if let Some(encoded) = value.strip_prefix("lf:") {
        return decode_primitive_list(encoded, "float", str::parse).map(NativeArg::Floats);
    }
    if let Some(encoded) = value.strip_prefix("lb:") {
        return decode_primitive_list(encoded, "boolean", str::parse).map(NativeArg::Bools);
    }
    if let Some(encoded) = value.strip_prefix("la:") {
        return decode_string_list(encoded).map(NativeArg::Strings);
    }
    if let Some(encoded) = value.strip_prefix("los:") {
        return decode_optional_list(encoded, "string", |value| {
            let encoded = value
                .strip_prefix('s')
                .ok_or_else(|| "nullable string value is missing its `s` tag".to_string())?;
            decode_text(encoded)
        })
        .map(NativeArg::NullableStrings);
    }
    if let Some(encoded) = value.strip_prefix("loi:") {
        return decode_optional_list(encoded, "integer", parse_tagged_primitive)
            .map(NativeArg::NullableInts);
    }
    if let Some(encoded) = value.strip_prefix("lof:") {
        return decode_optional_list(encoded, "float", parse_tagged_primitive)
            .map(NativeArg::NullableFloats);
    }
    if let Some(encoded) = value.strip_prefix("lob:") {
        return decode_optional_list(encoded, "boolean", parse_tagged_primitive)
            .map(NativeArg::NullableBools);
    }
    if let Some(encoded) = value.strip_prefix("lon:") {
        return encoded
            .parse()
            .map(NativeArg::Nulls)
            .map_err(|error| format!("invalid all-null list length: {error}"));
    }
    if let Some(encoded) = value.strip_prefix("lss:") {
        return decode_string_rows(encoded).map(NativeArg::StringRows);
    }
    if let Some(rest) = value.strip_prefix("h:") {
        let fields = rest.split(':').collect::<Vec<_>>();
        let [owner, id, generation, type_name] = fields.as_slice() else {
            return Err("invalid handle argument".to_string());
        };
        let owner = decode_text(owner)?;
        let type_name = decode_text(type_name)?;
        return Ok(NativeArg::Handle(NativeHandle {
            owner,
            id: id
                .parse()
                .map_err(|error| format!("invalid handle id: {error}"))?,
            generation: generation
                .parse()
                .map_err(|error| format!("invalid handle generation: {error}"))?,
            type_name,
        }));
    }
    Err("unsupported argument encoding".to_string())
}

fn decode_text(encoded: &str) -> Result<String, String> {
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|error| format!("invalid base64 text: {error}"))?;
    String::from_utf8(bytes).map_err(|error| format!("invalid UTF-8 text: {error}"))
}

fn decode_string_list(encoded: &str) -> Result<Vec<String>, String> {
    if encoded.is_empty() {
        return Ok(Vec::new());
    }
    encoded.split(',').map(decode_text).collect()
}

fn decode_primitive_list<T>(
    encoded: &str,
    label: &str,
    parse: impl Fn(&str) -> Result<T, T::Err>,
) -> Result<Vec<T>, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    if encoded.is_empty() {
        return Ok(Vec::new());
    }
    encoded
        .split(',')
        .map(|value| parse(value).map_err(|error| format!("invalid {label} list value: {error}")))
        .collect()
}

fn decode_optional_list<T>(
    encoded: &str,
    label: &str,
    parse: impl Fn(&str) -> Result<T, String>,
) -> Result<Vec<Option<T>>, String> {
    if encoded.is_empty() {
        return Ok(Vec::new());
    }
    encoded
        .split(',')
        .map(|value| {
            if value == "n" {
                Ok(None)
            } else {
                parse(value)
                    .map(Some)
                    .map_err(|error| format!("invalid nullable {label} argument: {error}"))
            }
        })
        .collect()
}

fn parse_tagged_primitive<T>(value: &str) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value
        .strip_prefix('v')
        .ok_or_else(|| "value is missing its `v` tag".to_string())?
        .parse::<T>()
        .map_err(|error| error.to_string())
}

fn decode_string_rows(encoded: &str) -> Result<Vec<Vec<String>>, String> {
    if encoded.is_empty() {
        return Ok(Vec::new());
    }
    encoded
        .split(',')
        .map(|encoded_row| {
            let row = decode_text(encoded_row)?;
            let (count, values) = row
                .split_once('|')
                .ok_or_else(|| "invalid nested string row".to_string())?;
            let count = count
                .parse::<usize>()
                .map_err(|error| format!("invalid nested string row count: {error}"))?;
            let values = decode_string_list(values)?;
            if values.len() != count {
                return Err(format!(
                    "nested string row declared {count} values but encoded {}",
                    values.len()
                ));
            }
            Ok(values)
        })
        .collect()
}

fn encode_string_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| STANDARD.encode(value))
        .collect::<Vec<_>>()
        .join(",")
}

fn encode_string_rows(rows: &[Vec<String>]) -> String {
    rows.iter()
        .map(|row| {
            let values = row
                .iter()
                .map(|value| STANDARD.encode(value))
                .collect::<Vec<_>>()
                .join(",");
            STANDARD.encode(format!("{}|{values}", row.len()))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn encode_schema(values: &[TerlanPolarsColumnSchema]) -> String {
    values
        .iter()
        .map(|column| {
            format!(
                "{}:{}",
                STANDARD.encode(&column.name),
                STANDARD.encode(&column.data_type)
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn adapter_result_error(code: &str, message: &str) -> String {
    format!(
        "result_err {} {}",
        STANDARD.encode(code),
        STANDARD.encode(message)
    )
}

fn encode_bytes_result(result: Result<Vec<u8>, TerlanPolarsError>) -> String {
    match result {
        Ok(bytes) if bytes.is_empty() => "result_ok_bytes".to_string(),
        Ok(bytes) => format!("result_ok_bytes {}", STANDARD.encode(bytes)),
        Err(error) => adapter_result_error(error.code(), error.message()),
    }
}

fn protocol_error(code: &str, message: &str) -> String {
    format!("err {} {}", STANDARD.encode(code), STANDARD.encode(message))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call_line(operation: &str, args: &[String]) -> String {
        call_line_with_request_id(1, operation, args)
    }

    fn call_line_with_request_id(request_id: u64, operation: &str, args: &[String]) -> String {
        let mut parts = vec![
            "call".to_string(),
            request_id.to_string(),
            STANDARD.encode(operation),
        ];
        parts.extend(args.iter().cloned());
        parts.join(" ")
    }

    fn reply_payload(reply: &str) -> &str {
        reply
            .strip_prefix(&format!("reply 1 {CREDIT_WINDOW} "))
            .unwrap_or_else(|| panic!("expected correlated reply, got `{reply}`"))
    }

    fn parse_string_reply(reply: &str, expected_tag: &str) -> String {
        let parts = reply_payload(reply).split_whitespace().collect::<Vec<_>>();
        let [tag, value] = parts.as_slice() else {
            panic!("expected {expected_tag} string reply, got `{reply}`");
        };
        assert_eq!(*tag, expected_tag);
        decode_text(value).expect("string reply should decode")
    }

    #[cfg(feature = "real-polars")]
    fn parse_bytes_reply(reply: &str, expected_tag: &str) -> Vec<u8> {
        let parts = reply_payload(reply).split_whitespace().collect::<Vec<_>>();
        let [tag, value] = parts.as_slice() else {
            panic!("expected {expected_tag} Bytes reply, got `{reply}`");
        };
        assert_eq!(*tag, expected_tag);
        STANDARD.decode(value).expect("Bytes reply should decode")
    }

    fn string_arg(value: &str) -> String {
        format!("s:{}", STANDARD.encode(value))
    }

    fn int_arg(value: i64) -> String {
        format!("i:{value}")
    }

    fn string_list_arg(values: &[&str]) -> String {
        format!(
            "ls:{}",
            values
                .iter()
                .map(|value| STANDARD.encode(value))
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    fn string_rows_arg(rows: &[&[&str]]) -> String {
        let rows = rows
            .iter()
            .map(|row| {
                let values = row
                    .iter()
                    .map(|value| STANDARD.encode(value))
                    .collect::<Vec<_>>()
                    .join(",");
                STANDARD.encode(format!("{}|{values}", row.len()))
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("lss:{rows}")
    }

    fn handle_arg(handle: &NativeHandle) -> String {
        format!(
            "h:{}:{}:{}:{}",
            STANDARD.encode(&handle.owner),
            handle.id,
            handle.generation,
            STANDARD.encode(&handle.type_name)
        )
    }

    #[cfg(feature = "real-polars")]
    fn parse_result_handle(reply: &str) -> NativeHandle {
        let parts = reply_payload(reply).split_whitespace().collect::<Vec<_>>();
        let [tag, owner, id, generation, type_name] = parts.as_slice() else {
            panic!("expected result_ok_handle reply, got `{reply}`");
        };
        assert_eq!(*tag, "result_ok_handle");
        NativeHandle {
            owner: decode_text(owner).expect("handle owner should decode"),
            id: id.parse().expect("handle id should parse"),
            generation: generation.parse().expect("handle generation should parse"),
            type_name: decode_text(type_name).expect("handle type should decode"),
        }
    }

    #[cfg(feature = "real-polars")]
    fn parse_plain_handle(reply: &str) -> NativeHandle {
        let parts = reply_payload(reply).split_whitespace().collect::<Vec<_>>();
        let [tag, owner, id, generation, type_name] = parts.as_slice() else {
            panic!("expected ok_handle reply, got `{reply}`");
        };
        assert_eq!(*tag, "ok_handle");
        NativeHandle {
            owner: decode_text(owner).expect("handle owner should decode"),
            id: id.parse().expect("handle id should parse"),
            generation: generation.parse().expect("handle generation should parse"),
            type_name: decode_text(type_name).expect("handle type should decode"),
        }
    }

    #[cfg(feature = "real-polars")]
    fn fixture_path(name: &str) -> String {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test")
            .join("fixtures")
            .join(name)
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn parser_decodes_call_request() {
        let line = call_line(
            "polars.dataframe.select",
            &[string_arg("path.csv"), string_list_arg(&["name", "city"])],
        );

        let request = parse_request(&line).expect("request should parse");

        assert_eq!(request.request_id, 1);
        assert_eq!(request.operation, "polars.dataframe.select");
        assert!(matches!(request.args.first(), Some(NativeArg::Text(path)) if path == "path.csv"));
        assert!(
            matches!(request.args.get(1), Some(NativeArg::Strings(values)) if values == &["name", "city"])
        );
    }

    #[test]
    fn parser_decodes_integer_argument() {
        let line = call_line("polars.dataframe.head", &[int_arg(3)]);

        let request = parse_request(&line).expect("request should parse");

        assert_eq!(request.operation, "polars.dataframe.head");
        assert!(matches!(request.args.first(), Some(NativeArg::Int(3))));
    }

    #[test]
    fn parser_decodes_float_and_boolean_arguments() {
        let request = parse_request(&call_line(
            "polars.dataframe.filter_eq",
            &["f:1.5".to_string(), "b:true".to_string()],
        ))
        .expect("request should parse");

        assert!(matches!(request.args.first(), Some(NativeArg::Float(value)) if *value == 1.5));
        assert!(matches!(request.args.get(1), Some(NativeArg::Bool(true))));
    }

    #[test]
    fn parser_decodes_atoms_and_typed_primitive_lists() {
        let request = parse_request(&call_line(
            "polars.protocol.typed",
            &[
                format!("a:{}", STANDARD.encode("int32")),
                format!(
                    "la:{},{}",
                    STANDARD.encode("int64"),
                    STANDARD.encode("float64")
                ),
                "li:1,-2".to_string(),
                "lf:1.5,-2.25".to_string(),
                "lb:true,false".to_string(),
            ],
        ))
        .expect("request should parse");

        assert!(matches!(request.args.first(), Some(NativeArg::Text(value)) if value == "int32"));
        assert!(
            matches!(request.args.get(1), Some(NativeArg::Strings(values)) if values == &["int64", "float64"])
        );
        assert!(matches!(request.args.get(2), Some(NativeArg::Ints(values)) if values == &[1, -2]));
        assert!(
            matches!(request.args.get(3), Some(NativeArg::Floats(values)) if values == &[1.5, -2.25])
        );
        assert!(
            matches!(request.args.get(4), Some(NativeArg::Bools(values)) if values == &[true, false])
        );
    }

    #[test]
    fn parser_decodes_nullable_primitive_lists() {
        let request = parse_request(&call_line(
            "polars.protocol.nullable",
            &[
                format!("los:s{},n", STANDARD.encode("Ada")),
                "loi:v1,n".to_string(),
                "lof:n,v2.5".to_string(),
                "lob:vtrue,n".to_string(),
                "lon:2".to_string(),
            ],
        ))
        .expect("request should parse");

        assert!(
            matches!(request.args.first(), Some(NativeArg::NullableStrings(values)) if values == &[Some("Ada".to_string()), None])
        );
        assert!(
            matches!(request.args.get(1), Some(NativeArg::NullableInts(values)) if values == &[Some(1), None])
        );
        assert!(
            matches!(request.args.get(2), Some(NativeArg::NullableFloats(values)) if values == &[None, Some(2.5)])
        );
        assert!(
            matches!(request.args.get(3), Some(NativeArg::NullableBools(values)) if values == &[Some(true), None])
        );
        assert!(matches!(request.args.get(4), Some(NativeArg::Nulls(2))));
    }

    #[test]
    fn parser_decodes_nested_string_rows() {
        let request = parse_request(&call_line(
            "polars.dataframe.from_rows",
            &[string_rows_arg(&[&["Ada", "London"], &[]])],
        ))
        .expect("request should parse");

        assert!(matches!(
            request.args.first(),
            Some(NativeArg::StringRows(rows))
                if rows == &[vec!["Ada".to_string(), "London".to_string()], vec![]]
        ));
    }

    #[test]
    fn helper_applies_declarative_expression_udf() {
        let mut worker = Worker::default();
        let parameter = parse_string_reply(
            &worker.execute_line(&call_line("polars.expr.udf_parameter", &[int_arg(0)])),
            "result_ok_string",
        );
        let udf = parse_string_reply(
            &worker.execute_line(&call_line(
                "polars.expr.define_udf",
                &[int_arg(1), string_arg("Int64"), string_arg(&parameter)],
            )),
            "result_ok_string",
        );
        let column = parse_string_reply(
            &worker.execute_line(&call_line("polars.expr.col", &[string_arg("value")])),
            "ok_string",
        );
        let applied = parse_string_reply(
            &worker.execute_line(&call_line(
                "polars.expr.apply_udf",
                &[string_arg(&udf), string_list_arg(&[&column])],
            )),
            "result_ok_string",
        );

        assert!(applied.contains("\"op\":\"column\""));
        assert!(!applied.contains("\"op\":\"parameter\""));

        let wrong_arity = worker.execute_line(&call_line(
            "polars.expr.apply_udf",
            &[string_arg(&udf), string_list_arg(&[])],
        ));
        assert!(reply_payload(&wrong_arity).starts_with("result_err "));
    }

    #[test]
    fn helper_encodes_sql_expression_results() {
        let mut worker = Worker::default();
        let valid = worker.execute_line(&call_line("polars.expr.sql", &[string_arg("score + 1")]));
        assert!(reply_payload(&valid).starts_with("result_ok_string "));

        let invalid = worker.execute_line(&call_line("polars.expr.sql", &[string_arg("")]));
        assert!(reply_payload(&invalid).starts_with("result_err "));
    }

    #[test]
    fn helper_preserves_stable_generic_sort_option() {
        let mut worker = Worker::default();
        let column = parse_string_reply(
            &worker.execute_line(&call_line("polars.expr.col", &[string_arg("value")])),
            "ok_string",
        );
        let sorted = parse_string_reply(
            &worker.execute_line(&call_line(
                "polars.expr.sort_with_stability",
                &[
                    string_arg(&column),
                    "b:false".to_string(),
                    "b:false".to_string(),
                    "b:true".to_string(),
                ],
            )),
            "ok_string",
        );
        let payload: serde_json::Value =
            serde_json::from_str(&sorted).expect("stable sort expression JSON");

        assert_eq!(payload["op"], "sort_options");
        assert_eq!(payload["maintain_order"], true);
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn helper_round_trips_dataframe_through_arrow_ipc_bytes() {
        let mut worker = Worker::default();
        let source = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let bytes = parse_bytes_reply(
            &worker.execute_line(&call_line(
                "polars.dataframe.to_arrow_ipc",
                &[handle_arg(&source)],
            )),
            "result_ok_bytes",
        );
        let restored = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.from_arrow_ipc",
            &[format!("x:{}", STANDARD.encode(bytes))],
        )));

        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&restored)]
            ))),
            "ok_int 2"
        );
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.width",
                &[handle_arg(&restored)]
            ))),
            "ok_int 3"
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn helper_renders_all_supported_svg_plot_types() {
        let mut worker = Worker::default();
        let source = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let cases = [
            (
                "polars.visualization.scatter_svg",
                vec!["age", "age", "city", "Scatter"],
            ),
            ("polars.visualization.line_svg", vec!["age", "age", "Line"]),
            (
                "polars.visualization.bar_svg",
                vec!["city", "age", "", "Bar"],
            ),
            (
                "polars.visualization.histogram_svg",
                vec!["age", "", "Histogram"],
            ),
            (
                "polars.visualization.box_svg",
                vec!["city", "age", "", "Box"],
            ),
        ];

        for (operation, values) in cases {
            let mut args = vec![handle_arg(&source)];
            args.extend(values.into_iter().map(string_arg));
            let bytes = parse_bytes_reply(
                &worker.execute_line(&call_line(operation, &args)),
                "result_ok_bytes",
            );
            assert!(std::str::from_utf8(&bytes).unwrap().contains("<svg"));
        }
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn helper_composes_plot_options_and_renders_grouped_line() {
        let mut worker = Worker::default();
        let source = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let options = parse_string_reply(
            &worker.execute_line(&call_line("polars.visualization.options", &[])),
            "ok_string",
        );
        let options = parse_string_reply(
            &worker.execute_line(&call_line(
                "polars.visualization.options.color",
                &[string_arg(&options), string_arg("city")],
            )),
            "result_ok_string",
        );
        let options = parse_string_reply(
            &worker.execute_line(&call_line(
                "polars.visualization.options.dimensions",
                &[
                    string_arg(&options),
                    "i:720".to_string(),
                    "i:480".to_string(),
                ],
            )),
            "result_ok_string",
        );
        let bytes = parse_bytes_reply(
            &worker.execute_line(&call_line(
                "polars.visualization.render_svg",
                &[
                    handle_arg(&source),
                    string_arg("line"),
                    string_arg("age"),
                    string_arg("age"),
                    string_arg(&options),
                ],
            )),
            "result_ok_bytes",
        );
        let svg = std::str::from_utf8(&bytes).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("720"));
        assert!(svg.contains("480"));
    }

    #[cfg(feature = "plotly-html")]
    #[test]
    fn helper_renders_all_optional_plotly_html_types() {
        let mut worker = Worker::default();
        let source = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let cases = [
            (
                "polars.visualization.scatter_html",
                vec!["age", "age", "city", "Scatter"],
            ),
            ("polars.visualization.line_html", vec!["age", "age", "Line"]),
            (
                "polars.visualization.bar_html",
                vec!["city", "age", "", "Bar"],
            ),
            (
                "polars.visualization.histogram_html",
                vec!["age", "", "Histogram"],
            ),
            (
                "polars.visualization.box_html",
                vec!["city", "age", "", "Box"],
            ),
        ];

        for (operation, values) in cases {
            let mut args = vec![handle_arg(&source)];
            args.extend(values.into_iter().map(string_arg));
            let bytes = parse_bytes_reply(
                &worker.execute_line(&call_line(operation, &args)),
                "result_ok_bytes",
            );
            let html = std::str::from_utf8(&bytes).unwrap();
            assert!(html.contains("plotly") || html.contains("Plotly"));
        }
    }

    #[cfg(feature = "plot-png")]
    #[test]
    fn helper_renders_all_optional_png_types() {
        let mut worker = Worker::default();
        let source = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let cases = [
            (
                "polars.visualization.scatter_png",
                vec!["age", "age", "city", "Scatter"],
            ),
            ("polars.visualization.line_png", vec!["age", "age", "Line"]),
            (
                "polars.visualization.bar_png",
                vec!["city", "age", "", "Bar"],
            ),
            (
                "polars.visualization.histogram_png",
                vec!["age", "", "Histogram"],
            ),
            (
                "polars.visualization.box_png",
                vec!["city", "age", "", "Box"],
            ),
        ];

        for (operation, values) in cases {
            let mut args = vec![handle_arg(&source)];
            args.extend(values.into_iter().map(string_arg));
            let bytes = parse_bytes_reply(
                &worker.execute_line(&call_line(operation, &args)),
                "result_ok_bytes",
            );
            assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        }
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn helper_builds_declarative_style_and_renders_bounded_html() {
        let mut worker = Worker::default();
        let source = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let style = parse_string_reply(
            &worker.execute_line(&call_line("polars.table.gt", &[])),
            "ok_string",
        );
        let style = parse_string_reply(
            &worker.execute_line(&call_line(
                "polars.table.tab_header",
                &[
                    string_arg(&style),
                    string_arg("People <table>"),
                    string_arg("A & B"),
                ],
            )),
            "result_ok_string",
        );
        let style = parse_string_reply(
            &worker.execute_line(&call_line(
                "polars.table.tab_stub",
                &[string_arg(&style), string_arg("name")],
            )),
            "result_ok_string",
        );
        let style = parse_string_reply(
            &worker.execute_line(&call_line(
                "polars.table.fmt_number",
                &[
                    string_arg(&style),
                    string_list_arg(&["age"]),
                    "i:2".to_string(),
                    "b:true".to_string(),
                ],
            )),
            "result_ok_string",
        );
        let html = parse_bytes_reply(
            &worker.execute_line(&call_line(
                "polars.table.to_html",
                &[handle_arg(&source), string_arg(&style), "i:1".to_string()],
            )),
            "result_ok_bytes",
        );
        let html = std::str::from_utf8(&html).unwrap();
        assert!(html.contains("People &lt;table&gt;"));
        assert!(html.contains("A &amp; B"));
        assert!(html.contains("scope=\"row\""));
        assert!(html.contains("1 of 2 rows rendered"));
    }

    #[test]
    fn parser_preserves_handle_type_name() {
        let line = call_line(
            "polars.dataframe.height",
            &[handle_arg(&NativeHandle {
                owner: "fixture-owner".to_string(),
                id: 7,
                generation: 3,
                type_name: DATAFRAME_TYPE.to_string(),
            })],
        );

        let request = parse_request(&line).expect("request should parse");

        assert!(
            matches!(request.args.first(), Some(NativeArg::Handle(handle)) if handle.id == 7 && handle.generation == 3 && handle.type_name == DATAFRAME_TYPE)
        );
    }

    #[test]
    fn default_worker_reports_unavailable_read_csv_as_result_error() {
        let mut worker = Worker::default();
        let line = call_line("polars.dataframe.read_csv", &[string_arg("missing.csv")]);

        let reply = worker.execute_line(&line);

        assert!(reply_payload(&reply).starts_with("result_err "));
    }

    #[test]
    fn worker_rejects_handle_with_wrong_type_name() {
        let mut worker = Worker::default();
        let handle = NativeHandle {
            owner: worker.owner.clone(),
            id: 1,
            generation: 1,
            type_name: "other.Type".to_string(),
        };
        let reply = worker.execute_line(&call_line(
            "polars.dataframe.height",
            &[handle_arg(&handle)],
        ));

        assert!(reply_payload(&reply).starts_with("err "));
        assert!(reply.contains(&STANDARD.encode("stale_native_handle")));
    }

    #[test]
    fn worker_correlates_replies_and_advertises_credit() {
        let mut worker = Worker::default();
        let reply = worker.execute_line(&call_line_with_request_id(
            42,
            "polars.dataframe.read_csv",
            &[string_arg("missing.csv")],
        ));

        assert!(reply.starts_with(&format!("reply 42 {CREDIT_WINDOW} result_err ")));
    }

    #[test]
    fn worker_dispatches_replacement_and_numeric_binning_expressions() {
        let mut worker = Worker::default();
        let input = expr_col("value").unwrap();
        let series_reply = worker.execute_line(&call_line(
            "polars.series.from_ints",
            &[string_arg("breaks"), "li:1,3".to_string()],
        ));
        let breakpoint_series = parse_result_handle(&series_reply);
        let old = expr_lit(&TerlanPolarsScalar::Int(2)).unwrap();
        let new = expr_lit(&TerlanPolarsScalar::Int(20)).unwrap();
        let default = expr_lit(&TerlanPolarsScalar::Int(-1)).unwrap();
        let calls = [
            (
                "polars.expr.replace",
                vec![string_arg(&input), string_arg(&old), string_arg(&new)],
            ),
            (
                "polars.expr.replace_strict",
                vec![string_arg(&input), string_arg(&old), string_arg(&new)],
            ),
            (
                "polars.expr.replace_or_default",
                vec![
                    string_arg(&input),
                    string_arg(&old),
                    string_arg(&new),
                    string_arg(&default),
                ],
            ),
            (
                "polars.expr.cut",
                vec![
                    string_arg(&input),
                    "lf:1,3".to_string(),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.cut_labeled",
                vec![
                    string_arg(&input),
                    "lf:1,3".to_string(),
                    string_list_arg(&["low", "mid", "high"]),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.qcut",
                vec![
                    string_arg(&input),
                    "lf:0.5".to_string(),
                    "b:false".to_string(),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.qcut_labeled",
                vec![
                    string_arg(&input),
                    "lf:0.5".to_string(),
                    string_list_arg(&["low", "high"]),
                    "b:false".to_string(),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.qcut_equal_frequency",
                vec![
                    string_arg(&input),
                    int_arg(2),
                    "b:false".to_string(),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.qcut_equal_frequency_labeled",
                vec![
                    string_arg(&input),
                    int_arg(2),
                    string_list_arg(&["low", "high"]),
                    "b:false".to_string(),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.bin_intervals",
                vec![
                    string_arg(&input),
                    "li:1,3".to_string(),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.bin_intervals",
                vec![
                    string_arg(&input),
                    string_list_arg(&["a", "m"]),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.bin_intervals_labeled",
                vec![
                    string_arg(&input),
                    int_arg(2),
                    string_list_arg(&["low", "high"]),
                    "b:true".to_string(),
                    "b:true".to_string(),
                ],
            ),
            (
                "polars.expr.bin_intervals",
                vec![
                    string_arg(&input),
                    handle_arg(&breakpoint_series),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.bin_quantiles",
                vec![
                    string_arg(&input),
                    "lf:0.25,0.25,0.75".to_string(),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.bin_quantiles_labeled",
                vec![
                    string_arg(&input),
                    int_arg(2),
                    string_list_arg(&["low", "high"]),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.expr.bin_ranks",
                vec![string_arg(&input), int_arg(2), "b:false".to_string()],
            ),
            (
                "polars.expr.bin_ranks_labeled",
                vec![
                    string_arg(&input),
                    "lf:0.5".to_string(),
                    string_list_arg(&["low", "high"]),
                    "b:true".to_string(),
                ],
            ),
        ];

        for (operation, args) in calls {
            let reply = worker.execute_line(&call_line(operation, &args));
            assert!(
                reply_payload(&reply).starts_with("ok_string "),
                "{operation} failed: {reply}"
            );
        }

        let input_reply = worker.execute_line(&call_line(
            "polars.series.from_ints",
            &[string_arg("value"), "li:-2,-1,0,1,2".to_string()],
        ));
        let input_series = parse_result_handle(&input_reply);
        let size_reply = worker.execute_line(&call_line(
            "polars.series.estimated_size_with_unit",
            &[handle_arg(&input_series), string_arg("kb")],
        ));
        assert!(reply_payload(&size_reply).starts_with("result_ok_float "));
        let series_calls = [
            (
                "polars.series.bin_intervals",
                vec![
                    handle_arg(&input_series),
                    "li:-1,1".to_string(),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.series.bin_intervals_labeled",
                vec![
                    handle_arg(&input_series),
                    handle_arg(&breakpoint_series),
                    string_list_arg(&["low", "mid", "high"]),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.series.bin_quantiles",
                vec![
                    handle_arg(&input_series),
                    "lf:0.5".to_string(),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.series.bin_quantiles_labeled",
                vec![
                    handle_arg(&input_series),
                    int_arg(2),
                    string_list_arg(&["low", "high"]),
                    "b:false".to_string(),
                    "b:false".to_string(),
                ],
            ),
            (
                "polars.series.bin_ranks",
                vec![handle_arg(&input_series), int_arg(2), "b:false".to_string()],
            ),
            (
                "polars.series.bin_ranks_labeled",
                vec![
                    handle_arg(&input_series),
                    "lf:0.5".to_string(),
                    string_list_arg(&["low", "high"]),
                    "b:false".to_string(),
                ],
            ),
        ];
        for (operation, args) in series_calls {
            let reply = worker.execute_line(&call_line(operation, &args));
            assert!(
                reply_payload(&reply).starts_with("result_ok_handle "),
                "{operation} failed: {reply}"
            );
        }
    }

    #[test]
    fn worker_dispatches_datetime_and_repeat_with_expression_arguments() {
        let mut worker = Worker::default();
        let year = expr_col("year").unwrap();
        let month = expr_col("month").unwrap();
        let day = expr_col("day").unwrap();
        let zero = expr_lit(&TerlanPolarsScalar::Int(0)).unwrap();
        let ambiguous = expr_lit(&TerlanPolarsScalar::String("raise".into())).unwrap();
        let components = [
            year.as_str(),
            month.as_str(),
            day.as_str(),
            zero.as_str(),
            zero.as_str(),
            zero.as_str(),
            zero.as_str(),
            ambiguous.as_str(),
        ];
        let datetime = worker.execute_line(&call_line(
            "polars.expr.datetime",
            &[
                string_list_arg(&components),
                string_arg("us"),
                string_arg(""),
            ],
        ));
        assert!(
            reply_payload(&datetime).starts_with("ok_string "),
            "datetime failed: {datetime}"
        );

        let date = worker.execute_line(&call_line(
            "polars.expr.date_components",
            &[string_arg(&year), string_arg(&month), string_arg(&day)],
        ));
        assert!(
            reply_payload(&date).starts_with("ok_string "),
            "date failed: {date}"
        );

        let logarithm = worker.execute_line(&call_line("polars.expr.log10", &[string_arg(&year)]));
        assert!(
            reply_payload(&logarithm).starts_with("ok_string "),
            "log10 failed: {logarithm}"
        );

        let count = expr_lit(&TerlanPolarsScalar::Int(3)).unwrap();
        let repeated = worker.execute_line(&call_line(
            "polars.expr.repeat",
            &[string_arg(&year), string_arg(&count)],
        ));
        assert!(
            reply_payload(&repeated).starts_with("ok_string "),
            "repeat failed: {repeated}"
        );
    }

    #[test]
    fn worker_dispatches_interpolation_peak_run_and_reshape_expressions() {
        let mut worker = Worker::default();
        let input = expr_col("value").unwrap();
        let by = expr_col("coordinate").unwrap();
        let calls = [
            (
                "polars.expr.interpolate_by",
                vec![string_arg(&input), string_arg(&by)],
            ),
            ("polars.expr.peak_min", vec![string_arg(&input)]),
            ("polars.expr.peak_max", vec![string_arg(&input)]),
            ("polars.expr.rle", vec![string_arg(&input)]),
            ("polars.expr.rle_id", vec![string_arg(&input)]),
            (
                "polars.expr.reshape_expression",
                vec![string_arg(&input), "li:-1,2".to_string()],
            ),
        ];

        for (operation, args) in calls {
            let reply = worker.execute_line(&call_line(operation, &args));
            assert!(
                reply_payload(&reply).starts_with("ok_string "),
                "{operation} failed: {reply}"
            );
        }
    }

    #[test]
    fn worker_dispatches_exponentially_weighted_expressions() {
        let mut worker = Worker::default();
        let input = expr_col("value").unwrap();
        let by = expr_col("time").unwrap();
        let calls = [
            (
                "polars.expr.ewm_mean",
                vec![
                    string_arg(&input),
                    "f:0.5".to_string(),
                    "b:false".to_string(),
                    int_arg(1),
                    "b:true".to_string(),
                ],
            ),
            (
                "polars.expr.ewm_std",
                vec![
                    string_arg(&input),
                    "f:0.5".to_string(),
                    "b:false".to_string(),
                    "b:true".to_string(),
                    int_arg(1),
                    "b:true".to_string(),
                ],
            ),
            (
                "polars.expr.ewm_var",
                vec![
                    string_arg(&input),
                    "f:0.5".to_string(),
                    "b:false".to_string(),
                    "b:true".to_string(),
                    int_arg(1),
                    "b:true".to_string(),
                ],
            ),
            (
                "polars.expr.ewm_mean_by",
                vec![string_arg(&input), string_arg(&by), string_arg("1i")],
            ),
        ];

        for (operation, args) in calls {
            let reply = worker.execute_line(&call_line(operation, &args));
            assert!(
                reply_payload(&reply).starts_with("ok_string "),
                "{operation} failed: {reply}"
            );
        }
    }

    #[test]
    fn worker_dispatches_advanced_aggregation_and_window_expressions() {
        let mut worker = Worker::default();
        let input = expr_col("value").unwrap();
        let group = expr_col("group").unwrap();
        let element = expr_element().unwrap();
        let evaluation = expr_sum(&element).unwrap();
        let calls = [
            (
                "polars.expr.over_join",
                vec![string_arg(&input), string_list_arg(&[&group])],
            ),
            (
                "polars.expr.over_ordered",
                vec![
                    string_arg(&input),
                    string_list_arg(&[&group]),
                    string_list_arg(&[&input]),
                    "b:true".to_string(),
                    "b:false".to_string(),
                    string_arg("groups_to_rows"),
                ],
            ),
            (
                "polars.expr.cumulative_eval",
                vec![string_arg(&input), string_arg(&evaluation), int_arg(1)],
            ),
        ];

        for (operation, args) in calls {
            let reply = worker.execute_line(&call_line(operation, &args));
            assert!(
                reply_payload(&reply).starts_with("ok_string "),
                "{operation} failed: {reply}"
            );
        }

        let removed =
            worker.execute_line(&call_line("polars.expr.agg_groups", &[string_arg(&input)]));
        assert!(reply_payload(&removed).starts_with("err "));
    }

    #[test]
    fn worker_dispatches_advanced_string_expressions() {
        let mut worker = Worker::default();
        let input = expr_col("text").unwrap();
        let pattern = expr_lit(&TerlanPolarsScalar::String(".".into())).unwrap();
        let length = expr_lit(&TerlanPolarsScalar::Int(6)).unwrap();
        let calls = [
            (
                "polars.expr.string_contains_literal",
                vec![string_arg(&input), string_arg(&pattern)],
            ),
            (
                "polars.expr.string_find_literal",
                vec![string_arg(&input), string_arg(&pattern)],
            ),
            (
                "polars.expr.string_find",
                vec![
                    string_arg(&input),
                    string_arg(&pattern),
                    "b:true".to_string(),
                ],
            ),
            (
                "polars.expr.string_count_matches",
                vec![
                    string_arg(&input),
                    string_arg(&pattern),
                    "b:true".to_string(),
                ],
            ),
            (
                "polars.expr.string_pad_start",
                vec![string_arg(&input), string_arg(&length), string_arg("*")],
            ),
            (
                "polars.expr.string_pad_end",
                vec![string_arg(&input), string_arg(&length), string_arg("*")],
            ),
            (
                "polars.expr.string_zfill",
                vec![string_arg(&input), string_arg(&length)],
            ),
            ("polars.expr.string_hex_encode", vec![string_arg(&input)]),
            (
                "polars.expr.string_hex_decode",
                vec![string_arg(&input), "b:true".to_string()],
            ),
            ("polars.expr.string_base64_encode", vec![string_arg(&input)]),
            (
                "polars.expr.string_base64_decode",
                vec![string_arg(&input), "b:true".to_string()],
            ),
            (
                "polars.expr.string_normalize",
                vec![string_arg(&input), string_arg("nfc")],
            ),
            ("polars.expr.string_reverse", vec![string_arg(&input)]),
            ("polars.expr.string_escape_regex", vec![string_arg(&input)]),
        ];

        for (operation, args) in calls {
            let reply = worker.execute_line(&call_line(operation, &args));
            assert!(
                reply_payload(&reply).starts_with("ok_string "),
                "{operation} failed: {reply}"
            );
        }
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_reads_csv_and_reports_observers() {
        let mut worker = Worker::default();
        let handle = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));

        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&handle)]
            ))),
            "ok_int 2"
        );
        assert_eq!(
            reply_payload(
                &worker.execute_line(&call_line("polars.dataframe.width", &[handle_arg(&handle)]))
            ),
            "ok_int 3"
        );

        let columns_reply = worker.execute_line(&call_line(
            "polars.dataframe.columns",
            &[handle_arg(&handle)],
        ));
        assert!(reply_payload(&columns_reply).starts_with("ok_strings "));
        assert!(columns_reply.contains(&STANDARD.encode("name")));
        assert!(columns_reply.contains(&STANDARD.encode("age")));
        assert!(columns_reply.contains(&STANDARD.encode("city")));

        let schema_reply = worker.execute_line(&call_line(
            "polars.dataframe.schema",
            &[handle_arg(&handle)],
        ));
        assert!(reply_payload(&schema_reply).starts_with("ok_schema "));
        assert!(schema_reply.contains(&format!(
            "{}:{}",
            STANDARD.encode("age"),
            STANDARD.encode("Int64")
        )));

        let dtypes_reply = worker.execute_line(&call_line(
            "polars.dataframe.dtypes",
            &[handle_arg(&handle)],
        ));
        assert!(reply_payload(&dtypes_reply).starts_with("ok_strings "));
        assert!(dtypes_reply.contains(&STANDARD.encode("Int64")));

        let size_reply = worker.execute_line(&call_line(
            "polars.dataframe.estimated_size_with_unit",
            &[handle_arg(&handle), string_arg("mb")],
        ));
        assert!(reply_payload(&size_reply).starts_with("result_ok_float "));

        let gathered = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.gather_every",
            &[handle_arg(&handle), int_arg(2), int_arg(0)],
        )));
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&gathered)]
            ))),
            "ok_int 1"
        );

        let dropped = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.drop_columns_with_options",
            &[
                handle_arg(&handle),
                string_list_arg(&["absent"]),
                "b:false".to_string(),
            ],
        )));
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.width",
                &[handle_arg(&dropped)]
            ))),
            "ok_int 3"
        );

        let age = expr_col("age").unwrap();
        let zero = expr_lit(&TerlanPolarsScalar::Int(0)).unwrap();
        let predicate = expr_gt(&age, &zero).unwrap();
        let filtered = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.filter_all",
            &[handle_arg(&handle), string_list_arg(&[&predicate])],
        )));
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&filtered)]
            ))),
            "ok_int 2"
        );

        let ada = expr_lit(&TerlanPolarsScalar::String("Ada".to_string())).unwrap();
        let age_36 = expr_lit(&TerlanPolarsScalar::Int(36)).unwrap();
        let constrained = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.filter_constraints",
            &[
                handle_arg(&handle),
                string_list_arg(&["name", "age"]),
                string_list_arg(&[&ada, &age_36]),
            ],
        )));
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&constrained)]
            ))),
            "ok_int 1"
        );

        let ninety = expr_lit(&TerlanPolarsScalar::Int(90)).unwrap();
        let old = expr_lit(&TerlanPolarsScalar::String("old".to_string())).unwrap();
        let other = expr_lit(&TerlanPolarsScalar::String("other".to_string())).unwrap();
        let chain_reply = worker.execute_line(&call_line(
            "polars.expr.when_chain",
            &[
                string_list_arg(&[&expr_lt(&age, &ninety).unwrap()]),
                string_list_arg(&[&old]),
                string_arg(&other),
            ],
        ));
        assert!(reply_payload(&chain_reply).starts_with("ok_string "));
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_constructs_rows_and_rejects_ragged_data() {
        let mut worker = Worker::default();
        let dataframe = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.from_rows",
            &[
                string_list_arg(&["name", "city"]),
                string_rows_arg(&[&["Ada", "London"], &["Grace", "Arlington"]]),
            ],
        )));
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&dataframe)]
            ))),
            "ok_int 2"
        );
        assert!(reply_payload(&worker.execute_line(&call_line(
            "polars.dataframe.from_rows",
            &[
                string_list_arg(&["name", "city"]),
                string_rows_arg(&[&["Ada"]]),
            ],
        )))
        .starts_with("result_err "));
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_materializes_bounded_rows() {
        let mut worker = Worker::default();
        let dataframe = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("nullable.csv"))],
        )));
        let reply = worker.execute_line(&call_line(
            "polars.dataframe.rows",
            &[handle_arg(&dataframe), int_arg(1)],
        ));
        assert!(reply_payload(&reply).starts_with("result_ok_string_rows "));
        let encoded = reply_payload(&reply)
            .strip_prefix("result_ok_string_rows ")
            .expect("row payload");
        assert_eq!(
            decode_string_rows(encoded).expect("decode rows"),
            [vec!["Ada".to_string(), "10".to_string()]]
        );
        assert!(reply_payload(&worker.execute_line(&call_line(
            "polars.dataframe.rows",
            &[handle_arg(&dataframe), int_arg(-1)]
        )))
        .starts_with("result_err "));
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_select_returns_new_handle() {
        let mut worker = Worker::default();
        let handle = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let selected = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.select",
            &[handle_arg(&handle), string_list_arg(&["name", "city"])],
        )));

        assert_ne!(selected.id, handle.id);
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.width",
                &[handle_arg(&selected)]
            ))),
            "ok_int 2"
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_filters_and_reports_missing_columns() {
        let mut worker = Worker::default();
        let handle = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let filtered = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.filter_eq",
            &[handle_arg(&handle), string_arg("age"), int_arg(36)],
        )));

        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&filtered)]
            ))),
            "ok_int 1"
        );
        assert!(reply_payload(&worker.execute_line(&call_line(
            "polars.dataframe.filter_eq",
            &[handle_arg(&handle), string_arg("missing"), int_arg(1)]
        )))
        .starts_with("result_err "));
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_sorts_by_column() {
        let mut worker = Worker::default();
        let handle = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let sorted = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.sort_by",
            &[handle_arg(&handle), string_arg("age"), "b:true".to_string()],
        )));
        let first = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.head",
            &[handle_arg(&sorted), int_arg(1)],
        )));
        let oldest = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.filter_eq",
            &[handle_arg(&first), string_arg("age"), int_arg(85)],
        )));

        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&oldest)]
            ))),
            "ok_int 1"
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_groups_and_counts_rows() {
        let mut worker = Worker::default();
        let handle = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("groups.csv"))],
        )));
        let grouped = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.group_count",
            &[handle_arg(&handle), string_list_arg(&["city"])],
        )));

        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&grouped)]
            ))),
            "ok_int 2"
        );
        assert!(reply_payload(&worker.execute_line(&call_line(
            "polars.dataframe.group_count",
            &[handle_arg(&handle), string_list_arg(&[])]
        )))
        .starts_with("result_err "));
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_collects_lazy_queries_and_rejects_stale_plans() {
        let mut worker = Worker::default();
        let dataframe = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let plan = parse_plain_handle(&worker.execute_line(&call_line(
            "polars.dataframe.lazy",
            &[handle_arg(&dataframe)],
        )));
        assert_eq!(plan.type_name, LAZY_FRAME_TYPE);
        let filtered = parse_plain_handle(&worker.execute_line(&call_line(
            "polars.lazy_frame.filter_eq",
            &[handle_arg(&plan), string_arg("age"), int_arg(85)],
        )));
        let selected = parse_plain_handle(&worker.execute_line(&call_line(
            "polars.lazy_frame.select",
            &[handle_arg(&filtered), string_list_arg(&["name", "age"])],
        )));
        let collected = parse_result_handle(&worker.execute_line(&call_line(
            "polars.lazy_frame.collect",
            &[handle_arg(&selected)],
        )));
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&collected)]
            ))),
            "ok_int 1"
        );

        let invalid = parse_plain_handle(&worker.execute_line(&call_line(
            "polars.lazy_frame.select",
            &[handle_arg(&plan), string_list_arg(&["missing"])],
        )));
        assert!(reply_payload(&worker.execute_line(&call_line(
            "polars.lazy_frame.collect",
            &[handle_arg(&invalid)]
        )))
        .starts_with("result_err "));

        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.lazy_frame.dispose",
                &[handle_arg(&selected)]
            ))),
            "ok_unit"
        );
        assert!(reply_payload(&worker.execute_line(&call_line(
            "polars.lazy_frame.collect",
            &[handle_arg(&selected)]
        )))
        .starts_with("err "));
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_head_returns_new_handle() {
        let mut worker = Worker::default();
        let handle = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let headed = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.head",
            &[handle_arg(&handle), int_arg(1)],
        )));

        assert_ne!(headed.id, handle.id);
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&headed)]
            ))),
            "ok_int 1"
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_disposes_and_rejects_stale_or_forged_handles() {
        let mut worker = Worker::default();
        let handle = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));

        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.dispose",
                &[handle_arg(&handle)]
            ))),
            "ok_unit"
        );
        assert!(reply_payload(&worker.execute_line(&call_line(
            "polars.dataframe.height",
            &[handle_arg(&handle)]
        )))
        .starts_with("err "));
        assert!(reply_payload(&worker.execute_line(&call_line(
            "polars.dataframe.dispose",
            &[handle_arg(&handle)]
        )))
        .starts_with("err "));

        let forged = NativeHandle {
            generation: handle.generation + 1,
            ..handle
        };
        let forged_reply = worker.execute_line(&call_line(
            "polars.dataframe.height",
            &[handle_arg(&forged)],
        ));
        assert!(reply_payload(&forged_reply).starts_with("err "));
        assert!(forged_reply.contains(&STANDARD.encode("stale_native_handle")));
        assert!(worker.frames.is_empty());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_executes_sql_and_rejects_stale_contexts() {
        let mut worker = Worker::default();
        let dataframe = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_csv",
            &[string_arg(&fixture_path("people.csv"))],
        )));
        let context =
            parse_plain_handle(&worker.execute_line(&call_line("polars.sql_context.new", &[])));
        assert_eq!(context.type_name, SQL_CONTEXT_TYPE);
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.sql_context.register",
                &[
                    handle_arg(&context),
                    string_arg("people"),
                    handle_arg(&dataframe)
                ]
            ))),
            "result_ok_unit"
        );
        let tables_reply = worker.execute_line(&call_line(
            "polars.sql_context.tables",
            &[handle_arg(&context)],
        ));
        assert_eq!(
            decode_string_list(
                reply_payload(&tables_reply)
                    .strip_prefix("ok_strings ")
                    .expect("table list payload")
            )
            .expect("decode tables"),
            ["people"]
        );

        let plan = parse_result_handle(&worker.execute_line(&call_line(
            "polars.sql_context.execute",
            &[
                handle_arg(&context),
                string_arg("SELECT name FROM people WHERE age > 40 ORDER BY age"),
            ],
        )));
        let result = parse_result_handle(&worker.execute_line(&call_line(
            "polars.lazy_frame.collect",
            &[handle_arg(&plan)],
        )));
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&result)]
            ))),
            "ok_int 1"
        );

        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.sql_context.dispose",
                &[handle_arg(&context)]
            ))),
            "ok_unit"
        );
        assert!(reply_payload(&worker.execute_line(&call_line(
            "polars.sql_context.tables",
            &[handle_arg(&context)]
        )))
        .starts_with("err "));
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_helper_reads_sqlite_through_connectorx() {
        use rusqlite::Connection;

        let path = std::env::temp_dir().join(format!(
            "terlan-polars-helper-database-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        let connection = Connection::open(&path).expect("open SQLite fixture");
        connection
            .execute_batch(
                "CREATE TABLE people(name TEXT NOT NULL, age INTEGER NOT NULL);
                 INSERT INTO people VALUES ('Ada', 36), ('Grace', 85);",
            )
            .expect("seed SQLite fixture");
        drop(connection);

        let mut worker = Worker::default();
        let dataframe = parse_result_handle(&worker.execute_line(&call_line(
            "polars.dataframe.read_database_uri",
            &[
                string_arg(&format!("sqlite://{}", path.display())),
                string_arg("SELECT name, age FROM people ORDER BY age"),
            ],
        )));
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.height",
                &[handle_arg(&dataframe)]
            ))),
            "ok_int 2"
        );
        assert_eq!(
            reply_payload(&worker.execute_line(&call_line(
                "polars.dataframe.width",
                &[handle_arg(&dataframe)]
            ))),
            "ok_int 2"
        );

        std::fs::remove_file(path).expect("remove SQLite fixture");
    }
}
