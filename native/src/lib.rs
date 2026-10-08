#![forbid(unsafe_code)]
//! Rust adapter skeleton for the external `terlan-polars` package.
//!
//! Inputs:
//! - Opaque Terlan DataFrame handles supplied by the future native package ABI.
//! - Curated Polars operations selected by `bindings/polars.mapping.toml`.
//!
//! Outputs:
//! - Native adapter functions that translate between Terlan package calls and
//!   package-owned Polars values.
//!
//! Transformation:
//! - The default build records callable adapter boundaries without depending on
//!   the upstream `polars` crate. The `real-polars` feature links the real
//!   crate and executes the first DataFrame operations.

pub mod bridge;
pub mod data_types;
pub mod database;
pub mod dataframe_extended;
pub mod expressions;
pub mod interchange;
pub mod io_options;
pub mod lazyframe_extended;
pub mod parity;
pub mod plot_options;
pub mod series;
pub mod series_extended;
pub mod sql;
pub mod table_style;
mod tensor_interop;
pub mod udf;
pub mod visualization;

#[cfg(all(test, feature = "real-polars"))]
mod cheatsheet_compat_test;
#[cfg(test)]
mod cloud_io_test;
#[cfg(all(test, feature = "real-polars"))]
mod data_type_test;
#[cfg(all(test, feature = "real-polars"))]
mod dataframe_extended_test;
#[cfg(all(test, feature = "real-polars"))]
mod expression_sort_test;
#[cfg(all(test, feature = "real-polars"))]
mod interchange_test;
#[cfg(all(test, feature = "real-polars"))]
mod io_options_test;
#[cfg(all(test, feature = "real-polars"))]
mod lazyframe_extended_test;
#[cfg(all(test, feature = "real-polars"))]
mod polars_2_semantics_test;
#[cfg(all(test, feature = "real-polars"))]
mod series_test;
#[cfg(all(test, feature = "real-polars"))]
mod udf_test;

pub use database::{read_database_env, read_database_uri};
pub use dataframe_extended::*;
pub use interchange::{
    dataframe_from_arrow_ipc, dataframe_to_arrow_ipc, series_from_arrow_ipc, series_to_arrow_ipc,
};
pub use io_options::{
    csv_read_options, csv_read_options_with_column_names, csv_read_options_with_column_policies,
    csv_read_options_with_infer_schema_files, csv_write_options, ipc_custom_metadata,
    ipc_read_options, ipc_write_options, json_read_options, parquet_metadata, parquet_read_options,
    parquet_row_count, parquet_write_options, read_csv_with_options, read_ipc_with_options,
    read_json_with_options, read_parquet_with_options, write_csv_with_options,
    write_ipc_with_options, write_parquet_with_options,
};
pub use lazyframe_extended::*;
pub use plot_options::{
    plot_options, with_plot_color, with_plot_dimensions, with_plot_facet, with_plot_title,
};
pub use series::*;
pub use series_extended::*;
pub use sql::{
    sql_context_execute, sql_context_new, sql_context_register, sql_context_tables,
    sql_context_unregister, sql_functions, sql_keywords, sql_table_identifiers,
    TerlanPolarsSqlContext,
};
pub use table_style::{
    cols_label, data_color, fmt_nanoplot, fmt_number, gt, tab_header, tab_stub, table_html,
};
pub use tensor_interop::{dataframe_tensor_packet, series_tensor_packet};
pub use udf::{apply_expression_udf, define_expression_udf, expression_udf_parameter};
pub use visualization::{
    plot_bar_html, plot_bar_png, plot_bar_svg, plot_box_html, plot_box_png, plot_box_svg,
    plot_histogram_html, plot_histogram_png, plot_histogram_svg, plot_line_html, plot_line_png,
    plot_line_svg, plot_scatter_html, plot_scatter_png, plot_scatter_svg, plot_with_html,
    plot_with_png, plot_with_svg,
};

const MAX_MATERIALIZED_ROWS: usize = 10_000;
#[cfg(feature = "real-polars")]
const MAX_GENERATED_SAMPLE_ROWS: usize = 10_000_000;

/// Native DataFrame handle placeholder.
///
/// Inputs:
/// - None.
///
/// Outputs:
/// - A cloneable Rust marker used only by the package skeleton.
///
/// Transformation:
/// - Reserves the adapter-side handle name while the real Polars ownership and
///   lifetime contract is designed.
#[cfg(not(feature = "real-polars"))]
#[derive(Clone, Debug)]
pub struct TerlanPolarsDataFrame;

/// Native DataFrame wrapper for the real Polars feature.
///
/// Inputs:
/// - A `polars::prelude::DataFrame` produced by package-owned Rust code.
///
/// Outputs:
/// - A typed adapter value that Terlan can only observe through curated
///   functions.
///
/// Transformation:
/// - Keeps the upstream Polars type private to the native adapter boundary.
#[cfg(feature = "real-polars")]
#[derive(Clone, Debug)]
pub struct TerlanPolarsDataFrame {
    inner: polars::prelude::DataFrame,
}

/// Native LazyFrame handle placeholder.
#[cfg(not(feature = "real-polars"))]
#[derive(Clone)]
pub struct TerlanPolarsLazyFrame;

/// Package-owned real Polars lazy query plan.
#[cfg(feature = "real-polars")]
#[derive(Clone)]
pub struct TerlanPolarsLazyFrame {
    inner: polars::prelude::LazyFrame,
}

/// Native owned DataFrame collection placeholder.
#[cfg(not(feature = "real-polars"))]
#[derive(Clone, Debug)]
pub struct TerlanPolarsFrameSet;

/// Package-owned collection of independently owned DataFrames.
#[cfg(feature = "real-polars")]
#[derive(Clone, Debug)]
pub struct TerlanPolarsFrameSet {
    /// Frames retained by the collection.
    frames: Vec<TerlanPolarsDataFrame>,
}

/// Native owned LazyFrame collection placeholder.
#[cfg(not(feature = "real-polars"))]
#[derive(Clone)]
pub struct TerlanPolarsLazyFrameSet;

/// Package-owned collection of independently owned lazy plans.
#[cfg(feature = "real-polars")]
#[derive(Clone)]
pub struct TerlanPolarsLazyFrameSet {
    /// Plans retained by the collection.
    plans: Vec<TerlanPolarsLazyFrame>,
}

/// Native Series handle placeholder.
#[cfg(not(feature = "real-polars"))]
#[derive(Clone, Debug)]
pub struct TerlanPolarsSeries;

/// Package-owned real Polars Series.
#[cfg(feature = "real-polars")]
#[derive(Clone, Debug)]
pub struct TerlanPolarsSeries {
    inner: polars::prelude::Series,
}

/// One column in a DataFrame schema.
///
/// The native adapter exposes data types as text so the public package ABI is
/// not coupled to upstream Polars enum variants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerlanPolarsColumnSchema {
    /// Column name.
    pub name: String,
    /// Stable textual Polars data type descriptor.
    pub data_type: String,
}

/// Scalar literal accepted by DataFrame equality predicates.
#[derive(Clone, Debug, PartialEq)]
pub enum TerlanPolarsScalar {
    /// UTF-8 string scalar.
    String(String),
    /// Signed 64-bit integer scalar.
    Int(i64),
    /// IEEE 754 double-precision scalar.
    Float(f64),
    /// Boolean scalar.
    Bool(bool),
}

/// Native Polars adapter error placeholder.
///
/// Inputs:
/// - Static error code and message supplied by adapter functions.
///
/// Outputs:
/// - A typed Rust error value that can later map into `std.core.Error.Error`.
///
/// Transformation:
/// - Keeps adapter failures explicit while the real Polars error conversion is
///   still pending.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerlanPolarsError {
    code: &'static str,
    message: String,
}

impl TerlanPolarsError {
    /// Creates an adapter error from stable fields.
    ///
    /// Inputs:
    /// - `code`: stable machine-readable error code.
    /// - `message`: human-readable diagnostic text.
    ///
    /// Outputs:
    /// - `TerlanPolarsError` with owned message storage.
    ///
    /// Transformation:
    /// - Normalizes native and Polars failures into the future Terlan error
    ///   conversion shape.
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Returns the Terlan error-code atom name.
    ///
    /// Inputs:
    /// - `self`: adapter error value.
    ///
    /// Outputs:
    /// - Stable atom-name text without target-specific exception data.
    ///
    /// Transformation:
    /// - Exposes the first half of the `std.core.Error.Error` conversion shape.
    pub fn code(&self) -> &'static str {
        self.code
    }

    /// Returns the Terlan error message.
    ///
    /// Inputs:
    /// - `self`: adapter error value.
    ///
    /// Outputs:
    /// - Stable UTF-8 message text for `std.core.Error.Error`.
    ///
    /// Transformation:
    /// - Exposes the second half of the `std.core.Error.Error` conversion
    ///   shape.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Splits the adapter error into the future Terlan ABI fields.
    ///
    /// Inputs:
    /// - `self`: adapter error value.
    ///
    /// Outputs:
    /// - `(code, message)` tuple matching package binding metadata.
    ///
    /// Transformation:
    /// - Makes error conversion testable before the native target links real
    ///   Polars errors.
    pub fn into_parts(self) -> (&'static str, String) {
        (self.code, self.message)
    }
}

#[cfg(feature = "real-polars")]
impl From<polars::error::PolarsError> for TerlanPolarsError {
    /// Converts a Polars error into the stable adapter error shape.
    ///
    /// Inputs:
    /// - `error`: upstream Polars error.
    ///
    /// Outputs:
    /// - `TerlanPolarsError` with code `polars_error`.
    ///
    /// Transformation:
    /// - Erases upstream error variants from the Terlan ABI while preserving a
    ///   useful human-readable message.
    fn from(error: polars::error::PolarsError) -> Self {
        Self::new("polars_error", error.to_string())
    }
}

impl From<std::io::Error> for TerlanPolarsError {
    fn from(error: std::io::Error) -> Self {
        Self::new("io_error", error.to_string())
    }
}

/// Builds the current unavailable-native adapter error.
///
/// Inputs:
/// - None.
///
/// Outputs:
/// - `TerlanPolarsError` with stable code and message fields.
///
/// Transformation:
/// - Centralizes the temporary error returned by stubbed adapter functions.
#[cfg(not(feature = "real-polars"))]
fn unavailable_error() -> TerlanPolarsError {
    TerlanPolarsError::new(
        "native_unavailable",
        "terlan-polars requires the Rust native adapter target capability",
    )
}

/// Reads a CSV file into a native DataFrame.
///
/// Inputs:
/// - `path`: UTF-8 filesystem path supplied by Terlan.
///
/// Outputs:
/// - `Ok(TerlanPolarsDataFrame)` once the real Polars adapter is linked.
/// - `Err(TerlanPolarsError)` in the current stub implementation.
///
/// Transformation:
/// - Reserves the Rust function boundary for `polars.DataFrame.read_csv`.
#[cfg(not(feature = "real-polars"))]
pub fn read_csv(_path: &str) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Reads a CSV file into a native DataFrame with the real Polars feature.
///
/// Inputs:
/// - `path`: UTF-8 filesystem path supplied by Terlan.
///
/// Outputs:
/// - `Ok(TerlanPolarsDataFrame)` containing a package-owned Polars DataFrame.
/// - `Err(TerlanPolarsError)` when Polars cannot read the file.
///
/// Transformation:
/// - Opens the CSV through the Polars reader and wraps the resulting DataFrame
///   behind the adapter type.
#[cfg(feature = "real-polars")]
pub fn read_csv(path: &str) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::SerReader;

    let file = std::fs::File::open(path).map_err(|error| {
        TerlanPolarsError::new(
            "csv_read_error",
            format!("failed to open CSV file `{path}`: {error}"),
        )
    })?;
    let df = polars::prelude::CsvReader::new(file).finish()?;

    Ok(TerlanPolarsDataFrame { inner: df })
}

/// Reads a CSV and asks Polars to infer ISO date columns.
#[cfg(not(feature = "real-polars"))]
pub fn read_csv_dates(_path: &str) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Reads csv dates data into an owned Polars value.
pub fn read_csv_dates(path: &str) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{CsvParseOptions, CsvReadOptions, SerReader};

    let inner = CsvReadOptions::default()
        .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Writes a DataFrame as CSV.
#[cfg(not(feature = "real-polars"))]
pub fn write_csv(_df: &TerlanPolarsDataFrame, _path: &str) -> Result<(), TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Writes an owned Polars value as csv data.
pub fn write_csv(df: &TerlanPolarsDataFrame, path: &str) -> Result<(), TerlanPolarsError> {
    use polars::prelude::{CsvWriter, SerWriter};

    let mut file = std::fs::File::create(path).map_err(|error| {
        TerlanPolarsError::new(
            "csv_write_error",
            format!("failed to create CSV file `{path}`: {error}"),
        )
    })?;
    let mut inner = df.inner.clone();
    CsvWriter::new(&mut file)
        .include_header(true)
        .finish(&mut inner)?;
    Ok(())
}

macro_rules! unavailable_dataframe_io {
    ($read:ident, $write:ident) => {
        #[cfg(not(feature = "real-polars"))]
        pub fn $read(_path: &str) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
            Err(unavailable_error())
        }

        #[cfg(not(feature = "real-polars"))]
        pub fn $write(_df: &TerlanPolarsDataFrame, _path: &str) -> Result<(), TerlanPolarsError> {
            Err(unavailable_error())
        }
    };
}

unavailable_dataframe_io!(read_parquet, write_parquet);
unavailable_dataframe_io!(read_json, write_json);
unavailable_dataframe_io!(read_ndjson, write_ndjson);
unavailable_dataframe_io!(read_ipc, write_ipc);

#[cfg(feature = "real-polars")]
/// Reads parquet data into an owned Polars value.
pub fn read_parquet(path: &str) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{ParquetReader, SerReader};
    Ok(TerlanPolarsDataFrame {
        inner: ParquetReader::new(std::fs::File::open(path)?).finish()?,
    })
}

#[cfg(feature = "real-polars")]
/// Writes an owned Polars value as parquet data.
pub fn write_parquet(df: &TerlanPolarsDataFrame, path: &str) -> Result<(), TerlanPolarsError> {
    use polars::prelude::ParquetWriter;
    let mut inner = df.inner.clone();
    ParquetWriter::new(std::fs::File::create(path)?).finish(&mut inner)?;
    Ok(())
}

#[cfg(feature = "real-polars")]
fn read_json_format(
    path: &str,
    format: polars::prelude::JsonFormat,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{JsonReader, SerReader};
    Ok(TerlanPolarsDataFrame {
        inner: JsonReader::new(std::fs::File::open(path)?)
            .with_json_format(format)
            .finish()?,
    })
}

#[cfg(feature = "real-polars")]
fn write_json_format(
    df: &TerlanPolarsDataFrame,
    path: &str,
    format: polars::prelude::JsonFormat,
) -> Result<(), TerlanPolarsError> {
    use polars::prelude::{JsonWriter, SerWriter};
    let mut inner = df.inner.clone();
    JsonWriter::new(std::fs::File::create(path)?)
        .with_json_format(format)
        .finish(&mut inner)?;
    Ok(())
}

#[cfg(feature = "real-polars")]
/// Reads json data into an owned Polars value.
pub fn read_json(path: &str) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    read_json_format(path, polars::prelude::JsonFormat::Json)
}

#[cfg(feature = "real-polars")]
/// Writes an owned Polars value as json data.
pub fn write_json(df: &TerlanPolarsDataFrame, path: &str) -> Result<(), TerlanPolarsError> {
    write_json_format(df, path, polars::prelude::JsonFormat::Json)
}

#[cfg(feature = "real-polars")]
/// Reads ndjson data into an owned Polars value.
pub fn read_ndjson(path: &str) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    read_json_format(path, polars::prelude::JsonFormat::JsonLines)
}

#[cfg(feature = "real-polars")]
/// Writes an owned Polars value as ndjson data.
pub fn write_ndjson(df: &TerlanPolarsDataFrame, path: &str) -> Result<(), TerlanPolarsError> {
    write_json_format(df, path, polars::prelude::JsonFormat::JsonLines)
}

#[cfg(feature = "real-polars")]
/// Reads ipc data into an owned Polars value.
pub fn read_ipc(path: &str) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{IpcReader, SerReader};
    Ok(TerlanPolarsDataFrame {
        inner: IpcReader::new(std::fs::File::open(path)?).finish()?,
    })
}

#[cfg(feature = "real-polars")]
/// Writes an owned Polars value as ipc data.
pub fn write_ipc(df: &TerlanPolarsDataFrame, path: &str) -> Result<(), TerlanPolarsError> {
    use polars::prelude::{IpcWriter, SerWriter};
    let mut inner = df.inner.clone();
    IpcWriter::new(std::fs::File::create(path)?).finish(&mut inner)?;
    Ok(())
}

/// Constructs a string-valued DataFrame from rows.
#[cfg(not(feature = "real-polars"))]
pub fn from_rows(
    columns: &[String],
    rows: &[Vec<String>],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    validate_row_widths(columns, rows)?;
    Err(unavailable_error())
}

/// Constructs a real string-valued DataFrame from rows.
#[cfg(feature = "real-polars")]
pub fn from_rows(
    columns: &[String],
    rows: &[Vec<String>],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{Column, DataFrame};

    validate_row_widths(columns, rows)?;
    let mut values = vec![Vec::<String>::with_capacity(rows.len()); columns.len()];
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            values[index].push(value.clone());
        }
    }
    let columns = columns
        .iter()
        .cloned()
        .zip(values)
        .map(|(name, values)| Column::new(name.into(), values))
        .collect::<Vec<_>>();
    Ok(TerlanPolarsDataFrame {
        inner: DataFrame::new_infer_height(columns)?,
    })
}

fn validate_row_widths(columns: &[String], rows: &[Vec<String>]) -> Result<(), TerlanPolarsError> {
    if let Some((index, row)) = rows
        .iter()
        .enumerate()
        .find(|(_, row)| row.len() != columns.len())
    {
        return Err(TerlanPolarsError::new(
            "invalid_row_width",
            format!(
                "row {index} has {} values but the DataFrame declares {} columns",
                row.len(),
                columns.len()
            ),
        ));
    }
    Ok(())
}

/// Returns a DataFrame row count.
///
/// Inputs:
/// - `df`: native DataFrame handle.
///
/// Outputs:
/// - Row count as `usize`.
///
/// Transformation:
/// - Reserves the Rust function boundary for the Terlan `height` receiver
///   method while returning the current stub value.
#[cfg(not(feature = "real-polars"))]
pub fn height(_df: &TerlanPolarsDataFrame) -> usize {
    0
}

#[cfg(feature = "real-polars")]
/// Executes the height operation through the Terlan Polars adapter.
pub fn height(df: &TerlanPolarsDataFrame) -> usize {
    df.inner.height()
}

/// Returns a DataFrame column count.
///
/// Inputs:
/// - `df`: native DataFrame handle.
///
/// Outputs:
/// - Column count as `usize`.
///
/// Transformation:
/// - Reserves the Rust function boundary for the Terlan `width` receiver method
///   while returning the current stub value.
#[cfg(not(feature = "real-polars"))]
pub fn width(_df: &TerlanPolarsDataFrame) -> usize {
    0
}

#[cfg(feature = "real-polars")]
/// Executes the width operation through the Terlan Polars adapter.
pub fn width(df: &TerlanPolarsDataFrame) -> usize {
    df.inner.width()
}

/// Returns Polars' estimated heap size for a materialized frame.
#[cfg(not(feature = "real-polars"))]
pub fn estimated_size(_df: &TerlanPolarsDataFrame) -> usize {
    0
}

#[cfg(feature = "real-polars")]
/// Executes the estimated size operation through the Terlan Polars adapter.
pub fn estimated_size(df: &TerlanPolarsDataFrame) -> usize {
    df.inner.estimated_size(false)
}

/// Returns Polars' estimated heap size scaled to a Python-compatible size unit.
pub fn estimated_size_with_unit(
    df: &TerlanPolarsDataFrame,
    unit: &str,
) -> Result<f64, TerlanPolarsError> {
    scale_estimated_size(estimated_size(df), unit)
}

pub(crate) fn scale_estimated_size(bytes: usize, unit: &str) -> Result<f64, TerlanPolarsError> {
    let divisor = match unit {
        "b" | "bytes" => 1.0,
        "kb" | "kilobytes" => 1024.0,
        "mb" | "megabytes" => 1024.0_f64.powi(2),
        "gb" | "gigabytes" => 1024.0_f64.powi(3),
        "tb" | "terabytes" => 1024.0_f64.powi(4),
        _ => {
            return Err(TerlanPolarsError::new(
                "invalid_size_unit",
                "size unit must be b, kb, mb, gb, tb, or its full plural name",
            ))
        }
    };
    Ok(bytes as f64 / divisor)
}

/// Reports the package and pinned native dependency versions.
pub fn show_versions() -> String {
    format!(
        "terlan-polars {}\npolars py-2.0.0 (rust 0.55.1, rev 22a147de)\nplotlars-core 0.12.6\nplotlars-plotters 0.12.6\nplotlars-plotly 0.12.6 (optional)\n",
        env!("CARGO_PKG_VERSION")
    )
}

/// Renders a bounded, column-oriented overview of a DataFrame.
#[cfg(not(feature = "real-polars"))]
pub fn glimpse(_df: &TerlanPolarsDataFrame) -> Result<String, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Renders a bounded, column-oriented overview of a DataFrame.
pub fn glimpse(df: &TerlanPolarsDataFrame) -> Result<String, TerlanPolarsError> {
    use std::fmt::Write;

    const MAX_COLUMNS: usize = 256;
    const MAX_VALUES: usize = 10;
    const MAX_OUTPUT: usize = 1_048_576;

    let mut output = format!(
        "Rows: {}\nColumns: {}\n",
        df.inner.height(),
        df.inner.width()
    );
    for column in df.inner.columns().iter().take(MAX_COLUMNS) {
        write!(output, "$ {} <{}> ", column.name(), column.dtype()).expect("write to String");
        for index in 0..column.len().min(MAX_VALUES) {
            if index > 0 {
                output.push_str(", ");
            }
            let value = column.get(index)?;
            write!(output, "{value}").expect("write to String");
        }
        if column.len() > MAX_VALUES {
            output.push_str(", …");
        }
        output.push('\n');
        if output.len() >= MAX_OUTPUT {
            output.truncate(MAX_OUTPUT.saturating_sub(32));
            output.push_str("\n… inspection output truncated\n");
            return Ok(output);
        }
    }
    if df.inner.width() > MAX_COLUMNS {
        output.push_str("… remaining columns omitted\n");
    }
    Ok(output)
}

/// Produces the standard count/null/mean/std/range summary table.
#[cfg(not(feature = "real-polars"))]
pub fn describe(_df: &TerlanPolarsDataFrame) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Produces the standard count/null/mean/std/range summary table.
pub fn describe(df: &TerlanPolarsDataFrame) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{Column, DataFrame, DataType, QuantileMethod};

    let labels = [
        "count",
        "null_count",
        "mean",
        "std",
        "min",
        "25%",
        "50%",
        "75%",
        "max",
    ];
    let mut columns = vec![Column::new("statistic".into(), labels)];
    for column in df.inner.columns() {
        if column.dtype().is_numeric() {
            let cast = column.cast(&DataType::Float64)?;
            let scalar = |value: polars::prelude::Scalar| match value.into_value() {
                polars::prelude::AnyValue::Float64(value) => Some(value),
                polars::prelude::AnyValue::Float32(value) => Some(f64::from(value)),
                polars::prelude::AnyValue::Null => None,
                _ => None,
            };
            let values = vec![
                Some((column.len() - column.null_count()) as f64),
                Some(column.null_count() as f64),
                scalar(cast.mean_reduce()?),
                scalar(cast.std_reduce(1)?),
                scalar(cast.min_reduce()?),
                scalar(cast.quantile_reduce(0.25, QuantileMethod::Nearest)?),
                scalar(cast.quantile_reduce(0.50, QuantileMethod::Nearest)?),
                scalar(cast.quantile_reduce(0.75, QuantileMethod::Nearest)?),
                scalar(cast.max_reduce()?),
            ];
            columns.push(Column::new(column.name().clone(), values));
        } else {
            let render = |value: polars::prelude::Scalar| {
                (!value.is_null()).then(|| value.into_value().to_string())
            };
            let values = vec![
                Some((column.len() - column.null_count()).to_string()),
                Some(column.null_count().to_string()),
                None,
                None,
                render(column.min_reduce()?),
                None,
                None,
                None,
                render(column.max_reduce()?),
            ];
            columns.push(Column::new(column.name().clone(), values));
        }
    }
    Ok(TerlanPolarsDataFrame {
        inner: DataFrame::new(labels.len(), columns)?,
    })
}

/// Compares two frames using Polars equality, where any null prevents equality.
#[cfg(not(feature = "real-polars"))]
pub fn frames_equal(_left: &TerlanPolarsDataFrame, _right: &TerlanPolarsDataFrame) -> bool {
    false
}

#[cfg(feature = "real-polars")]
/// Executes the frames equal operation through the Terlan Polars adapter.
pub fn frames_equal(left: &TerlanPolarsDataFrame, right: &TerlanPolarsDataFrame) -> bool {
    left.inner.equals(&right.inner)
}

/// Compares two frames using Polars equality with null equal to null.
#[cfg(not(feature = "real-polars"))]
pub fn frames_equal_missing(_left: &TerlanPolarsDataFrame, _right: &TerlanPolarsDataFrame) -> bool {
    false
}

#[cfg(feature = "real-polars")]
/// Executes the frames equal missing operation through the Terlan Polars adapter.
pub fn frames_equal_missing(left: &TerlanPolarsDataFrame, right: &TerlanPolarsDataFrame) -> bool {
    left.inner.equals_missing(&right.inner)
}

fn validate_sample_seed(seed: Option<i64>) -> Result<Option<u64>, TerlanPolarsError> {
    seed.map(|value| {
        u64::try_from(value).map_err(|_| {
            TerlanPolarsError::new("invalid_sample_seed", "sample seed cannot be negative")
        })
    })
    .transpose()
}

fn validate_sample_count(count: i64) -> Result<usize, TerlanPolarsError> {
    usize::try_from(count).map_err(|_| {
        TerlanPolarsError::new("invalid_sample_size", "sample row count cannot be negative")
    })
}

#[cfg(feature = "real-polars")]
fn validate_generated_sample_size(
    df: &TerlanPolarsDataFrame,
    count: usize,
) -> Result<(), TerlanPolarsError> {
    let maximum = df.inner.height().max(MAX_GENERATED_SAMPLE_ROWS);
    if count > maximum {
        return Err(TerlanPolarsError::new(
            "sample_size_too_large",
            format!("sample row count {count} exceeds the safe maximum of {maximum}"),
        ));
    }
    Ok(())
}

fn validate_sample_fraction(fraction: f64) -> Result<(), TerlanPolarsError> {
    if !fraction.is_finite() || fraction < 0.0 {
        return Err(TerlanPolarsError::new(
            "invalid_sample_fraction",
            "sample fraction must be finite and non-negative",
        ));
    }
    Ok(())
}

fn sample_rows_n_with_seed(
    df: &TerlanPolarsDataFrame,
    count: i64,
    with_replacement: bool,
    shuffle: bool,
    seed: Option<i64>,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    let count = validate_sample_count(count)?;
    let seed = validate_sample_seed(seed)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, count, with_replacement, shuffle, seed);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        validate_generated_sample_size(df, count)?;
        Ok(TerlanPolarsDataFrame {
            inner: df
                .inner
                .sample_n_literal(count, with_replacement, Some(shuffle), seed)?,
        })
    }
}

/// Samples a fixed number of DataFrame rows.
pub fn sample_rows_n(
    df: &TerlanPolarsDataFrame,
    count: i64,
    with_replacement: bool,
    shuffle: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    sample_rows_n_with_seed(df, count, with_replacement, shuffle, None)
}

/// Samples a fixed number of DataFrame rows reproducibly.
pub fn sample_rows_n_seeded(
    df: &TerlanPolarsDataFrame,
    count: i64,
    with_replacement: bool,
    shuffle: bool,
    seed: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    sample_rows_n_with_seed(df, count, with_replacement, shuffle, Some(seed))
}

fn sample_rows_fraction_with_seed(
    df: &TerlanPolarsDataFrame,
    fraction: f64,
    with_replacement: bool,
    shuffle: bool,
    seed: Option<i64>,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    validate_sample_fraction(fraction)?;
    let seed = validate_sample_seed(seed)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, fraction, with_replacement, shuffle, seed);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        let count = (df.inner.height() as f64 * fraction) as usize;
        validate_generated_sample_size(df, count)?;
        Ok(TerlanPolarsDataFrame {
            inner: df
                .inner
                .sample_n_literal(count, with_replacement, Some(shuffle), seed)?,
        })
    }
}

/// Samples a fraction of DataFrame rows.
pub fn sample_rows_fraction(
    df: &TerlanPolarsDataFrame,
    fraction: f64,
    with_replacement: bool,
    shuffle: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    sample_rows_fraction_with_seed(df, fraction, with_replacement, shuffle, None)
}

/// Samples a fraction of DataFrame rows reproducibly.
pub fn sample_rows_fraction_seeded(
    df: &TerlanPolarsDataFrame,
    fraction: f64,
    with_replacement: bool,
    shuffle: bool,
    seed: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    sample_rows_fraction_with_seed(df, fraction, with_replacement, shuffle, Some(seed))
}

/// Returns a Boolean Series marking rows that occur exactly once.
#[cfg(not(feature = "real-polars"))]
pub fn row_is_unique(_df: &TerlanPolarsDataFrame) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the row is unique operation through the Terlan Polars adapter.
pub fn row_is_unique(df: &TerlanPolarsDataFrame) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::{IntoSeries, NamedFrom};
    if df.inner.width() == 0 {
        return Ok(TerlanPolarsSeries {
            inner: polars::prelude::Series::new(
                "row_is_unique".into(),
                vec![df.inner.height() == 1; df.inner.height()],
            ),
        });
    }
    let mut inner = df.inner.is_unique()?.into_series();
    inner.rename("row_is_unique".into());
    Ok(TerlanPolarsSeries { inner })
}

/// Returns a Boolean Series marking every member of a duplicated row group.
#[cfg(not(feature = "real-polars"))]
pub fn row_is_duplicated(
    _df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the row is duplicated operation through the Terlan Polars adapter.
pub fn row_is_duplicated(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::{IntoSeries, NamedFrom};
    if df.inner.width() == 0 {
        return Ok(TerlanPolarsSeries {
            inner: polars::prelude::Series::new(
                "row_is_duplicated".into(),
                vec![df.inner.height() > 1; df.inner.height()],
            ),
        });
    }
    let mut inner = df.inner.is_duplicated()?.into_series();
    inner.rename("row_is_duplicated".into());
    Ok(TerlanPolarsSeries { inner })
}

#[cfg(feature = "real-polars")]
fn validate_row_hash_seed(seed: i64) -> Result<u64, TerlanPolarsError> {
    u64::try_from(seed).map_err(|_| {
        TerlanPolarsError::new("invalid_row_hash_seed", "row hash seed cannot be negative")
    })
}

/// Hashes and combines every row using Polars' default row hasher.
#[cfg(not(feature = "real-polars"))]
pub fn row_hashes(_df: &TerlanPolarsDataFrame) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the row hashes operation through the Terlan Polars adapter.
pub fn row_hashes(df: &TerlanPolarsDataFrame) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::{IntoSeries, NamedFrom};
    if df.inner.height() == 0 {
        return Ok(TerlanPolarsSeries {
            inner: polars::prelude::Series::new("row_hash".into(), Vec::<u64>::new()),
        });
    }
    let mut frame = df.inner.clone();
    let mut inner = frame.hash_rows(None)?.into_series();
    inner.rename("row_hash".into());
    Ok(TerlanPolarsSeries { inner })
}

