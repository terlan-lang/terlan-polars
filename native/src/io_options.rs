//! Validated, immutable I/O option descriptors for the Terlan package API.

use serde::{Deserialize, Serialize};

use crate::{TerlanPolarsDataFrame, TerlanPolarsError};

const NONE: i64 = -1;

/// Options used to read one CSV file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CsvReadOptions {
    has_header: bool,
    separator: u8,
    quote_char: Option<u8>,
    eol_char: u8,
    encoding: String,
    null_mode: String,
    null_values: Vec<String>,
    null_columns: Vec<String>,
    missing_is_null: bool,
    truncate_ragged_lines: bool,
    comment_prefix: Option<String>,
    try_parse_dates: bool,
    decimal_comma: bool,
    rechunk: bool,
    n_threads: Option<usize>,
    low_memory: bool,
    n_rows: Option<usize>,
    columns: Vec<String>,
    projection: Vec<usize>,
    schema_names: Vec<String>,
    schema_types: Vec<String>,
    schema_overwrite_names: Vec<String>,
    schema_overwrite_types: Vec<String>,
    dtype_overwrite: Vec<String>,
    chunk_size: usize,
    skip_rows: usize,
    skip_lines: usize,
    skip_rows_after_header: usize,
    infer_schema_length: Option<usize>,
    raise_if_empty: bool,
    ignore_errors: bool,
    row_index_name: Option<String>,
    row_index_offset: u32,
    #[serde(default)]
    column_names_overwrite: Vec<String>,
    #[serde(default = "default_infer_schema_files")]
    infer_schema_files: usize,
    #[serde(default = "default_raise_policy")]
    extra_columns_policy: String,
    #[serde(default = "default_raise_policy")]
    missing_columns_policy: String,
}

fn default_infer_schema_files() -> usize {
    10
}

fn default_raise_policy() -> String {
    "raise".into()
}

/// Options used to write one CSV file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CsvWriteOptions {
    include_bom: bool,
    include_header: bool,
    batch_size: usize,
    date_format: Option<String>,
    time_format: Option<String>,
    datetime_format: Option<String>,
    float_scientific: Option<bool>,
    float_precision: Option<usize>,
    decimal_comma: bool,
    separator: u8,
    quote_char: u8,
    null_value: String,
    line_terminator: String,
    quote_style: String,
    n_threads: Option<usize>,
}

/// Options used to read JSON or newline-delimited JSON.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonReadOptions {
    lines: bool,
    infer_schema_length: Option<usize>,
    batch_size: usize,
    ignore_errors: bool,
    projection: Vec<String>,
    schema_names: Vec<String>,
    schema_types: Vec<String>,
    schema_overwrite_names: Vec<String>,
    schema_overwrite_types: Vec<String>,
    rechunk: bool,
}

/// Options used to read one Parquet file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParquetReadOptions {
    slice_offset: Option<usize>,
    slice_length: Option<usize>,
    columns: Vec<String>,
    projection: Vec<usize>,
    row_index_name: Option<String>,
    row_index_offset: u32,
    low_memory: bool,
    parallel: String,
    rechunk: bool,
    include_file_path_name: Option<String>,
}

/// Options used to write one Parquet file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParquetWriteOptions {
    compression: String,
    compression_level: Option<i32>,
    statistics: String,
    row_group_size: Option<usize>,
    data_page_size: Option<usize>,
    parallel: bool,
    metadata_keys: Vec<String>,
    metadata_values: Vec<String>,
}

/// Options used to read one Arrow IPC file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcReadOptions {
    n_rows: Option<usize>,
    columns: Vec<String>,
    projection: Vec<usize>,
    row_index_name: Option<String>,
    row_index_offset: u32,
    memory_mapped: bool,
    rechunk: bool,
    include_file_path_name: Option<String>,
}

/// Options used to write one Arrow IPC file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpcWriteOptions {
    compression: String,
    compatibility: String,
    record_batch_size: Option<usize>,
    record_batch_statistics: bool,
    parallel: bool,
    metadata_keys: Vec<String>,
    metadata_values: Vec<String>,
}