/// Hashes and combines every row reproducibly with a non-negative seed.
#[cfg(not(feature = "real-polars"))]
pub fn row_hashes_seeded(
    _df: &TerlanPolarsDataFrame,
    _seed: i64,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the row hashes seeded operation through the Terlan Polars adapter.
pub fn row_hashes_seeded(
    df: &TerlanPolarsDataFrame,
    seed: i64,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::{IntoSeries, PlSeedableRandomStateQuality, SeedableFromU64SeedExt};
    let seed = validate_row_hash_seed(seed)?;
    if df.inner.height() == 0 {
        use polars::prelude::NamedFrom;
        return Ok(TerlanPolarsSeries {
            inner: polars::prelude::Series::new("row_hash".into(), Vec::<u64>::new()),
        });
    }
    let hasher = PlSeedableRandomStateQuality::seed_from_u64(seed);
    let mut frame = df.inner.clone();
    let mut inner = frame.hash_rows(Some(hasher))?.into_series();
    inner.rename("row_hash".into());
    Ok(TerlanPolarsSeries { inner })
}

/// Returns DataFrame column names.
///
/// Inputs:
/// - `df`: native DataFrame handle.
///
/// Outputs:
/// - Owned UTF-8 column names.
///
/// Transformation:
/// - Reserves the Rust function boundary for the Terlan `columns` receiver
///   method while returning the current stub value.
#[cfg(not(feature = "real-polars"))]
pub fn columns(_df: &TerlanPolarsDataFrame) -> Vec<String> {
    Vec::new()
}

#[cfg(feature = "real-polars")]
/// Executes the columns operation through the Terlan Polars adapter.
pub fn columns(df: &TerlanPolarsDataFrame) -> Vec<String> {
    df.inner
        .get_column_names()
        .into_iter()
        .map(|name| name.to_string())
        .collect()
}

/// Returns up to `limit` DataFrame rows as display strings.
#[cfg(not(feature = "real-polars"))]
pub fn rows(
    _df: &TerlanPolarsDataFrame,
    _limit: usize,
) -> Result<Vec<Vec<String>>, TerlanPolarsError> {
    validate_rows_limit(_limit)?;
    Err(unavailable_error())
}

/// Returns up to `limit` real DataFrame rows as display strings.
#[cfg(feature = "real-polars")]
pub fn rows(
    df: &TerlanPolarsDataFrame,
    limit: usize,
) -> Result<Vec<Vec<String>>, TerlanPolarsError> {
    validate_rows_limit(limit)?;
    (0..df.inner.height().min(limit))
        .map(|index| {
            let row = df.inner.get_row(index)?;
            Ok(row.0.into_iter().map(display_value).collect())
        })
        .collect()
}

#[cfg(feature = "real-polars")]
fn display_value(value: polars::prelude::AnyValue<'_>) -> String {
    match value {
        polars::prelude::AnyValue::String(value) => value.to_string(),
        polars::prelude::AnyValue::StringOwned(value) => value.to_string(),
        polars::prelude::AnyValue::Null => "null".to_string(),
        value => value.to_string(),
    }
}

fn validate_rows_limit(limit: usize) -> Result<(), TerlanPolarsError> {
    if limit > MAX_MATERIALIZED_ROWS {
        return Err(TerlanPolarsError::new(
            "row_limit_too_large",
            format!("rows limit {limit} exceeds the package maximum of {MAX_MATERIALIZED_ROWS}"),
        ));
    }
    Ok(())
}

/// Extracts one DataFrame column as an independently owned Series.
#[cfg(not(feature = "real-polars"))]
pub fn column_series(
    _df: &TerlanPolarsDataFrame,
    _name: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the column series operation through the Terlan Polars adapter.
pub fn column_series(
    df: &TerlanPolarsDataFrame,
    name: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Ok(TerlanPolarsSeries {
        inner: df.inner.column(name)?.as_materialized_series().clone(),
    })
}

/// Constructs a UTF-8 Series from Terlan strings.
#[cfg(not(feature = "real-polars"))]
pub fn series_from_strings(
    _name: &str,
    _values: &[String],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_ints(
    _name: &str,
    _values: &[i64],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series from ints operation through the Terlan Polars adapter.
pub fn series_from_ints(
    name: &str,
    values: &[i64],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::NamedFrom;
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::new(name.into(), values),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_floats(
    _name: &str,
    _values: &[f64],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series from floats operation through the Terlan Polars adapter.
pub fn series_from_floats(
    name: &str,
    values: &[f64],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::NamedFrom;
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::new(name.into(), values),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_bools(
    _name: &str,
    _values: &[bool],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_nullable_strings(
    _name: &str,
    _values: &[Option<String>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series from nullable strings operation through the Terlan Polars adapter.
pub fn series_from_nullable_strings(
    name: &str,
    values: &[Option<String>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::NamedFrom;
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::new(name.into(), values),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_nullable_ints(
    _name: &str,
    _values: &[Option<i64>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series from nullable ints operation through the Terlan Polars adapter.
pub fn series_from_nullable_ints(
    name: &str,
    values: &[Option<i64>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::NamedFrom;
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::new(name.into(), values),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_nullable_floats(
    _name: &str,
    _values: &[Option<f64>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series from nullable floats operation through the Terlan Polars adapter.
pub fn series_from_nullable_floats(
    name: &str,
    values: &[Option<f64>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::NamedFrom;
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::new(name.into(), values),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_nullable_bools(
    _name: &str,
    _values: &[Option<bool>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series from nullable bools operation through the Terlan Polars adapter.
pub fn series_from_nullable_bools(
    name: &str,
    values: &[Option<bool>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::NamedFrom;
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::new(name.into(), values),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_dates(
    _name: &str,
    _values: &[String],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series from dates operation through the Terlan Polars adapter.
pub fn series_from_dates(
    name: &str,
    values: &[String],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use chrono::NaiveDate;
    use polars::prelude::NamedFrom;

    let values = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|error| {
                TerlanPolarsError::new(
                    "invalid_date",
                    format!("invalid ISO date at index {index} (`{value}`): {error}"),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::new(name.into(), values),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_nullable_dates(
    _name: &str,
    _values: &[Option<String>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series from nullable dates operation through the Terlan Polars adapter.
pub fn series_from_nullable_dates(
    name: &str,
    values: &[Option<String>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use chrono::NaiveDate;
    use polars::prelude::NamedFrom;

    let values = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value
                .as_deref()
                .map(|value| {
                    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|error| {
                        TerlanPolarsError::new(
                            "invalid_date",
                            format!("invalid ISO date at index {index} (`{value}`): {error}"),
                        )
                    })
                })
                .transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::new(name.into(), values),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_datetimes(
    _name: &str,
    _values: &[String],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series from datetimes operation through the Terlan Polars adapter.
pub fn series_from_datetimes(
    name: &str,
    values: &[String],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use chrono::NaiveDateTime;
    use polars::prelude::{DatetimeChunked, IntoSeries, TimeUnit};

    let values = values
        .iter()
        .enumerate()
        .map(|(index, value)| parse_iso_datetime(value, index))
        .collect::<Result<Vec<NaiveDateTime>, _>>()?;
    Ok(TerlanPolarsSeries {
        inner: DatetimeChunked::from_naive_datetime(name.into(), values, TimeUnit::Microseconds)
            .into_series(),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_from_nullable_datetimes(
    _name: &str,
    _values: &[Option<String>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series from nullable datetimes operation through the Terlan Polars adapter.
pub fn series_from_nullable_datetimes(
    name: &str,
    values: &[Option<String>],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use chrono::NaiveDateTime;
    use polars::prelude::{DatetimeChunked, IntoSeries, TimeUnit};

    let values = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value
                .as_deref()
                .map(|value| parse_iso_datetime(value, index))
                .transpose()
        })
        .collect::<Result<Vec<Option<NaiveDateTime>>, _>>()?;
    Ok(TerlanPolarsSeries {
        inner: DatetimeChunked::from_naive_datetime_options(
            name.into(),
            values,
            TimeUnit::Microseconds,
        )
        .into_series(),
    })
}

#[cfg(feature = "real-polars")]
fn parse_iso_datetime(
    value: &str,
    index: usize,
) -> Result<chrono::NaiveDateTime, TerlanPolarsError> {
    chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f").map_err(|error| {
        TerlanPolarsError::new(
            "invalid_datetime",
            format!("invalid ISO local datetime at index {index} (`{value}`): {error}"),
        )
    })
}

#[cfg(feature = "real-polars")]
/// Executes the series from bools operation through the Terlan Polars adapter.
pub fn series_from_bools(
    name: &str,
    values: &[bool],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::NamedFrom;
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::new(name.into(), values),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_empty(
    _name: &str,
    _data_type: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series empty operation through the Terlan Polars adapter.
pub fn series_empty(name: &str, data_type: &str) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::full_null(
            name.into(),
            0,
            &expressions::parse_data_type(data_type)?,
        ),
    })
}

#[cfg(feature = "real-polars")]
/// Executes the series from strings operation through the Terlan Polars adapter.
pub fn series_from_strings(
    name: &str,
    values: &[String],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::NamedFrom;
    Ok(TerlanPolarsSeries {
        inner: polars::prelude::Series::new(name.into(), values),
    })
}

/// Executes the series name operation through the Terlan Polars adapter.
pub fn series_name(series: &TerlanPolarsSeries) -> String {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        String::new()
    }
    #[cfg(feature = "real-polars")]
    {
        series.inner.name().to_string()
    }
}

/// Executes the series len operation through the Terlan Polars adapter.
pub fn series_len(series: &TerlanPolarsSeries) -> usize {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        0
    }
    #[cfg(feature = "real-polars")]
    {
        series.inner.len()
    }
}

/// Executes the series null count operation through the Terlan Polars adapter.
pub fn series_null_count(series: &TerlanPolarsSeries) -> usize {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        0
    }
    #[cfg(feature = "real-polars")]
    {
        series.inner.null_count()
    }
}

/// Compares two Series using Polars equality, where any null prevents equality.
pub fn series_equal(left: &TerlanPolarsSeries, right: &TerlanPolarsSeries) -> bool {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        false
    }
    #[cfg(feature = "real-polars")]
    {
        left.inner.equals(&right.inner)
    }
}

/// Compares two Series using Polars equality with null equal to null.
pub fn series_equal_missing(left: &TerlanPolarsSeries, right: &TerlanPolarsSeries) -> bool {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        false
    }
    #[cfg(feature = "real-polars")]
    {
        left.inner.equals_missing(&right.inner)
    }
}

/// Executes the series data type operation through the Terlan Polars adapter.
pub fn series_data_type(series: &TerlanPolarsSeries) -> String {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        String::new()
    }
    #[cfg(feature = "real-polars")]
    {
        format!("{:?}", series.inner.dtype())
    }
}

#[cfg(not(feature = "real-polars"))]
pub fn series_values(
    _series: &TerlanPolarsSeries,
    limit: usize,
) -> Result<Vec<String>, TerlanPolarsError> {
    validate_rows_limit(limit)?;
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series values operation through the Terlan Polars adapter.
pub fn series_values(
    series: &TerlanPolarsSeries,
    limit: usize,
) -> Result<Vec<String>, TerlanPolarsError> {
    validate_rows_limit(limit)?;
    (0..series.inner.len().min(limit))
        .map(|index| Ok(display_value(series.inner.get(index)?)))
        .collect()
}

#[cfg(not(feature = "real-polars"))]
pub fn series_cast(
    _series: &TerlanPolarsSeries,
    _data_type: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series cast operation through the Terlan Polars adapter.
pub fn series_cast(
    series: &TerlanPolarsSeries,
    data_type: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    let data_type = expressions::parse_data_type(data_type)?;
    if let Some(message) = expressions::polars2_cast_error(series.inner.dtype(), &data_type, false)
    {
        return Err(TerlanPolarsError::new("invalid_cast", message));
    }
    Ok(TerlanPolarsSeries {
        inner: series.inner.cast(&data_type)?,
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn series_to_frame(
    _series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the series to frame operation through the Terlan Polars adapter.
pub fn series_to_frame(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::IntoColumn;
    Ok(TerlanPolarsDataFrame {
        inner: polars::prelude::DataFrame::new_infer_height(vec![series
            .inner
            .clone()
            .into_column()])?,
    })
}

/// Returns ordered DataFrame schema entries.
#[cfg(not(feature = "real-polars"))]
pub fn schema(_df: &TerlanPolarsDataFrame) -> Vec<TerlanPolarsColumnSchema> {
    Vec::new()
}

/// Returns ordered DataFrame schema entries from real Polars metadata.
#[cfg(feature = "real-polars")]
pub fn schema(df: &TerlanPolarsDataFrame) -> Vec<TerlanPolarsColumnSchema> {
    df.inner
        .get_column_names()
        .into_iter()
        .zip(df.inner.dtypes())
        .map(|(name, data_type)| TerlanPolarsColumnSchema {
            name: name.to_string(),
            data_type: format!("{data_type:?}"),
        })
        .collect()
}

/// Returns ordered Polars data-type names for every DataFrame column.
pub fn data_types(df: &TerlanPolarsDataFrame) -> Vec<String> {
    schema(df)
        .into_iter()
        .map(|column| column.data_type)
        .collect()
}

/// Filters a DataFrame by scalar equality.
#[cfg(not(feature = "real-polars"))]
pub fn filter_eq(
    _df: &TerlanPolarsDataFrame,
    _column: &str,
    _value: &TerlanPolarsScalar,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Sorts a DataFrame by one column.
#[cfg(not(feature = "real-polars"))]
pub fn sort_by(
    _df: &TerlanPolarsDataFrame,
    _column: &str,
    _descending: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Groups a DataFrame by key columns and counts rows.
#[cfg(not(feature = "real-polars"))]
pub fn group_count(
    _df: &TerlanPolarsDataFrame,
    keys: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_group_keys",
            "group_count requires at least one key column",
        ));
    }
    Err(unavailable_error())
}

/// Converts a DataFrame into a lazy plan placeholder.
#[cfg(not(feature = "real-polars"))]
pub fn to_lazy(_df: &TerlanPolarsDataFrame) -> TerlanPolarsLazyFrame {
    TerlanPolarsLazyFrame
}

/// Converts a real DataFrame into a lazy plan.
#[cfg(feature = "real-polars")]
pub fn to_lazy(df: &TerlanPolarsDataFrame) -> TerlanPolarsLazyFrame {
    use polars::prelude::IntoLazy;

    TerlanPolarsLazyFrame {
        inner: df.inner.clone().lazy(),
    }
}

/// Adds a scalar-equality predicate to a lazy plan placeholder.
#[cfg(not(feature = "real-polars"))]
pub fn lazy_filter_eq(
    _plan: &TerlanPolarsLazyFrame,
    _column: &str,
    _value: &TerlanPolarsScalar,
) -> TerlanPolarsLazyFrame {
    TerlanPolarsLazyFrame
}

/// Adds a scalar-equality predicate to a real lazy plan.
#[cfg(feature = "real-polars")]
pub fn lazy_filter_eq(
    plan: &TerlanPolarsLazyFrame,
    column: &str,
    value: &TerlanPolarsScalar,
) -> TerlanPolarsLazyFrame {
    use polars::prelude::{col, lit};

    let literal = match value {
        TerlanPolarsScalar::String(value) => lit(value.clone()),
        TerlanPolarsScalar::Int(value) => lit(*value),
        TerlanPolarsScalar::Float(value) => lit(*value),
        TerlanPolarsScalar::Bool(value) => lit(*value),
    };
    TerlanPolarsLazyFrame {
        inner: plan.inner.clone().filter(col(column).eq(literal)),
    }
}

/// Adds a projection to a lazy plan placeholder.
#[cfg(not(feature = "real-polars"))]
pub fn lazy_select(_plan: &TerlanPolarsLazyFrame, _columns: &[String]) -> TerlanPolarsLazyFrame {
    TerlanPolarsLazyFrame
}

/// Adds a projection to a real lazy plan.
#[cfg(feature = "real-polars")]
pub fn lazy_select(plan: &TerlanPolarsLazyFrame, columns: &[String]) -> TerlanPolarsLazyFrame {
    use polars::prelude::col;

    TerlanPolarsLazyFrame {
        inner: plan
            .inner
            .clone()
            .select(columns.iter().map(col).collect::<Vec<_>>()),
    }
}

#[cfg(not(feature = "real-polars"))]
pub fn lazy_scan_csv(
    _path: &str,
    _parse_dates: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the lazy scan csv operation through the Terlan Polars adapter.
pub fn lazy_scan_csv(
    path: &str,
    parse_dates: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    lazy_scan_csv_with_storage_options(path, parse_dates, &[], &[])
}

#[cfg(not(feature = "real-polars"))]
pub fn lazy_scan_csv_with_storage_options(
    _path: &str,
    _parse_dates: bool,
    _keys: &[String],
    _values: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
fn scan_cloud_options(
    path: &str,
    keys: &[String],
    values: &[String],
) -> Result<Option<polars::io::cloud::CloudOptions>, TerlanPolarsError> {
    use polars::prelude::CloudScheme;

    if keys.len() != values.len() {
        return Err(TerlanPolarsError::new(
            "invalid_storage_options",
            "storage option keys and values must have the same length",
        ));
    }
    if keys.is_empty() {
        return Ok(None);
    }

    let options = polars::io::cloud::CloudOptions::from_untyped_config(
        CloudScheme::from_path(path),
        keys.iter()
            .zip(values.iter())
            .map(|(key, value)| (key, value.clone())),
    )?;
    Ok(Some(options))
}

#[cfg(feature = "real-polars")]
/// Scans CSV with explicit per-call cloud storage configuration.
pub fn lazy_scan_csv_with_storage_options(
    path: &str,
    parse_dates: bool,
    keys: &[String],
    values: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    use polars::prelude::{LazyCsvReader, LazyFileListReader, PlRefPath};
    Ok(TerlanPolarsLazyFrame {
        inner: LazyCsvReader::new(PlRefPath::new(path))
            .with_try_parse_dates(parse_dates)
            .with_missing_is_null(true)
            .with_cloud_options(scan_cloud_options(path, keys, values)?)
            .finish()?,
    })
}

macro_rules! unavailable_lazy_scan {
    ($name:ident) => {
        #[cfg(not(feature = "real-polars"))]
        pub fn $name(_path: &str) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
            Err(unavailable_error())
        }
    };
}

unavailable_lazy_scan!(lazy_scan_parquet);
unavailable_lazy_scan!(lazy_scan_ndjson);
unavailable_lazy_scan!(lazy_scan_ipc);

macro_rules! unavailable_lazy_scan_with_storage_options {
    ($name:ident) => {
        #[cfg(not(feature = "real-polars"))]
        pub fn $name(
            _path: &str,
            _keys: &[String],
            _values: &[String],
        ) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
            Err(unavailable_error())
        }
    };
}

unavailable_lazy_scan_with_storage_options!(lazy_scan_parquet_with_storage_options);
unavailable_lazy_scan_with_storage_options!(lazy_scan_ndjson_with_storage_options);
unavailable_lazy_scan_with_storage_options!(lazy_scan_ipc_with_storage_options);

#[cfg(feature = "real-polars")]
/// Executes the lazy scan parquet operation through the Terlan Polars adapter.
pub fn lazy_scan_parquet(path: &str) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    lazy_scan_parquet_with_storage_options(path, &[], &[])
}

#[cfg(feature = "real-polars")]
/// Scans Parquet with explicit per-call cloud storage configuration.
pub fn lazy_scan_parquet_with_storage_options(
    path: &str,
    keys: &[String],
    values: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    use polars::prelude::{LazyFrame, PlRefPath, ScanArgsParquet};
    let options = ScanArgsParquet {
        cloud_options: scan_cloud_options(path, keys, values)?,
        ..Default::default()
    };
    Ok(TerlanPolarsLazyFrame {
        inner: LazyFrame::scan_parquet(PlRefPath::new(path), options)?,
    })
}

#[cfg(feature = "real-polars")]
/// Executes the lazy scan ndjson operation through the Terlan Polars adapter.
pub fn lazy_scan_ndjson(path: &str) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    lazy_scan_ndjson_with_storage_options(path, &[], &[])
}

#[cfg(feature = "real-polars")]
/// Scans newline-delimited JSON with explicit per-call cloud storage configuration.
pub fn lazy_scan_ndjson_with_storage_options(
    path: &str,
    keys: &[String],
    values: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    use polars::prelude::{LazyFileListReader, LazyJsonLineReader, PlRefPath};
    Ok(TerlanPolarsLazyFrame {
        inner: LazyJsonLineReader::new(PlRefPath::new(path))
            .with_cloud_options(scan_cloud_options(path, keys, values)?)
            .finish()?,
    })
}

#[cfg(feature = "real-polars")]
/// Executes the lazy scan ipc operation through the Terlan Polars adapter.
pub fn lazy_scan_ipc(path: &str) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    lazy_scan_ipc_with_storage_options(path, &[], &[])
}

#[cfg(feature = "real-polars")]
/// Scans Arrow IPC with explicit per-call cloud storage configuration.
pub fn lazy_scan_ipc_with_storage_options(
    path: &str,
    keys: &[String],
    values: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    use polars::prelude::{LazyFrame, PlRefPath, UnifiedScanArgs};
    let scan_args = UnifiedScanArgs {
        cloud_options: scan_cloud_options(path, keys, values)?,
        ..Default::default()
    };
    Ok(TerlanPolarsLazyFrame {
        inner: LazyFrame::scan_ipc(PlRefPath::new(path), Default::default(), scan_args)?,
    })
}

macro_rules! unavailable_lazy_write {
    ($name:ident) => {
        #[cfg(not(feature = "real-polars"))]
        pub fn $name(_plan: &TerlanPolarsLazyFrame, _path: &str) -> Result<(), TerlanPolarsError> {
            Err(unavailable_error())
        }
    };
}

unavailable_lazy_write!(lazy_write_csv);
unavailable_lazy_write!(lazy_write_parquet);
unavailable_lazy_write!(lazy_write_ndjson);
unavailable_lazy_write!(lazy_write_ipc);

#[cfg(feature = "real-polars")]
fn lazy_write_format(
    plan: &TerlanPolarsLazyFrame,
    path: &str,
    format: polars::prelude::FileWriteFormat,
) -> Result<(), TerlanPolarsError> {
    use polars::prelude::{PlRefPath, SinkDestination, SinkTarget, UnifiedSinkArgs};
    plan.inner
        .clone()
        .sink(
            SinkDestination::File {
                target: SinkTarget::Path(PlRefPath::new(path.to_string())),
            },
            format,
            UnifiedSinkArgs::default(),
        )?
        .collect()?;
    Ok(())
}

/// Executes a lazy plan directly into a CSV streaming sink.
#[cfg(feature = "real-polars")]
pub fn lazy_write_csv(plan: &TerlanPolarsLazyFrame, path: &str) -> Result<(), TerlanPolarsError> {
    use polars::prelude::{CsvWriterOptions, FileWriteFormat};
    lazy_write_format(
        plan,
        path,
        FileWriteFormat::Csv(CsvWriterOptions::default()),
    )
}

/// Executes a lazy plan directly into a Parquet streaming sink.
#[cfg(feature = "real-polars")]
pub fn lazy_write_parquet(
    plan: &TerlanPolarsLazyFrame,
    path: &str,
) -> Result<(), TerlanPolarsError> {
    use polars::prelude::{FileWriteFormat, ParquetWriteOptions};
    lazy_write_format(
        plan,
        path,
        FileWriteFormat::Parquet(ParquetWriteOptions::default().into()),
    )
}

/// Executes a lazy plan directly into an NDJSON streaming sink.
#[cfg(feature = "real-polars")]
pub fn lazy_write_ndjson(
    plan: &TerlanPolarsLazyFrame,
    path: &str,
) -> Result<(), TerlanPolarsError> {
    use polars::prelude::{FileWriteFormat, NDJsonWriterOptions};
    lazy_write_format(
        plan,
        path,
        FileWriteFormat::NDJson(NDJsonWriterOptions::default()),
    )
}

/// Executes a lazy plan directly into an Arrow IPC streaming sink.
#[cfg(feature = "real-polars")]
pub fn lazy_write_ipc(plan: &TerlanPolarsLazyFrame, path: &str) -> Result<(), TerlanPolarsError> {
    use polars::prelude::{FileWriteFormat, IpcWriterOptions};
    lazy_write_format(
        plan,
        path,
        FileWriteFormat::Ipc(IpcWriterOptions::default()),
    )
}

#[cfg(feature = "real-polars")]
fn lazy_write_partitioned_format(
    plan: &TerlanPolarsLazyFrame,
    base_path: &str,
    keys: &[String],
    include_keys: bool,
    max_rows_per_file: i64,
    approximate_bytes_per_file: i64,
    format: polars::prelude::FileWriteFormat,
) -> Result<(), TerlanPolarsError> {
    use polars::prelude::{PartitionStrategy, PlRefPath, SinkDestination, UnifiedSinkArgs};

    if base_path.trim().is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_io_options",
            "partitioned sink base path cannot be empty",
        ));
    }
    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_io_options",
            "partitioned sink requires at least one key expression",
        ));
    }
    let max_rows_per_file = u32::try_from(max_rows_per_file).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_io_options",
            "partitioned sink maximum rows must fit an unsigned 32-bit integer",
        )
    })?;
    let approximate_bytes_per_file = u64::try_from(approximate_bytes_per_file).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_io_options",
            "partitioned sink approximate bytes cannot be negative",
        )
    })?;
    plan.inner
        .clone()
        .sink(
            SinkDestination::Partitioned {
                base_path: PlRefPath::new(base_path.to_string()),
                file_path_provider: None,
                partition_strategy: PartitionStrategy::Keyed {
                    keys: expressions::compile_many(keys)?,
                    include_keys,
                    keys_pre_grouped: false,
                },
                max_rows_per_file,
                approximate_bytes_per_file,
            },
            format,
            UnifiedSinkArgs {
                mkdir: true,
                ..UnifiedSinkArgs::default()
            },
        )?
        .collect()?;
    Ok(())
}

macro_rules! partitioned_sink_stub {
    ($name:ident) => {
        #[cfg(not(feature = "real-polars"))]
        pub fn $name(
            _plan: &TerlanPolarsLazyFrame,
            _base_path: &str,
            keys: &[String],
            _include_keys: bool,
            _max_rows_per_file: i64,
            _approximate_bytes_per_file: i64,
        ) -> Result<(), TerlanPolarsError> {
            if keys.is_empty() {
                return Err(TerlanPolarsError::new(
                    "invalid_io_options",
                    "partitioned sink requires at least one key expression",
                ));
            }
            Err(unavailable_error())
        }
    };
}

partitioned_sink_stub!(lazy_write_csv_partitioned);
partitioned_sink_stub!(lazy_write_parquet_partitioned);
partitioned_sink_stub!(lazy_write_ndjson_partitioned);
partitioned_sink_stub!(lazy_write_ipc_partitioned);

#[cfg(feature = "real-polars")]
/// Executes a lazy plan into keyed CSV partitions.
pub fn lazy_write_csv_partitioned(
    plan: &TerlanPolarsLazyFrame,
    base_path: &str,
    keys: &[String],
    include_keys: bool,
    max_rows_per_file: i64,
    approximate_bytes_per_file: i64,
) -> Result<(), TerlanPolarsError> {
    lazy_write_partitioned_format(
        plan,
        base_path,
        keys,
        include_keys,
        max_rows_per_file,
        approximate_bytes_per_file,
        polars::prelude::FileWriteFormat::Csv(Default::default()),
    )
}

#[cfg(feature = "real-polars")]
/// Executes a lazy plan into keyed Parquet partitions.
pub fn lazy_write_parquet_partitioned(
    plan: &TerlanPolarsLazyFrame,
    base_path: &str,
    keys: &[String],
    include_keys: bool,
    max_rows_per_file: i64,
    approximate_bytes_per_file: i64,
) -> Result<(), TerlanPolarsError> {
    lazy_write_partitioned_format(
        plan,
        base_path,
        keys,
        include_keys,
        max_rows_per_file,
        approximate_bytes_per_file,
        polars::prelude::FileWriteFormat::Parquet(Default::default()),
    )
}

#[cfg(feature = "real-polars")]
/// Executes a lazy plan into keyed NDJSON partitions.
pub fn lazy_write_ndjson_partitioned(
    plan: &TerlanPolarsLazyFrame,
    base_path: &str,
    keys: &[String],
    include_keys: bool,
    max_rows_per_file: i64,
    approximate_bytes_per_file: i64,
) -> Result<(), TerlanPolarsError> {
    lazy_write_partitioned_format(
        plan,
        base_path,
        keys,
        include_keys,
        max_rows_per_file,
        approximate_bytes_per_file,
        polars::prelude::FileWriteFormat::NDJson(Default::default()),
    )
}

#[cfg(feature = "real-polars")]
/// Executes a lazy plan into keyed Arrow IPC partitions.
pub fn lazy_write_ipc_partitioned(
    plan: &TerlanPolarsLazyFrame,
    base_path: &str,
    keys: &[String],
    include_keys: bool,
    max_rows_per_file: i64,
    approximate_bytes_per_file: i64,
) -> Result<(), TerlanPolarsError> {
    lazy_write_partitioned_format(
        plan,
        base_path,
        keys,
        include_keys,
        max_rows_per_file,
        approximate_bytes_per_file,
        polars::prelude::FileWriteFormat::Ipc(Default::default()),
    )
}

#[cfg(not(feature = "real-polars"))]
pub fn lazy_select_exprs(
    _plan: &TerlanPolarsLazyFrame,
    _expressions: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the lazy select exprs operation through the Terlan Polars adapter.
pub fn lazy_select_exprs(
    plan: &TerlanPolarsLazyFrame,
    expressions: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Ok(TerlanPolarsLazyFrame {
        inner: plan
            .inner
            .clone()
            .select(expressions::compile_many(expressions)?),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn lazy_select_exprs_sequential(
    _plan: &TerlanPolarsLazyFrame,
    _expressions: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Projects expressions sequentially instead of scheduling them in parallel.
#[cfg(feature = "real-polars")]
pub fn lazy_select_exprs_sequential(
    plan: &TerlanPolarsLazyFrame,
    expressions: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Ok(TerlanPolarsLazyFrame {
        inner: plan
            .inner
            .clone()
            .select_seq(expressions::compile_many(expressions)?),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn lazy_with_columns(
    _plan: &TerlanPolarsLazyFrame,
    _expressions: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the lazy with columns operation through the Terlan Polars adapter.
pub fn lazy_with_columns(
    plan: &TerlanPolarsLazyFrame,
    expressions: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Ok(TerlanPolarsLazyFrame {
        inner: plan
            .inner
            .clone()
            .with_columns(expressions::compile_many(expressions)?),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn lazy_with_columns_sequential(
    _plan: &TerlanPolarsLazyFrame,
    _expressions: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Adds or replaces columns sequentially instead of scheduling expressions in
/// parallel.
#[cfg(feature = "real-polars")]
pub fn lazy_with_columns_sequential(
    plan: &TerlanPolarsLazyFrame,
    expressions: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Ok(TerlanPolarsLazyFrame {
        inner: plan
            .inner
            .clone()
            .with_columns_seq(expressions::compile_many(expressions)?),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn lazy_filter_expr(
    _plan: &TerlanPolarsLazyFrame,
    _predicate: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the lazy filter expr operation through the Terlan Polars adapter.
pub fn lazy_filter_expr(
    plan: &TerlanPolarsLazyFrame,
    predicate: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    let mut predicate = expressions::compile_many(&[predicate.to_string()])?;
    Ok(TerlanPolarsLazyFrame {
        inner: plan.inner.clone().filter(predicate.remove(0)),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn lazy_remove_where(
    _plan: &TerlanPolarsLazyFrame,
    _predicate: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Removes rows where a predicate is true while retaining rows where it is
/// false or null.
#[cfg(feature = "real-polars")]
pub fn lazy_remove_where(
    plan: &TerlanPolarsLazyFrame,
    predicate: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    let mut predicate = expressions::compile_many(&[predicate.to_string()])?;
    Ok(TerlanPolarsLazyFrame {
        inner: plan.inner.clone().remove(predicate.remove(0)),
    })
}

#[cfg(not(feature = "real-polars"))]
pub fn lazy_group_agg(
    _plan: &TerlanPolarsLazyFrame,
    _keys: &[String],
    _aggregations: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the lazy group agg operation through the Terlan Polars adapter.
pub fn lazy_group_agg(
    plan: &TerlanPolarsLazyFrame,
    keys: &[String],
    aggregations: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_group_keys",
            "lazy_group_agg requires at least one key expression",
        ));
    }
    Ok(TerlanPolarsLazyFrame {
        inner: plan
            .inner
            .clone()
            .group_by(expressions::compile_many(keys)?)
            .agg(expressions::compile_many(aggregations)?),
    })
}

/// Executes the lazy sort operation through the Terlan Polars adapter.
pub fn lazy_sort(
    plan: &TerlanPolarsLazyFrame,
    column: &str,
    descending: bool,
) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, column, descending);
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().sort(
                [column],
                polars::prelude::SortMultipleOptions::default().with_order_descending(descending),
            ),
        }
    }
}

fn validate_multi_sort_options(
    columns: &[String],
    descending: &[bool],
    nulls_last: &[bool],
) -> Result<(), TerlanPolarsError> {
    if columns.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_sort_columns",
            "multi-column ordering requires at least one key column",
        ));
    }
    if descending.len() != columns.len() || nulls_last.len() != columns.len() {
        return Err(TerlanPolarsError::new(
            "invalid_sort_options",
            "sort columns, descending flags, and null-placement flags must have equal lengths",
        ));
    }
    Ok(())
}

fn validate_row_selection_limit(limit: i64) -> Result<usize, TerlanPolarsError> {
    if !(0..=u32::MAX as i64).contains(&limit) {
        return Err(TerlanPolarsError::new(
            "invalid_row_limit",
            "row selection limit must be between 0 and 4294967295",
        ));
    }
    Ok(limit as usize)
}

/// Sorts a lazy frame by multiple columns with per-key options.
pub fn lazy_sort_rows_by(
    plan: &TerlanPolarsLazyFrame,
    columns: &[String],
    descending: &[bool],
    nulls_last: &[bool],
    maintain_order: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_multi_sort_options(columns, descending, nulls_last)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, maintain_order);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::SortMultipleOptions;
        let options = SortMultipleOptions::default()
            .with_order_descending_multi(descending.iter().copied())
            .with_nulls_last_multi(nulls_last.iter().copied())
            .with_maintain_order(maintain_order);
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().sort(
                columns.iter().map(String::as_str).collect::<Vec<_>>(),
                options,
            ),
        })
    }
}

/// Keeps the greatest rows under a multi-key ordering.
pub fn lazy_top_rows_by(
    plan: &TerlanPolarsLazyFrame,
    columns: &[String],
    limit: i64,
    nulls_last: &[bool],
    maintain_order: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    let limit = validate_row_selection_limit(limit)?;
    let descending = vec![true; columns.len()];
    let sorted = lazy_sort_rows_by(plan, columns, &descending, nulls_last, maintain_order)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (sorted, limit);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsLazyFrame {
            inner: sorted.inner.limit(limit as polars::prelude::IdxSize),
        })
    }
}

/// Keeps the smallest rows under a multi-key ordering.
pub fn lazy_bottom_rows_by(
    plan: &TerlanPolarsLazyFrame,
    columns: &[String],
    limit: i64,
    nulls_last: &[bool],
    maintain_order: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    let limit = validate_row_selection_limit(limit)?;
    let descending = vec![false; columns.len()];
    let sorted = lazy_sort_rows_by(plan, columns, &descending, nulls_last, maintain_order)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (sorted, limit);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsLazyFrame {
            inner: sorted.inner.limit(limit as polars::prelude::IdxSize),
        })
    }
}

/// Sorts a materialized frame by multiple columns with per-key options.
pub fn sort_rows_by(
    df: &TerlanPolarsDataFrame,
    columns: &[String],
    descending: &[bool],
    nulls_last: &[bool],
    maintain_order: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_sort_rows_by(
        &to_lazy(df),
        columns,
        descending,
        nulls_last,
        maintain_order,
    )?)
}

/// Keeps the greatest materialized rows under a multi-key ordering.
pub fn top_rows_by(
    df: &TerlanPolarsDataFrame,
    columns: &[String],
    limit: i64,
    nulls_last: &[bool],
    maintain_order: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_top_rows_by(
        &to_lazy(df),
        columns,
        limit,
        nulls_last,
        maintain_order,
    )?)
}

/// Keeps the smallest materialized rows under a multi-key ordering.
pub fn bottom_rows_by(
    df: &TerlanPolarsDataFrame,
    columns: &[String],
    limit: i64,
    nulls_last: &[bool],
    maintain_order: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_bottom_rows_by(
        &to_lazy(df),
        columns,
        limit,
        nulls_last,
        maintain_order,
    )?)
}

/// Executes the lazy limit operation through the Terlan Polars adapter.
pub fn lazy_limit(plan: &TerlanPolarsLazyFrame, limit: usize) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, limit);
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().limit(limit as polars::prelude::IdxSize),
        }
    }
}

/// Keeps at most the final requested number of deferred rows.
pub fn lazy_tail(plan: &TerlanPolarsLazyFrame, limit: usize) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, limit);
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().tail(limit as polars::prelude::IdxSize),
        }
    }
}

/// Reduces every deferred column to its non-null sum.
pub fn lazy_column_sums(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().sum()]),
        }
    }
}

/// Reduces every deferred numeric column to its mean.
pub fn lazy_column_means(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().mean()]),
        }
    }
}

/// Reduces every deferred numeric column to its median.
pub fn lazy_column_medians(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().median()]),
        }
    }
}

/// Reduces every deferred column to its minimum value.
pub fn lazy_column_minima(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().min()]),
        }
    }
}

/// Reduces every deferred column to its maximum value.
pub fn lazy_column_maxima(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().max()]),
        }
    }
}

fn validate_column_ddof(ddof: i64) -> Result<u8, TerlanPolarsError> {
    u8::try_from(ddof).map_err(|_| {
        TerlanPolarsError::new(
            "invalid_degrees_of_freedom",
            "column reduction ddof must be between 0 and 255",
        )
    })
}

fn validate_column_quantile_probability(probability: f64) -> Result<(), TerlanPolarsError> {
    if probability.is_finite() && (0.0..=1.0).contains(&probability) {
        Ok(())
    } else {
        Err(TerlanPolarsError::new(
            "invalid_quantile_probability",
            "column quantile probability must be finite and between 0 and 1",
        ))
    }
}

fn validate_column_quantile_method(method: &str) -> Result<(), TerlanPolarsError> {
    match method {
        "nearest" | "lower" | "higher" | "midpoint" | "linear" | "equiprobable" => Ok(()),
        _ => Err(TerlanPolarsError::new(
            "invalid_quantile_method",
            "column quantile method must be nearest, lower, higher, midpoint, linear, or equiprobable",
        )),
    }
}

#[cfg(feature = "real-polars")]
fn parse_column_quantile_method(
    method: &str,
) -> Result<polars::prelude::QuantileMethod, TerlanPolarsError> {
    use polars::prelude::QuantileMethod;
    match method {
        "nearest" => Ok(QuantileMethod::Nearest),
        "lower" => Ok(QuantileMethod::Lower),
        "higher" => Ok(QuantileMethod::Higher),
        "midpoint" => Ok(QuantileMethod::Midpoint),
        "linear" => Ok(QuantileMethod::Linear),
        "equiprobable" => Ok(QuantileMethod::Equiprobable),
        _ => Err(TerlanPolarsError::new(
            "invalid_quantile_method",
            "column quantile method must be nearest, lower, higher, midpoint, linear, or equiprobable",
        )),
    }
}

/// Reduces every deferred numeric column to its product.
pub fn lazy_column_products(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().product()]),
        }
    }
}

/// Reduces every deferred numeric column to its variance.
pub fn lazy_column_variances(
    plan: &TerlanPolarsLazyFrame,
    ddof: i64,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    let ddof = validate_column_ddof(ddof)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, ddof);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().var(ddof)]),
        })
    }
}

/// Reduces every deferred numeric column to its standard deviation.
pub fn lazy_column_stddevs(
    plan: &TerlanPolarsLazyFrame,
    ddof: i64,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    let ddof = validate_column_ddof(ddof)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, ddof);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().std(ddof)]),
        })
    }
}

/// Reduces every deferred numeric column to a configurable quantile.
pub fn lazy_column_quantiles(
    plan: &TerlanPolarsLazyFrame,
    probability: f64,
    method: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_column_quantile_probability(probability)?;
    validate_column_quantile_method(method)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, method);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{all, lit};
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all()
                .as_expr()
                .quantile(lit(probability), parse_column_quantile_method(method)?)]),
        })
    }
}

/// Counts non-null values in every deferred column.
pub fn lazy_column_non_null_counts(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().count()]),
        }
    }
}

/// Counts all values, including nulls, in every deferred column.
pub fn lazy_column_lengths(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().len()]),
        }
    }
}

/// Counts exact distinct values in every deferred column.
pub fn lazy_column_unique_counts(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().select([all().as_expr().n_unique()]),
        }
    }
}

/// Estimates distinct-value cardinality in every deferred column.
pub fn lazy_column_approx_unique_counts(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::all;
        TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .select([all().as_expr().approx_n_unique()]),
        }
    }
}

fn validate_unique_keep(value: &str) -> Result<(), TerlanPolarsError> {
    match value {
        "KeepFirst" | "KeepLast" | "KeepAny" | "KeepNone" | "keep_first" | "keep_last"
        | "keep_any" | "keep_none" => Ok(()),
        _ => Err(TerlanPolarsError::new(
            "invalid_unique_keep",
            format!(
                "unsupported unique keep strategy `{value}`; expected KeepFirst, KeepLast, KeepAny, or KeepNone"
            ),
        )),
    }
}

#[cfg(feature = "real-polars")]
fn parse_unique_keep(
    value: &str,
) -> Result<polars::prelude::UniqueKeepStrategy, TerlanPolarsError> {
    use polars::prelude::UniqueKeepStrategy;
    validate_unique_keep(value)?;
    Ok(match value {
        "KeepFirst" | "keep_first" => UniqueKeepStrategy::First,
        "KeepLast" | "keep_last" => UniqueKeepStrategy::Last,
        "KeepAny" | "keep_any" => UniqueKeepStrategy::Any,
        "KeepNone" | "keep_none" => UniqueKeepStrategy::None,
        _ => unreachable!("validated unique keep strategy"),
    })
}

fn validate_slice_length(length: i64) -> Result<usize, TerlanPolarsError> {
    if !(0..=u32::MAX as i64).contains(&length) {
        return Err(TerlanPolarsError::new(
            "invalid_slice_length",
            "row slice length must be between 0 and 4294967295",
        ));
    }
    Ok(length as usize)
}

fn validate_gather_indices(indices: &[i64]) -> Result<Vec<u32>, TerlanPolarsError> {
    indices
        .iter()
        .enumerate()
        .map(|(position, index)| {
            u32::try_from(*index).map_err(|_| {
                TerlanPolarsError::new(
                    "invalid_row_index",
                    format!(
                        "row index at position {position} must be between 0 and 4294967295; found {index}"
                    ),
                )
            })
        })
        .collect()
}

/// Drops duplicate rows without materializing a lazy plan.
pub fn lazy_unique_rows(
    plan: &TerlanPolarsLazyFrame,
    subset: &[String],
    keep: &str,
    maintain_order: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_unique_keep(keep)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, subset, maintain_order);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::cols;
        let subset = (!subset.is_empty()).then(|| cols(subset.iter().cloned()));
        let keep = parse_unique_keep(keep)?;
        let inner = if maintain_order {
            plan.inner.clone().unique_stable(subset, keep)
        } else {
            plan.inner.clone().unique(subset, keep)
        };
        Ok(TerlanPolarsLazyFrame { inner })
    }
}

/// Drops rows containing nulls without materializing a lazy plan.
pub fn lazy_drop_null_rows(
    plan: &TerlanPolarsLazyFrame,
    subset: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, subset);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::cols;
        let subset = (!subset.is_empty()).then(|| cols(subset.iter().cloned()));
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().drop_nulls(subset),
        })
    }
}

/// Selects a bounded row range without materializing a lazy plan.
pub fn lazy_slice_rows(
    plan: &TerlanPolarsLazyFrame,
    offset: i64,
    length: i64,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    let length = validate_slice_length(length)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, offset, length);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .slice(offset, length as polars::prelude::IdxSize),
        })
    }
}

/// Gathers deferred rows by unsigned indices, optionally producing null rows
/// instead of collection errors for indices beyond the eventual frame height.
pub fn lazy_gather_rows(
    plan: &TerlanPolarsLazyFrame,
    indices: &[i64],
    null_on_out_of_bounds: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    let indices = validate_gather_indices(indices)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, indices, null_on_out_of_bounds);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{DataFrame, IdxCa, IntoColumn, IntoLazy, NamedFrom};
        let index_frame =
            DataFrame::new_infer_height(vec![
                IdxCa::new("row_index".into(), indices).into_column()
            ])?;
        Ok(TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .gather(index_frame.lazy(), null_on_out_of_bounds),
        })
    }
}

/// Removes every deferred row while retaining the logical schema.
pub fn lazy_clear_rows(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().clear(),
        }
    }
}

/// Keeps only the first deferred row.
pub fn lazy_first_row(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().first(),
        }
    }
}

/// Keeps only the final deferred row.
pub fn lazy_last_row(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().last(),
        }
    }
}

fn validate_rename_columns(
    operation: &str,
    existing: &[String],
    new: &[String],
) -> Result<(), TerlanPolarsError> {
    if existing.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_rename_columns",
            format!("{operation} requires at least one source column"),
        ));
    }
    if existing.len() != new.len() {
        return Err(TerlanPolarsError::new(
            "invalid_rename_columns",
            format!("{operation} requires the same number of source and destination columns"),
        ));
    }
    Ok(())
}

fn validate_drop_columns(operation: &str, columns: &[String]) -> Result<(), TerlanPolarsError> {
    if columns.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_drop_columns",
            format!("{operation} requires at least one column"),
        ));
    }
    Ok(())
}

fn validate_row_index(name: &str, offset: i64) -> Result<usize, TerlanPolarsError> {
    if name.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_row_index",
            "row-index name cannot be empty",
        ));
    }
    if !(0..=u32::MAX as i64).contains(&offset) {
        return Err(TerlanPolarsError::new(
            "invalid_row_index",
            "row-index offset must be between 0 and 4294967295",
        ));
    }
    Ok(offset as usize)
}

/// Renames lazy columns simultaneously.
pub fn lazy_rename_columns(
    plan: &TerlanPolarsLazyFrame,
    existing: &[String],
    new: &[String],
    strict: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_rename_columns("lazy_rename_columns", existing, new)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, strict);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().rename(
                existing.iter().map(String::as_str),
                new.iter().map(String::as_str),
                strict,
            ),
        })
    }
}

/// Removes named columns from a lazy plan.
pub fn lazy_drop_columns(
    plan: &TerlanPolarsLazyFrame,
    columns: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_drop_columns("lazy_drop_columns", columns)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::cols;
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().drop(cols(columns.iter().cloned())),
        })
    }
}

/// Reverses all rows in a lazy plan.
pub fn lazy_reverse_rows(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().reverse(),
        }
    }
}

/// Prepends an unsigned row index to a lazy plan.
pub fn lazy_with_row_index(
    plan: &TerlanPolarsLazyFrame,
    name: &str,
    offset: i64,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    let offset = validate_row_index(name, offset)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, offset);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .with_row_index(name, Some(offset as polars::prelude::IdxSize)),
        })
    }
}

/// Fills null values throughout a lazy frame with one expression.
pub fn lazy_fill_null_values(
    plan: &TerlanPolarsLazyFrame,
    value: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, value);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .fill_null(expressions::compile_one(value)?),
        })
    }
}

/// Fills floating-point NaN values throughout a lazy frame with one expression.
pub fn lazy_fill_nan_values(
    plan: &TerlanPolarsLazyFrame,
    value: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, value);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .fill_nan(expressions::compile_one(value)?),
        })
    }
}

/// Removes rows containing NaN values in an optional column subset.
pub fn lazy_drop_nan_rows(
    plan: &TerlanPolarsLazyFrame,
    subset: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, subset);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::cols;
        let subset = (!subset.is_empty()).then(|| cols(subset.iter().cloned()));
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().drop_nans(subset),
        })
    }
}

/// Produces one row containing every column's null count.
pub fn lazy_null_counts(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().null_count(),
        }
    }
}

/// Resolves a lazy plan's schema without collecting its rows.
pub fn lazy_schema(
    plan: &TerlanPolarsLazyFrame,
) -> Result<Vec<TerlanPolarsColumnSchema>, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        let schema = plan.inner.clone().collect_schema()?;
        Ok(schema
            .iter()
            .map(|(name, data_type)| TerlanPolarsColumnSchema {
                name: name.to_string(),
                data_type: format!("{data_type:?}"),
            })
            .collect())
    }
}

/// Inserts an explicit cache node into a lazy plan.
pub fn lazy_cache(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().cache(),
        }
    }
}

fn validate_lazy_optimization(name: &str) -> Result<(), TerlanPolarsError> {
    match name {
        "projection_pushdown"
        | "predicate_pushdown"
        | "simplify_expression"
        | "slice_pushdown"
        | "common_subplan_elimination"
        | "common_subexpression_elimination"
        | "cluster_with_columns"
        | "check_order"
        | "eager"
        | "gpu"
        | "row_estimate"
        | "streaming"
        | "type_coercion"
        | "type_check" => Ok(()),
        _ => Err(TerlanPolarsError::new(
            "invalid_lazy_optimization",
            format!(
                "unsupported lazy optimization `{name}`; expected projection_pushdown, predicate_pushdown, simplify_expression, slice_pushdown, common_subplan_elimination, common_subexpression_elimination, cluster_with_columns, check_order, eager, gpu, row_estimate, streaming, type_coercion, or type_check"
            ),
        )),
    }
}

/// Creates a derived plan with all optional optimizer passes disabled. Polars
/// retains type coercion because it is required for a valid logical plan.
pub fn lazy_without_optimizations(plan: &TerlanPolarsLazyFrame) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = plan;
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().without_optimizations(),
        }
    }
}

/// Toggles one named optimizer pass on a derived lazy plan.
pub fn lazy_set_optimization(
    plan: &TerlanPolarsLazyFrame,
    name: &str,
    enabled: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_lazy_optimization(name)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, enabled);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        let inner = match name {
            "projection_pushdown" => plan.inner.clone().with_projection_pushdown(enabled),
            "predicate_pushdown" => plan.inner.clone().with_predicate_pushdown(enabled),
            "simplify_expression" => plan.inner.clone().with_simplify_expr(enabled),
            "slice_pushdown" => plan.inner.clone().with_slice_pushdown(enabled),
            "common_subplan_elimination" => plan.inner.clone().with_comm_subplan_elim(enabled),
            "common_subexpression_elimination" => {
                plan.inner.clone().with_comm_subexpr_elim(enabled)
            }
            "cluster_with_columns" => plan.inner.clone().with_cluster_with_columns(enabled),
            "check_order" => plan.inner.clone().with_check_order(enabled),
            "eager" => plan.inner.clone()._with_eager(enabled),
            "gpu" => plan.inner.clone().with_gpu(enabled),
            "row_estimate" => plan.inner.clone().with_row_estimate(enabled),
            "streaming" => plan.inner.clone().with_streaming(enabled),
            "type_coercion" => plan.inner.clone().with_type_coercion(enabled),
            "type_check" => plan.inner.clone().with_type_check(enabled),
            _ => unreachable!("validated lazy optimization"),
        };
        Ok(TerlanPolarsLazyFrame { inner })
    }
}

/// Shifts every lazy column by a signed row count.
pub fn lazy_shift_rows(plan: &TerlanPolarsLazyFrame, periods: i64) -> TerlanPolarsLazyFrame {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, periods);
        TerlanPolarsLazyFrame
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::lit;
        TerlanPolarsLazyFrame {
            inner: plan.inner.clone().shift(lit(periods)),
        }
    }
}

/// Shifts every lazy column and fills the resulting gap from an expression.
pub fn lazy_shift_and_fill_rows(
    plan: &TerlanPolarsLazyFrame,
    periods: i64,
    fill_value: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, periods, fill_value);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::lit;
        Ok(TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .shift_and_fill(lit(periods), expressions::compile_one(fill_value)?),
        })
    }
}

#[cfg(not(feature = "real-polars"))]
pub fn lazy_left_join(
    _left: &TerlanPolarsLazyFrame,
    _right: &TerlanPolarsLazyFrame,
    _keys: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

fn validate_join_keys(
    operation: &str,
    kind: &str,
    left_on: &[String],
    right_on: &[String],
) -> Result<(), TerlanPolarsError> {
    if kind != "cross" && left_on.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_join_keys",
            format!("{operation} requires at least one key column"),
        ));
    }
    if left_on.len() != right_on.len() {
        return Err(TerlanPolarsError::new(
            "invalid_join_keys",
            format!("{operation} requires the same number of left and right key columns"),
        ));
    }
    Ok(())
}

#[cfg(feature = "real-polars")]
fn parse_join_type(kind: &str) -> Result<polars::prelude::JoinType, TerlanPolarsError> {
    use polars::prelude::JoinType;
    match kind {
        "inner" => Ok(JoinType::Inner),
        "left" => Ok(JoinType::Left),
        "right" => Ok(JoinType::Right),
        "full" => Ok(JoinType::Full),
        "semi" => Ok(JoinType::Semi),
        "anti" => Ok(JoinType::Anti),
        "cross" => Ok(JoinType::Cross),
        _ => Err(TerlanPolarsError::new(
            "invalid_join_type",
            format!(
                "unsupported join type `{kind}`; expected inner, left, right, full, semi, anti, or cross"
            ),
        )),
    }
}

fn validate_join_validation(value: &str) -> Result<(), TerlanPolarsError> {
    match value {
        "ManyToMany" | "ManyToOne" | "OneToMany" | "OneToOne" | "many_to_many"
        | "many_to_one" | "one_to_many" | "one_to_one" => Ok(()),
        _ => Err(TerlanPolarsError::new(
            "invalid_join_validation",
            format!(
                "unsupported join validation `{value}`; expected ManyToMany, ManyToOne, OneToMany, or OneToOne"
            ),
        )),
    }
}

fn validate_join_order(value: &str) -> Result<(), TerlanPolarsError> {
    match value {
        "Unordered" | "PreserveLeft" | "PreserveRight" | "PreserveLeftRight"
        | "PreserveRightLeft" | "unordered" | "preserve_left" | "preserve_right"
        | "preserve_left_right" | "preserve_right_left" => Ok(()),
        _ => Err(TerlanPolarsError::new(
            "invalid_join_order",
            format!(
                "unsupported join order `{value}`; expected Unordered, PreserveLeft, PreserveRight, PreserveLeftRight, or PreserveRightLeft"
            ),
        )),
    }
}

#[cfg(feature = "real-polars")]
fn parse_join_validation(
    value: &str,
) -> Result<polars::prelude::JoinValidation, TerlanPolarsError> {
    use polars::prelude::JoinValidation;
    validate_join_validation(value)?;
    Ok(match value {
        "ManyToMany" | "many_to_many" => JoinValidation::ManyToMany,
        "ManyToOne" | "many_to_one" => JoinValidation::ManyToOne,
        "OneToMany" | "one_to_many" => JoinValidation::OneToMany,
        "OneToOne" | "one_to_one" => JoinValidation::OneToOne,
        _ => unreachable!("validated join cardinality"),
    })
}

#[cfg(feature = "real-polars")]
fn parse_join_order(value: &str) -> Result<polars::prelude::MaintainOrderJoin, TerlanPolarsError> {
    use polars::prelude::MaintainOrderJoin;
    validate_join_order(value)?;
    Ok(match value {
        "Unordered" | "unordered" => MaintainOrderJoin::None,
        "PreserveLeft" | "preserve_left" => MaintainOrderJoin::Left,
        "PreserveRight" | "preserve_right" => MaintainOrderJoin::Right,
        "PreserveLeftRight" | "preserve_left_right" => MaintainOrderJoin::LeftRight,
        "PreserveRightLeft" | "preserve_right_left" => MaintainOrderJoin::RightLeft,
        _ => unreachable!("validated join order"),
    })
}

#[cfg(feature = "real-polars")]
fn parse_closed_window(value: &str) -> Result<polars::prelude::ClosedWindow, TerlanPolarsError> {
    use polars::prelude::ClosedWindow;
    match value {
        "left" => Ok(ClosedWindow::Left),
        "right" => Ok(ClosedWindow::Right),
        "both" => Ok(ClosedWindow::Both),
        "none" => Ok(ClosedWindow::None),
        _ => Err(TerlanPolarsError::new(
            "invalid_closed_window",
            format!(
                "unsupported closed-window value `{value}`; expected left, right, both, or none"
            ),
        )),
    }
}

#[cfg(feature = "real-polars")]
fn parse_window_label(value: &str) -> Result<polars::prelude::Label, TerlanPolarsError> {
    use polars::prelude::Label;
    match value {
        "left" => Ok(Label::Left),
        "right" => Ok(Label::Right),
        "data_point" => Ok(Label::DataPoint),
        _ => Err(TerlanPolarsError::new(
            "invalid_window_label",
            format!("unsupported window label `{value}`; expected left, right, or data_point"),
        )),
    }
}

/// Joins two lazy plans with independently named keys and explicit join options.
#[cfg(not(feature = "real-polars"))]
pub fn lazy_join(
    _left: &TerlanPolarsLazyFrame,
    _right: &TerlanPolarsLazyFrame,
    left_on: &[String],
    right_on: &[String],
    kind: &str,
    _suffix: &str,
    _nulls_equal: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_join_keys("lazy_join", kind, left_on, right_on)?;
    Err(unavailable_error())
}

/// Joins two lazy plans with checked cardinality and deterministic row ordering.
#[cfg(not(feature = "real-polars"))]
pub fn lazy_join_with_options(
    _left: &TerlanPolarsLazyFrame,
    _right: &TerlanPolarsLazyFrame,
    left_on: &[String],
    right_on: &[String],
    kind: &str,
    _suffix: &str,
    _nulls_equal: bool,
    validation: &str,
    order: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_join_keys("lazy_join_with_options", kind, left_on, right_on)?;
    validate_join_validation(validation)?;
    validate_join_order(order)?;
    Err(unavailable_error())
}

/// Joins two lazy plans using cross-table boolean predicates.
pub fn lazy_join_where(
    left: &TerlanPolarsLazyFrame,
    right: &TerlanPolarsLazyFrame,
    predicates: &[String],
    suffix: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    if predicates.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_join_predicates",
            "lazy_join_where requires at least one predicate",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right, suffix);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsLazyFrame {
            inner: left
                .inner
                .clone()
                .join_builder()
                .with(right.inner.clone())
                .suffix(suffix)
                .join_where(expressions::compile_many(predicates)?),
        })
    }
}

/// Performs a sorted nearest-key join without materializing either plan.
pub fn lazy_asof_join(
    left: &TerlanPolarsLazyFrame,
    right: &TerlanPolarsLazyFrame,
    left_on: &str,
    right_on: &str,
    left_by: &[String],
    right_by: &[String],
    strategy: &str,
    tolerance: &str,
    suffix: &str,
    allow_equal: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    if left_on.is_empty() || right_on.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_join_keys",
            "lazy_asof_join requires one left and one right key column",
        ));
    }
    if left_by.len() != right_by.len() {
        return Err(TerlanPolarsError::new(
            "invalid_join_keys",
            "lazy_asof_join requires the same number of left and right grouping keys",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right, strategy, tolerance, suffix, allow_equal);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{col, AsOfOptions, AsofStrategy, JoinArgs, JoinType, Scalar};
        let strategy = match strategy {
            "backward" => AsofStrategy::Backward,
            "forward" => AsofStrategy::Forward,
            "nearest" => AsofStrategy::Nearest,
            _ => {
                return Err(TerlanPolarsError::new(
                    "invalid_asof_strategy",
                    format!("unsupported as-of strategy `{strategy}`; expected backward, forward, or nearest"),
                ));
            }
        };
        let (numeric_tolerance, duration_tolerance) =
            if let Some(value) = tolerance.strip_suffix('i') {
                let value = value.parse::<i64>().map_err(|_| {
                    TerlanPolarsError::new(
                        "invalid_asof_tolerance",
                        format!("invalid integer as-of tolerance `{tolerance}`"),
                    )
                })?;
                (Some(Scalar::from(value)), None)
            } else {
                (None, (!tolerance.is_empty()).then(|| tolerance.into()))
            };
        let options = AsOfOptions {
            strategy,
            tolerance: numeric_tolerance,
            tolerance_str: duration_tolerance,
            left_by: (!left_by.is_empty())
                .then(|| left_by.iter().cloned().map(Into::into).collect()),
            right_by: (!right_by.is_empty())
                .then(|| right_by.iter().cloned().map(Into::into).collect()),
            allow_eq: allow_equal,
            ..AsOfOptions::default()
        };
        Ok(TerlanPolarsLazyFrame {
            inner: left.inner.clone().join(
                right.inner.clone(),
                [col(left_on)],
                [col(right_on)],
                JoinArgs::new(JoinType::AsOf(Box::new(options))).with_suffix(Some(suffix.into())),
            )?,
        })
    }
}

/// Builds a dynamic time/index-window aggregation plan.
pub fn lazy_dynamic_group_agg(
    plan: &TerlanPolarsLazyFrame,
    index_column: &str,
    group_by: &[String],
    aggregations: &[String],
    every: &str,
    period: &str,
    offset: &str,
    closed: &str,
    label: &str,
    include_boundaries: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    if index_column.is_empty() || aggregations.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_dynamic_group",
            "dynamic grouping requires an index column and at least one aggregation",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (
            plan,
            group_by,
            every,
            period,
            offset,
            closed,
            label,
            include_boundaries,
        );
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{col, Duration, DynamicGroupOptions};
        let options = DynamicGroupOptions {
            every: Duration::try_parse(every)?,
            period: Duration::try_parse(period)?,
            offset: Duration::try_parse(offset)?,
            closed_window: parse_closed_window(closed)?,
            label: parse_window_label(label)?,
            include_boundaries,
            ..DynamicGroupOptions::default()
        };
        Ok(TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .group_by_dynamic(
                    col(index_column),
                    expressions::compile_many(group_by)?,
                    options,
                )
                .agg(expressions::compile_many(aggregations)?),
        })
    }
}

/// Builds a rolling time/index-window aggregation plan.
pub fn lazy_rolling_group_agg(
    plan: &TerlanPolarsLazyFrame,
    index_column: &str,
    group_by: &[String],
    aggregations: &[String],
    period: &str,
    offset: &str,
    closed: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    if index_column.is_empty() || aggregations.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_rolling_group",
            "rolling grouping requires an index column and at least one aggregation",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, group_by, period, offset, closed);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{col, Duration, RollingGroupOptions};
        let options = RollingGroupOptions {
            period: Duration::try_parse(period)?,
            offset: Duration::try_parse(offset)?,
            closed_window: parse_closed_window(closed)?,
            ..RollingGroupOptions::default()
        };
        Ok(TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .rolling(
                    col(index_column),
                    expressions::compile_many(group_by)?,
                    options,
                )
                .agg(expressions::compile_many(aggregations)?),
        })
    }
}

/// Explodes list-like columns without materializing a lazy plan.
pub fn lazy_explode(
    plan: &TerlanPolarsLazyFrame,
    columns: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    lazy_explode_with_options(plan, columns, false, true)
}

/// Explodes list-like columns with explicit Polars 2.0 null and empty-list behavior.
pub fn lazy_explode_with_options(
    plan: &TerlanPolarsLazyFrame,
    columns: &[String],
    empty_as_null: bool,
    keep_nulls: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    if columns.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_explode_columns",
            "lazy_explode requires at least one column",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, empty_as_null, keep_nulls);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{cols, ExplodeOptions};
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().explode(
                cols(columns.iter().cloned()),
                ExplodeOptions {
                    empty_as_null,
                    keep_nulls,
                },
            ),
        })
    }
}

/// Converts selected wide columns into variable/value rows lazily.
pub fn lazy_unpivot(
    plan: &TerlanPolarsLazyFrame,
    on: &[String],
    index: &[String],
    variable_name: &str,
    value_name: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    if on.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_unpivot_columns",
            "lazy_unpivot requires at least one value column",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, index, variable_name, value_name);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{cols, UnpivotArgsDSL};
        Ok(TerlanPolarsLazyFrame {
            inner: plan.inner.clone().unpivot(UnpivotArgsDSL {
                on: Some(cols(on.iter().cloned())),
                index: cols(index.iter().cloned()),
                variable_name: Some(variable_name.into()),
                value_name: Some(value_name.into()),
            }),
        })
    }
}

/// Expands selected struct columns into their fields lazily.
pub fn lazy_unnest(
    plan: &TerlanPolarsLazyFrame,
    columns: &[String],
    separator: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    if columns.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_unnest_columns",
            "lazy_unnest requires at least one column",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, separator);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::cols;
        let separator = (!separator.is_empty()).then(|| separator.into());
        Ok(TerlanPolarsLazyFrame {
            inner: plan
                .inner
                .clone()
                .unnest(cols(columns.iter().cloned()), separator),
        })
    }
}

#[cfg(feature = "real-polars")]
/// Executes the lazy join operation through the Terlan Polars adapter.
pub fn lazy_join(
    left: &TerlanPolarsLazyFrame,
    right: &TerlanPolarsLazyFrame,
    left_on: &[String],
    right_on: &[String],
    kind: &str,
    suffix: &str,
    nulls_equal: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    use polars::prelude::{col, JoinArgs};
    validate_join_keys("lazy_join", kind, left_on, right_on)?;
    let mut args = JoinArgs::new(parse_join_type(kind)?).with_suffix(Some(suffix.into()));
    args.nulls_equal = nulls_equal;
    Ok(TerlanPolarsLazyFrame {
        inner: left.inner.clone().join(
            right.inner.clone(),
            left_on.iter().map(col).collect::<Vec<_>>(),
            right_on.iter().map(col).collect::<Vec<_>>(),
            args,
        )?,
    })
}

/// Joins two lazy plans while explicitly coalescing or retaining duplicate key columns.
pub fn lazy_join_with_coalesce(
    left: &TerlanPolarsLazyFrame,
    right: &TerlanPolarsLazyFrame,
    left_on: &[String],
    right_on: &[String],
    kind: &str,
    suffix: &str,
    nulls_equal: bool,
    coalesce: bool,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_join_keys("lazy_join_with_coalesce", kind, left_on, right_on)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right, suffix, nulls_equal, coalesce);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{col, JoinArgs, JoinCoalesce};
        let mut args = JoinArgs::new(parse_join_type(kind)?).with_suffix(Some(suffix.into()));
        args.nulls_equal = nulls_equal;
        args.coalesce = if coalesce {
            JoinCoalesce::CoalesceColumns
        } else {
            JoinCoalesce::KeepColumns
        };
        Ok(TerlanPolarsLazyFrame {
            inner: left.inner.clone().join(
                right.inner.clone(),
                left_on.iter().map(col).collect::<Vec<_>>(),
                right_on.iter().map(col).collect::<Vec<_>>(),
                args,
            )?,
        })
    }
}

/// Joins two lazy plans with checked cardinality and deterministic row ordering.
#[cfg(feature = "real-polars")]
pub fn lazy_join_with_options(
    left: &TerlanPolarsLazyFrame,
    right: &TerlanPolarsLazyFrame,
    left_on: &[String],
    right_on: &[String],
    kind: &str,
    suffix: &str,
    nulls_equal: bool,
    validation: &str,
    order: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    use polars::prelude::{col, JoinArgs};
    validate_join_keys("lazy_join_with_options", kind, left_on, right_on)?;
    let mut args = JoinArgs::new(parse_join_type(kind)?).with_suffix(Some(suffix.into()));
    args.nulls_equal = nulls_equal;
    args.validation = parse_join_validation(validation)?;
    args.maintain_order = parse_join_order(order)?;
    Ok(TerlanPolarsLazyFrame {
        inner: left.inner.clone().join(
            right.inner.clone(),
            left_on.iter().map(col).collect::<Vec<_>>(),
            right_on.iter().map(col).collect::<Vec<_>>(),
            args,
        )?,
    })
}

#[cfg(feature = "real-polars")]
/// Executes the lazy left join operation through the Terlan Polars adapter.
pub fn lazy_left_join(
    left: &TerlanPolarsLazyFrame,
    right: &TerlanPolarsLazyFrame,
    keys: &[String],
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    use polars::prelude::{col, JoinArgs, JoinType};
    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_join_keys",
            "lazy_left_join requires at least one key column",
        ));
    }
    let on = keys.iter().map(col).collect::<Vec<_>>();
    Ok(TerlanPolarsLazyFrame {
        inner: left.inner.clone().join(
            right.inner.clone(),
            on.clone(),
            on,
            JoinArgs::new(JoinType::Left),
        )?,
    })
}

/// Executes the lazy describe plan operation through the Terlan Polars adapter.
pub fn lazy_describe_plan(
    plan: &TerlanPolarsLazyFrame,
    optimized: bool,
) -> Result<String, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, optimized);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(plan.inner.explain(optimized)?)
    }
}

/// Returns a Graphviz DOT rendering of a logical or optimized lazy plan.
pub fn lazy_describe_plan_dot(
    plan: &TerlanPolarsLazyFrame,
    optimized: bool,
) -> Result<String, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, optimized);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        let ir = if optimized {
            plan.inner.clone().to_alp_optimized()?
        } else {
            plan.inner.clone().to_alp()?
        };
        Ok(format!("{}", ir.display_dot()))
    }
}

/// Renders the logical or optimized query plan as a tree.
pub fn lazy_describe_plan_tree(
    plan: &TerlanPolarsLazyFrame,
    optimized: bool,
) -> Result<String, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (plan, optimized);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        if optimized {
            Ok(plan.inner.describe_optimized_plan_tree()?)
        } else {
            Ok(plan.inner.describe_plan_tree()?)
        }
    }
}

/// Executes a lazy query and returns Polars' per-node timing table. The query
/// result is intentionally discarded because the native boundary returns one
/// owned resource per call.
#[cfg(not(feature = "real-polars"))]
pub fn lazy_profile_plan(
    _plan: &TerlanPolarsLazyFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the lazy profile plan operation through the Terlan Polars adapter.
pub fn lazy_profile_plan(
    plan: &TerlanPolarsLazyFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    let started = std::time::Instant::now();
    plan.inner.clone().describe_optimized_plan()?;
    let optimized = started.elapsed().as_micros() as u64;
    plan.inner.clone().collect()?;
    let finished = started.elapsed().as_micros() as u64;
    let profile = polars::df!(
        "node" => ["optimization", "query"],
        "start" => [0_u64, optimized],
        "end" => [optimized, finished],
    )?;
    Ok(TerlanPolarsDataFrame { inner: profile })
}

/// Executes a lazy query and returns its result and profile as a two-frame set.
#[cfg(not(feature = "real-polars"))]
pub fn lazy_profile(
    _plan: &TerlanPolarsLazyFrame,
) -> Result<TerlanPolarsFrameSet, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes a lazy query and returns its result and profile as a two-frame set.
pub fn lazy_profile(
    plan: &TerlanPolarsLazyFrame,
) -> Result<TerlanPolarsFrameSet, TerlanPolarsError> {
    let started = std::time::Instant::now();
    plan.inner.clone().describe_optimized_plan()?;
    let optimized = started.elapsed().as_micros() as u64;
    let result = plan.inner.clone().collect()?;
    let finished = started.elapsed().as_micros() as u64;
    let profile = polars::df!(
        "node" => ["optimization", "query"],
        "start" => [0_u64, optimized],
        "end" => [optimized, finished],
    )?;
    Ok(TerlanPolarsFrameSet {
        frames: vec![
            TerlanPolarsDataFrame { inner: result },
            TerlanPolarsDataFrame { inner: profile },
        ],
    })
}

/// Reports unavailable collection for a placeholder lazy plan.
#[cfg(not(feature = "real-polars"))]
pub fn collect_lazy(
    _plan: &TerlanPolarsLazyFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(not(feature = "real-polars"))]
pub fn collect_lazy_streaming(
    _plan: &TerlanPolarsLazyFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Reports unavailable in-memory collection for a placeholder lazy plan.
#[cfg(not(feature = "real-polars"))]
pub fn collect_lazy_in_memory(
    _plan: &TerlanPolarsLazyFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the collect lazy streaming operation through the Terlan Polars adapter.
pub fn collect_lazy_streaming(
    plan: &TerlanPolarsLazyFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_polars_plan(plan.inner.clone().with_streaming(true))
}

/// Executes a real lazy plan with the Polars 2.0 default streaming engine.
#[cfg(feature = "real-polars")]
pub fn collect_lazy(
    plan: &TerlanPolarsLazyFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    match collect_lazy_streaming(plan) {
        Err(error) if error.code() == "native_execution_panic" => collect_lazy_in_memory(plan),
        result => result,
    }
}

/// Executes a real lazy plan with the legacy in-memory engine.
#[cfg(feature = "real-polars")]
pub fn collect_lazy_in_memory(
    plan: &TerlanPolarsLazyFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_polars_plan(plan.inner.clone())
}

/// Materializes one Polars plan while containing upstream panic paths.
#[cfg(feature = "real-polars")]
fn collect_polars_plan(
    plan: polars::prelude::LazyFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plan.collect())) {
        Ok(result) => Ok(TerlanPolarsDataFrame { inner: result? }),
        Err(_) => Err(TerlanPolarsError::new(
            "native_execution_panic",
            "Polars aborted query execution; the adapter contained the failure",
        )),
    }
}

/// Groups a real Polars DataFrame by key columns and counts rows.
#[cfg(feature = "real-polars")]
pub fn group_count(
    df: &TerlanPolarsDataFrame,
    keys: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{col, len, Expr, IntoLazy};

    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_group_keys",
            "group_count requires at least one key column",
        ));
    }
    let key_exprs = keys.iter().map(col).collect::<Vec<Expr>>();
    let inner = df
        .inner
        .clone()
        .lazy()
        .group_by_stable(key_exprs)
        .agg([len().alias("count")])
        .collect()?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Sorts a real Polars DataFrame by one column.
#[cfg(feature = "real-polars")]
pub fn sort_by(
    df: &TerlanPolarsDataFrame,
    column: &str,
    descending: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::SortMultipleOptions;

    let inner = df.inner.sort(
        [column],
        SortMultipleOptions::default().with_order_descending(descending),
    )?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Filters a real Polars DataFrame by scalar equality.
#[cfg(feature = "real-polars")]
pub fn filter_eq(
    df: &TerlanPolarsDataFrame,
    column: &str,
    value: &TerlanPolarsScalar,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{col, lit, IntoLazy};

    let literal = match value {
        TerlanPolarsScalar::String(value) => lit(value.clone()),
        TerlanPolarsScalar::Int(value) => lit(*value),
        TerlanPolarsScalar::Float(value) => lit(*value),
        TerlanPolarsScalar::Bool(value) => lit(*value),
    };
    let inner = df
        .inner
        .clone()
        .lazy()
        .filter(col(column).eq(literal))
        .collect()?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Selects DataFrame columns.
///
/// Inputs:
/// - `df`: native DataFrame handle.
/// - `columns`: UTF-8 column names supplied by Terlan.
///
/// Outputs:
/// - `Ok(TerlanPolarsDataFrame)` once the real Polars adapter is linked.
/// - `Err(TerlanPolarsError)` in the current stub implementation.
///
/// Transformation:
/// - Reserves the Rust function boundary for the Terlan `select` receiver
///   method without exposing Polars internals to Terlan source.
#[cfg(not(feature = "real-polars"))]
pub fn select(
    _df: &TerlanPolarsDataFrame,
    _columns: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Selects DataFrame columns with the real Polars feature.
///
/// Inputs:
/// - `df`: native DataFrame handle.
/// - `columns`: UTF-8 column names supplied by Terlan.
///
/// Outputs:
/// - `Ok(TerlanPolarsDataFrame)` containing the projected DataFrame.
/// - `Err(TerlanPolarsError)` when Polars rejects the projection.
///
/// Transformation:
/// - Copies Terlan-owned column names into the Polars selection API and wraps
///   the projected DataFrame behind the adapter type.
#[cfg(feature = "real-polars")]
pub fn select(
    df: &TerlanPolarsDataFrame,
    columns: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    let selected = df.inner.select(columns.iter().map(String::as_str))?;

    Ok(TerlanPolarsDataFrame { inner: selected })
}

/// Returns the first rows of a DataFrame.
///
/// Inputs:
/// - `df`: native DataFrame handle.
/// - `limit`: requested row count.
///
/// Outputs:
/// - `Ok(TerlanPolarsDataFrame)` once the real Polars adapter is linked.
/// - `Err(TerlanPolarsError)` in the current stub implementation.
///
/// Transformation:
/// - Reserves the Rust function boundary for the Terlan `head` receiver method
///   without exposing Polars internals to Terlan source.
#[cfg(not(feature = "real-polars"))]
pub fn head(
    _df: &TerlanPolarsDataFrame,
    _limit: usize,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Returns the first rows of a DataFrame with the real Polars feature.
///
/// Inputs:
/// - `df`: native DataFrame handle.
/// - `limit`: requested row count.
///
/// Outputs:
/// - `Ok(TerlanPolarsDataFrame)` containing up to `limit` rows.
///
/// Transformation:
/// - Delegates to Polars `head`, clones the resulting frame, and keeps the
///   result behind the adapter type.
#[cfg(feature = "real-polars")]
pub fn head(
    df: &TerlanPolarsDataFrame,
    limit: usize,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Ok(TerlanPolarsDataFrame {
        inner: df.inner.head(Some(limit)),
    })
}

/// Returns at most the final requested number of rows.
pub fn tail(
    df: &TerlanPolarsDataFrame,
    limit: usize,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, limit);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsDataFrame {
            inner: df.inner.tail(Some(limit)),
        })
    }
}

/// Removes every row while retaining names and data types.
pub fn clear_rows(df: &TerlanPolarsDataFrame) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = df;
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsDataFrame {
            inner: df.inner.clear(),
        })
    }
}

/// Consolidates every column into a single contiguous chunk.
pub fn rechunk(df: &TerlanPolarsDataFrame) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = df;
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        let mut inner = df.inner.clone();
        inner.rechunk_mut_par();
        Ok(TerlanPolarsDataFrame { inner })
    }
}

/// Reduces every column to its non-null sum.
pub fn column_sums(df: &TerlanPolarsDataFrame) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_sums(&to_lazy(df)))
}

/// Reduces every numeric column to its mean.
pub fn column_means(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_means(&to_lazy(df)))
}

/// Reduces every numeric column to its median.
pub fn column_medians(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_medians(&to_lazy(df)))
}

/// Reduces every column to its minimum value.
pub fn column_minima(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_minima(&to_lazy(df)))
}