/// Creates and serializes validated CSV reader options.
#[allow(clippy::too_many_arguments)]
pub fn csv_read_options(
    has_header: bool,
    separator: &str,
    quote_char: &str,
    eol_char: &str,
    encoding: &str,
    null_mode: &str,
    null_values: &[String],
    null_columns: &[String],
    missing_is_null: bool,
    truncate_ragged_lines: bool,
    comment_prefix: &str,
    try_parse_dates: bool,
    decimal_comma: bool,
    rechunk: bool,
    n_threads: i64,
    low_memory: bool,
    n_rows: i64,
    columns: &[String],
    projection: &[i64],
    schema_names: &[String],
    schema_types: &[String],
    schema_overwrite_names: &[String],
    schema_overwrite_types: &[String],
    dtype_overwrite: &[String],
    chunk_size: i64,
    skip_rows: i64,
    skip_lines: i64,
    skip_rows_after_header: i64,
    infer_schema_length: i64,
    raise_if_empty: bool,
    ignore_errors: bool,
    row_index_name: &str,
    row_index_offset: i64,
) -> Result<String, TerlanPolarsError> {
    validate_parallel_lists("schema", schema_names, schema_types)?;
    validate_parallel_lists(
        "schema overwrite",
        schema_overwrite_names,
        schema_overwrite_types,
    )?;
    validate_null_values(null_mode, null_values, null_columns)?;
    let options = CsvReadOptions {
        has_header,
        separator: required_ascii_byte("CSV separator", separator)?,
        quote_char: optional_ascii_byte("CSV quote character", quote_char)?,
        eol_char: required_ascii_byte("CSV end-of-line character", eol_char)?,
        encoding: one_of("CSV encoding", encoding, &["utf8", "lossy_utf8"])?,
        null_mode: one_of(
            "CSV null mode",
            null_mode,
            &["none", "single", "all", "named"],
        )?,
        null_values: null_values.to_vec(),
        null_columns: null_columns.to_vec(),
        missing_is_null,
        truncate_ragged_lines,
        comment_prefix: optional_text("CSV comment prefix", comment_prefix, false)?,
        try_parse_dates,
        decimal_comma,
        rechunk,
        n_threads: optional_usize("CSV thread count", n_threads, true)?,
        low_memory,
        n_rows: optional_usize("CSV row limit", n_rows, true)?,
        columns: columns.to_vec(),
        projection: indexes("CSV projection", projection)?,
        schema_names: schema_names.to_vec(),
        schema_types: schema_types.to_vec(),
        schema_overwrite_names: schema_overwrite_names.to_vec(),
        schema_overwrite_types: schema_overwrite_types.to_vec(),
        dtype_overwrite: dtype_overwrite.to_vec(),
        chunk_size: positive_usize("CSV chunk size", chunk_size)?,
        skip_rows: nonnegative_usize("CSV skipped rows", skip_rows)?,
        skip_lines: nonnegative_usize("CSV skipped lines", skip_lines)?,
        skip_rows_after_header: nonnegative_usize(
            "CSV skipped rows after header",
            skip_rows_after_header,
        )?,
        infer_schema_length: optional_usize(
            "CSV schema inference length",
            infer_schema_length,
            true,
        )?,
        raise_if_empty,
        ignore_errors,
        row_index_name: optional_text("CSV row index name", row_index_name, false)?,
        row_index_offset: u32_value("CSV row index offset", row_index_offset)?,
        column_names_overwrite: Vec::new(),
        infer_schema_files: default_infer_schema_files(),
        extra_columns_policy: default_raise_policy(),
        missing_columns_policy: default_raise_policy(),
    };
    encode(&options)
}

/// Returns CSV reader options with explicit replacement column names.
pub fn csv_read_options_with_column_names(
    descriptor: &str,
    names: &[String],
) -> Result<String, TerlanPolarsError> {
    if names.iter().any(String::is_empty) {
        return invalid("CSV replacement column names cannot be empty");
    }
    let mut options: CsvReadOptions = decode(descriptor)?;
    options.column_names_overwrite = names.to_vec();
    encode(&options)
}

/// Returns CSV reader options with the Polars 2.0 multi-file inference limit.
pub fn csv_read_options_with_infer_schema_files(
    descriptor: &str,
    count: i64,
) -> Result<String, TerlanPolarsError> {
    options_infer_schema_files(
        descriptor,
        positive_usize("CSV schema inference file count", count)?,
    )
}

/// Returns CSV reader options with Polars 2 extra/missing column policies.
pub fn csv_read_options_with_column_policies(
    descriptor: &str,
    extra_columns: &str,
    missing_columns: &str,
) -> Result<String, TerlanPolarsError> {
    if !matches!(extra_columns, "raise" | "ignore") {
        return invalid("CSV extra-column policy must be raise or ignore");
    }
    if !matches!(missing_columns, "raise" | "insert") {
        return invalid("CSV missing-column policy must be raise or insert");
    }
    let mut options: CsvReadOptions = decode(descriptor)?;
    options.extra_columns_policy = extra_columns.into();
    options.missing_columns_policy = missing_columns.into();
    if extra_columns == "ignore" {
        options.truncate_ragged_lines = true;
    }
    encode(&options)
}

fn options_infer_schema_files(descriptor: &str, count: usize) -> Result<String, TerlanPolarsError> {
    let mut options: CsvReadOptions = decode(descriptor)?;
    options.infer_schema_files = count;
    encode(&options)
}

/// Creates and serializes validated CSV writer options.
#[allow(clippy::too_many_arguments)]
pub fn csv_write_options(
    include_bom: bool,
    include_header: bool,
    batch_size: i64,
    date_format: &str,
    time_format: &str,
    datetime_format: &str,
    float_scientific: i64,
    float_precision: i64,
    decimal_comma: bool,
    separator: &str,
    quote_char: &str,
    null_value: &str,
    line_terminator: &str,
    quote_style: &str,
    n_threads: i64,
) -> Result<String, TerlanPolarsError> {
    if line_terminator.is_empty() || line_terminator.contains('\0') {
        return invalid("CSV line terminator must be non-empty and NUL-free");
    }
    let options = CsvWriteOptions {
        include_bom,
        include_header,
        batch_size: positive_usize("CSV writer batch size", batch_size)?,
        date_format: optional_text("CSV date format", date_format, false)?,
        time_format: optional_text("CSV time format", time_format, false)?,
        datetime_format: optional_text("CSV datetime format", datetime_format, false)?,
        float_scientific: optional_bool("CSV scientific-float mode", float_scientific)?,
        float_precision: optional_usize("CSV float precision", float_precision, true)?,
        decimal_comma,
        separator: required_ascii_byte("CSV separator", separator)?,
        quote_char: required_ascii_byte("CSV quote character", quote_char)?,
        null_value: bounded_text("CSV null value", null_value)?,
        line_terminator: line_terminator.to_owned(),
        quote_style: one_of(
            "CSV quote style",
            quote_style,
            &["necessary", "always", "non_numeric", "never"],
        )?,
        n_threads: optional_usize("CSV writer thread count", n_threads, true)?,
    };
    encode(&options)
}