/// Reduces every column to its maximum value.
pub fn column_maxima(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_maxima(&to_lazy(df)))
}

/// Reduces every numeric column to its product.
pub fn column_products(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_products(&to_lazy(df)))
}

/// Reduces every numeric column to its variance.
pub fn column_variances(
    df: &TerlanPolarsDataFrame,
    ddof: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_variances(&to_lazy(df), ddof)?)
}

/// Reduces every numeric column to its standard deviation.
pub fn column_stddevs(
    df: &TerlanPolarsDataFrame,
    ddof: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_stddevs(&to_lazy(df), ddof)?)
}

/// Reduces every numeric column to a configurable quantile.
pub fn column_quantiles(
    df: &TerlanPolarsDataFrame,
    probability: f64,
    method: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_quantiles(&to_lazy(df), probability, method)?)
}

/// Counts non-null values in every column.
pub fn column_non_null_counts(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_non_null_counts(&to_lazy(df)))
}

/// Counts all values, including nulls, in every column.
pub fn column_lengths(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_lengths(&to_lazy(df)))
}

/// Counts exact distinct values in every column.
pub fn column_unique_counts(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_unique_counts(&to_lazy(df)))
}

/// Estimates distinct-value cardinality in every column.
pub fn column_approx_unique_counts(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_column_approx_unique_counts(&to_lazy(df)))
}

/// Drops duplicate rows using an optional key subset.
pub fn unique_rows(
    df: &TerlanPolarsDataFrame,
    subset: &[String],
    keep: &str,
    maintain_order: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    validate_unique_keep(keep)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, subset, maintain_order);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::PlSmallStr;
        let subset = (!subset.is_empty()).then(|| {
            subset
                .iter()
                .map(|name| PlSmallStr::from_str(name))
                .collect()
        });
        let keep = parse_unique_keep(keep)?;
        let inner = df.inner.unique_impl(maintain_order, subset, keep, None)?;
        Ok(TerlanPolarsDataFrame { inner })
    }
}

/// Drops rows containing nulls in an optional column subset.
pub fn drop_null_rows(
    df: &TerlanPolarsDataFrame,
    subset: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, subset);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        let subset = (!subset.is_empty()).then_some(subset);
        Ok(TerlanPolarsDataFrame {
            inner: df.inner.drop_nulls(subset)?,
        })
    }
}

/// Selects a bounded row range from a materialized frame.
pub fn slice_rows(
    df: &TerlanPolarsDataFrame,
    offset: i64,
    length: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    let length = validate_slice_length(length)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, offset, length);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsDataFrame {
            inner: df.inner.slice(offset, length),
        })
    }
}

/// Gathers materialized rows by unsigned indices without using unchecked
/// Polars indexing. Out-of-bounds indices either fail with a stable adapter
/// error or produce all-null rows.
pub fn gather_rows(
    df: &TerlanPolarsDataFrame,
    indices: &[i64],
    null_on_out_of_bounds: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    let indices = validate_gather_indices(indices)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, indices, null_on_out_of_bounds);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{IdxCa, NamedFrom};
        let height = df.inner.height();
        if !null_on_out_of_bounds {
            if let Some((position, index)) = indices
                .iter()
                .enumerate()
                .find(|(_, index)| **index as usize >= height)
            {
                return Err(TerlanPolarsError::new(
                    "row_index_out_of_bounds",
                    format!(
                        "row index at position {position} is out of bounds: index {index}, height {height}"
                    ),
                ));
            }
        }
        let indices = indices
            .into_iter()
            .map(|index| ((index as usize) < height).then_some(index))
            .collect::<Vec<_>>();
        Ok(TerlanPolarsDataFrame {
            inner: df.inner.take(&IdxCa::new("row_index".into(), indices))?,
        })
    }
}

/// Selects every `step`th materialized row beginning at `offset`.
pub fn gather_rows_every(
    df: &TerlanPolarsDataFrame,
    step: i64,
    offset: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    let step = usize::try_from(step).map_err(|_| {
        TerlanPolarsError::new("invalid_row_step", "row gather step must be positive")
    })?;
    if step == 0 {
        return Err(TerlanPolarsError::new(
            "invalid_row_step",
            "row gather step must be positive",
        ));
    }
    let offset = usize::try_from(offset).map_err(|_| {
        TerlanPolarsError::new("invalid_row_offset", "row gather offset cannot be negative")
    })?;
    let indices = (offset..height(df))
        .step_by(step)
        .map(|index| index as i64)
        .collect::<Vec<_>>();
    gather_rows(df, &indices, false)
}

/// Renames materialized columns simultaneously.
pub fn rename_columns(
    df: &TerlanPolarsDataFrame,
    existing: &[String],
    new: &[String],
    strict: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    validate_rename_columns("rename_columns", existing, new)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, strict);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoLazy;
        Ok(TerlanPolarsDataFrame {
            inner: df
                .inner
                .clone()
                .lazy()
                .rename(
                    existing.iter().map(String::as_str),
                    new.iter().map(String::as_str),
                    strict,
                )
                .collect()?,
        })
    }
}

/// Removes named columns from a materialized frame.
pub fn drop_columns(
    df: &TerlanPolarsDataFrame,
    columns: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    validate_drop_columns("drop_columns", columns)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = df;
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{cols, IntoLazy};
        Ok(TerlanPolarsDataFrame {
            inner: df
                .inner
                .clone()
                .lazy()
                .drop(cols(columns.iter().cloned()))
                .collect()?,
        })
    }
}

/// Removes named columns and optionally ignores names absent from the schema.
pub fn drop_columns_with_options(
    df: &TerlanPolarsDataFrame,
    requested: &[String],
    strict: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if strict {
        return drop_columns(df, requested);
    }
    validate_drop_columns("drop_columns", requested)?;
    let existing = columns(df)
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    let present = requested
        .iter()
        .filter(|name| existing.contains(name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if present.is_empty() {
        return Ok(df.clone());
    }
    drop_columns(df, &present)
}

/// Reverses all rows in a materialized frame.
pub fn reverse_rows(
    df: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = df;
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsDataFrame {
            inner: df.inner.reverse(),
        })
    }
}

/// Prepends an unsigned row index to a materialized frame.
pub fn with_row_index(
    df: &TerlanPolarsDataFrame,
    name: &str,
    offset: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    let offset = validate_row_index(name, offset)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, offset);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsDataFrame {
            inner: df
                .inner
                .with_row_index(name.into(), Some(offset as polars::prelude::IdxSize))?,
        })
    }
}

/// Fills null values throughout a materialized frame with one expression.
pub fn fill_null_values(
    df: &TerlanPolarsDataFrame,
    value: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, value);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoLazy;
        let plan = lazy_fill_null_values(
            &TerlanPolarsLazyFrame {
                inner: df.inner.clone().lazy(),
            },
            value,
        )?;
        Ok(TerlanPolarsDataFrame {
            inner: plan.inner.collect()?,
        })
    }
}

/// Fills null values throughout a materialized frame with a Polars strategy.
pub fn fill_null_with_strategy(
    df: &TerlanPolarsDataFrame,
    strategy: &str,
    limit: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    let limit = if limit < 0 {
        None
    } else {
        Some(
            usize::try_from(limit)
                .map_err(|_| TerlanPolarsError::new("invalid_limit", "fill limit is too large"))?,
        )
    };
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, strategy, limit);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsDataFrame {
            inner: df
                .inner
                .fill_null(series::fill_strategy(strategy, limit)?)?,
        })
    }
}

/// Fills floating-point NaN values throughout a materialized frame.
pub fn fill_nan_values(
    df: &TerlanPolarsDataFrame,
    value: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, value);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoLazy;
        let plan = lazy_fill_nan_values(
            &TerlanPolarsLazyFrame {
                inner: df.inner.clone().lazy(),
            },
            value,
        )?;
        Ok(TerlanPolarsDataFrame {
            inner: plan.inner.collect()?,
        })
    }
}

/// Removes rows containing NaN values in an optional column subset.
pub fn drop_nan_rows(
    df: &TerlanPolarsDataFrame,
    subset: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, subset);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoLazy;
        let plan = lazy_drop_nan_rows(
            &TerlanPolarsLazyFrame {
                inner: df.inner.clone().lazy(),
            },
            subset,
        )?;
        Ok(TerlanPolarsDataFrame {
            inner: plan.inner.collect()?,
        })
    }
}

/// Produces one row containing every column's null count.
pub fn null_counts(df: &TerlanPolarsDataFrame) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = df;
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsDataFrame {
            inner: df.inner.null_count(),
        })
    }
}

/// Shifts every column by a signed row count and fills the gap with nulls.
pub fn shift_rows(
    df: &TerlanPolarsDataFrame,
    periods: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, periods);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(TerlanPolarsDataFrame {
            inner: df.inner.shift(periods),
        })
    }
}

/// Shifts every column and fills the resulting gap from an expression.
pub fn shift_and_fill_rows(
    df: &TerlanPolarsDataFrame,
    periods: i64,
    fill_value: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, periods, fill_value);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        collect_lazy(&lazy_shift_and_fill_rows(
            &to_lazy(df),
            periods,
            fill_value,
        )?)
    }
}

/// Selects one or more compiled Polars expressions.
#[cfg(not(feature = "real-polars"))]
pub fn select_exprs(
    _df: &TerlanPolarsDataFrame,
    _expressions: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the select exprs operation through the Terlan Polars adapter.
pub fn select_exprs(
    df: &TerlanPolarsDataFrame,
    expressions: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::IntoLazy;
    collect_polars_plan(
        df.inner
            .clone()
            .lazy()
            .select(expressions::compile_many(expressions)?),
    )
}

/// Selects expressions using Polars' sequential execution strategy.
pub fn select_exprs_sequential(
    df: &TerlanPolarsDataFrame,
    expressions: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_select_exprs_sequential(&to_lazy(df), expressions)?)
}

/// Adds or replaces columns from compiled expressions.
#[cfg(not(feature = "real-polars"))]
pub fn with_columns_exprs(
    _df: &TerlanPolarsDataFrame,
    _expressions: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the with columns exprs operation through the Terlan Polars adapter.
pub fn with_columns_exprs(
    df: &TerlanPolarsDataFrame,
    expressions: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::IntoLazy;
    collect_polars_plan(
        df.inner
            .clone()
            .lazy()
            .with_columns(expressions::compile_many(expressions)?),
    )
}

/// Adds or replaces columns using Polars' sequential execution strategy.
pub fn with_columns_sequential(
    df: &TerlanPolarsDataFrame,
    expressions: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_with_columns_sequential(&to_lazy(df), expressions)?)
}

/// Filters rows with a compiled predicate expression.
#[cfg(not(feature = "real-polars"))]
pub fn filter_expr(
    _df: &TerlanPolarsDataFrame,
    _predicate: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Filters rows by the conjunction of one or more predicate expressions.
pub fn filter_all(
    df: &TerlanPolarsDataFrame,
    predicates: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if predicates.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "filter_all requires at least one predicate",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = df;
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoLazy;
        let predicate = expressions::compile_many(predicates)?
            .into_iter()
            .reduce(|left, right| left.and(right))
            .expect("non-empty predicates validated above");
        Ok(TerlanPolarsDataFrame {
            inner: df.inner.clone().lazy().filter(predicate).collect()?,
        })
    }
}

/// Filters rows by named equality constraints.
///
/// `columns` and `values` are parallel lists. Each value is an expression so
/// callers can supply mixed scalar types through `lit` while retaining a
/// homogeneous Terlan list at the native boundary.
pub fn filter_constraints(
    df: &TerlanPolarsDataFrame,
    columns: &[String],
    values: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if columns.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "filter_constraints requires at least one column and value",
        ));
    }
    if columns.len() != values.len() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "filter_constraints requires equal column and value counts",
        ));
    }
    let predicates = columns
        .iter()
        .zip(values)
        .map(|(column, value)| expressions::expr_eq(&expressions::expr_col(column)?, value))
        .collect::<Result<Vec<_>, _>>()?;
    filter_all(df, &predicates)
}

#[cfg(feature = "real-polars")]
/// Executes the filter expr operation through the Terlan Polars adapter.
pub fn filter_expr(
    df: &TerlanPolarsDataFrame,
    predicate: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::IntoLazy;
    let mut expressions = expressions::compile_many(&[predicate.to_string()])?;
    let inner = df
        .inner
        .clone()
        .lazy()
        .filter(expressions.remove(0))
        .collect()?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Removes rows where a predicate is true while retaining false and null rows.
pub fn remove_where(
    df: &TerlanPolarsDataFrame,
    predicate: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    collect_lazy(&lazy_remove_where(&to_lazy(df), predicate)?)
}

/// Groups by expressions and applies aggregation expressions.
#[cfg(not(feature = "real-polars"))]
pub fn group_agg(
    _df: &TerlanPolarsDataFrame,
    keys: &[String],
    _aggregations: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_group_keys",
            "group_agg requires at least one key expression",
        ));
    }
    Err(unavailable_error())
}

#[cfg(feature = "real-polars")]
/// Executes the group agg operation through the Terlan Polars adapter.
pub fn group_agg(
    df: &TerlanPolarsDataFrame,
    keys: &[String],
    aggregations: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::IntoLazy;
    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_group_keys",
            "group_agg requires at least one key expression",
        ));
    }
    let inner = df
        .inner
        .clone()
        .lazy()
        .group_by_stable(expressions::compile_many(keys)?)
        .agg(expressions::compile_many(aggregations)?)
        .collect()?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Left joins two DataFrames on identically named key columns.
#[cfg(not(feature = "real-polars"))]
pub fn left_join(
    _df: &TerlanPolarsDataFrame,
    _right: &TerlanPolarsDataFrame,
    keys: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_join_keys",
            "left_join requires at least one key column",
        ));
    }
    Err(unavailable_error())
}

/// Joins two DataFrames with independently named keys and explicit join options.
#[cfg(not(feature = "real-polars"))]
pub fn join(
    _df: &TerlanPolarsDataFrame,
    _right: &TerlanPolarsDataFrame,
    left_on: &[String],
    right_on: &[String],
    kind: &str,
    _suffix: &str,
    _nulls_equal: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    validate_join_keys("join", kind, left_on, right_on)?;
    Err(unavailable_error())
}

/// Materializes an equi join with checked cardinality and row ordering.
#[cfg(not(feature = "real-polars"))]
pub fn join_with_options(
    _df: &TerlanPolarsDataFrame,
    _right: &TerlanPolarsDataFrame,
    left_on: &[String],
    right_on: &[String],
    kind: &str,
    _suffix: &str,
    _nulls_equal: bool,
    validation: &str,
    order: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    validate_join_keys("join_with_options", kind, left_on, right_on)?;
    validate_join_validation(validation)?;
    validate_join_order(order)?;
    Err(unavailable_error())
}

/// Performs a materialized as-of join over sorted key columns.
pub fn asof_join(
    left: &TerlanPolarsDataFrame,
    right: &TerlanPolarsDataFrame,
    left_on: &str,
    right_on: &str,
    left_by: &[String],
    right_by: &[String],
    strategy: &str,
    tolerance: &str,
    suffix: &str,
    allow_equal: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (
            left,
            right,
            left_on,
            right_on,
            left_by,
            right_by,
            strategy,
            tolerance,
            suffix,
            allow_equal,
        );
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoLazy;
        let plan = lazy_asof_join(
            &TerlanPolarsLazyFrame {
                inner: left.inner.clone().lazy(),
            },
            &TerlanPolarsLazyFrame {
                inner: right.inner.clone().lazy(),
            },
            left_on,
            right_on,
            left_by,
            right_by,
            strategy,
            tolerance,
            suffix,
            allow_equal,
        )?;
        Ok(TerlanPolarsDataFrame {
            inner: plan.inner.collect()?,
        })
    }
}

/// Materializes a join using cross-table boolean predicates.
pub fn join_where(
    left: &TerlanPolarsDataFrame,
    right: &TerlanPolarsDataFrame,
    predicates: &[String],
    suffix: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right, suffix);
        if predicates.is_empty() {
            return Err(TerlanPolarsError::new(
                "invalid_join_predicates",
                "join_where requires at least one predicate",
            ));
        }
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoLazy;
        let joined = lazy_join_where(
            &TerlanPolarsLazyFrame {
                inner: left.inner.clone().lazy(),
            },
            &TerlanPolarsLazyFrame {
                inner: right.inner.clone().lazy(),
            },
            predicates,
            suffix,
        )?;
        Ok(TerlanPolarsDataFrame {
            inner: joined.inner.collect()?,
        })
    }
}

#[cfg(feature = "real-polars")]
/// Executes the join operation through the Terlan Polars adapter.
pub fn join(
    df: &TerlanPolarsDataFrame,
    right: &TerlanPolarsDataFrame,
    left_on: &[String],
    right_on: &[String],
    kind: &str,
    suffix: &str,
    nulls_equal: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{col, IntoLazy, JoinArgs};
    validate_join_keys("join", kind, left_on, right_on)?;
    let mut args = JoinArgs::new(parse_join_type(kind)?).with_suffix(Some(suffix.into()));
    args.nulls_equal = nulls_equal;
    let inner = df
        .inner
        .clone()
        .lazy()
        .join(
            right.inner.clone().lazy(),
            left_on.iter().map(col).collect::<Vec<_>>(),
            right_on.iter().map(col).collect::<Vec<_>>(),
            args,
        )?
        .collect()?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Materializes an equi join with checked cardinality and row ordering.
#[cfg(feature = "real-polars")]
pub fn join_with_options(
    df: &TerlanPolarsDataFrame,
    right: &TerlanPolarsDataFrame,
    left_on: &[String],
    right_on: &[String],
    kind: &str,
    suffix: &str,
    nulls_equal: bool,
    validation: &str,
    order: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::IntoLazy;
    let joined = lazy_join_with_options(
        &TerlanPolarsLazyFrame {
            inner: df.inner.clone().lazy(),
        },
        &TerlanPolarsLazyFrame {
            inner: right.inner.clone().lazy(),
        },
        left_on,
        right_on,
        kind,
        suffix,
        nulls_equal,
        validation,
        order,
    )?;
    Ok(TerlanPolarsDataFrame {
        inner: joined.inner.collect()?,
    })
}

/// Materializes a join with an explicit key-column coalescing policy.
pub fn join_with_coalesce(
    left: &TerlanPolarsDataFrame,
    right: &TerlanPolarsDataFrame,
    left_on: &[String],
    right_on: &[String],
    kind: &str,
    suffix: &str,
    nulls_equal: bool,
    coalesce: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (
            left,
            right,
            left_on,
            right_on,
            kind,
            suffix,
            nulls_equal,
            coalesce,
        );
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoLazy;
        let joined = lazy_join_with_coalesce(
            &TerlanPolarsLazyFrame {
                inner: left.inner.clone().lazy(),
            },
            &TerlanPolarsLazyFrame {
                inner: right.inner.clone().lazy(),
            },
            left_on,
            right_on,
            kind,
            suffix,
            nulls_equal,
            coalesce,
        )?;
        Ok(TerlanPolarsDataFrame {
            inner: joined.inner.collect()?,
        })
    }
}

#[cfg(feature = "real-polars")]
/// Executes the left join operation through the Terlan Polars adapter.
pub fn left_join(
    df: &TerlanPolarsDataFrame,
    right: &TerlanPolarsDataFrame,
    keys: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{col, IntoLazy, JoinArgs, JoinType};
    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_join_keys",
            "left_join requires at least one key column",
        ));
    }
    let on = keys.iter().map(col).collect::<Vec<_>>();
    let inner = df
        .inner
        .clone()
        .lazy()
        .join(
            right.inner.clone().lazy(),
            on.clone(),
            on,
            JoinArgs::new(JoinType::Left),
        )?
        .collect()?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Vertically concatenates two schema-compatible DataFrames.
#[cfg(not(feature = "real-polars"))]
pub fn concat_vertical(
    _df: &TerlanPolarsDataFrame,
    _other: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Vertically concatenates two frames after coercing columns to common supertypes.
#[cfg(not(feature = "real-polars"))]
pub fn concat_vertical_relaxed(
    _df: &TerlanPolarsDataFrame,
    _other: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Horizontally concatenates two DataFrames, optionally requiring equal heights.
#[cfg(not(feature = "real-polars"))]
pub fn concat_horizontal(
    _df: &TerlanPolarsDataFrame,
    _other: &TerlanPolarsDataFrame,
    _strict: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Horizontally concatenates frames and extends shorter inputs with nulls.
pub fn concat_horizontal_extend(
    df: &TerlanPolarsDataFrame,
    other: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    concat_horizontal(df, other, false)
}

#[cfg(feature = "real-polars")]
/// Executes the concat horizontal operation through the Terlan Polars adapter.
pub fn concat_horizontal(
    df: &TerlanPolarsDataFrame,
    other: &TerlanPolarsDataFrame,
    strict: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{concat_lf_horizontal, HConcatOptions, IntoLazy};
    let inner = concat_lf_horizontal(
        [df.inner.clone().lazy(), other.inner.clone().lazy()],
        HConcatOptions {
            strict,
            ..HConcatOptions::default()
        },
    )?
    .collect()?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Inserts missing regularly spaced temporal/index rows, optionally within groups.
pub fn upsample(
    df: &TerlanPolarsDataFrame,
    time_column: &str,
    every: &str,
    group_by: &[String],
    maintain_order: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if time_column.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_dynamic_group",
            "upsample time column cannot be empty",
        ));
    }
    if every.trim().is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_dynamic_group",
            "upsample interval cannot be empty",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, group_by, maintain_order);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{Duration, PlSmallStr, PolarsUpsample};
        let every = Duration::try_parse(every).map_err(|error| {
            TerlanPolarsError::new(
                "invalid_dynamic_group",
                format!("invalid upsample interval: {error}"),
            )
        })?;
        let group_by = group_by
            .iter()
            .map(|value| PlSmallStr::from_string(value.clone()))
            .collect::<Vec<_>>();
        let inner = if maintain_order {
            df.inner.upsample_stable(group_by, time_column, every)?
        } else {
            df.inner.upsample(group_by, time_column, every)?
        };
        Ok(TerlanPolarsDataFrame { inner })
    }
}

/// Updates existing columns from matching rows in another frame.
pub fn update(
    df: &TerlanPolarsDataFrame,
    other: &TerlanPolarsDataFrame,
    on: &[String],
    how: &str,
    include_nulls: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if !matches!(how, "left" | "inner" | "full" | "outer") {
        return Err(TerlanPolarsError::new(
            "invalid_join_type",
            "update join type must be left, inner, or full",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, other, on, include_nulls);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{
            col, lit, IntoLazy, JoinArgs, JoinCoalesce, JoinType, MaintainOrderJoin, PlSmallStr,
        };

        let left_columns = df
            .inner
            .get_column_names_owned()
            .into_iter()
            .map(|name| name.to_string())
            .collect::<Vec<_>>();
        let right_columns = other
            .inner
            .get_column_names_owned()
            .into_iter()
            .map(|name| name.to_string())
            .collect::<Vec<_>>();
        let mut occupied = left_columns
            .iter()
            .chain(right_columns.iter())
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        let mut unique_name = |base: &str| {
            let mut candidate = base.to_string();
            let mut suffix = 0_usize;
            while occupied.contains(&candidate) {
                suffix += 1;
                candidate = format!("{base}_{suffix}");
            }
            occupied.insert(candidate.clone());
            candidate
        };

        let row_index = on.is_empty().then(|| unique_name("__terlan_update_row"));
        let keys = row_index
            .as_ref()
            .map(|name| vec![name.clone()])
            .unwrap_or_else(|| on.to_vec());
        for key in &keys {
            if row_index.is_none() && (!left_columns.contains(key) || !right_columns.contains(key))
            {
                return Err(TerlanPolarsError::new(
                    "missing_column",
                    format!("update key column `{key}` must exist in both DataFrames"),
                ));
            }
        }

        let left = if let Some(name) = &row_index {
            df.inner
                .with_row_index(PlSmallStr::from_string(name.clone()), None)?
        } else {
            df.inner.clone()
        };
        let right = if let Some(name) = &row_index {
            other
                .inner
                .with_row_index(PlSmallStr::from_string(name.clone()), None)?
        } else {
            other.inner.clone()
        };

        let marker = unique_name("__terlan_update_match");
        let mut updates = Vec::new();
        let mut right_projection = keys.iter().map(col).collect::<Vec<_>>();
        for name in &left_columns {
            if !keys.contains(name) && right_columns.contains(name) {
                let temporary = unique_name(&format!("__terlan_update_{name}"));
                right_projection.push(col(name).alias(&temporary));
                updates.push((name.clone(), temporary));
            }
        }
        right_projection.push(lit(true).alias(&marker));

        let join_type = match how {
            "left" => JoinType::Left,
            "inner" => JoinType::Inner,
            "full" | "outer" => JoinType::Full,
            _ => unreachable!(),
        };
        let args = JoinArgs::new(join_type)
            .with_coalesce(JoinCoalesce::CoalesceColumns)
            // Python Polars 2.0 also defaults `update` to left order even
            // though ordinary lazy joins no longer preserve it by default.
            .with_maintain_order(MaintainOrderJoin::Left);
        let joined = left.lazy().join(
            right.lazy().select(right_projection),
            keys.iter().map(col).collect::<Vec<_>>(),
            keys.iter().map(col).collect::<Vec<_>>(),
            args,
        );

        let output = left_columns
            .iter()
            .map(|name| {
                updates
                    .iter()
                    .find(|(column, _)| column == name)
                    .map(|(_, temporary)| {
                        let matched = col(&marker).is_not_null();
                        let replace = if include_nulls {
                            matched
                        } else {
                            matched.and(col(temporary).is_not_null())
                        };
                        polars::lazy::dsl::when(replace)
                            .then(col(temporary))
                            .otherwise(col(name))
                            .alias(name)
                    })
                    .unwrap_or_else(|| col(name))
            })
            .collect::<Vec<_>>();
        Ok(TerlanPolarsDataFrame {
            inner: joined?.select(output).collect()?,
        })
    }
}

/// Applies a declarative projection independently to each materialized group.
pub fn map_groups(
    df: &TerlanPolarsDataFrame,
    keys: &[String],
    expressions: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if keys.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_group_keys",
            "map_groups requires at least one key column",
        ));
    }
    if expressions.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_expression",
            "map_groups requires at least one output expression",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = df;
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoLazy;
        let expressions = expressions::compile_many(expressions)?;
        let inner = df
            .inner
            .group_by_stable(keys)?
            .apply(move |group| group.lazy().select(expressions.clone()).collect())?;
        Ok(TerlanPolarsDataFrame { inner })
    }
}

/// Diagonally concatenates two DataFrames and null-fills absent columns.
#[cfg(not(feature = "real-polars"))]
pub fn concat_diagonal(
    _df: &TerlanPolarsDataFrame,
    _other: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(unavailable_error())
}

/// Explodes one or more list-like DataFrame columns.
pub fn explode_columns(
    df: &TerlanPolarsDataFrame,
    columns: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    explode_columns_with_options(df, columns, false, true)
}

/// Explodes list-like columns with explicit Polars 2.0 null and empty-list behavior.
pub fn explode_columns_with_options(
    df: &TerlanPolarsDataFrame,
    columns: &[String],
    empty_as_null: bool,
    keep_nulls: bool,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if columns.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_explode_columns",
            "explode_columns requires at least one column",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, empty_as_null, keep_nulls);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        let inner = df.inner.explode(
            columns.iter().map(String::as_str),
            polars::prelude::ExplodeOptions {
                empty_as_null,
                keep_nulls,
            },
        )?;
        Ok(TerlanPolarsDataFrame { inner })
    }
}

/// Converts selected wide columns into variable/value rows.
pub fn unpivot(
    df: &TerlanPolarsDataFrame,
    on: &[String],
    index: &[String],
    variable_name: &str,
    value_name: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if on.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_unpivot_columns",
            "unpivot requires at least one value column",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, index, variable_name, value_name);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{UnpivotArgsIR, UnpivotDF};
        let args = UnpivotArgsIR::new(
            df.inner.get_column_names_owned(),
            Some(on.iter().cloned().map(Into::into).collect()),
            index.iter().cloned().map(Into::into).collect(),
            Some(value_name.into()),
            Some(variable_name.into()),
        );
        let inner = df.inner.unpivot2(args)?;
        Ok(TerlanPolarsDataFrame { inner })
    }
}

/// Pivots long-form rows into columns using a named aggregation.
pub fn pivot(
    df: &TerlanPolarsDataFrame,
    on: &[String],
    index: &[String],
    values: &[String],
    aggregation: &str,
    separator: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if on.is_empty() || index.is_empty() || values.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_pivot_columns",
            "pivot requires on, index, and value columns",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, aggregation, separator);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use std::sync::Arc;

        use polars::frame::PivotColumnNaming;
        use polars::prelude::{cols, element, IntoLazy, UniqueKeepStrategy};
        let aggregate = match aggregation {
            "first" => element().first(),
            "last" => element().last(),
            "sum" => element().sum(),
            "mean" => element().mean(),
            "median" => element().median(),
            "min" => element().min(),
            "max" => element().max(),
            "len" => element().len(),
            _ => {
                return Err(TerlanPolarsError::new(
                    "invalid_pivot_aggregation",
                    format!("unsupported pivot aggregation `{aggregation}`; expected first, last, sum, mean, median, min, max, or len"),
                ));
            }
        };
        let on_columns = df
            .inner
            .select(on.iter().map(String::as_str))?
            .unique_stable(None, UniqueKeepStrategy::First, None)?;
        let inner = df
            .inner
            .clone()
            .lazy()
            .pivot(
                cols(on.iter().cloned()),
                Arc::new(on_columns),
                cols(index.iter().cloned()),
                cols(values.iter().cloned()),
                aggregate,
                true,
                separator.into(),
                PivotColumnNaming::Auto,
            )
            .collect()?;
        Ok(TerlanPolarsDataFrame { inner })
    }
}

/// Expands selected struct columns into ordinary DataFrame columns.
pub fn unnest_columns(
    df: &TerlanPolarsDataFrame,
    columns: &[String],
    separator: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    if columns.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_unnest_columns",
            "unnest_columns requires at least one column",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, separator);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        let inner = df.inner.unnest(
            columns.iter().cloned(),
            (!separator.is_empty()).then_some(separator),
        )?;
        Ok(TerlanPolarsDataFrame { inner })
    }
}

/// Transposes rows and columns, optionally retaining source column names.
pub fn transpose(
    df: &TerlanPolarsDataFrame,
    keep_names_as: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (df, keep_names_as);
        Err(unavailable_error())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{Column, DataFrame, DataType, NamedFrom, Series};

        if df.inner.height() == 0 {
            if !keep_names_as.is_empty() {
                let names = df
                    .inner
                    .get_column_names_owned()
                    .into_iter()
                    .map(|name| name.to_string())
                    .collect::<Vec<_>>();
                return Ok(TerlanPolarsDataFrame {
                    inner: DataFrame::new(
                        df.inner.width(),
                        vec![Series::new(keep_names_as.into(), names).into()],
                    )?,
                });
            }
            return Ok(TerlanPolarsDataFrame {
                inner: DataFrame::empty_with_height(df.inner.width()),
            });
        }
        if df.inner.width() == 0 {
            let mut columns =
                Vec::with_capacity(df.inner.height() + usize::from(!keep_names_as.is_empty()));
            if !keep_names_as.is_empty() {
                columns.push(Column::new_empty(keep_names_as.into(), &DataType::String));
            }
            columns.extend(
                (0..df.inner.height()).map(|index| {
                    Column::new_empty(format!("column_{index}").into(), &DataType::Null)
                }),
            );
            return Ok(TerlanPolarsDataFrame {
                inner: DataFrame::new(0, columns)?,
            });
        }
        let mut inner = df.inner.clone();
        let inner = inner.transpose((!keep_names_as.is_empty()).then_some(keep_names_as), None)?;
        Ok(TerlanPolarsDataFrame { inner })
    }
}

#[cfg(feature = "real-polars")]
/// Executes the concat diagonal operation through the Terlan Polars adapter.
pub fn concat_diagonal(
    df: &TerlanPolarsDataFrame,
    other: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{concat_lf_diagonal, IntoLazy, UnionArgs};
    let inner = concat_lf_diagonal(
        [df.inner.clone().lazy(), other.inner.clone().lazy()],
        UnionArgs::default(),
    )?
    .collect()?;
    Ok(TerlanPolarsDataFrame { inner })
}

#[cfg(feature = "real-polars")]
/// Executes the concat vertical operation through the Terlan Polars adapter.
pub fn concat_vertical(
    df: &TerlanPolarsDataFrame,
    other: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{concat, IntoLazy, UnionArgs};
    let inner = concat(
        [df.inner.clone().lazy(), other.inner.clone().lazy()],
        UnionArgs::default(),
    )?
    .collect()?;
    Ok(TerlanPolarsDataFrame { inner })
}