/// Creates and serializes validated JSON reader options.
#[allow(clippy::too_many_arguments)]
pub fn json_read_options(
    lines: bool,
    infer_schema_length: i64,
    batch_size: i64,
    ignore_errors: bool,
    projection: &[String],
    schema_names: &[String],
    schema_types: &[String],
    schema_overwrite_names: &[String],
    schema_overwrite_types: &[String],
    rechunk: bool,
) -> Result<String, TerlanPolarsError> {
    validate_parallel_lists("JSON schema", schema_names, schema_types)?;
    validate_parallel_lists(
        "JSON schema overwrite",
        schema_overwrite_names,
        schema_overwrite_types,
    )?;
    if ignore_errors && !lines {
        return invalid("JSON ignore_errors is only valid for newline-delimited JSON");
    }
    let options = JsonReadOptions {
        lines,
        infer_schema_length: optional_usize(
            "JSON schema inference length",
            infer_schema_length,
            false,
        )?,
        batch_size: positive_usize("JSON batch size", batch_size)?,
        ignore_errors,
        projection: projection.to_vec(),
        schema_names: schema_names.to_vec(),
        schema_types: schema_types.to_vec(),
        schema_overwrite_names: schema_overwrite_names.to_vec(),
        schema_overwrite_types: schema_overwrite_types.to_vec(),
        rechunk,
    };
    encode(&options)
}

/// Creates and serializes validated Parquet reader options.
#[allow(clippy::too_many_arguments)]
pub fn parquet_read_options(
    slice_offset: i64,
    slice_length: i64,
    columns: &[String],
    projection: &[i64],
    row_index_name: &str,
    row_index_offset: i64,
    low_memory: bool,
    parallel: &str,
    rechunk: bool,
    include_file_path_name: &str,
) -> Result<String, TerlanPolarsError> {
    let offset = optional_usize("Parquet slice offset", slice_offset, true)?;
    let length = optional_usize("Parquet slice length", slice_length, false)?;
    if offset.is_some() != length.is_some() {
        return invalid("Parquet slice offset and length must be set together");
    }
    let options = ParquetReadOptions {
        slice_offset: offset,
        slice_length: length,
        columns: columns.to_vec(),
        projection: indexes("Parquet projection", projection)?,
        row_index_name: optional_text("Parquet row index name", row_index_name, false)?,
        row_index_offset: u32_value("Parquet row index offset", row_index_offset)?,
        low_memory,
        parallel: one_of(
            "Parquet parallel strategy",
            parallel,
            &["auto", "none", "columns", "row_groups", "prefiltered"],
        )?,
        rechunk,
        include_file_path_name: optional_text(
            "Parquet source-path column name",
            include_file_path_name,
            false,
        )?,
    };
    encode(&options)
}

/// Creates and serializes validated Parquet writer options.
#[allow(clippy::too_many_arguments)]
pub fn parquet_write_options(
    compression: &str,
    compression_level: i64,
    statistics: &str,
    row_group_size: i64,
    data_page_size: i64,
    parallel: bool,
    metadata_keys: &[String],
    metadata_values: &[String],
) -> Result<String, TerlanPolarsError> {
    validate_parallel_lists("Parquet metadata", metadata_keys, metadata_values)?;
    let compression = one_of(
        "Parquet compression",
        compression,
        &[
            "uncompressed",
            "snappy",
            "gzip",
            "brotli",
            "zstd",
            "lz4_raw",
        ],
    )?;
    let level = optional_i32("Parquet compression level", compression_level)?;
    validate_compression_level(&compression, level)?;
    let options = ParquetWriteOptions {
        compression,
        compression_level: level,
        statistics: one_of(
            "Parquet statistics mode",
            statistics,
            &["none", "chunk", "page", "full"],
        )?,
        row_group_size: optional_usize("Parquet row group size", row_group_size, false)?,
        data_page_size: optional_usize("Parquet data page size", data_page_size, false)?,
        parallel,
        metadata_keys: metadata_keys.to_vec(),
        metadata_values: metadata_values.to_vec(),
    };
    encode(&options)
}

/// Creates and serializes validated Arrow IPC reader options.
pub fn ipc_read_options(
    n_rows: i64,
    columns: &[String],
    projection: &[i64],
    row_index_name: &str,
    row_index_offset: i64,
    memory_mapped: bool,
    rechunk: bool,
    include_file_path_name: &str,
) -> Result<String, TerlanPolarsError> {
    let options = IpcReadOptions {
        n_rows: optional_usize("IPC row limit", n_rows, true)?,
        columns: columns.to_vec(),
        projection: indexes("IPC projection", projection)?,
        row_index_name: optional_text("IPC row index name", row_index_name, false)?,
        row_index_offset: u32_value("IPC row index offset", row_index_offset)?,
        memory_mapped,
        rechunk,
        include_file_path_name: optional_text(
            "IPC source-path column name",
            include_file_path_name,
            false,
        )?,
    };
    encode(&options)
}