#[cfg(feature = "real-polars")]
/// Vertically concatenates two frames after coercing columns to common supertypes.
pub fn concat_vertical_relaxed(
    df: &TerlanPolarsDataFrame,
    other: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{concat, IntoLazy, UnionArgs};
    let inner = concat(
        [df.inner.clone().lazy(), other.inner.clone().lazy()],
        UnionArgs {
            to_supertypes: true,
            ..UnionArgs::default()
        },
    )?
    .collect()?;
    Ok(TerlanPolarsDataFrame { inner })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "real-polars")]
    fn asof_pivot_and_window_groups_work() {
        use polars::df;
        use polars::prelude::IntoLazy;
        let left = TerlanPolarsDataFrame {
            inner: df!("time" => [1_i64, 5, 10], "left_value" => [10_i64, 50, 100])
                .expect("asof left fixture"),
        };
        let right = TerlanPolarsDataFrame {
            inner: df!("time" => [2_i64, 6, 9], "right_value" => [20_i64, 60, 90])
                .expect("asof right fixture"),
        };
        let joined = asof_join(
            &left,
            &right,
            "time",
            "time",
            &[],
            &[],
            "backward",
            "",
            "_right",
            true,
        )
        .expect("asof join");
        assert_eq!(joined.inner.shape(), (3, 3));

        let long = TerlanPolarsDataFrame {
            inner: df!(
                "name" => ["A", "A", "B", "B"],
                "quarter" => ["Q1", "Q2", "Q1", "Q2"],
                "value" => [1_i64, 2, 3, 4]
            )
            .expect("pivot fixture"),
        };
        let pivoted = pivot(
            &long,
            &["quarter".to_string()],
            &["name".to_string()],
            &["value".to_string()],
            "sum",
            "_",
        )
        .expect("pivot");
        assert_eq!(pivoted.inner.shape(), (2, 3));

        let indexed = TerlanPolarsLazyFrame {
            inner: df!("index" => [0_i64, 1, 2, 3], "value" => [1_i64, 2, 3, 4])
                .expect("window fixture")
                .lazy(),
        };
        let value = expressions::expr_col("value").expect("value expression");
        let sum = expressions::expr_sum(&value).expect("sum expression");
        let sum = expressions::expr_alias(&sum, "total").expect("sum alias");
        let dynamic = lazy_dynamic_group_agg(
            &indexed,
            "index",
            &[],
            &[sum.clone()],
            "2i",
            "2i",
            "0i",
            "left",
            "data_point",
            false,
        )
        .expect("dynamic plan");
        assert_eq!(
            dynamic.inner.collect().expect("dynamic collect").shape(),
            (2, 2)
        );
        let rolling = lazy_rolling_group_agg(&indexed, "index", &[], &[sum], "2i", "-2i", "right")
            .expect("rolling plan");
        assert_eq!(
            rolling.inner.collect().expect("rolling collect").shape(),
            (4, 2)
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn temporal_and_list_expression_namespaces_work() {
        use polars::df;
        use polars::prelude::IntoLazy;
        let frame = df!(
            "group" => ["alpha", "beta"],
            "tags" => ["red|blue", "green|blue"],
            "numbers" => ["1|2", "3|4"],
            "stamp" => ["2024-01-02T03:04:05", "2024-02-03T04:05:06"]
        )
        .expect("expression fixture");
        let stamp = expressions::expr_col("stamp").expect("stamp");
        let stamp = expressions::expr_parse_datetime(&stamp).expect("datetime");
        let one_day = expressions::expr_lit(&TerlanPolarsScalar::String("1d".into()))
            .expect("one-day duration");
        let one_month = expressions::expr_lit(&TerlanPolarsScalar::String("1mo".into()))
            .expect("one-month duration");
        let temporal = [
            expressions::expr_year(&stamp),
            expressions::expr_month(&stamp),
            expressions::expr_day(&stamp),
            expressions::expr_weekday(&stamp),
            expressions::expr_hour(&stamp),
            expressions::expr_minute(&stamp),
            expressions::expr_second(&stamp),
            expressions::expr_millennium(&stamp),
            expressions::expr_century(&stamp),
            expressions::expr_is_leap_year(&stamp),
            expressions::expr_iso_year(&stamp),
            expressions::expr_days_in_month(&stamp),
            expressions::expr_quarter(&stamp),
            expressions::expr_week(&stamp),
            expressions::expr_ordinal_day(&stamp),
            expressions::expr_time_of_day(&stamp),
            expressions::expr_calendar_date(&stamp),
            expressions::expr_local_datetime(&stamp),
            expressions::expr_millisecond(&stamp),
            expressions::expr_microsecond(&stamp),
            expressions::expr_nanosecond(&stamp),
            expressions::expr_month_start(&stamp),
            expressions::expr_month_end(&stamp),
            expressions::expr_timestamp_milliseconds(&stamp),
            expressions::expr_timestamp_microseconds(&stamp),
            expressions::expr_timestamp_nanoseconds(&stamp),
            expressions::expr_temporal_truncate(&stamp, &one_day),
            expressions::expr_temporal_round(&stamp, &one_day),
            expressions::expr_temporal_offset_by(&stamp, &one_month),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("temporal expressions");
        let temporal = temporal
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("part_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("temporal aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&temporal).expect("compile temporal"))
            .collect()
            .expect("collect temporal");
        assert_eq!(out.shape(), (2, 29));
        assert_eq!(
            out.column("part_9")
                .expect("leap year")
                .get(0)
                .unwrap()
                .to_string(),
            "true"
        );
        assert_eq!(
            out.column("part_11")
                .expect("days in month")
                .get(0)
                .unwrap()
                .to_string(),
            "31"
        );
        assert_eq!(
            out.column("part_18")
                .expect("millisecond")
                .get(0)
                .unwrap()
                .to_string(),
            "0"
        );

        let duration_frame = df!(
            "start" => ["2024-01-01T00:00:00", "2024-01-01T00:00:00"],
            "finish" => ["2024-01-02T12:00:00", "2024-01-01T00:00:01"]
        )
        .expect("duration fixture");
        let start = expressions::expr_col("start").expect("duration start");
        let start = expressions::expr_parse_datetime(&start).expect("parsed duration start");
        let finish = expressions::expr_col("finish").expect("duration finish");
        let finish = expressions::expr_parse_datetime(&finish).expect("parsed duration finish");
        let duration = expressions::expr_subtract(&finish, &start).expect("duration expression");
        let totals = [
            expressions::expr_total_days(&duration, true),
            expressions::expr_total_hours(&duration, false),
            expressions::expr_total_minutes(&duration, true),
            expressions::expr_total_seconds(&duration, false),
            expressions::expr_total_milliseconds(&duration, true),
            expressions::expr_total_microseconds(&duration, false),
            expressions::expr_total_nanoseconds(&duration, true),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("duration total expressions");
        let totals = totals
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("total_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("duration total aliases");
        let out = duration_frame
            .lazy()
            .select(expressions::compile_many(&totals).expect("compile duration totals"))
            .collect()
            .expect("collect duration totals");
        assert_eq!(out.shape(), (2, 7));
        assert_eq!(
            out.column("total_1")
                .expect("total hours")
                .get(0)
                .unwrap()
                .to_string(),
            "36"
        );
        assert_eq!(
            out.column("total_3")
                .expect("total seconds")
                .get(1)
                .unwrap()
                .to_string(),
            "1"
        );

        let tags = expressions::expr_col("tags").expect("tags");
        let tags = expressions::expr_string_split(&tags, "|").expect("split");
        let zero = expressions::expr_lit(&TerlanPolarsScalar::Int(0)).expect("zero");
        let one = expressions::expr_lit(&TerlanPolarsScalar::Int(1)).expect("one");
        let blue = expressions::expr_lit(&TerlanPolarsScalar::String("blue".into())).expect("blue");
        let separator =
            expressions::expr_lit(&TerlanPolarsScalar::String("-".into())).expect("separator");
        let lists = [
            expressions::expr_list_len(&tags),
            expressions::expr_list_get(&tags, &zero),
            expressions::expr_list_contains(&tags, &blue),
            expressions::expr_list_sort(&tags),
            expressions::expr_list_join(&tags, &separator, true),
            expressions::expr_list_first(&tags),
            expressions::expr_list_last(&tags),
            expressions::expr_list_sort_with(&tags, true, true, true),
            expressions::expr_list_get_with(&tags, &zero, false),
            expressions::expr_list_contains_with(&tags, &blue, false),
            expressions::expr_list_slice(&tags, &zero, &one),
            expressions::expr_list_head(&tags, &one),
            expressions::expr_list_tail(&tags, &one),
            expressions::expr_list_shift(&tags, &one),
            expressions::expr_list_to_array(&tags, 2),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("list expressions");
        let lists = lists
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("list_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("list aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&lists).expect("compile lists"))
            .collect()
            .expect("collect lists");
        assert_eq!(out.shape(), (2, 15));

        let half = expressions::expr_lit(&TerlanPolarsScalar::Float(0.5)).expect("half");
        let indices = expressions::expr_lit(&TerlanPolarsScalar::String("0|1".into()))
            .expect("List gather indices");
        let indices =
            expressions::expr_string_split(&indices, "|").expect("split List gather indices");
        let index_list_type = expressions::data_type_list("int64").expect("List[Int64]");
        let indices =
            expressions::expr_cast(&indices, &index_list_type).expect("cast List gather indices");
        let other = expressions::expr_lit(&TerlanPolarsScalar::String("blue|yellow".into()))
            .expect("other List");
        let other = expressions::expr_string_split(&other, "|").expect("split other List");
        let list_struct =
            expressions::expr_list_to_struct(&tags, &["first".to_owned(), "second".to_owned()])
                .expect("List to Struct");
        let advanced_lists = [
            expressions::expr_list_drop_nulls(&tags),
            expressions::expr_list_sample_n(&tags, &one, false, false),
            expressions::expr_list_sample_n_seeded(&tags, &one, false, true, 42),
            expressions::expr_list_sample_fraction(&tags, &half, false, false),
            expressions::expr_list_sample_fraction_seeded(&tags, &half, false, true, 42),
            expressions::expr_list_gather(&tags, &indices, false),
            expressions::expr_list_gather_every(&tags, &one, &zero),
            Ok(list_struct),
            expressions::expr_list_count_matches(&tags, &blue),
            expressions::expr_list_set_union(&tags, &other),
            expressions::expr_list_set_difference(&tags, &other),
            expressions::expr_list_set_intersection(&tags, &other),
            expressions::expr_list_set_symmetric_difference(&tags, &other),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("advanced List expressions");
        let advanced_lists = advanced_lists
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("advanced_list_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("advanced List aliases");
        let out = frame
            .clone()
            .lazy()
            .select(
                expressions::compile_many(&advanced_lists)
                    .expect("compile advanced List expressions"),
            )
            .collect()
            .expect("collect advanced List expressions");
        assert_eq!(out.shape(), (2, 13));
        assert_eq!(
            out.column("advanced_list_8")
                .expect("List count matches")
                .get(0)
                .unwrap()
                .to_string(),
            "1"
        );

        let numeric_list = expressions::expr_col("numbers").expect("numbers");
        let numeric_list =
            expressions::expr_string_split(&numeric_list, "|").expect("split numeric List");
        let numeric_list_type = expressions::data_type_list("int64").expect("List[Int64]");
        let numeric_list =
            expressions::expr_cast(&numeric_list, &numeric_list_type).expect("numeric List cast");
        let two = expressions::expr_lit(&TerlanPolarsScalar::Int(2)).expect("two");
        let element = expressions::expr_element().expect("List element");
        let doubled = expressions::expr_multiply(&element, &two).expect("double List element");
        let element_sum = expressions::expr_sum(&element).expect("sum List elements");
        let numeric_lists = [
            expressions::expr_list_std(&numeric_list, 1),
            expressions::expr_list_var(&numeric_list, 1),
            expressions::expr_list_median(&numeric_list),
            expressions::expr_list_arg_min(&numeric_list),
            expressions::expr_list_arg_max(&numeric_list),
            expressions::expr_list_eval(&numeric_list, &doubled),
            expressions::expr_list_agg(&numeric_list, &element_sum),
            expressions::expr_list_diff_drop(&numeric_list, 1),
            expressions::expr_list_diff_ignore(&numeric_list, 1),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("numeric List expressions");
        let numeric_lists = numeric_lists
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("numeric_list_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("numeric List aliases");
        let out = frame
            .clone()
            .lazy()
            .select(
                expressions::compile_many(&numeric_lists)
                    .expect("compile numeric List expressions"),
            )
            .collect()
            .expect("collect numeric List expressions");
        assert_eq!(out.shape(), (2, 9));
        assert_eq!(
            out.column("numeric_list_6")
                .expect("List element aggregation")
                .get(0)
                .unwrap()
                .to_string(),
            "3"
        );

        let numbers = expressions::expr_col("numbers").expect("numbers");
        let numbers = expressions::expr_string_split(&numbers, "|").expect("split numbers");
        let array_type = expressions::data_type_array("int64", 2).expect("Array[Int64, 2]");
        let numbers = expressions::expr_cast(&numbers, &array_type).expect("numeric Array cast");
        let array_struct = expressions::expr_array_to_struct(&numbers).expect("Array to Struct");
        let element = expressions::expr_element().expect("Array element");
        let doubled = expressions::expr_multiply(&element, &two).expect("double Array element");
        let element_sum = expressions::expr_sum(&element).expect("sum Array elements");
        let arrays = [
            expressions::expr_array_len(&numbers),
            expressions::expr_array_sum(&numbers),
            expressions::expr_array_mean(&numbers),
            expressions::expr_array_min(&numbers),
            expressions::expr_array_max(&numbers),
            expressions::expr_array_get(&numbers, &one),
            expressions::expr_array_contains(&numbers, &two),
            expressions::expr_array_sort(&numbers),
            expressions::expr_array_to_list(&numbers),
            expressions::expr_array_std(&numbers, 1),
            expressions::expr_array_var(&numbers, 1),
            expressions::expr_array_median(&numbers),
            expressions::expr_array_arg_min(&numbers),
            expressions::expr_array_arg_max(&numbers),
            expressions::expr_array_count_matches(&numbers, &two),
            expressions::expr_struct_field_at(&array_struct, 0),
            expressions::expr_array_slice(&numbers, &zero, &one, false),
            expressions::expr_array_head(&numbers, &one, true),
            expressions::expr_array_tail(&numbers, &one, false),
            expressions::expr_array_shift(&numbers, &one),
            expressions::expr_array_eval(&numbers, &doubled, false),
            expressions::expr_array_agg(&numbers, &element_sum),
            expressions::expr_array_sort_with(&numbers, true, true, true),
            expressions::expr_array_get_with(&numbers, &one, false),
            expressions::expr_array_contains_with(&numbers, &two, false),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("array expressions");
        let arrays = arrays
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("array_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("array aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&arrays).expect("compile arrays"))
            .collect()
            .expect("collect arrays");
        assert_eq!(out.shape(), (2, 25));
        assert_eq!(
            out.column("array_0")
                .expect("array length")
                .get(0)
                .unwrap()
                .to_string(),
            "2"
        );
        assert_eq!(
            out.column("array_1")
                .expect("array sum")
                .get(0)
                .unwrap()
                .to_string(),
            "3"
        );
        assert_eq!(
            out.column("array_5")
                .expect("array get")
                .get(1)
                .unwrap()
                .to_string(),
            "4"
        );
        assert_eq!(
            out.column("array_6")
                .expect("array contains")
                .get(0)
                .unwrap()
                .to_string(),
            "true"
        );
        assert_eq!(
            out.column("array_14")
                .expect("array count matches")
                .get(0)
                .unwrap()
                .to_string(),
            "1"
        );
        assert_eq!(
            out.column("array_15")
                .expect("array Struct field")
                .get(1)
                .unwrap()
                .to_string(),
            "3"
        );
        assert_eq!(
            out.column("array_21")
                .expect("Array element aggregation")
                .get(0)
                .unwrap()
                .to_string(),
            "3"
        );
        assert_eq!(
            out.column("array_24")
                .expect("configurable Array contains")
                .get(0)
                .unwrap()
                .to_string(),
            "true"
        );

        let exploded = expressions::expr_array_explode(&numbers, false, false)
            .expect("explode Array expression");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&[exploded]).expect("compile Array explode"))
            .collect()
            .expect("collect Array explode");
        assert_eq!(out.shape(), (4, 1));

        let tags = expressions::expr_col("tags").expect("tags");
        let tags = expressions::expr_string_split(&tags, "|").expect("split tags");
        let string_array = expressions::data_type_array("string", 2).expect("Array[String, 2]");
        let tags = expressions::expr_cast(&tags, &string_array).expect("string Array cast");
        let separator =
            expressions::expr_lit(&TerlanPolarsScalar::String("-".into())).expect("separator");
        let joined = expressions::expr_array_join(&tags, &separator, true).expect("join Array");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&[joined]).expect("compile Array join"))
            .collect()
            .expect("collect Array join");
        assert_eq!(out.shape(), (2, 1));

        let group = expressions::expr_col("group").expect("group");
        let enum_type = expressions::data_type_enum(&["alpha".into(), "beta".into()])
            .expect("Enum[alpha, beta]");
        let category = expressions::expr_cast(&group, &enum_type).expect("enum cast");
        let categorical = [
            expressions::expr_categorical_categories(&category),
            expressions::expr_categorical_len_bytes(&category),
            expressions::expr_categorical_len_chars(&category),
            expressions::expr_categorical_starts_with(&category, "a"),
            expressions::expr_categorical_ends_with(&category, "a"),
            expressions::expr_categorical_slice(&category, 1, 3),
            expressions::expr_categorical_slice_to_end(&category, 1),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("categorical expressions");
        let categorical = categorical
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("categorical_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("categorical aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&categorical).expect("compile categorical"))
            .collect()
            .expect("collect categorical");
        assert_eq!(out.shape(), (2, 7));
        assert_eq!(
            out.column("categorical_1")
                .expect("categorical byte length")
                .get(0)
                .unwrap()
                .to_string(),
            "5"
        );
        assert_eq!(
            out.column("categorical_3")
                .expect("categorical starts with")
                .get(0)
                .unwrap()
                .to_string(),
            "true"
        );

        let binary_type = "binary";
        let tags = expressions::expr_col("tags").expect("tags");
        let tags = expressions::expr_cast(&tags, binary_type).expect("binary cast");
        let binary_literal = |value: &str| {
            let value = expressions::expr_lit(&TerlanPolarsScalar::String(value.into()))?;
            expressions::expr_cast(&value, binary_type)
        };
        let blue = binary_literal("blue").expect("binary blue");
        let red = binary_literal("red").expect("binary red");
        let suffix = binary_literal("blue").expect("binary suffix");
        let zero = expressions::expr_lit(&TerlanPolarsScalar::Int(0)).expect("zero");
        let one = expressions::expr_lit(&TerlanPolarsScalar::Int(1)).expect("one");
        let three = expressions::expr_lit(&TerlanPolarsScalar::Int(3)).expect("three");
        let hex = expressions::expr_binary_hex_encode(&tags).expect("hex encode");
        let hex_binary = expressions::expr_cast(&hex, binary_type).expect("encoded hex binary");
        let base64 = expressions::expr_binary_base64_encode(&tags).expect("base64 encode");
        let base64_binary =
            expressions::expr_cast(&base64, binary_type).expect("encoded base64 binary");
        let first_byte = expressions::expr_binary_head(&tags, &one).expect("first binary byte");
        let binary = [
            expressions::expr_binary_contains(&tags, &blue),
            expressions::expr_binary_starts_with(&tags, &red),
            expressions::expr_binary_ends_with(&tags, &suffix),
            expressions::expr_binary_size_bytes(&tags),
            expressions::expr_binary_get(&tags, &zero),
            expressions::expr_binary_head(&tags, &three),
            expressions::expr_binary_tail(&tags, &three),
            expressions::expr_binary_slice(&tags, &one, &three),
            Ok(hex.clone()),
            expressions::expr_binary_hex_decode(&hex_binary, true),
            Ok(base64.clone()),
            expressions::expr_binary_base64_decode(&base64_binary, true),
            expressions::expr_binary_reinterpret(&first_byte, "uint8", true),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("binary expressions");
        let binary = binary
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("binary_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("binary aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&binary).expect("compile binary"))
            .collect()
            .expect("collect binary");
        assert_eq!(out.shape(), (2, 13));
        assert_eq!(
            out.column("binary_0")
                .expect("binary contains")
                .get(0)
                .unwrap()
                .to_string(),
            "true"
        );
        assert_eq!(
            out.column("binary_3")
                .expect("binary size")
                .get(0)
                .unwrap()
                .to_string(),
            "8"
        );
        assert_eq!(
            out.column("binary_4")
                .expect("binary get")
                .get(0)
                .unwrap()
                .to_string(),
            "114"
        );
        assert_eq!(
            out.column("binary_12")
                .expect("binary reinterpret")
                .get(1)
                .unwrap()
                .to_string(),
            "103"
        );

        let group = expressions::expr_col("group").expect("group");
        let group = expressions::expr_alias(&group, "kind").expect("kind alias");
        let tags = expressions::expr_col("tags").expect("tags");
        let tags = expressions::expr_alias(&tags, "labels").expect("labels alias");
        let record = expressions::expr_as_struct(&[group, tags]).expect("record struct");
        let renamed =
            expressions::expr_struct_rename_fields(&record, &["category".into(), "payload".into()])
                .expect("rename struct fields");
        let one = expressions::expr_lit(&TerlanPolarsScalar::Int(1)).expect("one");
        let marker = expressions::expr_alias(&one, "marker").expect("marker alias");
        let extended =
            expressions::expr_struct_with_fields(&record, &[marker]).expect("extend struct");
        let structs = [
            expressions::expr_struct_field_at(&record, 0),
            expressions::expr_struct_field(&renamed, "category"),
            expressions::expr_struct_json_encode(&record),
            expressions::expr_struct_field(&extended, "marker"),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("struct expressions");
        let structs = structs
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("struct_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("struct aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&structs).expect("compile structs"))
            .collect()
            .expect("collect structs");
        assert_eq!(out.shape(), (2, 4));
        assert_eq!(
            out.column("struct_0")
                .expect("positional struct field")
                .get(0)
                .unwrap()
                .to_string(),
            "\"alpha\""
        );
        assert_eq!(
            out.column("struct_3")
                .expect("added struct field")
                .get(1)
                .unwrap()
                .to_string(),
            "1"
        );

        let fields = expressions::expr_struct_fields(&record, &["kind".into(), "labels".into()])
            .expect("select struct fields");
        let out = frame
            .lazy()
            .select(expressions::compile_many(&[fields]).expect("compile struct fields"))
            .collect()
            .expect("collect struct fields");
        assert_eq!(out.shape(), (2, 2));
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn random_and_bitwise_expression_namespaces_work() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!("value" => [1_i64, 3, 7, 8]).expect("numeric expression fixture");
        let value = expressions::expr_col("value").expect("value expression");
        let two = expressions::expr_lit(&TerlanPolarsScalar::Int(2)).expect("sample size");
        let half = expressions::expr_lit(&TerlanPolarsScalar::Float(0.5)).expect("sample fraction");

        let shuffles = [
            expressions::expr_shuffle(&value),
            expressions::expr_shuffle_seeded(&value, 42),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("shuffle expressions");
        let shuffles = shuffles
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("shuffle_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("shuffle aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&shuffles).expect("compile shuffles"))
            .collect()
            .expect("collect shuffles");
        assert_eq!(out.shape(), (4, 2));

        let samples = [
            expressions::expr_sample_n(&value, &two, false, false),
            expressions::expr_sample_n_seeded(&value, &two, false, true, 42),
            expressions::expr_sample_fraction(&value, &half, false, false),
            expressions::expr_sample_fraction_seeded(&value, &half, false, true, 42),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("sample expressions");
        let samples = samples
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("sample_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("sample aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&samples).expect("compile samples"))
            .collect()
            .expect("collect samples");
        assert_eq!(out.shape(), (2, 4));
        assert!(expressions::expr_shuffle_seeded(&value, -1).is_err());
        assert!(expressions::expr_sample_n_seeded(&value, &two, false, false, -1).is_err());

        let bit_counts = [
            expressions::expr_bitwise_count_ones(&value),
            expressions::expr_bitwise_count_zeros(&value),
            expressions::expr_bitwise_leading_ones(&value),
            expressions::expr_bitwise_leading_zeros(&value),
            expressions::expr_bitwise_trailing_ones(&value),
            expressions::expr_bitwise_trailing_zeros(&value),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("bit-count expressions");
        let bit_counts = bit_counts
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("bit_count_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("bit-count aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&bit_counts).expect("compile bit counts"))
            .collect()
            .expect("collect bit counts");
        assert_eq!(out.shape(), (4, 6));
        assert_eq!(
            out.column("bit_count_0")
                .expect("count ones")
                .get(2)
                .unwrap()
                .to_string(),
            "3"
        );

        let reductions = [
            expressions::expr_alias(
                &expressions::expr_bitwise_and(&value).expect("bitwise and"),
                "and",
            ),
            expressions::expr_alias(
                &expressions::expr_bitwise_or(&value).expect("bitwise or"),
                "or",
            ),
            expressions::expr_alias(
                &expressions::expr_bitwise_xor(&value).expect("bitwise xor"),
                "xor",
            ),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("bitwise reduction aliases");
        let out = frame
            .lazy()
            .select(expressions::compile_many(&reductions).expect("compile bitwise reductions"))
            .collect()
            .expect("collect bitwise reductions");
        assert_eq!(out.shape(), (1, 3));
        assert_eq!(
            out.column("and")
                .expect("and result")
                .get(0)
                .unwrap()
                .to_string(),
            "0"
        );
        assert_eq!(
            out.column("or")
                .expect("or result")
                .get(0)
                .unwrap()
                .to_string(),
            "15"
        );
        assert_eq!(
            out.column("xor")
                .expect("xor result")
                .get(0)
                .unwrap()
                .to_string(),
            "13"
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn cumulative_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame =
            df!("value" => [Some(2_i64), None, Some(3), Some(4)]).expect("cumulative fixture");
        let value = expressions::expr_col("value").expect("cumulative value expression");
        let cumulative = [
            expressions::expr_cum_count(&value, false),
            expressions::expr_cum_count(&value, true),
            expressions::expr_cum_sum(&value, false),
            expressions::expr_cum_sum(&value, true),
            expressions::expr_cum_product(&value, false),
            expressions::expr_cum_product(&value, true),
            expressions::expr_cum_min(&value, false),
            expressions::expr_cum_min(&value, true),
            expressions::expr_cum_max(&value, false),
            expressions::expr_cum_max(&value, true),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("cumulative expressions");
        let cumulative = cumulative
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("cumulative_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("cumulative aliases");
        let out = frame
            .lazy()
            .select(expressions::compile_many(&cumulative).expect("compile cumulative"))
            .collect()
            .expect("collect cumulative expressions");
        assert_eq!(out.shape(), (4, 10));
        let rendered = |column: &str, row: usize| {
            out.column(column)
                .expect("cumulative result column")
                .get(row)
                .expect("cumulative result value")
                .to_string()
        };
        assert_eq!(rendered("cumulative_0", 3), "3");
        assert_eq!(rendered("cumulative_1", 0), "3");
        assert_eq!(rendered("cumulative_2", 3), "9");
        assert_eq!(rendered("cumulative_3", 0), "9");
        assert_eq!(rendered("cumulative_4", 3), "24");
        assert_eq!(rendered("cumulative_5", 0), "24");
        assert_eq!(rendered("cumulative_6", 3), "2");
        assert_eq!(rendered("cumulative_7", 0), "2");
        assert_eq!(rendered("cumulative_8", 3), "4");
        assert_eq!(rendered("cumulative_9", 0), "4");
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn numeric_transform_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!(
            "value" => [-2.5_f64, -1.25, 1.25, 2.5],
            "positive" => [1.0_f64, 2.0, 4.0, std::f64::consts::E],
            "large" => [1234.56_f64, -1234.56, 0.0123456, -0.0123456],
            "probability" => [0.25_f64, 0.25, 0.25, 0.25],
            "integer" => [1_i8, 2, 3, 4],
        )
        .expect("numeric transform fixture");
        let value = expressions::expr_col("value").expect("value expression");
        let positive = expressions::expr_col("positive").expect("positive expression");
        let large = expressions::expr_col("large").expect("large expression");
        let probability = expressions::expr_col("probability").expect("probability expression");
        let integer = expressions::expr_col("integer").expect("integer expression");
        let zero = expressions::expr_lit(&TerlanPolarsScalar::Float(0.0)).expect("zero literal");
        let one = expressions::expr_lit(&TerlanPolarsScalar::Float(1.0)).expect("one literal");
        let minus_one =
            expressions::expr_lit(&TerlanPolarsScalar::Float(-1.0)).expect("minus one literal");
        let two = expressions::expr_lit(&TerlanPolarsScalar::Float(2.0)).expect("two literal");

        let transforms = [
            expressions::expr_round_with_mode(&value, 0, "half_to_even"),
            expressions::expr_round_with_mode(&value, 0, "half_away_from_zero"),
            expressions::expr_round_with_mode(&value, 0, "to_zero"),
            expressions::expr_round_sig_figs(&large, 3),
            expressions::expr_truncate(&value, 1),
            expressions::expr_floor(&value),
            expressions::expr_ceil(&value),
            expressions::expr_abs(&value),
            expressions::expr_clip(&value, &minus_one, &one),
            expressions::expr_clip_min(&value, &zero),
            expressions::expr_clip_max(&value, &zero),
            expressions::expr_log(&positive, &two),
            expressions::expr_log1p(&positive),
            expressions::expr_exp(&positive),
            expressions::expr_pi(),
            expressions::expr_product(&positive),
            expressions::expr_entropy(&probability, 2.0, false),
            expressions::expr_skew(&value, true),
            expressions::expr_kurtosis(&value, true, true),
            expressions::expr_upper_bound(&integer),
            expressions::expr_lower_bound(&integer),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("numeric transform expressions");
        let transforms = transforms
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("numeric_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("numeric transform aliases");
        let out = frame
            .lazy()
            .select(expressions::compile_many(&transforms).expect("compile numeric transforms"))
            .collect()
            .expect("collect numeric transforms");

        assert_eq!(out.shape(), (4, 21));
        let float_at = |column: &str, row: usize| {
            out.column(column)
                .expect("numeric transform result column")
                .f64()
                .expect("Float64 numeric transform result")
                .get(row)
                .expect("numeric transform result value")
        };
        assert_eq!(float_at("numeric_0", 0), -2.0);
        assert_eq!(float_at("numeric_0", 3), 2.0);
        assert_eq!(float_at("numeric_1", 0), -3.0);
        assert_eq!(float_at("numeric_1", 3), 3.0);
        assert_eq!(float_at("numeric_2", 0), -2.0);
        assert_eq!(float_at("numeric_2", 3), 2.0);
        assert_eq!(float_at("numeric_3", 0), 1230.0);
        assert!((float_at("numeric_3", 2) - 0.0123).abs() < 1.0e-12);
        assert_eq!(float_at("numeric_4", 1), -1.2);
        assert_eq!(float_at("numeric_5", 1), -2.0);
        assert_eq!(float_at("numeric_6", 1), -1.0);
        assert_eq!(float_at("numeric_7", 1), 1.25);
        assert_eq!(float_at("numeric_8", 0), -1.0);
        assert_eq!(float_at("numeric_8", 3), 1.0);
        assert_eq!(float_at("numeric_9", 0), 0.0);
        assert_eq!(float_at("numeric_10", 3), 0.0);
        assert_eq!(float_at("numeric_11", 2), 2.0);
        assert!((float_at("numeric_12", 0) - std::f64::consts::LN_2).abs() < 1.0e-12);
        assert!((float_at("numeric_13", 0) - std::f64::consts::E).abs() < 1.0e-12);
        assert!((float_at("numeric_14", 0) - std::f64::consts::PI).abs() < 1.0e-12);
        assert!((float_at("numeric_15", 0) - 8.0 * std::f64::consts::E).abs() < 1.0e-12);
        assert!((float_at("numeric_16", 0) - 2.0).abs() < 1.0e-12);
        assert!(float_at("numeric_17", 0).abs() < 1.0e-12);
        assert!((float_at("numeric_18", 0) + 1.64).abs() < 1.0e-12);
        assert_eq!(
            out.column("numeric_19")
                .expect("upper bound")
                .get(0)
                .unwrap()
                .to_string(),
            "127"
        );
        assert_eq!(
            out.column("numeric_20")
                .expect("lower bound")
                .get(0)
                .unwrap()
                .to_string(),
            "-128"
        );

        assert!(expressions::expr_round_with_mode(&value, 0, "unknown").is_err());
        assert!(expressions::expr_round_sig_figs(&value, 0).is_err());
        assert!(expressions::expr_truncate(&value, -1).is_err());
        assert!(expressions::expr_entropy(&probability, 1.0, false).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn scalar_aggregation_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!(
            "value" => [3_i64, 1, 3, 2],
            "other" => [2_i64, 4, 1, 5],
        )
        .expect("aggregation fixture");
        let value = expressions::expr_col("value").expect("value expression");
        let other = expressions::expr_col("other").expect("other expression");
        let quarter =
            expressions::expr_lit(&TerlanPolarsScalar::Float(0.25)).expect("quantile probability");
        let aggregations = [
            expressions::expr_count(&value),
            expressions::expr_len_values(&value),
            expressions::expr_quantile(&value, &quarter, "linear"),
            expressions::expr_mode(&value, true),
            expressions::expr_arg_min(&value),
            expressions::expr_arg_max(&value),
            expressions::expr_dot(&value, &other),
            expressions::expr_implode(&value, true),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("aggregation expressions");
        let aggregations = aggregations
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("aggregation_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("aggregation aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&aggregations).expect("compile aggregations"))
            .collect()
            .expect("collect aggregations");
        assert_eq!(out.shape(), (1, 8));
        let rendered = |column: &str| {
            out.column(column)
                .expect("aggregation result column")
                .get(0)
                .expect("aggregation result value")
                .to_string()
        };
        assert_eq!(rendered("aggregation_0"), "4");
        assert_eq!(rendered("aggregation_1"), "4");
        assert_eq!(rendered("aggregation_2"), "1.75");
        assert_eq!(rendered("aggregation_3"), "3");
        assert_eq!(rendered("aggregation_4"), "1");
        assert_eq!(rendered("aggregation_5"), "0");
        assert_eq!(rendered("aggregation_6"), "23");
        let imploded = out
            .column("aggregation_7")
            .expect("imploded result")
            .list()
            .expect("List aggregation result")
            .get_as_series(0)
            .expect("imploded values");
        assert_eq!(
            imploded
                .i64()
                .expect("Int64 imploded values")
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![3, 1, 3, 2]
        );

        let unique = expressions::expr_sort_ascending(
            &expressions::expr_unique(&value).expect("unique expression"),
        )
        .expect("sorted unique expression");
        let arg_unique = expressions::expr_arg_unique(&value).expect("arg-unique expression");
        let variable = frame
            .lazy()
            .select(
                expressions::compile_many(&[
                    expressions::expr_alias(&unique, "unique").unwrap(),
                    expressions::expr_alias(&arg_unique, "arg_unique").unwrap(),
                ])
                .expect("compile variable-length aggregations"),
            )
            .collect()
            .expect("collect variable-length aggregations");
        assert_eq!(variable.shape(), (3, 2));
        assert_eq!(
            variable
                .column("unique")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            variable
                .column("arg_unique")
                .unwrap()
                .u32()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![0, 1, 3]
        );

        let nullable = df!("value" => [None, Some(2_i64), None, Some(4_i64)])
            .expect("nullable aggregation fixture");
        let nullable_value = expressions::expr_col("value").unwrap();
        let null_aware = nullable
            .clone()
            .lazy()
            .select(
                expressions::compile_many(&[
                    expressions::expr_alias(
                        &expressions::expr_first_non_null(&nullable_value).unwrap(),
                        "first",
                    )
                    .unwrap(),
                    expressions::expr_alias(
                        &expressions::expr_last_non_null(&nullable_value).unwrap(),
                        "last",
                    )
                    .unwrap(),
                    expressions::expr_alias(
                        &expressions::expr_count(&nullable_value).unwrap(),
                        "count",
                    )
                    .unwrap(),
                    expressions::expr_alias(
                        &expressions::expr_len_values(&nullable_value).unwrap(),
                        "length",
                    )
                    .unwrap(),
                ])
                .expect("compile null-aware aggregations"),
            )
            .collect()
            .expect("collect null-aware aggregations");
        assert_eq!(null_aware.shape(), (1, 4));
        assert_eq!(
            null_aware
                .column("first")
                .unwrap()
                .get(0)
                .unwrap()
                .to_string(),
            "2"
        );
        assert_eq!(
            null_aware
                .column("last")
                .unwrap()
                .get(0)
                .unwrap()
                .to_string(),
            "4"
        );
        assert_eq!(
            null_aware
                .column("count")
                .unwrap()
                .get(0)
                .unwrap()
                .to_string(),
            "2"
        );
        assert_eq!(
            null_aware
                .column("length")
                .unwrap()
                .get(0)
                .unwrap()
                .to_string(),
            "4"
        );

        let without_nulls = nullable
            .lazy()
            .select(
                expressions::compile_many(
                    &[expressions::expr_drop_nulls(&nullable_value).unwrap()],
                )
                .unwrap(),
            )
            .collect()
            .expect("drop nulls");
        assert_eq!(without_nulls.shape(), (2, 1));
        assert_eq!(
            without_nulls
                .column("value")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![2, 4]
        );

        let nan_frame = df!(
            "value" => [Some(f64::NAN), Some(1.0_f64), None, Some(2.0_f64)]
        )
        .expect("NaN fixture");
        let nan_value = expressions::expr_col("value").unwrap();
        let without_nans = nan_frame
            .lazy()
            .select(
                expressions::compile_many(&[expressions::expr_drop_nans(&nan_value).unwrap()])
                    .unwrap(),
            )
            .collect()
            .expect("drop NaNs");
        assert_eq!(without_nans.shape(), (3, 1));
        let without_nans = without_nans.column("value").unwrap().f64().unwrap();
        assert_eq!(without_nans.get(0), Some(1.0));
        assert_eq!(without_nans.get(1), None);
        assert_eq!(without_nans.get(2), Some(2.0));

        let singleton = df!("value" => [7_i64]).expect("singleton fixture");
        let singleton_value = expressions::expr_col("value").unwrap();
        let item = singleton
            .lazy()
            .select(
                expressions::compile_many(&[
                    expressions::expr_item(&singleton_value, false).unwrap()
                ])
                .unwrap(),
            )
            .collect()
            .expect("collect single item");
        assert_eq!(
            item.column("value").unwrap().get(0).unwrap().to_string(),
            "7"
        );

        assert!(expressions::expr_quantile(&value, &quarter, "unknown").is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn generic_sequence_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!(
            "value" => [4_i64, 1, 3, 2],
            "sorted" => [1_i64, 3, 3, 5],
            "positive" => [1.0_f64, 2.0, 4.0, 8.0],
            "repeats" => [1_u32, 2, 0, 3],
            "other" => [8_i64, 7, 6, 5],
        )
        .expect("generic sequence fixture");
        let value = expressions::expr_col("value").unwrap();
        let sorted = expressions::expr_col("sorted").unwrap();
        let positive = expressions::expr_col("positive").unwrap();
        let repeats = expressions::expr_col("repeats").unwrap();
        let other = expressions::expr_col("other").unwrap();
        let zero = expressions::expr_lit(&TerlanPolarsScalar::Int(0)).unwrap();
        let one = expressions::expr_lit(&TerlanPolarsScalar::Int(1)).unwrap();
        let two = expressions::expr_lit(&TerlanPolarsScalar::Int(2)).unwrap();
        let three = expressions::expr_lit(&TerlanPolarsScalar::Int(3)).unwrap();

        let row_preserving = [
            expressions::expr_rechunk(&value),
            expressions::expr_arg_sort(&value, false, false),
            expressions::expr_sort_with(&value, false, false),
            expressions::expr_reverse(&value),
            expressions::expr_shift(&value, &one),
            expressions::expr_shift_and_fill(&value, &one, &zero),
            expressions::expr_repeat_by(&value, &repeats),
            expressions::expr_diff(&value, &one, "ignore"),
            expressions::expr_pct_change(&positive, &one),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("row-preserving sequence expressions");
        let row_preserving = row_preserving
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("sequence_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&row_preserving).unwrap())
            .collect()
            .expect("collect row-preserving sequence expressions");
        assert_eq!(out.shape(), (4, 9));
        let int_values = |column: &str| {
            out.column(column)
                .unwrap()
                .i64()
                .unwrap()
                .iter()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            int_values("sequence_0"),
            vec![Some(4), Some(1), Some(3), Some(2)]
        );
        assert_eq!(
            out.column("sequence_1")
                .unwrap()
                .u32()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![1, 3, 2, 0]
        );
        assert_eq!(
            int_values("sequence_2"),
            vec![Some(1), Some(2), Some(3), Some(4)]
        );
        assert_eq!(
            int_values("sequence_3"),
            vec![Some(2), Some(3), Some(1), Some(4)]
        );
        assert_eq!(
            int_values("sequence_4"),
            vec![None, Some(4), Some(1), Some(3)]
        );
        assert_eq!(
            int_values("sequence_5"),
            vec![Some(0), Some(4), Some(1), Some(3)]
        );
        assert_eq!(
            int_values("sequence_7"),
            vec![None, Some(-3), Some(2), Some(-1)]
        );
        assert_eq!(
            out.column("sequence_8")
                .unwrap()
                .f64()
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![None, Some(1.0), Some(1.0), Some(1.0)]
        );
        let repeated = out
            .column("sequence_6")
            .unwrap()
            .list()
            .unwrap()
            .get_as_series(1)
            .unwrap();
        assert_eq!(
            repeated
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![1, 1]
        );

        let scalar = [
            expressions::expr_index_of(&sorted, &three),
            expressions::expr_search_sorted(&sorted, &three, "left", false),
            expressions::expr_search_sorted(&sorted, &three, "right", false),
            expressions::expr_get_expression(&value, &two, false),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
        let scalar = scalar
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("scalar_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let scalar = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&scalar).unwrap())
            .collect()
            .expect("collect sequence scalar lookups");
        assert_eq!(scalar.shape(), (1, 4));
        assert_eq!(
            scalar
                .column("scalar_0")
                .unwrap()
                .get(0)
                .unwrap()
                .to_string(),
            "1"
        );
        assert_eq!(
            scalar
                .column("scalar_1")
                .unwrap()
                .get(0)
                .unwrap()
                .to_string(),
            "1"
        );
        assert_eq!(
            scalar
                .column("scalar_2")
                .unwrap()
                .get(0)
                .unwrap()
                .to_string(),
            "3"
        );
        assert_eq!(
            scalar
                .column("scalar_3")
                .unwrap()
                .get(0)
                .unwrap()
                .to_string(),
            "3"
        );

        let collect_values = |expression: String| {
            frame
                .clone()
                .lazy()
                .select(expressions::compile_many(&[expression]).unwrap())
                .collect()
                .unwrap()
                .column("value")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            collect_values(expressions::expr_slice_expression(&value, &one, &two).unwrap()),
            vec![1, 3]
        );
        assert_eq!(
            collect_values(expressions::expr_append(&value, &other, false).unwrap()),
            vec![4, 1, 3, 2, 8, 7, 6, 5]
        );
        assert_eq!(
            collect_values(expressions::expr_tail_expression(&value, 2).unwrap()),
            vec![3, 2]
        );
        assert_eq!(
            collect_values(expressions::expr_gather(&value, &two, false).unwrap()),
            vec![3]
        );
        assert_eq!(
            collect_values(expressions::expr_diff(&value, &one, "drop").unwrap()),
            vec![-3, 2, -1]
        );
        assert_eq!(
            collect_values(expressions::expr_gather_every(&value, 2, 1).unwrap()),
            vec![1, 2]
        );
        let nine = expressions::expr_lit(&TerlanPolarsScalar::Int(9)).unwrap();
        assert_eq!(
            collect_values(expressions::expr_extend_constant(&value, &nine, &two).unwrap()),
            vec![4, 1, 3, 2, 9, 9]
        );

        let nullable = df!("value" => [Some(2_i64), None, Some(1_i64), None]).unwrap();
        let nullable_value = expressions::expr_col("value").unwrap();
        let sorted_nulls = nullable
            .lazy()
            .select(
                expressions::compile_many(&[expressions::expr_sort_with(
                    &nullable_value,
                    false,
                    true,
                )
                .unwrap()])
                .unwrap(),
            )
            .collect()
            .unwrap();
        assert_eq!(
            sorted_nulls
                .column("value")
                .unwrap()
                .i64()
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![Some(1), Some(2), None, None]
        );

        assert!(expressions::expr_search_sorted(&sorted, &three, "invalid", false).is_err());
        assert!(expressions::expr_diff(&value, &one, "invalid").is_err());
        assert!(expressions::expr_gather_every(&value, 0, 0).is_err());
        assert!(expressions::expr_gather_every(&value, 1, -1).is_err());
        assert!(expressions::expr_tail_expression(&value, -1).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn ranking_and_top_k_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!(
            "value" => [5_i64, 1, 3, 3, 2],
            "key" => [2_i64, 5, 1, 4, 3],
        )
        .expect("ranking fixture");
        let value = expressions::expr_col("value").unwrap();
        let key = expressions::expr_col("key").unwrap();
        let two = expressions::expr_lit(&TerlanPolarsScalar::Int(2)).unwrap();
        let ranks = [
            expressions::expr_rank(&value, "average", false),
            expressions::expr_rank(&value, "min", false),
            expressions::expr_rank(&value, "max", false),
            expressions::expr_rank(&value, "dense", false),
            expressions::expr_rank(&value, "ordinal", false),
            expressions::expr_rank(&value, "dense", true),
            expressions::expr_rank_random(&value, false, 42),
            expressions::expr_rank_random(&value, false, 42),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("rank expressions");
        let ranks = ranks
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("rank_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let ranked = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&ranks).unwrap())
            .collect()
            .expect("collect ranks");
        assert_eq!(ranked.shape(), (5, 8));
        assert_eq!(
            ranked
                .column("rank_0")
                .unwrap()
                .f64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![5.0, 1.0, 3.5, 3.5, 2.0]
        );
        let integer_rank = |column: &str| {
            ranked
                .column(column)
                .unwrap()
                .u32()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>()
        };
        assert_eq!(integer_rank("rank_1"), vec![5, 1, 3, 3, 2]);
        assert_eq!(integer_rank("rank_2"), vec![5, 1, 4, 4, 2]);
        assert_eq!(integer_rank("rank_3"), vec![4, 1, 3, 3, 2]);
        assert_eq!(integer_rank("rank_4"), vec![5, 1, 3, 4, 2]);
        assert_eq!(integer_rank("rank_5"), vec![1, 4, 2, 2, 3]);
        assert_eq!(integer_rank("rank_6"), integer_rank("rank_7"));
        assert_eq!(integer_rank("rank_6"), vec![5, 1, 3, 4, 2]);

        let selections = [
            expressions::expr_top_k(&value, &two),
            expressions::expr_bottom_k(&value, &two),
            expressions::expr_top_k_by(&value, &two, std::slice::from_ref(&key), &[false]),
            expressions::expr_bottom_k_by(&value, &two, std::slice::from_ref(&key), &[false]),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("top/bottom-k expressions");
        let selections = selections
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("selection_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let selected = frame
            .lazy()
            .select(expressions::compile_many(&selections).unwrap())
            .collect()
            .expect("collect top/bottom-k expressions");
        assert_eq!(selected.shape(), (2, 4));
        let selected_values = |column: &str| {
            selected
                .column(column)
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>()
        };
        // Polars only guarantees membership for expression-level top/bottom-k;
        // the order of the selected values is intentionally unspecified.
        let sorted_selection = |column: &str| {
            let mut values = selected_values(column);
            values.sort_unstable();
            values
        };
        assert_eq!(sorted_selection("selection_0"), vec![3, 5]);
        assert_eq!(sorted_selection("selection_1"), vec![1, 2]);
        assert_eq!(sorted_selection("selection_2"), vec![1, 3]);
        assert_eq!(sorted_selection("selection_3"), vec![3, 5]);

        assert!(expressions::expr_rank(&value, "random", false).is_err());
        assert!(expressions::expr_rank(&value, "unknown", false).is_err());
        assert!(expressions::expr_rank_random(&value, false, -1).is_err());
        assert!(expressions::expr_top_k_by(&value, &two, &[], &[]).is_err());
        assert!(expressions::expr_top_k_by(&value, &two, std::slice::from_ref(&key), &[]).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn replacement_and_numeric_binning_expression_namespace_works() {
        use polars::df;
        use polars::prelude::{DataType, IntoLazy};

        let frame = df!("value" => [1_i64, 2, 3, 2]).expect("replacement fixture");
        let value = expressions::expr_col("value").unwrap();
        let two = expressions::expr_lit(&TerlanPolarsScalar::Int(2)).unwrap();
        let twenty = expressions::expr_lit(&TerlanPolarsScalar::Int(20)).unwrap();
        let minus_one = expressions::expr_lit(&TerlanPolarsScalar::Int(-1)).unwrap();
        let replacements = [
            expressions::expr_replace(&value, &two, &twenty),
            expressions::expr_replace_or_default(&value, &two, &twenty, &minus_one),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, expression)| {
            expressions::expr_alias(expression, &format!("replacement_{index}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
        let replaced = frame
            .lazy()
            .select(expressions::compile_many(&replacements).unwrap())
            .collect()
            .expect("collect replacements");
        let replacement_values = |column: &str| {
            replaced
                .column(column)
                .unwrap()
                .cast(&DataType::Int64)
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>()
        };
        assert_eq!(replacement_values("replacement_0"), vec![1, 20, 3, 20]);
        assert_eq!(replacement_values("replacement_1"), vec![-1, 20, -1, 20]);

        let strict = df!("value" => [2_i64, 2])
            .unwrap()
            .lazy()
            .select(
                expressions::compile_many(&[expressions::expr_replace_strict(
                    &value, &two, &twenty,
                )
                .unwrap()])
                .unwrap(),
            )
            .collect()
            .expect("collect strict replacement");
        assert_eq!(
            strict
                .column("value")
                .unwrap()
                .cast(&DataType::Int64)
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![20, 20]
        );

        let frame = df!("value" => [0.5_f64, 1.5, 2.5, 3.5]).expect("binning fixture");
        let value = expressions::expr_col("value").unwrap();
        let labels = vec!["low".to_string(), "mid".to_string(), "high".to_string()];
        let binary_labels = vec!["low".to_string(), "high".to_string()];
        let bins = [
            expressions::expr_cut(&value, &[1.0, 3.0], false, false),
            expressions::expr_cut_labeled(&value, &[1.0, 3.0], &labels, false, false),
            expressions::expr_qcut(&value, &[0.5], false, false, false),
            expressions::expr_qcut_labeled(&value, &[0.5], &binary_labels, false, false, false),
            expressions::expr_qcut_equal_frequency_labeled(
                &value,
                2,
                &binary_labels,
                false,
                false,
                false,
            ),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, expression)| expressions::expr_alias(expression, &format!("bin_{index}")))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
        let binned = frame
            .lazy()
            .select(expressions::compile_many(&bins).unwrap())
            .collect()
            .expect("collect bins");
        let bin_values = |column: &str| {
            binned
                .column(column)
                .unwrap()
                .cast(&DataType::String)
                .unwrap()
                .str()
                .unwrap()
                .iter()
                .map(|value| value.unwrap().to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            bin_values("bin_0"),
            vec!["(-inf, 1]", "(1, 3]", "(1, 3]", "(3, inf]"]
        );
        assert_eq!(bin_values("bin_1"), vec!["low", "mid", "mid", "high"]);
        assert_eq!(
            bin_values("bin_2"),
            vec!["(-inf, 2]", "(-inf, 2]", "(2, inf]", "(2, inf]"]
        );
        assert_eq!(bin_values("bin_3"), vec!["low", "low", "high", "high"]);
        assert_eq!(bin_values("bin_4"), vec!["low", "low", "high", "high"]);

        assert!(expressions::expr_cut(&value, &[1.0, 1.0], false, false).is_err());
        assert!(expressions::expr_cut(&value, &[f64::INFINITY], false, false).is_err());
        assert!(expressions::expr_qcut(&value, &[1.1], false, false, false).is_err());
        assert!(
            expressions::expr_cut_labeled(&value, &[1.0, 3.0], &binary_labels, false, false,)
                .is_err()
        );
        assert!(expressions::expr_qcut_equal_frequency(&value, 0, false, false, false).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn interpolation_peak_run_and_reshape_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!(
            "value" => [Some(0.0_f64), None, Some(8.0), None, Some(16.0)],
            "coordinate" => [0.0_f64, 1.0, 4.0, 6.0, 8.0],
        )
        .expect("interpolation fixture");
        let value = expressions::expr_col("value").unwrap();
        let coordinate = expressions::expr_col("coordinate").unwrap();
        let interpolated = frame
            .lazy()
            .select(
                expressions::compile_many(&[
                    expressions::expr_interpolate_by(&value, &coordinate).unwrap()
                ])
                .unwrap(),
            )
            .collect()
            .expect("collect coordinate interpolation");
        assert_eq!(
            interpolated
                .column("value")
                .unwrap()
                .f64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![0.0, 2.0, 8.0, 12.0, 16.0]
        );

        let frame = df!("value" => [1_i64, 3, 2, 4, 1]).expect("peak fixture");
        let value = expressions::expr_col("value").unwrap();
        let peaks = [
            expressions::expr_peak_min(&value),
            expressions::expr_peak_max(&value),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, expression)| expressions::expr_alias(expression, &format!("peak_{index}")))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
        let peaks = frame
            .lazy()
            .select(expressions::compile_many(&peaks).unwrap())
            .collect()
            .expect("collect peaks");
        let peak_values = |column: &str| {
            peaks
                .column(column)
                .unwrap()
                .bool()
                .unwrap()
                .iter()
                .map(Option::unwrap)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            peak_values("peak_0"),
            vec![false, false, true, false, false]
        );
        assert_eq!(peak_values("peak_1"), vec![false, true, false, true, false]);

        let frame = df!("value" => [1_i64, 1, 2, 2, 2, 1]).expect("run fixture");
        let value = expressions::expr_col("value").unwrap();
        let encoded = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&[expressions::expr_rle(&value).unwrap()]).unwrap())
            .collect()
            .expect("collect run-length encoding");
        let fields = encoded
            .column("value")
            .unwrap()
            .struct_()
            .unwrap()
            .fields_as_series();
        assert_eq!(
            fields[0]
                .u32()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![2, 3, 1]
        );
        assert_eq!(
            fields[1]
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![1, 2, 1]
        );
        let run_ids = frame
            .clone()
            .lazy()
            .select(
                expressions::compile_many(&[expressions::expr_rle_id(&value).unwrap()]).unwrap(),
            )
            .collect()
            .expect("collect run IDs");
        assert_eq!(
            run_ids
                .column("value")
                .unwrap()
                .u32()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![0, 0, 1, 1, 1, 2]
        );

        let reshaped = frame
            .lazy()
            .select(
                expressions::compile_many(&[expressions::expr_reshape(&value, &[-1, 2]).unwrap()])
                    .unwrap(),
            )
            .collect()
            .expect("collect reshape");
        let arrays = reshaped.column("value").unwrap().array().unwrap();
        assert_eq!(arrays.len(), 3);
        assert_eq!(
            arrays
                .get_as_series(1)
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![2, 2]
        );

        assert!(expressions::expr_reshape(&value, &[]).is_err());
        assert!(expressions::expr_reshape(&value, &[-1, -1]).is_err());
        assert!(expressions::expr_reshape(&value, &[-2, 2]).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn exponentially_weighted_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!(
            "value" => [1.0_f64, 2.0, 3.0, 4.0],
            "time" => [0_i64, 1, 2, 3],
        )
        .expect("exponentially weighted fixture");
        let value = expressions::expr_col("value").unwrap();
        let time = expressions::expr_col("time").unwrap();
        let weighted = [
            expressions::expr_ewm_sum(&value, 0.5, false, 1, true),
            expressions::expr_ewm_mean(&value, 0.5, false, 1, true),
            expressions::expr_ewm_std(&value, 0.5, false, true, 1, true),
            expressions::expr_ewm_var(&value, 0.5, false, true, 1, true),
            expressions::expr_ewm_mean_by(&value, &time, "1i"),
            expressions::expr_ewm_sum_by(&value, &time, "1i"),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, expression)| {
            expressions::expr_alias(expression, &format!("weighted_{index}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
        let weighted = frame
            .lazy()
            .select(expressions::compile_many(&weighted).unwrap())
            .collect()
            .expect("collect exponentially weighted expressions");
        let values = |column: &str| {
            weighted
                .column(column)
                .unwrap()
                .f64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>()
        };
        let sum = values("weighted_0");
        assert_eq!(sum, vec![1.0, 2.5, 4.25, 6.125]);
        let mean = values("weighted_1");
        assert_eq!(mean, vec![1.0, 1.5, 2.25, 3.125]);
        assert_eq!(values("weighted_4"), mean);
        assert_eq!(values("weighted_5"), sum);
        let standard_deviation = values("weighted_2");
        let variance = values("weighted_3");
        assert_eq!(standard_deviation.len(), 4);
        assert_eq!(variance.len(), 4);
        for (standard_deviation, variance) in standard_deviation.iter().zip(&variance) {
            assert!((standard_deviation * standard_deviation - variance).abs() < 1.0e-12);
        }

        assert!(expressions::expr_ewm_mean(&value, 0.0, false, 1, true).is_err());
        assert!(expressions::expr_ewm_mean(&value, f64::NAN, false, 1, true).is_err());
        assert!(expressions::expr_ewm_var(&value, 0.5, false, false, -1, true).is_err());
        for half_life in ["0i", "-1i", "1mo", "invalid"] {
            let expression = expressions::expr_ewm_mean_by(&value, &time, half_life).unwrap();
            assert!(expressions::compile_many(&[expression]).is_err());
        }
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn advanced_aggregation_and_window_expression_namespace_works() {
        use polars::df;
        use polars::prelude::{col, IntoLazy};

        let frame = df!(
            "value" => [1_i64, 2, 3, 4],
            "group" => ["a", "a", "b", "b"],
        )
        .expect("advanced window fixture");
        let value = expressions::expr_col("value").unwrap();
        let group = expressions::expr_col("group").unwrap();
        let element = expressions::expr_element().unwrap();
        let cumulative_sum = expressions::expr_sum(&element).unwrap();
        let cumulative = frame
            .clone()
            .lazy()
            .select(
                expressions::compile_many(&[expressions::expr_cumulative_eval(
                    &value,
                    &cumulative_sum,
                    1,
                )
                .unwrap()])
                .unwrap(),
            )
            .collect()
            .expect("collect cumulative evaluation");
        assert_eq!(
            cumulative
                .column("value")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![1, 3, 6, 10]
        );

        let joined = frame
            .clone()
            .lazy()
            .select(
                expressions::compile_many(&[expressions::expr_over_join(
                    &value,
                    std::slice::from_ref(&group),
                )
                .unwrap()])
                .unwrap(),
            )
            .collect()
            .expect("collect join-mapped window");
        let joined = joined.column("value").unwrap().list().unwrap();
        assert_eq!(
            joined
                .get_as_series(0)
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(
            joined
                .get_as_series(3)
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![3, 4]
        );

        let first = expressions::expr_first_non_null(&value).unwrap();
        let ordered = frame
            .clone()
            .lazy()
            .select(
                expressions::compile_many(&[expressions::expr_over_ordered(
                    &first,
                    std::slice::from_ref(&group),
                    std::slice::from_ref(&value),
                    true,
                    false,
                    "groups_to_rows",
                )
                .unwrap()])
                .unwrap(),
            )
            .collect()
            .expect("collect ordered window");
        assert_eq!(
            ordered
                .column("value")
                .unwrap()
                .i64()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![2, 2, 4, 4]
        );

        let removed =
            expressions::expr_agg_groups(&value).expect_err("Polars 2.0 removes agg_groups");
        assert_eq!(removed.code(), "unsupported_expression");
        let group_indices = frame
            .with_row_index("index".into(), None)
            .unwrap()
            .lazy()
            .group_by_stable([col("group")])
            .agg([col("index")])
            .collect()
            .expect("collect replacement aggregation group indices");
        let group_indices = group_indices.column("index").unwrap().list().unwrap();
        assert_eq!(
            group_indices
                .get_as_series(0)
                .unwrap()
                .u32()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(
            group_indices
                .get_as_series(1)
                .unwrap()
                .u32()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![2, 3]
        );

        assert!(expressions::expr_cumulative_eval(&value, &cumulative_sum, -1).is_err());
        assert!(expressions::expr_over_join(&value, &[]).is_err());
        assert!(expressions::expr_over_ordered(
            &value,
            std::slice::from_ref(&group),
            &[],
            false,
            false,
            "groups_to_rows",
        )
        .is_err());
        assert!(expressions::expr_over_ordered(
            &value,
            std::slice::from_ref(&group),
            std::slice::from_ref(&value),
            false,
            false,
            "invalid",
        )
        .is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn advanced_string_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame =
            df!("text" => ["café", "-7", "a.b", "e\u{301}"]).expect("advanced String fixture");
        let text = expressions::expr_col("text").unwrap();
        let dot = expressions::expr_lit(&TerlanPolarsScalar::String(".".into())).unwrap();
        let regex = expressions::expr_lit(&TerlanPolarsScalar::String("[é.]".into())).unwrap();
        let count_pattern =
            expressions::expr_lit(&TerlanPolarsScalar::String("[ae]".into())).unwrap();
        let length = expressions::expr_lit(&TerlanPolarsScalar::Int(6)).unwrap();
        let zfill_length = expressions::expr_lit(&TerlanPolarsScalar::Int(4)).unwrap();
        let hex = expressions::expr_lit(&TerlanPolarsScalar::String("636166c3a9".into())).unwrap();
        let base64 = expressions::expr_lit(&TerlanPolarsScalar::String("Y2Fmw6k=".into())).unwrap();
        let expressions = [
            expressions::expr_string_contains_literal(&text, &dot),
            expressions::expr_string_find_literal(&text, &dot),
            expressions::expr_string_find(&text, &regex, true),
            expressions::expr_string_count_matches(&text, &count_pattern, false),
            expressions::expr_string_pad_start(&text, &length, "*"),
            expressions::expr_string_pad_end(&text, &length, "*"),
            expressions::expr_string_zfill(&text, &zfill_length),
            expressions::expr_string_hex_encode(&text),
            expressions::expr_string_base64_encode(&text),
            expressions::expr_string_hex_decode(&hex, true),
            expressions::expr_string_base64_decode(&base64, true),
            expressions::expr_string_normalize(&text, "nfc"),
            expressions::expr_string_reverse(&text),
            expressions::expr_string_escape_regex(&text),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, expression)| expressions::expr_alias(expression, &format!("string_{index}")))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&expressions).unwrap())
            .collect()
            .expect("collect advanced String expressions");
        assert_eq!(out.shape(), (4, 14));
        assert_eq!(
            out.column("string_0")
                .unwrap()
                .bool()
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![Some(false), Some(false), Some(true), Some(false)]
        );
        assert_eq!(
            out.column("string_1")
                .unwrap()
                .u32()
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![None, None, Some(1), None]
        );
        assert_eq!(
            out.column("string_2")
                .unwrap()
                .u32()
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![Some(3), None, Some(1), None]
        );
        assert_eq!(
            out.column("string_3")
                .unwrap()
                .u32()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![1, 0, 1, 1]
        );
        assert_eq!(
            out.column("string_4").unwrap().str().unwrap().get(0),
            Some("**café")
        );
        assert_eq!(
            out.column("string_5").unwrap().str().unwrap().get(0),
            Some("café**")
        );
        assert_eq!(
            out.column("string_6").unwrap().str().unwrap().get(1),
            Some("-007")
        );
        assert_eq!(
            out.column("string_7").unwrap().str().unwrap().get(0),
            Some("636166c3a9")
        );
        assert_eq!(
            out.column("string_8").unwrap().str().unwrap().get(0),
            Some("Y2Fmw6k=")
        );
        assert_eq!(
            out.column("string_9").unwrap().binary().unwrap().get(0),
            Some("café".as_bytes())
        );
        assert_eq!(
            out.column("string_10").unwrap().binary().unwrap().get(0),
            Some("café".as_bytes())
        );
        assert_eq!(
            out.column("string_11").unwrap().str().unwrap().get(3),
            Some("é")
        );
        assert_eq!(
            out.column("string_12").unwrap().str().unwrap().get(0),
            Some("éfac")
        );
        assert_eq!(
            out.column("string_13").unwrap().str().unwrap().get(2),
            Some("a\\.b")
        );

        assert!(expressions::expr_string_pad_start(&text, &length, "").is_err());
        assert!(expressions::expr_string_pad_end(&text, &length, "**").is_err());
        assert!(expressions::expr_string_normalize(&text, "invalid").is_err());

        let invalid_regex = expressions::expr_lit(&TerlanPolarsScalar::String("(".into())).unwrap();
        let invalid_regex = expressions::expr_string_find(&text, &invalid_regex, true).unwrap();
        assert!(frame
            .lazy()
            .select(expressions::compile_many(&[invalid_regex]).unwrap())
            .collect()
            .is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn fixed_rolling_expression_family_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!("value" => [1.0_f64, 2.0, 3.0, 4.0, 5.0]).expect("fixed rolling fixture");
        let value = expressions::expr_col("value").expect("rolling value expression");
        let rolling = [
            expressions::expr_rolling_min(&value, 3, 2, false),
            expressions::expr_rolling_max(&value, 3, 2, false),
            expressions::expr_rolling_mean(&value, 3, 2, false),
            expressions::expr_rolling_sum(&value, 3, 2, false),
            expressions::expr_rolling_median(&value, 3, 2, false),
            expressions::expr_rolling_var(&value, 3, 2, false),
            expressions::expr_rolling_std(&value, 3, 2, false),
            expressions::expr_rolling_sum(&value, 3, 3, true),
            expressions::expr_rolling_quantile(&value, 3, 3, false, 0.25, "linear"),
            expressions::expr_rolling_rank(&value, 3, 3, false, "average", 42),
            expressions::expr_rolling_skew(&value, 3, 3, false, true),
            expressions::expr_rolling_kurtosis(&value, 5, 5, false, true, true),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("fixed rolling expressions");
        let rolling = rolling
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("rolling_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("fixed rolling aliases");
        let out = frame
            .lazy()
            .select(expressions::compile_many(&rolling).expect("compile fixed rolling"))
            .collect()
            .expect("collect fixed rolling expressions");

        assert_eq!(out.shape(), (5, 12));
        let value_at = |column: &str, row: usize| {
            out.column(column)
                .expect("fixed rolling result column")
                .f64()
                .expect("floating fixed rolling result")
                .get(row)
        };
        assert_eq!(value_at("rolling_0", 0), None);
        assert_eq!(value_at("rolling_0", 4), Some(3.0));
        assert_eq!(value_at("rolling_1", 4), Some(5.0));
        assert_eq!(value_at("rolling_2", 4), Some(4.0));
        assert_eq!(value_at("rolling_3", 4), Some(12.0));
        assert_eq!(value_at("rolling_4", 4), Some(4.0));
        assert_eq!(value_at("rolling_5", 4), Some(1.0));
        assert_eq!(value_at("rolling_6", 4), Some(1.0));
        assert_eq!(value_at("rolling_7", 0), None);
        assert_eq!(value_at("rolling_7", 1), Some(6.0));
        assert_eq!(value_at("rolling_7", 4), None);
        assert_eq!(value_at("rolling_8", 4), Some(3.5));
        assert_eq!(value_at("rolling_9", 4), Some(3.0));
        assert!(
            value_at("rolling_10", 4)
                .expect("rolling skew result")
                .abs()
                < 1.0e-12
        );
        assert!(
            (value_at("rolling_11", 4).expect("rolling kurtosis result") + 1.3).abs() < 1.0e-12
        );

        let invalid_window = expressions::expr_rolling_sum(&value, 0, 0, false)
            .expect_err("zero-sized rolling window must fail");
        assert_eq!(invalid_window.code(), "invalid_expression");
        let invalid_samples = expressions::expr_rolling_sum(&value, 2, 3, false)
            .expect_err("minimum samples above window size must fail");
        assert_eq!(invalid_samples.code(), "invalid_expression");
        assert!(expressions::expr_rolling_quantile(&value, 3, 3, false, 1.1, "linear").is_err());
        assert!(expressions::expr_rolling_quantile(&value, 3, 3, false, 0.5, "unknown").is_err());
        assert!(expressions::expr_rolling_rank(&value, 3, 3, false, "average", -1).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn weighted_rolling_expression_family_executes_and_validates_weights() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!("value" => [1.0_f64, 2.0, 3.0, 4.0]).expect("weighted rolling fixture");
        let value = expressions::expr_col("value").expect("weighted rolling column");
        let weights = [1.0, 2.0, 3.0];
        let rolling = [
            expressions::expr_rolling_min_weighted(&value, 3, 3, false, &weights),
            expressions::expr_rolling_max_weighted(&value, 3, 3, false, &weights),
            expressions::expr_rolling_mean_weighted(&value, 3, 3, false, &weights),
            expressions::expr_rolling_sum_weighted(&value, 3, 3, false, &weights),
            expressions::expr_rolling_median_weighted(&value, 3, 3, false, &weights),
            expressions::expr_rolling_var_weighted(&value, 3, 3, false, &weights),
            expressions::expr_rolling_std_weighted(&value, 3, 3, false, &weights),
            expressions::expr_rolling_quantile_weighted(
                &value, 3, 3, false, &weights, 0.25, "linear",
            ),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("weighted rolling expressions")
        .iter()
        .enumerate()
        .map(|(index, expression)| {
            expressions::expr_alias(expression, &format!("weighted_{index}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("weighted rolling aliases");
        let out = frame
            .lazy()
            .select(expressions::compile_many(&rolling).expect("compile weighted rolling"))
            .collect()
            .expect("collect weighted rolling");

        assert_eq!(out.shape(), (4, 8));
        assert_eq!(
            out.column("weighted_3")
                .expect("weighted sum")
                .f64()
                .expect("weighted sum type")
                .get(2),
            Some(14.0)
        );
        assert!(expressions::expr_rolling_sum_weighted(&value, 3, 3, false, &[1.0, 2.0]).is_err());
        assert!(
            expressions::expr_rolling_sum_weighted(&value, 3, 3, false, &[1.0, f64::NAN, 3.0],)
                .is_err()
        );
        assert!(expressions::expr_rolling_quantile_weighted(
            &value,
            3,
            3,
            false,
            &[0.0, 0.0, 0.0],
            0.5,
            "linear",
        )
        .is_err());

        let nullable = TerlanPolarsDataFrame {
            inner: df!("value" => [Some(1.0_f64), None, Some(3.0), Some(4.0)])
                .expect("nullable weighted fixture"),
        };
        let weighted = expressions::expr_rolling_sum_weighted(&value, 3, 2, false, &weights)
            .expect("nullable weighted expression");
        let error = select_exprs(&nullable, &[weighted])
            .expect_err("nullable weighted panic path must become an adapter error");
        assert!(
            matches!(error.code(), "native_execution_panic" | "polars_error"),
            "unexpected nullable weighted error: {}",
            error.message()
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn rolling_by_expression_family_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!(
            "index" => [1_i64, 2, 4, 7, 8],
            "value" => [1.0_f64, 2.0, 3.0, 4.0, 5.0],
        )
        .expect("rolling-by fixture");
        let value = expressions::expr_col("value").expect("rolling-by value expression");
        let index = expressions::expr_col("index").expect("rolling-by index expression");
        let rolling = [
            expressions::expr_rolling_min_by(&value, &index, "2i", 1, "right"),
            expressions::expr_rolling_max_by(&value, &index, "2i", 1, "right"),
            expressions::expr_rolling_mean_by(&value, &index, "2i", 1, "right"),
            expressions::expr_rolling_sum_by(&value, &index, "2i", 1, "right"),
            expressions::expr_rolling_median_by(&value, &index, "2i", 1, "right"),
            expressions::expr_rolling_var_by(&value, &index, "2i", 1, "right"),
            expressions::expr_rolling_std_by(&value, &index, "2i", 1, "right"),
            expressions::expr_rolling_quantile_by(&value, &index, "2i", 1, "right", 0.25, "linear"),
            expressions::expr_rolling_rank_by(&value, &index, "2i", 1, "right", "average", 42),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("rolling-by expressions");
        let rolling = rolling
            .iter()
            .enumerate()
            .map(|(position, expression)| {
                expressions::expr_alias(expression, &format!("rolling_by_{position}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("rolling-by aliases");
        let out = frame
            .lazy()
            .select(expressions::compile_many(&rolling).expect("compile rolling-by expressions"))
            .collect()
            .expect("collect rolling-by expressions");

        assert_eq!(out.shape(), (5, 9));
        let value_at = |column: &str, row: usize| {
            out.column(column)
                .expect("rolling-by result column")
                .f64()
                .expect("floating rolling-by result")
                .get(row)
        };
        assert_eq!(value_at("rolling_by_0", 1), Some(1.0));
        assert_eq!(value_at("rolling_by_1", 1), Some(2.0));
        assert_eq!(value_at("rolling_by_2", 1), Some(1.5));
        assert_eq!(value_at("rolling_by_3", 1), Some(3.0));
        assert_eq!(value_at("rolling_by_4", 1), Some(1.5));
        assert_eq!(value_at("rolling_by_5", 1), Some(0.5));
        assert!(
            (value_at("rolling_by_6", 1).expect("rolling-by standard deviation") - 0.5_f64.sqrt())
                .abs()
                < 1.0e-12
        );
        assert_eq!(value_at("rolling_by_7", 1), Some(1.25));
        assert_eq!(value_at("rolling_by_8", 1), Some(2.0));
        assert_eq!(value_at("rolling_by_3", 2), Some(3.0));
        assert_eq!(value_at("rolling_by_3", 4), Some(9.0));

        assert!(expressions::expr_rolling_sum_by(&value, &index, "2i", -1, "right").is_err());
        assert!(expressions::expr_rolling_sum_by(&value, &index, "2i", 1, "invalid").is_err());
        assert!(
            expressions::expr_rolling_rank_by(&value, &index, "2i", 1, "left", "average", 42,)
                .is_err()
        );
        let invalid_duration =
            expressions::expr_rolling_sum_by(&value, &index, "invalid", 1, "right")
                .expect("invalid duration remains serializable");
        assert!(expressions::compile_many(&[invalid_duration]).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn callback_free_expression_name_namespace_works() {
        use polars::df;
        use polars::prelude::{DataType, IntoLazy};

        let frame = df!(
            "Value One" => [1_i64, 2],
            "Group Label" => ["alpha", "beta"],
        )
        .expect("expression-name fixture");
        let value = expressions::expr_col("Value One").expect("value expression");
        let group = expressions::expr_col("Group Label").expect("group expression");
        let temporary = expressions::expr_alias(&value, "temporary").expect("temporary alias");
        let named = [
            expressions::expr_keep_name(&temporary),
            expressions::expr_replace_name(&value, r"\s+", "_", false),
            expressions::expr_lowercase_names(&group),
            expressions::expr_uppercase_names(&value),
            expressions::expr_prefix(&group, "pre_"),
            expressions::expr_suffix(&value, "_post"),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("root name expressions");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&named).expect("compile root name expressions"))
            .collect()
            .expect("collect root name expressions");
        assert_eq!(
            out.get_column_names()
                .iter()
                .map(|name| name.as_str())
                .collect::<Vec<_>>(),
            [
                "Value One",
                "Value_One",
                "group label",
                "VALUE ONE",
                "pre_Group Label",
                "Value One_post",
            ]
        );

        let record = expressions::expr_as_struct(&[value, group]).expect("Struct expression");
        let prefixed = expressions::expr_prefix_field_names(&record, "pre_")
            .and_then(|expression| expressions::expr_alias(&expression, "prefixed"))
            .expect("prefixed Struct fields");
        let suffixed = expressions::expr_suffix_field_names(&record, "_post")
            .and_then(|expression| expressions::expr_alias(&expression, "suffixed"))
            .expect("suffixed Struct fields");
        let out = frame
            .lazy()
            .select(
                expressions::compile_many(&[prefixed, suffixed])
                    .expect("compile Struct field name expressions"),
            )
            .collect()
            .expect("collect Struct field name expressions");
        let field_names = |column: &str| match out.column(column).expect("Struct column").dtype() {
            DataType::Struct(fields) => fields
                .iter()
                .map(|field| field.name().as_str().to_string())
                .collect::<Vec<_>>(),
            data_type => panic!("expected Struct dtype, found {data_type:?}"),
        };
        assert_eq!(
            field_names("prefixed"),
            ["pre_Value One", "pre_Group Label"]
        );
        assert_eq!(
            field_names("suffixed"),
            ["Value One_post", "Group Label_post"]
        );
        assert!(expressions::expr_replace_name(&temporary, "", "value", true).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn boolean_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!(
            "order" => [1_i64, 2, 3, 4],
            "value" => [1.0_f64, 1.0, f64::NAN, f64::INFINITY],
            "other" => [1.0_f64, 1.000_001, f64::NAN, f64::NEG_INFINITY],
            "nullable" => [Some(1_i64), None, None, Some(2)],
            "nullable_other" => [Some(1_i64), None, Some(3), None],
            "flag" => [Some(true), Some(false), None, Some(true)],
            "tags" => ["a|b", "b|c", "a|d", "d|e"]
        )
        .expect("Boolean fixture");
        let order = expressions::expr_col("order").expect("order expression");
        let value = expressions::expr_col("value").expect("value expression");
        let other = expressions::expr_col("other").expect("other expression");
        let nullable = expressions::expr_col("nullable").expect("nullable expression");
        let nullable_other =
            expressions::expr_col("nullable_other").expect("other nullable expression");
        let flag = expressions::expr_col("flag").expect("Boolean expression");
        let one = expressions::expr_lit(&TerlanPolarsScalar::Int(1)).expect("one");
        let three = expressions::expr_lit(&TerlanPolarsScalar::Int(3)).expect("three");
        let positive = expressions::expr_gt(&order, &one).expect("positive predicate");
        let bounded = expressions::expr_lt(&order, &three).expect("bounded predicate");
        let tags = expressions::expr_col("tags").expect("tags expression");
        let members = expressions::expr_string_split(&tags, "|").expect("membership lists");
        let needle =
            expressions::expr_lit(&TerlanPolarsScalar::String("a".into())).expect("needle");
        let horizontal = expressions::expr_any_horizontal(&[positive.clone(), bounded.clone()])
            .expect("horizontal any");
        let rowwise = [
            expressions::expr_is_null(&nullable),
            expressions::expr_is_not_null(&nullable),
            expressions::expr_is_finite(&value),
            expressions::expr_is_infinite(&value),
            expressions::expr_is_nan(&value),
            expressions::expr_is_not_nan(&value),
            expressions::expr_is_first_distinct(&value),
            expressions::expr_is_last_distinct(&value),
            expressions::expr_is_unique(&value),
            expressions::expr_is_duplicated(&value),
            expressions::expr_eq_missing(&nullable, &nullable_other),
            expressions::expr_neq_missing(&nullable, &nullable_other),
            expressions::expr_logical_and(&positive, &bounded),
            expressions::expr_logical_or(&positive, &bounded),
            expressions::expr_between_closed(&order, &one, &three, "left"),
            expressions::expr_is_in(&needle, &members, true),
            expressions::expr_is_close(&value, &other, 0.000_01, 0.000_01, true),
            Ok(horizontal),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("Boolean row expressions");
        let rowwise = rowwise
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("boolean_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("Boolean aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&rowwise).expect("compile Boolean rows"))
            .collect()
            .expect("collect Boolean rows");
        assert_eq!(out.shape(), (4, 18));
        let bool_value = |column: &str, row: usize| {
            out.column(column)
                .expect("Boolean result column")
                .bool()
                .expect("Boolean result")
                .get(row)
                .expect("Boolean value")
        };
        assert!(bool_value("boolean_0", 1));
        assert!(bool_value("boolean_3", 3));
        assert!(bool_value("boolean_4", 2));
        assert!(bool_value("boolean_10", 1));
        assert!(bool_value("boolean_14", 0));
        assert!(!bool_value("boolean_14", 2));
        assert!(bool_value("boolean_15", 2));
        assert!(bool_value("boolean_16", 2));

        let reductions = [
            expressions::expr_any(&flag, true),
            expressions::expr_all_true(&flag, true),
            expressions::expr_is_empty(&flag, true),
            expressions::expr_has_nulls(&flag),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("Boolean reductions");
        let reductions = reductions
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("boolean_reduction_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("Boolean reduction aliases");
        let out = frame
            .lazy()
            .select(expressions::compile_many(&reductions).expect("compile Boolean reductions"))
            .collect()
            .expect("collect Boolean reductions");
        assert_eq!(out.shape(), (1, 4));
        assert!(expressions::expr_between_closed(&order, &one, &three, "invalid").is_err());
        assert!(expressions::expr_is_close(&value, &other, -1.0, 0.0, false).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn trigonometry_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!(
            "radians" => [0.0_f64, std::f64::consts::FRAC_PI_4, std::f64::consts::FRAC_PI_2],
            "unit" => [-0.5_f64, 0.0, 0.5],
            "positive" => [1.0_f64, 2.0, 3.0],
            "degrees" => [0.0_f64, 45.0, 90.0],
            "y" => [0.0_f64, 1.0, 1.0],
            "x" => [1.0_f64, 1.0, 0.0]
        )
        .expect("trigonometry fixture");
        let radians = expressions::expr_col("radians").expect("radian expression");
        let unit = expressions::expr_col("unit").expect("unit-domain expression");
        let positive = expressions::expr_col("positive").expect("positive expression");
        let degrees = expressions::expr_col("degrees").expect("degree expression");
        let y = expressions::expr_col("y").expect("y expression");
        let x = expressions::expr_col("x").expect("x expression");
        let trigonometry = [
            expressions::expr_cos(&radians),
            expressions::expr_cot(&radians),
            expressions::expr_sin(&radians),
            expressions::expr_tan(&radians),
            expressions::expr_arccos(&unit),
            expressions::expr_arcsin(&unit),
            expressions::expr_arctan(&radians),
            expressions::expr_arctan2(&y, &x),
            expressions::expr_cosh(&radians),
            expressions::expr_sinh(&radians),
            expressions::expr_tanh(&radians),
            expressions::expr_arccosh(&positive),
            expressions::expr_arcsinh(&radians),
            expressions::expr_arctanh(&unit),
            expressions::expr_degrees(&radians),
            expressions::expr_radians(&degrees),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("trigonometry expressions");
        let trigonometry = trigonometry
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("trigonometry_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("trigonometry aliases");
        let out = frame
            .lazy()
            .select(expressions::compile_many(&trigonometry).expect("compile trigonometry"))
            .collect()
            .expect("collect trigonometry");
        assert_eq!(out.shape(), (3, 16));
        let value = |column: &str, row: usize| {
            out.column(column)
                .expect("trigonometry result column")
                .f64()
                .expect("Float64 trigonometry result")
                .get(row)
                .expect("trigonometry result value")
        };
        assert!((value("trigonometry_2", 2) - 1.0).abs() < 1.0e-12);
        assert!((value("trigonometry_7", 2) - std::f64::consts::FRAC_PI_2).abs() < 1.0e-12);
        assert!((value("trigonometry_14", 2) - 90.0).abs() < 1.0e-12);
        assert!((value("trigonometry_15", 1) - std::f64::consts::FRAC_PI_4).abs() < 1.0e-12);
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn statistics_expression_namespace_works() {
        use polars::df;
        use polars::prelude::IntoLazy;

        let frame = df!(
            "label" => ["one", "three", "seven", "eight"],
            "value" => [1.0_f64, 3.0, 7.0, 8.0]
        )
        .expect("statistics fixture");
        let value = expressions::expr_col("value").expect("value expression");
        let label = expressions::expr_col("label").expect("label expression");
        let statistics = [
            expressions::expr_std(&value, 1),
            expressions::expr_var(&value, 1),
            expressions::expr_min(&value),
            expressions::expr_median(&value),
            expressions::expr_min_by(&label, &value),
            expressions::expr_max_by(&label, &value),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("statistics expressions");
        let statistics = statistics
            .iter()
            .enumerate()
            .map(|(index, expression)| {
                expressions::expr_alias(expression, &format!("statistic_{index}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("statistics aliases");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&statistics).expect("compile statistics"))
            .collect()
            .expect("collect statistics");
        assert_eq!(out.shape(), (1, 6));
        assert_eq!(
            out.column("statistic_2")
                .expect("minimum")
                .get(0)
                .unwrap()
                .to_string(),
            "1.0"
        );
        assert_eq!(
            out.column("statistic_4")
                .expect("min by")
                .get(0)
                .unwrap()
                .to_string(),
            "\"one\""
        );
        assert_eq!(
            out.column("statistic_5")
                .expect("max by")
                .get(0)
                .unwrap()
                .to_string(),
            "\"eight\""
        );
        assert!(expressions::expr_std(&value, -1).is_err());
        assert!(expressions::expr_var(&value, 256).is_err());

        let nan_frame = df!("value" => [1.0_f64, f64::NAN, 3.0]).expect("NaN fixture");
        let nan_statistics = [
            expressions::expr_alias(
                &expressions::expr_nan_min(&value).expect("NaN minimum"),
                "nan_min",
            ),
            expressions::expr_alias(
                &expressions::expr_nan_max(&value).expect("NaN maximum"),
                "nan_max",
            ),
        ]
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("NaN statistics aliases");
        let out = nan_frame
            .lazy()
            .select(expressions::compile_many(&nan_statistics).expect("compile NaN statistics"))
            .collect()
            .expect("collect NaN statistics");
        assert!(out
            .column("nan_min")
            .expect("NaN minimum result")
            .f64()
            .expect("Float64 minimum")
            .get(0)
            .expect("minimum value")
            .is_nan());
        assert!(out
            .column("nan_max")
            .expect("NaN maximum result")
            .f64()
            .expect("Float64 maximum")
            .get(0)
            .expect("maximum value")
            .is_nan());

        let automatic =
            expressions::expr_histogram_auto(&value, false, false).expect("automatic histogram");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&[automatic]).expect("compile automatic histogram"))
            .collect()
            .expect("collect automatic histogram");
        assert_eq!(out.shape(), (10, 1));

        let counted =
            expressions::expr_histogram_count(&value, 2, true, true).expect("counted histogram");
        let out = frame
            .clone()
            .lazy()
            .select(expressions::compile_many(&[counted]).expect("compile counted histogram"))
            .collect()
            .expect("collect counted histogram");
        assert_eq!(out.shape(), (2, 1));
        assert!(expressions::expr_histogram_count(&value, 0, false, false).is_err());

        let bins = series_from_floats("bins", &[1.0, 4.0, 8.0]).expect("histogram bins");
        let bins = expressions::expr_series(&bins).expect("histogram bins expression");
        let explicit = expressions::expr_histogram_bins(&value, &bins, true, true)
            .expect("explicit histogram");
        let out = frame
            .lazy()
            .select(expressions::compile_many(&[explicit]).expect("compile explicit histogram"))
            .collect()
            .expect("collect explicit histogram");
        assert_eq!(out.shape(), (2, 1));
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn relational_join_modes_and_concatenation_work() {
        use polars::df;
        let left = TerlanPolarsDataFrame {
            inner: df!("id" => [1_i64, 2, 3], "name" => ["Ada", "Grace", "Edsger"])
                .expect("left fixture"),
        };
        let right = TerlanPolarsDataFrame {
            inner:
                df!("person_id" => [2_i64, 3, 4], "team" => ["compilers", "systems", "databases"])
                    .expect("right fixture"),
        };
        let left_on = vec!["id".to_string()];
        let right_on = vec!["person_id".to_string()];
        for (kind, expected_height, expected_width) in [
            ("inner", 2, 3),
            ("left", 3, 3),
            ("right", 3, 3),
            ("full", 4, 4),
            ("semi", 2, 2),
            ("anti", 1, 2),
        ] {
            let result = join(&left, &right, &left_on, &right_on, kind, "_team", false)
                .unwrap_or_else(|error| panic!("{kind} join failed: {}", error.message()));
            assert_eq!(
                result.inner.shape(),
                (expected_height, expected_width),
                "{kind}"
            );
        }
        let cross = join(&left, &right, &[], &[], "cross", "_team", false).expect("cross join");
        assert_eq!(cross.inner.shape(), (9, 4));
        let checked = join_with_options(
            &left,
            &right,
            &left_on,
            &right_on,
            "inner",
            "_team",
            false,
            "OneToOne",
            "PreserveLeft",
        )
        .expect("checked ordered join");
        assert_eq!(checked.inner.shape(), (2, 3));
        assert_eq!(
            checked
                .inner
                .column("id")
                .expect("ordered join key")
                .i64()
                .expect("Int64 ordered join key")
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
        let checked_lazy = lazy_join_with_options(
            &to_lazy(&left),
            &to_lazy(&right),
            &left_on,
            &right_on,
            "inner",
            "_team",
            false,
            "OneToOne",
            "PreserveRight",
        )
        .expect("checked ordered lazy join");
        assert_eq!(
            collect_lazy(&checked_lazy)
                .expect("collect checked ordered lazy join")
                .inner
                .shape(),
            (2, 3)
        );
        let duplicate_right = TerlanPolarsDataFrame {
            inner: df!("person_id" => [2_i64, 2], "team" => ["one", "two"])
                .expect("duplicate right fixture"),
        };
        assert!(join_with_options(
            &left,
            &duplicate_right,
            &left_on,
            &right_on,
            "inner",
            "_team",
            false,
            "OneToOne",
            "Unordered",
        )
        .is_err());
        assert_eq!(
            join_with_options(
                &left,
                &right,
                &left_on,
                &right_on,
                "inner",
                "_team",
                false,
                "invalid",
                "Unordered",
            )
            .expect_err("invalid join validation")
            .code(),
            "invalid_join_validation"
        );
        assert_eq!(
            join_with_options(
                &left,
                &right,
                &left_on,
                &right_on,
                "inner",
                "_team",
                false,
                "ManyToMany",
                "invalid",
            )
            .expect_err("invalid join order")
            .code(),
            "invalid_join_order"
        );
        let predicate = expressions::expr_gt(
            &expressions::expr_col("id").expect("left predicate column"),
            &expressions::expr_col("person_id").expect("right predicate column"),
        )
        .expect("non-equi predicate");
        let joined =
            join_where(&left, &right, &[predicate.clone()], "_team").expect("eager non-equi join");
        assert_eq!(joined.inner.shape(), (1, 4));
        let joined = lazy_join_where(&to_lazy(&left), &to_lazy(&right), &[predicate], "_team")
            .expect("lazy non-equi join");
        assert_eq!(
            collect_lazy(&joined)
                .expect("collect non-equi join")
                .inner
                .shape(),
            (1, 4)
        );
        assert_eq!(
            join_where(&left, &right, &[], "_team")
                .expect_err("empty non-equi predicates")
                .code(),
            "invalid_join_predicates"
        );
        assert_eq!(
            concat_vertical(&left, &left)
                .expect("vertical concat")
                .inner
                .shape(),
            (6, 2)
        );
        assert_eq!(
            concat_horizontal(&left, &right, true)
                .expect("horizontal concat")
                .inner
                .shape(),
            (3, 4)
        );
        let short = TerlanPolarsDataFrame {
            inner: df!("score" => [10_i64, 20]).expect("short horizontal fixture"),
        };
        assert!(concat_horizontal(&left, &short, true).is_err());
        let extended = concat_horizontal_extend(&left, &short).expect("horizontal extend");
        assert_eq!(extended.inner.shape(), (3, 3));
        assert_eq!(
            extended
                .inner
                .column("score")
                .expect("extended score")
                .null_count(),
            1
        );
        assert_eq!(
            concat_diagonal(&left, &right)
                .expect("diagonal concat")
                .inner
                .shape(),
            (6, 4)
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_explode_uses_polars_2_defaults_and_empty_transpose_works() {
        use polars::prelude::{DataFrame, DataType, IntoSeries, ListChunked, NamedFrom, Series};

        let values = Series::new("".into(), &[1_i64, 2]);
        let empty = Series::new_empty("".into(), &DataType::Int64);
        let items = ListChunked::from_iter([Some(values), Some(empty), None].into_iter())
            .with_name("items".into())
            .into_series();
        let frame = TerlanPolarsDataFrame {
            inner: DataFrame::new(3, vec![items.into()]).expect("list DataFrame"),
        };
        let columns = vec!["items".to_string()];

        let default = explode_columns(&frame, &columns).expect("Polars 2 explode defaults");
        assert_eq!(default.inner.shape(), (3, 1));
        assert_eq!(default.inner.column("items").unwrap().null_count(), 1);
        let legacy = explode_columns_with_options(&frame, &columns, true, true)
            .expect("legacy empty-list explode");
        assert_eq!(legacy.inner.shape(), (4, 1));
        assert_eq!(legacy.inner.column("items").unwrap().null_count(), 2);

        let lazy_default = collect_lazy(
            &lazy_explode(&to_lazy(&frame), &columns).expect("lazy Polars 2 explode defaults"),
        )
        .expect("collect lazy explode");
        assert_eq!(lazy_default.inner.shape(), (3, 1));

        let empty = TerlanPolarsDataFrame {
            inner: DataFrame::empty(),
        };
        assert_eq!(
            transpose(&empty, "")
                .expect("transpose empty DataFrame")
                .inner
                .shape(),
            (0, 0)
        );

        let zero_rows = TerlanPolarsDataFrame {
            inner: DataFrame::new(
                0,
                vec![
                    polars::prelude::Column::new_empty("left".into(), &DataType::Int64),
                    polars::prelude::Column::new_empty("right".into(), &DataType::String),
                ],
            )
            .expect("zero-row DataFrame"),
        };
        let named = transpose(&zero_rows, "source").expect("named zero-row transpose");
        assert_eq!(named.inner.shape(), (2, 1));
        let source = named.inner.column("source").unwrap().str().unwrap();
        assert_eq!(source.get(0), Some("left"));
        assert_eq!(source.get(1), Some("right"));

        let zero_columns = TerlanPolarsDataFrame {
            inner: DataFrame::empty_with_height(2),
        };
        assert_eq!(
            transpose(&zero_columns, "")
                .expect("zero-column transpose")
                .inner
                .shape(),
            (0, 2)
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_row_set_operations_work() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!(
                "id" => [1_i64, 1, 2, 3],
                "value" => [Some("a"), Some("a"), None, Some("c")]
            )
            .expect("row-set fixture"),
        };
        let id = vec!["id".to_string()];
        let value = vec!["value".to_string()];

        let first = unique_rows(&frame, &id, "KeepFirst", true).expect("stable first unique");
        assert_eq!(first.inner.shape(), (3, 2));
        assert_eq!(
            first
                .inner
                .column("id")
                .expect("unique id")
                .i64()
                .expect("Int64 unique id")
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            unique_rows(&frame, &id, "KeepNone", true)
                .expect("remove duplicate groups")
                .inner
                .shape(),
            (2, 2)
        );
        assert_eq!(
            unique_rows(&frame, &id, "KeepLast", true)
                .expect("stable last unique")
                .inner
                .shape(),
            (3, 2)
        );
        assert_eq!(
            unique_rows(&frame, &id, "KeepAny", false)
                .expect("optimizer-friendly unique")
                .inner
                .shape(),
            (3, 2)
        );
        assert_eq!(
            drop_null_rows(&frame, &value)
                .expect("drop null rows")
                .inner
                .shape(),
            (3, 2)
        );
        assert_eq!(
            slice_rows(&frame, 1, 2)
                .expect("positive slice")
                .inner
                .shape(),
            (2, 2)
        );
        assert_eq!(
            slice_rows(&frame, -2, 2)
                .expect("negative-offset slice")
                .inner
                .shape(),
            (2, 2)
        );

        let lazy_unique =
            lazy_unique_rows(&to_lazy(&frame), &id, "keep_last", true).expect("lazy stable unique");
        assert_eq!(
            collect_lazy(&lazy_unique)
                .expect("collect lazy unique")
                .inner
                .shape(),
            (3, 2)
        );
        let lazy_non_null =
            lazy_drop_null_rows(&to_lazy(&frame), &value).expect("lazy null removal");
        assert_eq!(
            collect_lazy(&lazy_non_null)
                .expect("collect lazy null removal")
                .inner
                .shape(),
            (3, 2)
        );
        let lazy_slice = lazy_slice_rows(&to_lazy(&frame), -3, 2).expect("lazy row slice");
        assert_eq!(
            collect_lazy(&lazy_slice)
                .expect("collect lazy slice")
                .inner
                .shape(),
            (2, 2)
        );

        assert_eq!(
            unique_rows(&frame, &id, "invalid", true)
                .expect_err("invalid unique strategy")
                .code(),
            "invalid_unique_keep"
        );
        assert_eq!(
            slice_rows(&frame, 0, -1)
                .expect_err("negative slice length")
                .code(),
            "invalid_slice_length"
        );
        assert_eq!(
            lazy_slice_rows(&to_lazy(&frame), 0, -1)
                .err()
                .expect("negative lazy slice length")
                .code(),
            "invalid_slice_length"
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_schema_and_layout_operations_work() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!("id" => [10_i64, 20, 30], "name" => ["Ada", "Grace", "Edsger"])
                .expect("schema-layout fixture"),
        };
        let existing = vec!["id".to_string(), "name".to_string()];
        let renamed_names = vec!["person_id".to_string(), "person_name".to_string()];
        let renamed = rename_columns(&frame, &existing, &renamed_names, true)
            .expect("strict simultaneous rename");
        assert_eq!(columns(&renamed), renamed_names);

        let ignored_missing = rename_columns(
            &frame,
            &["missing".to_string()],
            &["ignored".to_string()],
            false,
        )
        .expect("non-strict missing rename");
        assert_eq!(columns(&ignored_missing), vec!["id", "name"]);

        let dropped = drop_columns(&frame, &["name".to_string()]).expect("drop name column");
        assert_eq!(columns(&dropped), vec!["id"]);

        let reversed = reverse_rows(&frame).expect("reverse rows");
        assert_eq!(
            reversed
                .inner
                .column("id")
                .expect("reversed id")
                .i64()
                .expect("Int64 reversed id")
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![30, 20, 10]
        );

        let indexed = with_row_index(&frame, "row_nr", 5).expect("add row index");
        assert_eq!(columns(&indexed), vec!["row_nr", "id", "name"]);
        assert_eq!(
            indexed
                .inner
                .column("row_nr")
                .expect("row index")
                .u32()
                .expect("UInt32 row index")
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![5, 6, 7]
        );

        let plan = to_lazy(&frame);
        let lazy_renamed =
            lazy_rename_columns(&plan, &existing, &renamed_names, true).expect("lazy rename");
        assert_eq!(
            columns(&collect_lazy(&lazy_renamed).expect("collect lazy rename")),
            renamed_names
        );
        let lazy_dropped =
            lazy_drop_columns(&plan, &["name".to_string()]).expect("lazy drop column");
        assert_eq!(
            columns(&collect_lazy(&lazy_dropped).expect("collect lazy drop")),
            vec!["id"]
        );
        let lazy_reversed = lazy_reverse_rows(&plan);
        assert_eq!(
            collect_lazy(&lazy_reversed)
                .expect("collect lazy reverse")
                .inner
                .column("id")
                .expect("lazy reversed id")
                .i64()
                .expect("Int64 lazy reversed id")
                .get(0),
            Some(30)
        );
        let lazy_indexed = lazy_with_row_index(&plan, "row_nr", 8).expect("lazy row index");
        assert_eq!(
            columns(&collect_lazy(&lazy_indexed).expect("collect lazy index")),
            vec!["row_nr", "id", "name"]
        );

        assert_eq!(
            rename_columns(&frame, &existing, &["only_one".to_string()], true)
                .expect_err("mismatched rename lists")
                .code(),
            "invalid_rename_columns"
        );
        assert_eq!(
            drop_columns(&frame, &[])
                .expect_err("empty drop list")
                .code(),
            "invalid_drop_columns"
        );
        assert_eq!(
            with_row_index(&frame, "row_nr", -1)
                .expect_err("negative row-index offset")
                .code(),
            "invalid_row_index"
        );
        assert_eq!(
            lazy_with_row_index(&plan, "", 0)
                .err()
                .expect("empty lazy row-index name")
                .code(),
            "invalid_row_index"
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_missing_value_operations_work() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!("score" => [Some(1.0_f64), None, Some(f64::NAN)])
                .expect("missing-value fixture"),
        };
        let zero = expressions::expr_lit(&TerlanPolarsScalar::Float(0.0)).expect("zero expression");
        let negative_one = expressions::expr_lit(&TerlanPolarsScalar::Float(-1.0))
            .expect("negative-one expression");
        let score = vec!["score".to_string()];

        let filled_null = fill_null_values(&frame, &zero).expect("fill null values");
        assert_eq!(
            filled_null
                .inner
                .column("score")
                .expect("null-filled score")
                .null_count(),
            0
        );
        let filled_nan = fill_nan_values(&frame, &negative_one).expect("fill NaN values");
        assert_eq!(
            filled_nan
                .inner
                .column("score")
                .expect("NaN-filled score")
                .f64()
                .expect("Float64 NaN-filled score")
                .get(2),
            Some(-1.0)
        );
        assert_eq!(
            drop_nan_rows(&frame, &score)
                .expect("drop NaN rows")
                .inner
                .shape(),
            (2, 1)
        );
        let counts = null_counts(&frame).expect("null counts");
        assert_eq!(counts.inner.shape(), (1, 1));
        assert_eq!(
            counts
                .inner
                .column("score")
                .expect("score null count")
                .u32()
                .expect("UInt32 null count")
                .get(0),
            Some(1)
        );

        let plan = to_lazy(&frame);
        let lazy_filled_null = lazy_fill_null_values(&plan, &zero).expect("lazy fill null values");
        assert_eq!(
            collect_lazy(&lazy_filled_null)
                .expect("collect lazy null fill")
                .inner
                .column("score")
                .expect("lazy null-filled score")
                .null_count(),
            0
        );
        let lazy_filled_nan =
            lazy_fill_nan_values(&plan, &negative_one).expect("lazy fill NaN values");
        assert_eq!(
            collect_lazy(&lazy_filled_nan)
                .expect("collect lazy NaN fill")
                .inner
                .shape(),
            (3, 1)
        );
        let lazy_dropped = lazy_drop_nan_rows(&plan, &[]).expect("lazy drop NaN rows");
        assert_eq!(
            collect_lazy(&lazy_dropped)
                .expect("collect lazy drop NaN")
                .inner
                .shape(),
            (2, 1)
        );
        assert_eq!(
            collect_lazy(&lazy_null_counts(&plan))
                .expect("collect lazy null counts")
                .inner
                .shape(),
            (1, 1)
        );
        assert!(fill_null_values(&frame, "not-an-expression").is_err());
        assert!(lazy_fill_nan_values(&plan, "not-an-expression").is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_shift_and_lazy_planning_operations_work() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!("value" => [10_i64, 20, 30]).expect("shift fixture"),
        };
        let negative_one =
            expressions::expr_lit(&TerlanPolarsScalar::Int(-1)).expect("fill expression");

        let shifted = shift_rows(&frame, 1).expect("shift rows forward");
        let shifted_values = shifted
            .inner
            .column("value")
            .expect("shifted value")
            .i64()
            .expect("Int64 shifted value");
        assert_eq!(
            [
                shifted_values.get(0),
                shifted_values.get(1),
                shifted_values.get(2)
            ],
            [None, Some(10), Some(20)]
        );
        let shifted_back = shift_rows(&frame, -1).expect("shift rows backward");
        let shifted_back_values = shifted_back
            .inner
            .column("value")
            .expect("back-shifted value")
            .i64()
            .expect("Int64 back-shifted value");
        assert_eq!(
            [
                shifted_back_values.get(0),
                shifted_back_values.get(1),
                shifted_back_values.get(2)
            ],
            [Some(20), Some(30), None]
        );
        let filled = shift_and_fill_rows(&frame, 1, &negative_one).expect("shift and fill rows");
        assert_eq!(
            filled
                .inner
                .column("value")
                .expect("filled value")
                .i64()
                .expect("Int64 filled value")
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            vec![-1, 10, 20]
        );

        let plan = to_lazy(&frame);
        assert_eq!(
            lazy_schema(&plan).expect("resolve lazy schema"),
            vec![TerlanPolarsColumnSchema {
                name: "value".to_string(),
                data_type: "Int64".to_string(),
            }]
        );
        assert_eq!(
            collect_lazy(&lazy_cache(&plan))
                .expect("collect cached plan")
                .inner
                .shape(),
            (3, 1)
        );
        let lazy_shifted = collect_lazy(&lazy_shift_rows(&plan, -1)).expect("collect lazy shift");
        let lazy_shifted_values = lazy_shifted
            .inner
            .column("value")
            .expect("lazy shifted value")
            .i64()
            .expect("Int64 lazy shifted value");
        assert_eq!(
            [
                lazy_shifted_values.get(0),
                lazy_shifted_values.get(1),
                lazy_shifted_values.get(2)
            ],
            [Some(20), Some(30), None]
        );
        assert_eq!(
            collect_lazy(
                &lazy_shift_and_fill_rows(&plan, 1, &negative_one).expect("lazy shift and fill"),
            )
            .expect("collect lazy shift and fill")
            .inner
            .column("value")
            .expect("lazy filled value")
            .i64()
            .expect("Int64 lazy filled value")
            .into_no_null_iter()
            .collect::<Vec<_>>(),
            vec![-1, 10, 20]
        );

        assert!(shift_and_fill_rows(&frame, 1, "not-an-expression").is_err());
        assert!(lazy_shift_and_fill_rows(&plan, 1, "not-an-expression").is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_tail_clear_rechunk_and_lazy_endpoint_operations_work() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!("value" => [10_i64, 20, 30]).expect("row-endpoint fixture"),
        };

        let tailed = tail(&frame, 2).expect("tail rows");
        let tail_values = tailed
            .inner
            .column("value")
            .expect("tail value")
            .i64()
            .expect("Int64 tail value");
        assert_eq!(
            [tail_values.get(0), tail_values.get(1)],
            [Some(20), Some(30)]
        );
        assert_eq!(
            tail(&frame, 10).expect("oversized tail").inner.shape(),
            (3, 1)
        );

        let cleared = clear_rows(&frame).expect("clear rows");
        assert_eq!(cleared.inner.shape(), (0, 1));
        assert_eq!(schema(&cleared), schema(&frame));

        let mut multi_chunk_inner = frame.inner.clone();
        multi_chunk_inner
            .vstack_mut(&frame.inner)
            .expect("build multi-chunk frame");
        assert!(multi_chunk_inner.first_col_n_chunks() > 1);
        let multi_chunk = TerlanPolarsDataFrame {
            inner: multi_chunk_inner,
        };
        let contiguous = rechunk(&multi_chunk).expect("rechunk frame");
        assert_eq!(contiguous.inner.shape(), (6, 1));
        assert_eq!(contiguous.inner.first_col_n_chunks(), 1);
        assert!(multi_chunk.inner.first_col_n_chunks() > 1);

        let plan = to_lazy(&frame);
        let lazy_tailed = collect_lazy(&lazy_tail(&plan, 2)).expect("collect lazy tail");
        assert_eq!(lazy_tailed.inner.shape(), (2, 1));
        assert_eq!(
            lazy_tailed
                .inner
                .column("value")
                .expect("lazy tail value")
                .i64()
                .expect("Int64 lazy tail value")
                .get(0),
            Some(20)
        );
        assert_eq!(
            collect_lazy(&lazy_clear_rows(&plan))
                .expect("collect lazy clear")
                .inner
                .shape(),
            (0, 1)
        );
        assert_eq!(
            collect_lazy(&lazy_first_row(&plan))
                .expect("collect lazy first")
                .inner
                .column("value")
                .expect("lazy first value")
                .i64()
                .expect("Int64 lazy first value")
                .get(0),
            Some(10)
        );
        assert_eq!(
            collect_lazy(&lazy_last_row(&plan))
                .expect("collect lazy last")
                .inner
                .column("value")
                .expect("lazy last value")
                .i64()
                .expect("Int64 lazy last value")
                .get(0),
            Some(30)
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_eager_and_lazy_column_reductions_work() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!("value" => [1_i64, 3, 5]).expect("column-reduction fixture"),
        };
        let integer_value = |reduced: &TerlanPolarsDataFrame| {
            reduced
                .inner
                .column("value")
                .expect("reduced value")
                .i64()
                .expect("Int64 reduced value")
                .get(0)
        };
        let float_value = |reduced: &TerlanPolarsDataFrame| {
            reduced
                .inner
                .column("value")
                .expect("reduced value")
                .f64()
                .expect("Float64 reduced value")
                .get(0)
        };

        assert_eq!(
            integer_value(&column_sums(&frame).expect("column sums")),
            Some(9)
        );
        assert_eq!(
            float_value(&column_means(&frame).expect("column means")),
            Some(3.0)
        );
        assert_eq!(
            float_value(&column_medians(&frame).expect("column medians")),
            Some(3.0)
        );
        assert_eq!(
            integer_value(&column_minima(&frame).expect("column minima")),
            Some(1)
        );
        assert_eq!(
            integer_value(&column_maxima(&frame).expect("column maxima")),
            Some(5)
        );

        let plan = to_lazy(&frame);
        assert_eq!(
            integer_value(&collect_lazy(&lazy_column_sums(&plan)).expect("lazy column sums")),
            Some(9)
        );
        assert_eq!(
            float_value(&collect_lazy(&lazy_column_means(&plan)).expect("lazy column means")),
            Some(3.0)
        );
        assert_eq!(
            float_value(&collect_lazy(&lazy_column_medians(&plan)).expect("lazy column medians")),
            Some(3.0)
        );
        assert_eq!(
            integer_value(&collect_lazy(&lazy_column_minima(&plan)).expect("lazy column minima")),
            Some(1)
        );
        assert_eq!(
            integer_value(&collect_lazy(&lazy_column_maxima(&plan)).expect("lazy column maxima")),
            Some(5)
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_statistical_column_reductions_validate_and_execute() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!("value" => [1_i64, 3, 5]).expect("statistical reduction fixture"),
        };
        let integer_value = |reduced: &TerlanPolarsDataFrame| {
            reduced
                .inner
                .column("value")
                .expect("reduced value")
                .i64()
                .expect("Int64 reduced value")
                .get(0)
        };
        let float_value = |reduced: &TerlanPolarsDataFrame| {
            reduced
                .inner
                .column("value")
                .expect("reduced value")
                .f64()
                .expect("Float64 reduced value")
                .get(0)
        };

        assert_eq!(
            integer_value(&column_products(&frame).expect("column products")),
            Some(15)
        );
        assert_eq!(
            float_value(&column_variances(&frame, 1).expect("column variances")),
            Some(4.0)
        );
        assert_eq!(
            float_value(&column_stddevs(&frame, 1).expect("column standard deviations")),
            Some(2.0)
        );
        assert_eq!(
            float_value(&column_quantiles(&frame, 0.25, "linear").expect("column quantiles")),
            Some(2.0)
        );

        let plan = to_lazy(&frame);
        assert_eq!(
            integer_value(
                &collect_lazy(&lazy_column_products(&plan)).expect("lazy column products")
            ),
            Some(15)
        );
        assert_eq!(
            float_value(
                &collect_lazy(&lazy_column_variances(&plan, 1).expect("lazy column variances"))
                    .expect("collect lazy column variances")
            ),
            Some(4.0)
        );
        assert_eq!(
            float_value(
                &collect_lazy(
                    &lazy_column_stddevs(&plan, 1).expect("lazy column standard deviations")
                )
                .expect("collect lazy column standard deviations")
            ),
            Some(2.0)
        );
        assert_eq!(
            float_value(
                &collect_lazy(
                    &lazy_column_quantiles(&plan, 0.25, "linear").expect("lazy column quantiles")
                )
                .expect("collect lazy column quantiles")
            ),
            Some(2.0)
        );

        assert_eq!(
            column_variances(&frame, -1)
                .expect_err("negative ddof")
                .code(),
            "invalid_degrees_of_freedom"
        );
        assert_eq!(
            lazy_column_stddevs(&plan, 256)
                .err()
                .expect("oversized ddof")
                .code(),
            "invalid_degrees_of_freedom"
        );
        assert_eq!(
            column_quantiles(&frame, 1.1, "linear")
                .expect_err("out-of-range probability")
                .code(),
            "invalid_quantile_probability"
        );
        assert_eq!(
            lazy_column_quantiles(&plan, f64::NAN, "linear")
                .err()
                .expect("non-finite probability")
                .code(),
            "invalid_quantile_probability"
        );
        assert_eq!(
            column_quantiles(&frame, 0.5, "invalid")
                .expect_err("invalid quantile method")
                .code(),
            "invalid_quantile_method"
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_eager_and_lazy_column_cardinality_summaries_work() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!("value" => [Some(1_i64), None, Some(1), Some(2)])
                .expect("cardinality fixture"),
        };
        let cardinality = |summary: &TerlanPolarsDataFrame| {
            summary
                .inner
                .column("value")
                .expect("cardinality value")
                .u32()
                .expect("UInt32 cardinality value")
                .get(0)
        };

        assert_eq!(
            cardinality(&column_non_null_counts(&frame).expect("non-null counts")),
            Some(3)
        );
        assert_eq!(
            cardinality(&column_lengths(&frame).expect("column lengths")),
            Some(4)
        );
        assert_eq!(
            cardinality(&column_unique_counts(&frame).expect("unique counts")),
            Some(3)
        );
        assert_eq!(
            cardinality(&column_approx_unique_counts(&frame).expect("approximate unique counts")),
            Some(3)
        );

        let plan = to_lazy(&frame);
        assert_eq!(
            cardinality(
                &collect_lazy(&lazy_column_non_null_counts(&plan)).expect("lazy non-null counts")
            ),
            Some(3)
        );
        assert_eq!(
            cardinality(&collect_lazy(&lazy_column_lengths(&plan)).expect("lazy column lengths")),
            Some(4)
        );
        assert_eq!(
            cardinality(
                &collect_lazy(&lazy_column_unique_counts(&plan)).expect("lazy unique counts")
            ),
            Some(3)
        );
        assert_eq!(
            cardinality(
                &collect_lazy(&lazy_column_approx_unique_counts(&plan))
                    .expect("lazy approximate unique counts")
            ),
            Some(3)
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_multi_key_sort_and_row_selection_work() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!(
                "group" => ["b", "a", "a", "b"],
                "score" => [1_i64, 1, 2, 2],
                "id" => [0_i64, 1, 2, 3]
            )
            .expect("ordering fixture"),
        };
        let ids = |selected: &TerlanPolarsDataFrame| {
            let values = selected
                .inner
                .column("id")
                .expect("id column")
                .i64()
                .expect("Int64 id column");
            (0..values.len())
                .map(|index| values.get(index).expect("non-null id"))
                .collect::<Vec<_>>()
        };
        let sort_columns = vec!["group".to_string(), "score".to_string()];
        let descending = vec![false, true];
        let sort_nulls_last = vec![false, false];
        let score = vec!["score".to_string()];
        let score_nulls_last = vec![false];

        let sorted = sort_rows_by(&frame, &sort_columns, &descending, &sort_nulls_last, true)
            .expect("eager multi-key sort");
        assert_eq!(ids(&sorted), [2, 1, 3, 0]);
        assert_eq!(
            ids(&top_rows_by(&frame, &score, 2, &score_nulls_last, true).expect("eager top rows")),
            [2, 3]
        );
        assert_eq!(
            ids(&bottom_rows_by(&frame, &score, 2, &score_nulls_last, true)
                .expect("eager bottom rows")),
            [0, 1]
        );

        let plan = to_lazy(&frame);
        let lazy_sorted =
            lazy_sort_rows_by(&plan, &sort_columns, &descending, &sort_nulls_last, true)
                .expect("lazy multi-key sort");
        assert_eq!(
            ids(&collect_lazy(&lazy_sorted).expect("collect lazy multi-key sort")),
            [2, 1, 3, 0]
        );
        let lazy_top =
            lazy_top_rows_by(&plan, &score, 2, &score_nulls_last, true).expect("lazy top rows");
        assert_eq!(
            ids(&collect_lazy(&lazy_top).expect("collect lazy top rows")),
            [2, 3]
        );
        let lazy_bottom = lazy_bottom_rows_by(&plan, &score, 2, &score_nulls_last, true)
            .expect("lazy bottom rows");
        assert_eq!(
            ids(&collect_lazy(&lazy_bottom).expect("collect lazy bottom rows")),
            [0, 1]
        );

        assert_eq!(
            sort_rows_by(&frame, &[], &[], &[], true)
                .expect_err("empty sort columns")
                .code(),
            "invalid_sort_columns"
        );
        assert_eq!(
            sort_rows_by(&frame, &sort_columns, &[false], &sort_nulls_last, true)
                .expect_err("mismatched descending flags")
                .code(),
            "invalid_sort_options"
        );
        assert_eq!(
            lazy_sort_rows_by(&plan, &sort_columns, &descending, &[false], true)
                .err()
                .expect("mismatched null-placement flags")
                .code(),
            "invalid_sort_options"
        );
        assert_eq!(
            top_rows_by(&frame, &score, -1, &score_nulls_last, true)
                .expect_err("negative eager row limit")
                .code(),
            "invalid_row_limit"
        );
        assert_eq!(
            lazy_bottom_rows_by(&plan, &score, -1, &score_nulls_last, true)
                .err()
                .expect("negative lazy row limit")
                .code(),
            "invalid_row_limit"
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_eager_and_lazy_row_gather_is_bounds_safe() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!(
                "id" => [10_i64, 20, 30],
                "name" => ["a", "b", "c"]
            )
            .expect("gather fixture"),
        };
        let ids = |selected: &TerlanPolarsDataFrame| {
            let values = selected
                .inner
                .column("id")
                .expect("id column")
                .i64()
                .expect("Int64 id column");
            (0..values.len())
                .map(|index| values.get(index))
                .collect::<Vec<_>>()
        };

        let gathered = gather_rows(&frame, &[2, 0, 2], false).expect("checked eager gather");
        assert_eq!(ids(&gathered), [Some(30), Some(10), Some(30)]);
        let empty = gather_rows(&frame, &[], false).expect("empty eager gather");
        assert_eq!(empty.inner.height(), 0);
        assert_eq!(empty.inner.width(), 2);

        assert_eq!(
            gather_rows(&frame, &[0, 3], false)
                .expect_err("strict eager out-of-bounds index")
                .code(),
            "row_index_out_of_bounds"
        );
        let nullable =
            gather_rows(&frame, &[0, 3], true).expect("nullable eager out-of-bounds gather");
        assert_eq!(ids(&nullable), [Some(10), None]);
        assert_eq!(
            gather_rows(&frame, &[-1], true)
                .expect_err("negative eager index")
                .code(),
            "invalid_row_index"
        );
        assert_eq!(
            gather_rows(&frame, &[u32::MAX as i64 + 1], true)
                .expect_err("unrepresentable eager index")
                .code(),
            "invalid_row_index"
        );

        let plan = to_lazy(&frame);
        let lazy_gathered =
            lazy_gather_rows(&plan, &[2, 0, 2], false).expect("checked lazy gather");
        assert_eq!(
            ids(&collect_lazy(&lazy_gathered).expect("collect checked lazy gather")),
            [Some(30), Some(10), Some(30)]
        );
        let lazy_nullable = lazy_gather_rows(&plan, &[0, 3], true).expect("nullable lazy gather");
        assert_eq!(
            ids(&collect_lazy(&lazy_nullable).expect("collect nullable lazy gather")),
            [Some(10), None]
        );
        let lazy_strict =
            lazy_gather_rows(&plan, &[3], false).expect("deferred strict lazy gather");
        assert!(collect_lazy(&lazy_strict).is_err());
        assert_eq!(
            lazy_gather_rows(&plan, &[-1], true)
                .err()
                .expect("negative lazy index")
                .code(),
            "invalid_row_index"
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn lazy_streaming_file_sinks_round_trip_supported_formats() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!(
                "id" => [1_i64, 2, 3],
                "name" => ["a", "b", "c"]
            )
            .expect("lazy sink fixture"),
        };
        let plan = to_lazy(&frame);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = |extension: &str| {
            std::env::temp_dir().join(format!(
                "terlan-polars-lazy-sink-{}-{nonce}.{extension}",
                std::process::id()
            ))
        };
        let verify = |loaded: TerlanPolarsDataFrame| {
            assert_eq!(loaded.inner.height(), 3);
            assert_eq!(columns(&loaded), ["id", "name"]);
        };

        let csv = path("csv");
        lazy_write_csv(&plan, csv.to_str().expect("UTF-8 CSV path")).expect("lazy CSV sink");
        verify(read_csv(csv.to_str().expect("UTF-8 CSV path")).expect("read lazy CSV sink"));

        let parquet = path("parquet");
        lazy_write_parquet(&plan, parquet.to_str().expect("UTF-8 Parquet path"))
            .expect("lazy Parquet sink");
        verify(
            read_parquet(parquet.to_str().expect("UTF-8 Parquet path"))
                .expect("read lazy Parquet sink"),
        );

        let ndjson = path("jsonl");
        lazy_write_ndjson(&plan, ndjson.to_str().expect("UTF-8 NDJSON path"))
            .expect("lazy NDJSON sink");
        verify(
            read_ndjson(ndjson.to_str().expect("UTF-8 NDJSON path"))
                .expect("read lazy NDJSON sink"),
        );

        let ipc = path("ipc");
        lazy_write_ipc(&plan, ipc.to_str().expect("UTF-8 IPC path")).expect("lazy IPC sink");
        verify(read_ipc(ipc.to_str().expect("UTF-8 IPC path")).expect("read lazy IPC sink"));

        for output in [&csv, &parquet, &ndjson, &ipc] {
            std::fs::remove_file(output).expect("remove lazy sink fixture");
        }

        let missing_parent = std::env::temp_dir()
            .join(format!("terlan-polars-missing-{nonce}"))
            .join("output.csv");
        assert!(lazy_write_csv(
            &plan,
            missing_parent.to_str().expect("UTF-8 missing-parent path")
        )
        .is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn lazy_plan_tree_and_profile_diagnostics_work() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!(
                "id" => [1_i64, 2, 3],
                "value" => [10_i64, 20, 30]
            )
            .expect("diagnostics fixture"),
        };
        let selected = lazy_select(&to_lazy(&frame), &["value".to_string()]);
        let plan = lazy_sort(&selected, "value", true);

        let logical = lazy_describe_plan_tree(&plan, false).expect("logical plan tree");
        let optimized = lazy_describe_plan_tree(&plan, true).expect("optimized plan tree");
        assert!(!logical.trim().is_empty());
        assert!(!optimized.trim().is_empty());

        let profile = lazy_profile_plan(&plan).expect("profile lazy plan");
        assert!(profile.inner.height() >= 1);
        assert_eq!(columns(&profile), ["node", "start", "end"]);

        let collected = collect_lazy(&plan).expect("plan remains reusable after profiling");
        assert_eq!(collected.inner.height(), 3);
        assert_eq!(columns(&collected), ["value"]);

        let invalid = lazy_select(&to_lazy(&frame), &["missing".to_string()]);
        assert!(lazy_profile_plan(&invalid).is_err());
        assert!(lazy_describe_plan_tree(&invalid, true).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn lazy_optimizer_controls_create_reusable_derived_plans() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!(
                "id" => [1_i64, 2, 3],
                "value" => [10_i64, 20, 30]
            )
            .expect("optimizer fixture"),
        };
        let plan = lazy_sort(&to_lazy(&frame), "value", true);
        let names = [
            "projection_pushdown",
            "predicate_pushdown",
            "simplify_expression",
            "slice_pushdown",
            "common_subplan_elimination",
            "common_subexpression_elimination",
            "cluster_with_columns",
            "check_order",
            "type_coercion",
            "type_check",
        ];

        for name in names {
            let configured =
                lazy_set_optimization(&plan, name, false).expect("known optimizer pass");
            let collected = collect_lazy(&configured).expect("collect configured plan");
            assert_eq!(collected.inner.height(), 3, "optimizer {name}");
            assert_eq!(columns(&collected), ["id", "value"], "optimizer {name}");
        }

        let unoptimized = lazy_without_optimizations(&plan);
        let collected = collect_lazy(&unoptimized).expect("collect unoptimized plan");
        assert_eq!(collected.inner.height(), 3);
        assert_eq!(
            collect_lazy(&plan)
                .expect("original plan remains reusable")
                .inner
                .height(),
            3
        );
        assert_eq!(
            lazy_set_optimization(&plan, "unknown", true)
                .err()
                .expect("unknown optimizer pass")
                .code(),
            "invalid_lazy_optimization"
        );
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn eager_and_lazy_sequential_expression_pipelines_work() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!(
                "id" => [1_i64, 2, 3],
                "value" => [10_i64, 20, 30]
            )
            .expect("sequential expression fixture"),
        };
        let id = expressions::expr_col("id").expect("id expression");
        let value = expressions::expr_col("value").expect("value expression");
        let total = expressions::expr_add(&id, &value).expect("total expression");
        let total = expressions::expr_alias(&total, "total").expect("total alias");
        let projection = vec![id, total.clone()];

        let selected = select_exprs_sequential(&frame, &projection)
            .expect("eager sequential expression selection");
        assert_eq!(columns(&selected), ["id", "total"]);
        assert_eq!(
            selected
                .inner
                .column("total")
                .expect("eager total")
                .i64()
                .expect("Int64 eager total")
                .get(2),
            Some(33)
        );
        let extended = with_columns_sequential(&frame, &[total.clone()])
            .expect("eager sequential column mutation");
        assert_eq!(columns(&extended), ["id", "value", "total"]);

        let plan = to_lazy(&frame);
        let lazy_selected = lazy_select_exprs_sequential(&plan, &projection)
            .expect("lazy sequential expression selection");
        assert_eq!(
            columns(&collect_lazy(&lazy_selected).expect("collect lazy sequential selection")),
            ["id", "total"]
        );
        let lazy_extended =
            lazy_with_columns_sequential(&plan, &[total]).expect("lazy sequential column mutation");
        assert_eq!(
            columns(&collect_lazy(&lazy_extended).expect("collect lazy sequential mutation")),
            ["id", "value", "total"]
        );

        assert!(select_exprs_sequential(&frame, &["not-an-expression".to_string()]).is_err());
        assert!(lazy_with_columns_sequential(&plan, &["not-an-expression".to_string()]).is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn eager_and_lazy_remove_where_preserves_null_predicates() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!(
                "id" => [1_i64, 2, 3],
                "value" => [Some(1_i64), None, Some(3)]
            )
            .expect("inverse filter fixture"),
        };
        let value = expressions::expr_col("value").expect("value expression");
        let threshold =
            expressions::expr_lit(&TerlanPolarsScalar::Int(1)).expect("threshold expression");
        let predicate = expressions::expr_gt(&value, &threshold).expect("predicate expression");

        let filtered = filter_expr(&frame, &predicate).expect("eager filter");
        assert_eq!(rows(&filtered, 3).expect("filtered rows"), [vec!["3", "3"]]);

        let removed = remove_where(&frame, &predicate).expect("eager inverse filter");
        assert_eq!(
            rows(&removed, 3).expect("eager inverse-filtered rows"),
            [vec!["1", "1"], vec!["2", "null"]]
        );

        let plan = to_lazy(&frame);
        let lazy_filtered = lazy_filter_expr(&plan, &predicate).expect("lazy filter");
        assert_eq!(
            rows(
                &collect_lazy(&lazy_filtered).expect("collect lazy filter"),
                3
            )
            .expect("lazy filtered rows"),
            [vec!["3", "3"]]
        );
        let lazy_removed = lazy_remove_where(&plan, &predicate).expect("lazy inverse filter");
        assert_eq!(
            rows(
                &collect_lazy(&lazy_removed).expect("collect lazy inverse filter"),
                3
            )
            .expect("lazy inverse-filtered rows"),
            [vec!["1", "1"], vec!["2", "null"]]
        );

        assert!(remove_where(&frame, "not-an-expression").is_err());
        assert!(lazy_remove_where(&plan, "not-an-expression").is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn expression_metadata_introspection_matches_polars() {
        let value = expressions::expr_col("value").expect("value column");
        let group = expressions::expr_col("group").expect("group column");
        let combined = expressions::expr_add(&value, &group).expect("combined expression");
        let aliased = expressions::expr_alias(&combined, "combined").expect("aliased expression");
        let columns = expressions::expr_cols(&["value".to_string(), "group".to_string()])
            .expect("multi-column selector");
        let literal = expressions::expr_lit(&TerlanPolarsScalar::Int(1)).expect("literal");
        let aliased_literal =
            expressions::expr_alias(&literal, "one").expect("aliased literal expression");

        assert_eq!(
            expressions::expr_meta_root_names(&aliased).expect("root names"),
            ["value", "group"]
        );
        assert_eq!(
            expressions::expr_meta_output_name(&aliased).expect("output name"),
            "combined"
        );
        assert!(expressions::expr_meta_has_multiple_outputs(&columns)
            .expect("multiple-output classification"));
        assert!(expressions::expr_meta_is_column(&value).expect("column classification"));
        assert!(!expressions::expr_meta_is_column(&aliased).expect("aliased column classification"));
        assert!(expressions::expr_meta_is_simple_projection(&value)
            .expect("simple projection classification"));
        assert!(expressions::expr_meta_is_column_selection(
            &expressions::expr_alias(&value, "renamed").expect("column alias"),
            true,
        )
        .expect("aliased selection classification"));
        assert!(expressions::expr_meta_is_literal(&aliased_literal, true)
            .expect("aliased literal classification"));
        assert!(!expressions::expr_meta_is_literal(&aliased_literal, false)
            .expect("strict literal classification"));
        assert!(!expressions::expr_meta_is_regex_projection(&columns)
            .expect("regex projection classification"));
        assert!(!expressions::expr_meta_format_tree(&aliased, false)
            .expect("expression tree")
            .is_empty());
        assert!(expressions::expr_meta_root_names("not-an-expression").is_err());
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn frame_and_series_equality_preserve_polars_null_semantics() {
        use polars::df;
        let left = TerlanPolarsDataFrame {
            inner: df!("value" => [Some(1_i64), None]).expect("left equality fixture"),
        };
        let right = TerlanPolarsDataFrame {
            inner: df!("value" => [Some(1_i64), None]).expect("right equality fixture"),
        };
        let renamed = TerlanPolarsDataFrame {
            inner: df!("renamed" => [Some(1_i64), None]).expect("renamed equality fixture"),
        };
        let changed = TerlanPolarsDataFrame {
            inner: df!("value" => [Some(2_i64), None]).expect("changed equality fixture"),
        };

        assert!(!frames_equal(&left, &right));
        assert!(frames_equal_missing(&left, &right));
        assert!(!frames_equal_missing(&left, &renamed));
        assert!(!frames_equal_missing(&left, &changed));

        let left_series =
            series_from_nullable_ints("left", &[Some(1), None]).expect("left nullable Series");
        let right_series =
            series_from_nullable_ints("right", &[Some(1), None]).expect("right nullable Series");
        assert!(!series_equal(&left_series, &right_series));
        assert!(series_equal_missing(&left_series, &right_series));

        let non_null_left = series_from_ints("left", &[1, 2]).expect("left Series");
        let non_null_right = series_from_ints("right", &[1, 2]).expect("right Series");
        assert!(series_equal(&non_null_left, &non_null_right));
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_row_sampling_is_seeded_and_bounds_safe() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!(
                "id" => [1_i64, 2, 3, 4],
                "value" => [10_i64, 20, 30, 40],
            )
            .expect("sampling fixture"),
        };

        let first =
            sample_rows_n_seeded(&frame, 3, false, true, 42).expect("first fixed-count sample");
        let second =
            sample_rows_n_seeded(&frame, 3, false, true, 42).expect("second fixed-count sample");
        assert!(frames_equal_missing(&first, &second));
        assert_eq!((height(&first), width(&first)), (3, 2));

        let fractional =
            sample_rows_fraction_seeded(&frame, 0.5, false, true, 42).expect("fractional sample");
        assert_eq!((height(&fractional), width(&fractional)), (2, 2));

        let replacement =
            sample_rows_n_seeded(&frame, 6, true, true, 42).expect("replacement sample");
        assert_eq!(height(&replacement), 6);
        assert!(sample_rows_n(&frame, 5, false, true).is_err());

        let negative_count =
            sample_rows_n(&frame, -1, false, true).expect_err("negative sample count must fail");
        assert_eq!(negative_count.code(), "invalid_sample_size");
        let negative_seed = sample_rows_n_seeded(&frame, 1, false, true, -1)
            .expect_err("negative sample seed must fail");
        assert_eq!(negative_seed.code(), "invalid_sample_seed");

        for fraction in [-0.1, f64::NAN] {
            let error = sample_rows_fraction(&frame, fraction, false, true)
                .expect_err("invalid sample fraction must fail");
            assert_eq!(error.code(), "invalid_sample_fraction");
        }

        let excessive_count =
            sample_rows_n(&frame, MAX_GENERATED_SAMPLE_ROWS as i64 + 1, true, true)
                .expect_err("excessive generated sample must fail before allocation");
        assert_eq!(excessive_count.code(), "sample_size_too_large");
        let excessive_fraction = sample_rows_fraction(&frame, 3_000_000.0, true, true)
            .expect_err("excessive fractional sample must fail before allocation");
        assert_eq!(excessive_fraction.code(), "sample_size_too_large");
    }

    #[test]
    #[cfg(feature = "real-polars")]
    fn dataframe_row_identity_masks_and_hashes_preserve_duplicate_groups() {
        use polars::df;
        let frame = TerlanPolarsDataFrame {
            inner: df!(
                "key" => ["a", "a", "b", "c", "c"],
                "value" => [1_i64, 1, 2, 3, 3],
            )
            .expect("row identity fixture"),
        };

        let unique = row_is_unique(&frame).expect("unique row mask");
        let duplicated = row_is_duplicated(&frame).expect("duplicated row mask");
        assert_eq!(series_name(&unique), "row_is_unique");
        assert_eq!(series_data_type(&unique), "Boolean");
        assert_eq!(
            series_values(&unique, 5).expect("unique values"),
            ["false", "false", "true", "false", "false"]
        );
        assert_eq!(
            series_values(&duplicated, 5).expect("duplicated values"),
            ["true", "true", "false", "true", "true"]
        );

        let default_hashes = row_hashes(&frame).expect("default row hashes");
        assert_eq!(series_data_type(&default_hashes), "UInt64");
        let hashes = series_values(&default_hashes, 5).expect("row hash values");
        assert_eq!(hashes[0], hashes[1]);
        assert_eq!(hashes[3], hashes[4]);
        assert_ne!(hashes[0], hashes[2]);

        let seeded = row_hashes_seeded(&frame, 42).expect("seeded row hashes");
        let seeded_again = row_hashes_seeded(&frame, 42).expect("repeated seeded row hashes");
        assert!(series_equal_missing(&seeded, &seeded_again));
        let changed_seed = row_hashes_seeded(&frame, 43).expect("different seeded row hashes");
        assert!(!series_equal_missing(&seeded, &changed_seed));

        let negative_seed =
            row_hashes_seeded(&frame, -1).expect_err("negative row hash seed must fail");
        assert_eq!(negative_seed.code(), "invalid_row_hash_seed");

        let empty = TerlanPolarsDataFrame {
            inner: polars::prelude::DataFrame::empty(),
        };
        assert_eq!(
            series_len(&row_is_unique(&empty).expect("empty unique mask")),
            0
        );
        assert_eq!(
            series_len(&row_is_duplicated(&empty).expect("empty duplicate mask")),
            0
        );
        assert_eq!(
            series_len(&row_hashes(&empty).expect("empty row hashes")),
            0
        );
        assert_eq!(
            series_len(&row_hashes_seeded(&empty, 42).expect("empty seeded row hashes")),
            0
        );
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

    /// Verifies the stubbed read path returns the stable unavailable error.
    #[cfg(not(feature = "real-polars"))]
    #[test]
    fn read_csv_returns_unavailable_error() {
        let err = read_csv("data.csv").expect_err("stub should return unavailable error");

        assert_eq!(err.code(), "native_unavailable");
        assert_eq!(
            err.message(),
            "terlan-polars requires the Rust native adapter target capability"
        );
    }

    /// Verifies adapter errors expose the future Terlan error fields.
    #[cfg(not(feature = "real-polars"))]
    #[test]
    fn adapter_error_converts_to_code_message_parts() {
        let (code, message) = unavailable_error().into_parts();

        assert_eq!(code, "native_unavailable");
        assert_eq!(
            message,
            "terlan-polars requires the Rust native adapter target capability"
        );
    }

    /// Verifies the stubbed DataFrame observers are callable.
    #[cfg(not(feature = "real-polars"))]
    #[test]
    fn dataframe_observers_return_stub_values() {
        let df = TerlanPolarsDataFrame;

        assert_eq!(height(&df), 0);
        assert_eq!(width(&df), 0);
        assert!(columns(&df).is_empty());
        assert!(schema(&df).is_empty());
    }

    /// Verifies real Polars linkage reads the committed CSV fixture.
    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_reads_csv_fixture() {
        let df = read_csv(&fixture_path("people.csv")).expect("fixture should read");

        assert_eq!(height(&df), 2);
        assert_eq!(width(&df), 3);
        assert_eq!(columns(&df), ["name", "age", "city"]);
        assert_eq!(
            schema(&df),
            [
                TerlanPolarsColumnSchema {
                    name: "name".to_string(),
                    data_type: "String".to_string(),
                },
                TerlanPolarsColumnSchema {
                    name: "age".to_string(),
                    data_type: "Int64".to_string(),
                },
                TerlanPolarsColumnSchema {
                    name: "city".to_string(),
                    data_type: "String".to_string(),
                },
            ]
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_constructs_string_rows_and_empty_frames() {
        let column_names = ["name".to_string(), "city".to_string()];
        let rows = [
            vec!["Ada".to_string(), "London".to_string()],
            vec!["Grace".to_string(), "Arlington".to_string()],
        ];
        let dataframe = from_rows(&column_names, &rows).expect("rows should construct");
        let empty = from_rows(&column_names, &[]).expect("empty rows should construct");

        assert_eq!(height(&dataframe), 2);
        assert_eq!(columns(&dataframe), ["name", "city"]);
        assert_eq!(height(&empty), 0);
        assert_eq!(width(&empty), 2);

        let ragged = from_rows(&column_names, &[vec!["Ada".to_string()]])
            .expect_err("ragged rows should fail");
        assert_eq!(ragged.code(), "invalid_row_width");
        assert!(from_rows(&["name".to_string(), "name".to_string()], &[]).is_err());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_constructs_and_casts_typed_series() {
        let integers = series_from_ints("numbers", &[1, 2, 3]).expect("integer Series");
        let floats = series_from_floats("values", &[1.5, 2.25]).expect("float Series");
        let booleans = series_from_bools("flags", &[true, false]).expect("boolean Series");
        let nullable_strings = series_from_nullable_strings(
            "names",
            &[Some("Ada".to_string()), None, Some("Grace".to_string())],
        )
        .expect("nullable string Series");
        let nullable_integers = series_from_nullable_ints("counts", &[Some(1), None, Some(3)])
            .expect("nullable integer Series");
        let nullable_floats = series_from_nullable_floats("weights", &[Some(1.5), None])
            .expect("nullable float Series");
        let nullable_booleans = series_from_nullable_bools("checks", &[Some(true), None])
            .expect("nullable boolean Series");
        let all_null_integers =
            series_from_nullable_ints("missing", &[None, None]).expect("all-null integer Series");
        let dates = series_from_dates(
            "dates",
            &["2024-01-02".to_string(), "2024-02-03".to_string()],
        )
        .expect("Date Series");
        let nullable_dates =
            series_from_nullable_dates("nullable_dates", &[Some("2024-01-02".to_string()), None])
                .expect("nullable Date Series");
        let datetimes = series_from_datetimes(
            "datetimes",
            &[
                "2024-01-02T03:04:05".to_string(),
                "2024-02-03T04:05:06.123456".to_string(),
            ],
        )
        .expect("Datetime Series");
        let nullable_datetimes = series_from_nullable_datetimes(
            "nullable_datetimes",
            &[Some("2024-01-02T03:04:05".to_string()), None],
        )
        .expect("nullable Datetime Series");
        let list_type = expressions::data_type_list("int64").expect("List data type");
        let nested_list_type =
            expressions::data_type_list(&list_type).expect("nested List data type");
        let array_type = expressions::data_type_array("float32", 3).expect("Array data type");
        let decimal_type = expressions::data_type_decimal(12, 2).expect("Decimal data type");
        let categorical_type = expressions::data_type_categorical().expect("Categorical data type");
        let enum_type = expressions::data_type_enum(&[
            "small".to_string(),
            "medium".to_string(),
            "large".to_string(),
        ])
        .expect("Enum data type");
        let timezone_datetime_type =
            expressions::data_type_datetime("DatetimeMicroseconds", "Europe/Riga")
                .expect("timezone-aware Datetime data type");
        let id_field = expressions::data_type_field("id", "int64").expect("integer Struct field");
        let tags_field =
            expressions::data_type_field("tags", &list_type).expect("nested Struct field");
        let struct_type =
            expressions::data_type_struct(&[id_field, tags_field]).expect("Struct data type");
        let empty_nested_list =
            series_empty("nested", &nested_list_type).expect("empty nested List Series");
        let empty_array = series_empty("array", &array_type).expect("empty Array Series");
        let empty_decimal = series_empty("decimal", &decimal_type).expect("empty Decimal Series");
        let empty_categorical =
            series_empty("category", &categorical_type).expect("empty Categorical Series");
        let empty_enum = series_empty("size", &enum_type).expect("empty Enum Series");
        let empty_timezone_datetime = series_empty("observed_at", &timezone_datetime_type)
            .expect("empty timezone-aware Datetime Series");
        let empty_struct = series_empty("record", &struct_type).expect("empty Struct Series");
        let empty_date = series_empty("dates", "date_type").expect("empty Date Series");
        let empty_canonical_uint128 =
            series_empty("wide_unsigned", "u_int128").expect("Terlan-canonical UInt128 Series");
        let int32 = series_cast(&integers, "int32").expect("typed integer cast");
        let int128 = series_cast(&integers, "int128").expect("Int128 cast");
        let uint128 = series_cast(&integers, "uint128").expect("UInt128 cast");
        let float16 = series_cast(&floats, "float16").expect("Float16 cast");
        let literal = expressions::expr_series(&integers).expect("Series literal expression");
        let literal = expressions::expr_alias(&literal, "literal_numbers")
            .expect("Series literal expression alias");
        let source = series_to_frame(&integers).expect("Series frame");
        let selected = select_exprs(&source, &[literal]).expect("select Series literal");
        let nullable = read_csv(&fixture_path("nullable.csv")).expect("nullable fixture");
        let scores = column_series(&nullable, "score").expect("nullable score Series");
        let score_literal = expressions::expr_series(&scores).expect("nullable Series literal");
        let score_literal = expressions::expr_alias(&score_literal, "literal_scores")
            .expect("nullable Series literal alias");
        let selected_scores =
            select_exprs(&nullable, &[score_literal]).expect("select nullable Series literal");

        assert_eq!(series_data_type(&integers), "Int64");
        assert_eq!(series_data_type(&floats), "Float64");
        assert_eq!(series_data_type(&booleans), "Boolean");
        assert_eq!(series_null_count(&nullable_strings), 1);
        assert_eq!(series_null_count(&nullable_integers), 1);
        assert_eq!(series_null_count(&nullable_floats), 1);
        assert_eq!(series_null_count(&nullable_booleans), 1);
        assert_eq!(series_null_count(&all_null_integers), 2);
        assert_eq!(series_data_type(&all_null_integers), "Int64");
        assert_eq!(series_data_type(&dates), "Date");
        assert_eq!(series_null_count(&nullable_dates), 1);
        assert!(series_data_type(&datetimes).starts_with("Datetime"));
        assert_eq!(series_null_count(&nullable_datetimes), 1);
        assert_eq!(series_len(&empty_nested_list), 0);
        assert_eq!(series_len(&empty_array), 0);
        assert_eq!(series_len(&empty_decimal), 0);
        assert_eq!(series_len(&empty_categorical), 0);
        assert_eq!(series_len(&empty_enum), 0);
        assert_eq!(series_len(&empty_timezone_datetime), 0);
        assert_eq!(series_len(&empty_struct), 0);
        assert!(series_data_type(&empty_nested_list).contains("List"));
        assert!(series_data_type(&empty_array).contains("Array"));
        assert!(series_data_type(&empty_decimal).contains("Decimal"));
        assert!(series_data_type(&empty_categorical).contains("Categorical"));
        assert!(series_data_type(&empty_enum).contains("Enum"));
        assert!(series_data_type(&empty_timezone_datetime).contains("Europe/Riga"));
        assert!(series_data_type(&empty_struct).contains("Struct"));
        assert_eq!(series_data_type(&empty_date), "Date");
        assert_eq!(series_len(&empty_date), 0);
        assert_eq!(series_data_type(&empty_canonical_uint128), "UInt128");
        assert_eq!(series_data_type(&int32), "Int32");
        assert_eq!(series_data_type(&int128), "Int128");
        assert_eq!(series_data_type(&uint128), "UInt128");
        assert_eq!(series_data_type(&float16), "Float16");
        assert_eq!(
            series_values(&integers, 3).expect("integer values"),
            ["1", "2", "3"]
        );
        assert_eq!(columns(&selected), ["literal_numbers"]);
        assert_eq!(
            rows(&selected, 3).expect("Series literal rows"),
            [vec!["1"], vec!["2"], vec!["3"]]
        );
        assert_eq!(
            rows(&selected_scores, 2).expect("nullable Series literal rows"),
            [vec!["10"], vec!["null"]]
        );
        assert_eq!(
            series_from_dates("bad", &["2024-13-99".to_string()])
                .expect_err("invalid date should fail")
                .code(),
            "invalid_date"
        );
        assert_eq!(
            expressions::data_type_decimal(2, 3)
                .expect_err("scale greater than precision should fail")
                .code(),
            "invalid_data_type"
        );
        assert_eq!(
            expressions::data_type_enum(&["duplicate".to_string(), "duplicate".to_string()])
                .expect_err("duplicate enum categories should fail")
                .code(),
            "invalid_data_type"
        );
        let invalid_timezone =
            expressions::data_type_datetime("DatetimeMicroseconds", "Not/A_Timezone")
                .expect("timezone descriptor construction is deferred");
        assert_eq!(
            series_empty("bad_timezone", &invalid_timezone)
                .expect_err("invalid timezone should fail")
                .code(),
            "invalid_data_type"
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_reports_malformed_csv() {
        assert!(read_csv(&fixture_path("malformed.csv")).is_err());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_materializes_bounded_rows_and_nulls() {
        let df = read_csv(&fixture_path("nullable.csv")).expect("fixture should read");

        assert_eq!(
            rows(&df, 1).expect("first row"),
            [vec!["Ada".to_string(), "10".to_string()]]
        );
        assert_eq!(
            rows(&df, 10).expect("all rows"),
            [
                vec!["Ada".to_string(), "10".to_string()],
                vec!["Grace".to_string(), "null".to_string()],
            ]
        );
        assert!(rows(&df, 0).expect("zero limit").is_empty());
        assert_eq!(
            rows(&df, MAX_MATERIALIZED_ROWS + 1)
                .expect_err("oversized limit should fail")
                .code(),
            "row_limit_too_large"
        );
    }

    /// Verifies real Polars linkage projects selected columns.
    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_selects_columns() {
        let df = read_csv(&fixture_path("people.csv")).expect("fixture should read");
        let selected = select(&df, &["name".to_string(), "city".to_string()])
            .expect("selection should succeed");

        assert_eq!(height(&selected), 2);
        assert_eq!(width(&selected), 2);
        assert_eq!(columns(&selected), ["name", "city"]);
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_filters_scalar_equality_and_reports_invalid_predicates() {
        let df = read_csv(&fixture_path("people.csv")).expect("fixture should read");
        let filtered = filter_eq(&df, "age", &TerlanPolarsScalar::Int(36))
            .expect("integer filter should succeed");

        assert_eq!(height(&filtered), 1);
        assert!(filter_eq(
            &df,
            "missing",
            &TerlanPolarsScalar::String("Ada".to_string())
        )
        .is_err());
        assert!(filter_eq(&df, "age", &TerlanPolarsScalar::String("36".to_string())).is_err());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_sorts_by_column_and_reports_missing_columns() {
        let df = read_csv(&fixture_path("people.csv")).expect("fixture should read");
        let sorted = sort_by(&df, "age", true).expect("descending sort should succeed");
        let first = head(&sorted, 1).expect("head should succeed");
        let oldest = filter_eq(&first, "age", &TerlanPolarsScalar::Int(85))
            .expect("oldest-row filter should succeed");

        assert_eq!(height(&oldest), 1);
        assert!(sort_by(&df, "missing", false).is_err());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_groups_and_counts_rows() {
        let df = read_csv(&fixture_path("groups.csv")).expect("fixture should read");
        let grouped = group_count(&df, &["city".to_string()]).expect("grouping should succeed");

        assert_eq!(height(&grouped), 2);
        assert_eq!(columns(&grouped), ["city", "count"]);
        let counts = grouped
            .inner
            .column("count")
            .expect("count column")
            .u32()
            .expect("count should be UInt32")
            .into_no_null_iter()
            .collect::<Vec<_>>();
        assert_eq!(counts, [2, 1]);
        let empty = group_count(&df, &[]).expect_err("empty keys should fail");
        assert_eq!(empty.code(), "invalid_group_keys");
        assert!(group_count(&df, &["missing".to_string()]).is_err());
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_builds_and_collects_lazy_queries_with_deferred_errors() {
        let df = read_csv(&fixture_path("people.csv")).expect("fixture should read");
        let plan = to_lazy(&df);
        let filtered = lazy_filter_eq(&plan, "age", &TerlanPolarsScalar::Int(85));
        let selected = lazy_select(&filtered, &["name".to_string(), "age".to_string()]);
        let collected = collect_lazy(&selected).expect("lazy query should collect");

        assert_eq!(height(&collected), 1);
        assert_eq!(columns(&collected), ["name", "age"]);

        let invalid = lazy_select(&plan, &["missing".to_string()]);
        assert!(collect_lazy(&invalid).is_err());
    }

    /// Verifies real Polars linkage returns the first requested rows.
    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_heads_rows() {
        let df = read_csv(&fixture_path("people.csv")).expect("fixture should read");
        let headed = head(&df, 1).expect("head should succeed");

        assert_eq!(height(&headed), 1);
        assert_eq!(width(&headed), 3);
        assert_eq!(columns(&headed), ["name", "age", "city"]);
    }

    /// Verifies real Polars errors are converted to stable adapter errors.
    #[cfg(feature = "real-polars")]
    #[test]
    fn real_polars_reports_missing_file() {
        let err = read_csv(&fixture_path("missing.csv")).expect_err("missing file should fail");

        assert_eq!(err.code(), "csv_read_error");
        assert!(err.message().contains("missing.csv"));
    }
}