/// Creates and serializes validated Arrow IPC writer options.
#[allow(clippy::too_many_arguments)]
pub fn ipc_write_options(
    compression: &str,
    compatibility: &str,
    record_batch_size: i64,
    record_batch_statistics: bool,
    parallel: bool,
    metadata_keys: &[String],
    metadata_values: &[String],
) -> Result<String, TerlanPolarsError> {
    validate_parallel_lists("IPC metadata", metadata_keys, metadata_values)?;
    let options = IpcWriteOptions {
        compression: one_of("IPC compression", compression, &["none", "lz4", "zstd"])?,
        compatibility: one_of(
            "IPC compatibility level",
            compatibility,
            &["oldest", "newest"],
        )?,
        record_batch_size: optional_usize("IPC record batch size", record_batch_size, false)?,
        record_batch_statistics,
        parallel,
        metadata_keys: metadata_keys.to_vec(),
        metadata_values: metadata_values.to_vec(),
    };
    encode(&options)
}

/// Reads a CSV file using one validated option descriptor.
#[cfg(feature = "real-polars")]
pub fn read_csv_with_options(
    path: &str,
    descriptor: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{
        CommentPrefix, CsvEncoding, CsvParseOptions, ExtraColumnsPolicy, LazyCsvReader,
        LazyFileListReader, MissingColumnsPolicy, NullValues, PlRefPath, PlSmallStr, RowIndex,
    };
    use std::num::NonZeroUsize;

    let value: CsvReadOptions = decode(descriptor)?;
    let null_values = match value.null_mode.as_str() {
        "none" => None,
        "single" => Some(NullValues::AllColumnsSingle(
            value.null_values[0].as_str().into(),
        )),
        "all" => Some(NullValues::AllColumns(
            value
                .null_values
                .iter()
                .map(|item| item.as_str().into())
                .collect(),
        )),
        "named" => Some(NullValues::Named(
            value
                .null_columns
                .iter()
                .zip(value.null_values.iter())
                .map(|(column, item)| (column.as_str().into(), item.as_str().into()))
                .collect(),
        )),
        _ => return invalid("CSV option descriptor contains an invalid null mode"),
    };
    let parse_options = CsvParseOptions::default()
        .with_separator(value.separator)
        .with_quote_char(value.quote_char)
        .with_eol_char(value.eol_char)
        .with_encoding(match value.encoding.as_str() {
            "utf8" => CsvEncoding::Utf8,
            "lossy_utf8" => CsvEncoding::LossyUtf8,
            _ => return invalid("CSV option descriptor contains an invalid encoding"),
        })
        .with_null_values(null_values)
        .with_missing_is_null(value.missing_is_null)
        .with_truncate_ragged_lines(value.truncate_ragged_lines)
        .with_comment_prefix(
            value
                .comment_prefix
                .as_deref()
                .map(CommentPrefix::new_from_str),
        )
        .with_try_parse_dates(value.try_parse_dates)
        .with_decimal_comma(value.decimal_comma);
    let extra_columns_policy = match value.extra_columns_policy.as_str() {
        "raise" => ExtraColumnsPolicy::Raise,
        "ignore" => ExtraColumnsPolicy::Ignore,
        _ => return invalid("CSV option descriptor contains an invalid extra-column policy"),
    };
    let missing_columns_policy = match value.missing_columns_policy.as_str() {
        "raise" => MissingColumnsPolicy::Raise,
        "insert" => MissingColumnsPolicy::Insert,
        _ => return invalid("CSV option descriptor contains an invalid missing-column policy"),
    };
    let mut reader = LazyCsvReader::new(PlRefPath::new(path))
        .map_parse_options(|_| parse_options.clone())
        .with_n_threads(value.n_threads)
        .with_low_memory(value.low_memory)
        .with_n_rows(value.n_rows)
        .with_has_header(value.has_header)
        .with_chunk_size(value.chunk_size)
        .with_skip_rows(value.skip_rows)
        .with_skip_lines(value.skip_lines)
        .with_skip_rows_after_header(value.skip_rows_after_header)
        .with_infer_schema_length(value.infer_schema_length)
        .with_infer_schema_files(NonZeroUsize::new(value.infer_schema_files).ok_or_else(|| {
            TerlanPolarsError::new(
                "invalid_io_options",
                "CSV schema inference file count must be greater than zero",
            )
        })?)
        .with_raise_if_empty(value.raise_if_empty)
        .with_ignore_errors(value.ignore_errors)
        .with_extra_columns_policy(extra_columns_policy)
        .with_missing_columns_policy(Some(missing_columns_policy));
    if !value.column_names_overwrite.is_empty() {
        reader = reader.with_column_names_overwrite(
            value
                .column_names_overwrite
                .iter()
                .map(|name| PlSmallStr::from_str(name))
                .collect::<Vec<_>>()
                .into(),
        );
    }
    if !value.schema_names.is_empty() {
        reader = reader.with_schema(Some(schema(&value.schema_names, &value.schema_types)?));
    }
    if !value.schema_overwrite_names.is_empty() {
        reader = reader.with_dtype_overwrite(Some(schema(
            &value.schema_overwrite_names,
            &value.schema_overwrite_types,
        )?));
    }
    if !value.dtype_overwrite.is_empty() {
        reader = reader.with_dtype_overwrite_by_position(Some(std::sync::Arc::new(
            value
                .dtype_overwrite
                .iter()
                .map(|data_type| crate::expressions::parse_data_type(data_type))
                .collect::<Result<Vec<_>, _>>()?,
        )));
    }
    if let Some(name) = value.row_index_name.as_ref() {
        reader = reader.with_row_index(Some(RowIndex {
            name: name.as_str().into(),
            offset: value.row_index_offset,
        }));
    }
    let mut inner = reader.finish()?.collect()?;
    if !value.columns.is_empty() {
        let mut names = value
            .row_index_name
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        names.extend(value.columns.iter().map(String::as_str));
        inner = inner.select(names)?;
    }
    if !value.projection.is_empty() {
        let source_names = inner
            .get_column_names()
            .into_iter()
            .filter(|name| value.row_index_name.as_deref() != Some(name.as_str()))
            .map(|name| name.as_str().to_string())
            .collect::<Vec<_>>();
        let mut names = value.row_index_name.clone().into_iter().collect::<Vec<_>>();
        names.extend(
            value
                .projection
                .iter()
                .map(|index| {
                    source_names.get(*index).cloned().ok_or_else(|| {
                        TerlanPolarsError::new(
                            "invalid_io_options",
                            format!("CSV projection index {index} is out of bounds"),
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        inner = inner.select(names.iter().map(String::as_str))?;
    }
    if value.rechunk {
        inner.rechunk_mut_par();
    }
    Ok(TerlanPolarsDataFrame { inner })
}

/// Writes a CSV file using one validated option descriptor.
#[cfg(feature = "real-polars")]
pub fn write_csv_with_options(
    frame: &TerlanPolarsDataFrame,
    path: &str,
    descriptor: &str,
) -> Result<(), TerlanPolarsError> {
    use std::num::NonZeroUsize;

    use polars::prelude::{CsvWriter, QuoteStyle, SerWriter};

    let value: CsvWriteOptions = decode(descriptor)?;
    let mut inner = frame.inner.clone();
    let mut writer = CsvWriter::new(std::fs::File::create(path)?)
        .include_bom(value.include_bom)
        .include_header(value.include_header)
        .with_batch_size(
            NonZeroUsize::new(value.batch_size)
                .ok_or_else(|| io_error("CSV writer batch size cannot be zero"))?,
        )
        .with_decimal_comma(value.decimal_comma)
        .with_separator(value.separator)
        .with_quote_char(value.quote_char)
        .with_null_value(value.null_value.into())
        .with_line_terminator(value.line_terminator.into())
        .with_quote_style(match value.quote_style.as_str() {
            "necessary" => QuoteStyle::Necessary,
            "always" => QuoteStyle::Always,
            "non_numeric" => QuoteStyle::NonNumeric,
            "never" => QuoteStyle::Never,
            _ => return invalid("CSV option descriptor contains an invalid quote style"),
        });
    if let Some(n_threads) = value.n_threads {
        writer = writer.n_threads(n_threads);
    }
    if let Some(format) = value.date_format {
        writer = writer.with_date_format(Some(format.into()));
    }
    if let Some(format) = value.time_format {
        writer = writer.with_time_format(Some(format.into()));
    }
    if let Some(format) = value.datetime_format {
        writer = writer.with_datetime_format(Some(format.into()));
    }
    if let Some(scientific) = value.float_scientific {
        writer = writer.with_float_scientific(Some(scientific));
    }
    if let Some(precision) = value.float_precision {
        writer = writer.with_float_precision(Some(precision));
    }
    writer.finish(&mut inner)?;
    Ok(())
}

/// Reads JSON using one validated option descriptor.
#[cfg(feature = "real-polars")]
pub fn read_json_with_options(
    path: &str,
    descriptor: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use std::num::NonZeroUsize;

    use polars::prelude::{JsonFormat, JsonReader, PlSmallStr, SerReader};

    let value: JsonReadOptions = decode(descriptor)?;
    let schema_overwrite = (!value.schema_overwrite_names.is_empty())
        .then(|| schema(&value.schema_overwrite_names, &value.schema_overwrite_types))
        .transpose()?;
    let mut reader = JsonReader::new(std::fs::File::open(path)?)
        .infer_schema_len(value.infer_schema_length.and_then(NonZeroUsize::new))
        .with_batch_size(
            NonZeroUsize::new(value.batch_size)
                .ok_or_else(|| io_error("JSON batch size cannot be zero"))?,
        )
        .with_ignore_errors(value.ignore_errors)
        .with_projection((!value.projection.is_empty()).then(|| {
            value
                .projection
                .iter()
                .map(|name| PlSmallStr::from_str(name))
                .collect()
        }))
        .with_json_format(if value.lines {
            JsonFormat::JsonLines
        } else {
            JsonFormat::Json
        })
        .set_rechunk(value.rechunk);
    if !value.schema_names.is_empty() {
        reader = reader.with_schema(schema(&value.schema_names, &value.schema_types)?);
    }
    if let Some(overwrite) = schema_overwrite.as_deref() {
        reader = reader.with_schema_overwrite(overwrite);
    }
    Ok(TerlanPolarsDataFrame {
        inner: reader.finish()?,
    })
}

/// Reads Parquet using one validated option descriptor.
#[cfg(feature = "real-polars")]
pub fn read_parquet_with_options(
    path: &str,
    descriptor: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{ParallelStrategy, ParquetReader, RowIndex, SerReader};

    let value: ParquetReadOptions = decode(descriptor)?;
    let mut reader = ParquetReader::new(std::fs::File::open(path)?)
        .set_low_memory(value.low_memory)
        .read_parallel(match value.parallel.as_str() {
            "auto" => ParallelStrategy::Auto,
            "none" => ParallelStrategy::None,
            "columns" => ParallelStrategy::Columns,
            "row_groups" => ParallelStrategy::RowGroups,
            "prefiltered" => ParallelStrategy::Prefiltered,
            _ => return invalid("Parquet option descriptor contains an invalid parallel mode"),
        })
        .with_slice(value.slice_offset.zip(value.slice_length))
        .with_columns((!value.columns.is_empty()).then_some(value.columns))
        .with_projection((!value.projection.is_empty()).then_some(value.projection))
        .set_rechunk(value.rechunk);
    if let Some(name) = value.row_index_name {
        reader = reader.with_row_index(Some(RowIndex {
            name: name.into(),
            offset: value.row_index_offset,
        }));
    }
    if let Some(name) = value.include_file_path_name {
        reader = reader.with_include_file_path(Some((name.into(), path.into())));
    }
    Ok(TerlanPolarsDataFrame {
        inner: reader.finish()?,
    })
}

/// Writes Parquet using one validated option descriptor.
#[cfg(feature = "real-polars")]
pub fn write_parquet_with_options(
    frame: &TerlanPolarsDataFrame,
    path: &str,
    descriptor: &str,
) -> Result<(), TerlanPolarsError> {
    use polars::prelude::{KeyValueMetadata, ParquetCompression, ParquetWriter};
    use polars_parquet::write::{BrotliLevel, GzipLevel, StatisticsOptions, ZstdLevel};

    let value: ParquetWriteOptions = decode(descriptor)?;
    let compression = match value.compression.as_str() {
        "uncompressed" => ParquetCompression::Uncompressed,
        "snappy" => ParquetCompression::Snappy,
        "gzip" => ParquetCompression::Gzip(
            value
                .compression_level
                .map(|level| {
                    u8::try_from(level)
                        .map_err(|_| io_error("gzip level must fit in UInt8"))
                        .and_then(|level| {
                            GzipLevel::try_new(level).map_err(|error| io_error(error.to_string()))
                        })
                })
                .transpose()?,
        ),
        "brotli" => ParquetCompression::Brotli(
            value
                .compression_level
                .map(|level| {
                    u32::try_from(level)
                        .map_err(|_| io_error("brotli level cannot be negative"))
                        .and_then(|level| {
                            BrotliLevel::try_new(level).map_err(|error| io_error(error.to_string()))
                        })
                })
                .transpose()?,
        ),
        "zstd" => ParquetCompression::Zstd(
            value
                .compression_level
                .map(ZstdLevel::try_new)
                .transpose()
                .map_err(|error| io_error(error.to_string()))?,
        ),
        "lz4_raw" => ParquetCompression::Lz4Raw,
        _ => return invalid("Parquet option descriptor contains an invalid compression mode"),
    };
    let statistics = match value.statistics.as_str() {
        "none" => StatisticsOptions {
            min_value: false,
            max_value: false,
            distinct_count: false,
            null_count: false,
            binary_statistics_truncate_length: None,
        },
        "chunk" => StatisticsOptions::default(),
        "page" => StatisticsOptions {
            binary_statistics_truncate_length: Some(64),
            ..StatisticsOptions::default()
        },
        "full" => StatisticsOptions {
            distinct_count: true,
            binary_statistics_truncate_length: None,
            ..StatisticsOptions::default()
        },
        _ => return invalid("Parquet option descriptor contains an invalid statistics mode"),
    };
    let metadata = (!value.metadata_keys.is_empty()).then(|| {
        KeyValueMetadata::from_static(
            value
                .metadata_keys
                .into_iter()
                .zip(value.metadata_values)
                .collect(),
        )
    });
    let mut inner = frame.inner.clone();
    ParquetWriter::new(std::fs::File::create(path)?)
        .with_compression(compression)
        .with_statistics(statistics)
        .with_row_group_size(value.row_group_size)
        .with_data_page_size(value.data_page_size)
        .set_parallel(value.parallel)
        .with_key_value_metadata(metadata)
        .finish(&mut inner)?;
    Ok(())
}

/// Reads Arrow IPC using one validated option descriptor.
#[cfg(feature = "real-polars")]
pub fn read_ipc_with_options(
    path: &str,
    descriptor: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{IpcReader, RowIndex, SerReader};

    let value: IpcReadOptions = decode(descriptor)?;
    if value.memory_mapped {
        return invalid(
            "IPC memory mapping is unavailable because Polars 0.55 requires an unsafe reader API",
        );
    }
    let mut reader = IpcReader::new(std::fs::File::open(path)?)
        .with_n_rows(value.n_rows)
        .with_columns((!value.columns.is_empty()).then_some(value.columns))
        .with_projection((!value.projection.is_empty()).then_some(value.projection))
        .set_rechunk(value.rechunk);
    if let Some(name) = value.row_index_name {
        reader = reader.with_row_index(Some(RowIndex {
            name: name.into(),
            offset: value.row_index_offset,
        }));
    }
    if let Some(name) = value.include_file_path_name {
        reader = reader.with_include_file_path(Some((name.into(), path.into())));
    }
    Ok(TerlanPolarsDataFrame {
        inner: reader.finish()?,
    })
}

/// Writes Arrow IPC using one validated option descriptor.
#[cfg(feature = "real-polars")]
pub fn write_ipc_with_options(
    frame: &TerlanPolarsDataFrame,
    path: &str,
    descriptor: &str,
) -> Result<(), TerlanPolarsError> {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use polars::prelude::{CompatLevel, IpcCompression, IpcWriter, SerWriter};

    let value: IpcWriteOptions = decode(descriptor)?;
    let mut writer = IpcWriter::new(std::fs::File::create(path)?)
        .with_compression(match value.compression.as_str() {
            "none" => None,
            "lz4" => Some(IpcCompression::LZ4),
            "zstd" => Some(IpcCompression::ZSTD(Default::default())),
            _ => return invalid("IPC option descriptor contains an invalid compression mode"),
        })
        .with_compat_level(match value.compatibility.as_str() {
            "oldest" => CompatLevel::oldest(),
            "newest" => CompatLevel::newest(),
            _ => return invalid("IPC option descriptor contains an invalid compatibility level"),
        })
        .with_record_batch_size(value.record_batch_size)
        .with_record_batch_statistics(value.record_batch_statistics)
        .with_parallel(value.parallel);
    if !value.metadata_keys.is_empty() {
        let metadata = value
            .metadata_keys
            .into_iter()
            .zip(value.metadata_values)
            .map(|(key, value)| (key.into(), value.into()))
            .collect::<BTreeMap<_, _>>();
        writer.set_custom_schema_metadata(Arc::new(metadata));
    }
    let mut inner = frame.inner.clone();
    writer.finish(&mut inner)?;
    Ok(())
}

/// Returns Arrow IPC schema custom metadata as deterministic JSON.
#[cfg(feature = "real-polars")]
pub fn ipc_custom_metadata(path: &str) -> Result<String, TerlanPolarsError> {
    use polars::prelude::{IpcReader, SerReader};

    let mut reader = IpcReader::new(std::fs::File::open(path)?);
    let metadata = reader.custom_metadata()?;
    let values = metadata
        .as_deref()
        .map(|metadata| {
            metadata
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    serde_json::to_string(&values)
        .map_err(|error| io_error(format!("cannot encode IPC metadata: {error}")))
}

/// Returns stable Parquet file metadata as deterministic JSON.
#[cfg(feature = "real-polars")]
pub fn parquet_metadata(path: &str) -> Result<String, TerlanPolarsError> {
    use polars::prelude::{ParquetReader, SerReader};

    let mut reader = ParquetReader::new(std::fs::File::open(path)?);
    let metadata = reader.get_metadata()?;
    let key_values = metadata
        .key_value_metadata
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|entry| (&entry.key, entry.value.as_deref()))
        .collect::<std::collections::BTreeMap<_, _>>();
    serde_json::to_string(&serde_json::json!({
        "created_by": metadata.created_by.as_deref(),
        "key_values": key_values,
        "num_rows": metadata.num_rows,
        "row_groups": metadata.row_groups.len(),
        "version": metadata.version,
    }))
    .map_err(|error| io_error(format!("cannot encode Parquet metadata: {error}")))
}

/// Returns the row count recorded in a Parquet file footer.
#[cfg(feature = "real-polars")]
pub fn parquet_row_count(path: &str) -> Result<i64, TerlanPolarsError> {
    use polars::prelude::{ParquetReader, SerReader};

    let mut reader = ParquetReader::new(std::fs::File::open(path)?);
    i64::try_from(reader.num_rows()?)
        .map_err(|_| io_error("Parquet row count exceeds the Terlan Int range"))
}

/// Reports unavailable metadata inspection without the real Polars feature.
#[cfg(not(feature = "real-polars"))]
pub fn ipc_custom_metadata(_path: &str) -> Result<String, TerlanPolarsError> {
    Err(TerlanPolarsError::new(
        "native_unavailable",
        "terlan-polars requires the Rust native adapter target capability",
    ))
}

/// Reports unavailable metadata inspection without the real Polars feature.
#[cfg(not(feature = "real-polars"))]
pub fn parquet_metadata(_path: &str) -> Result<String, TerlanPolarsError> {
    Err(TerlanPolarsError::new(
        "native_unavailable",
        "terlan-polars requires the Rust native adapter target capability",
    ))
}

/// Reports unavailable Parquet row-count inspection without real Polars.
#[cfg(not(feature = "real-polars"))]
pub fn parquet_row_count(_path: &str) -> Result<i64, TerlanPolarsError> {
    Err(TerlanPolarsError::new(
        "native_unavailable",
        "terlan-polars requires the Rust native adapter target capability",
    ))
}

#[cfg(not(feature = "real-polars"))]
macro_rules! unavailable_read {
    ($($name:ident),+ $(,)?) => {
        $(
            pub fn $name(
                _path: &str,
                _descriptor: &str,
            ) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
                Err(TerlanPolarsError::new(
                    "native_unavailable",
                    "terlan-polars requires the Rust native adapter target capability",
                ))
            }
        )+
    };
}

#[cfg(not(feature = "real-polars"))]
macro_rules! unavailable_write {
    ($($name:ident),+ $(,)?) => {
        $(
            pub fn $name(
                _frame: &TerlanPolarsDataFrame,
                _path: &str,
                _descriptor: &str,
            ) -> Result<(), TerlanPolarsError> {
                Err(TerlanPolarsError::new(
                    "native_unavailable",
                    "terlan-polars requires the Rust native adapter target capability",
                ))
            }
        )+
    };
}

#[cfg(not(feature = "real-polars"))]
unavailable_read!(
    read_csv_with_options,
    read_json_with_options,
    read_parquet_with_options,
    read_ipc_with_options,
);

#[cfg(not(feature = "real-polars"))]
unavailable_write!(
    write_csv_with_options,
    write_parquet_with_options,
    write_ipc_with_options,
);

fn encode<T: Serialize>(value: &T) -> Result<String, TerlanPolarsError> {
    serde_json::to_string(value)
        .map_err(|error| io_error(format!("cannot encode I/O options: {error}")))
}

fn decode<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, TerlanPolarsError> {
    if value.len() > 1_048_576 {
        return invalid("I/O option descriptor exceeds 1 MiB");
    }
    serde_json::from_str(value)
        .map_err(|error| io_error(format!("invalid I/O option descriptor: {error}")))
}

#[cfg(feature = "real-polars")]
fn schema(
    names: &[String],
    data_types: &[String],
) -> Result<std::sync::Arc<polars::prelude::Schema>, TerlanPolarsError> {
    use polars::prelude::{PlSmallStr, Schema};

    let fields = names
        .iter()
        .zip(data_types)
        .map(|(name, data_type)| {
            crate::expressions::parse_data_type(data_type)
                .map(|data_type| (PlSmallStr::from_str(name), data_type))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(std::sync::Arc::new(Schema::from_iter(fields)))
}

fn validate_null_values(
    mode: &str,
    values: &[String],
    columns: &[String],
) -> Result<(), TerlanPolarsError> {
    let valid = match mode {
        "none" => values.is_empty() && columns.is_empty(),
        "single" => values.len() == 1 && columns.is_empty(),
        "all" => !values.is_empty() && columns.is_empty(),
        "named" => !values.is_empty() && values.len() == columns.len(),
        _ => false,
    };
    if !valid {
        return invalid("CSV null mode does not match its null values and columns");
    }
    Ok(())
}

fn validate_parallel_lists(
    label: &str,
    left: &[String],
    right: &[String],
) -> Result<(), TerlanPolarsError> {
    if left.len() != right.len() {
        return invalid(format!("{label} names and values must have equal lengths"));
    }
    if left.iter().any(|name| name.is_empty()) {
        return invalid(format!("{label} names cannot be empty"));
    }
    Ok(())
}

fn validate_compression_level(
    compression: &str,
    level: Option<i32>,
) -> Result<(), TerlanPolarsError> {
    if level.is_some() && !matches!(compression, "gzip" | "brotli" | "zstd") {
        return invalid("compression levels are only valid for gzip, brotli, and zstd");
    }
    Ok(())
}

fn required_ascii_byte(label: &str, value: &str) -> Result<u8, TerlanPolarsError> {
    if value.len() != 1 || !value.is_ascii() {
        return invalid(format!("{label} must be exactly one ASCII byte"));
    }
    Ok(value.as_bytes()[0])
}

fn optional_ascii_byte(label: &str, value: &str) -> Result<Option<u8>, TerlanPolarsError> {
    if value.is_empty() {
        Ok(None)
    } else {
        required_ascii_byte(label, value).map(Some)
    }
}

fn optional_text(
    label: &str,
    value: &str,
    allow_nul: bool,
) -> Result<Option<String>, TerlanPolarsError> {
    if value.is_empty() {
        return Ok(None);
    }
    if !allow_nul && value.contains('\0') {
        return invalid(format!("{label} cannot contain NUL"));
    }
    bounded_text(label, value).map(Some)
}

fn bounded_text(label: &str, value: &str) -> Result<String, TerlanPolarsError> {
    if value.len() > 65_536 {
        return invalid(format!("{label} cannot exceed 65536 bytes"));
    }
    Ok(value.to_owned())
}

fn one_of(label: &str, value: &str, choices: &[&str]) -> Result<String, TerlanPolarsError> {
    if choices.contains(&value) {
        Ok(value.to_owned())
    } else {
        invalid(format!("{label} must be one of: {}", choices.join(", ")))
    }
}

fn optional_usize(
    label: &str,
    value: i64,
    allow_zero: bool,
) -> Result<Option<usize>, TerlanPolarsError> {
    if value == NONE {
        return Ok(None);
    }
    let value = nonnegative_usize(label, value)?;
    if value == 0 && !allow_zero {
        return invalid(format!("{label} must be greater than zero or -1"));
    }
    Ok(Some(value))
}

fn positive_usize(label: &str, value: i64) -> Result<usize, TerlanPolarsError> {
    let value = nonnegative_usize(label, value)?;
    if value == 0 {
        return invalid(format!("{label} must be greater than zero"));
    }
    Ok(value)
}

fn nonnegative_usize(label: &str, value: i64) -> Result<usize, TerlanPolarsError> {
    usize::try_from(value).map_err(|_| io_error(format!("{label} cannot be negative")))
}

fn u32_value(label: &str, value: i64) -> Result<u32, TerlanPolarsError> {
    u32::try_from(value).map_err(|_| io_error(format!("{label} must fit in UInt32")))
}

fn optional_i32(label: &str, value: i64) -> Result<Option<i32>, TerlanPolarsError> {
    if value == NONE {
        Ok(None)
    } else {
        i32::try_from(value)
            .map(Some)
            .map_err(|_| io_error(format!("{label} must fit in Int32")))
    }
}

fn optional_bool(label: &str, value: i64) -> Result<Option<bool>, TerlanPolarsError> {
    match value {
        -1 => Ok(None),
        0 => Ok(Some(false)),
        1 => Ok(Some(true)),
        _ => invalid(format!("{label} must be -1, 0, or 1")),
    }
}

fn indexes(label: &str, values: &[i64]) -> Result<Vec<usize>, TerlanPolarsError> {
    values
        .iter()
        .map(|value| nonnegative_usize(label, *value))
        .collect()
}

fn invalid<T>(message: impl Into<String>) -> Result<T, TerlanPolarsError> {
    Err(io_error(message))
}

fn io_error(message: impl Into<String>) -> TerlanPolarsError {
    TerlanPolarsError::new("invalid_io_options", message)
}
