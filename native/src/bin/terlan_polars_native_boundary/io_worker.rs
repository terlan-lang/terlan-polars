//! Native worker dispatch for immutable Polars I/O option descriptors.

use super::*;

impl Worker {
    pub(super) fn csv_read_options_with_column_names(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(descriptor), NativeArg::Strings(names)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "csv_read_options_with_column_names expects options and replacement names",
            );
        };
        expression_result_reply(csv_read_options_with_column_names(descriptor, names))
    }

    pub(super) fn csv_read_options_with_infer_schema_files(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(descriptor), NativeArg::Int(count)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "csv_read_options_with_infer_schema_files expects options and a positive count",
            );
        };
        expression_result_reply(csv_read_options_with_infer_schema_files(descriptor, *count))
    }

    pub(super) fn csv_read_options_with_column_policies(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(descriptor), NativeArg::Text(extra), NativeArg::Text(missing)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "CSV column policies expect options, an extra-column policy, and a missing-column policy",
            );
        };
        expression_result_reply(csv_read_options_with_column_policies(
            descriptor, extra, missing,
        ))
    }

    /// Creates one validated CSV reader option descriptor.
    pub(super) fn csv_read_options(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Bool(has_header), NativeArg::Text(separator), NativeArg::Text(quote_char), NativeArg::Text(eol_char), NativeArg::Text(encoding), NativeArg::Text(null_mode), NativeArg::Strings(null_values), NativeArg::Strings(null_columns), NativeArg::Bool(missing_is_null), NativeArg::Bool(truncate_ragged_lines), NativeArg::Text(comment_prefix), NativeArg::Bool(try_parse_dates), NativeArg::Bool(decimal_comma), NativeArg::Bool(rechunk), NativeArg::Int(n_threads), NativeArg::Bool(low_memory), NativeArg::Int(n_rows), NativeArg::Strings(columns), projection, NativeArg::Strings(schema_names), NativeArg::Strings(schema_types), NativeArg::Strings(schema_overwrite_names), NativeArg::Strings(schema_overwrite_types), NativeArg::Strings(dtype_overwrite), NativeArg::Int(chunk_size), NativeArg::Int(skip_rows), NativeArg::Int(skip_lines), NativeArg::Int(skip_rows_after_header), NativeArg::Int(infer_schema_length), NativeArg::Bool(raise_if_empty), NativeArg::Bool(ignore_errors), NativeArg::Text(row_index_name), NativeArg::Int(row_index_offset)] =
            args.as_slice()
        else {
            let supplied = args.len();
            return protocol_error(
                "native_bad_args",
                &format!(
                    "csv_read_options expects 33 flattened option values, received {supplied}"
                ),
            );
        };
        let projection = match projection {
            NativeArg::Ints(projection) => projection.as_slice(),
            NativeArg::Strings(projection) if projection.is_empty() => &[],
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "csv_read_options projection must be a list of integers",
                );
            }
        };
        expression_result_reply(csv_read_options(
            *has_header,
            separator,
            quote_char,
            eol_char,
            encoding,
            null_mode,
            null_values,
            null_columns,
            *missing_is_null,
            *truncate_ragged_lines,
            comment_prefix,
            *try_parse_dates,
            *decimal_comma,
            *rechunk,
            *n_threads,
            *low_memory,
            *n_rows,
            columns,
            projection,
            schema_names,
            schema_types,
            schema_overwrite_names,
            schema_overwrite_types,
            dtype_overwrite,
            *chunk_size,
            *skip_rows,
            *skip_lines,
            *skip_rows_after_header,
            *infer_schema_length,
            *raise_if_empty,
            *ignore_errors,
            row_index_name,
            *row_index_offset,
        ))
    }

    /// Creates one validated CSV writer option descriptor.
    pub(super) fn csv_write_options(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Bool(include_bom), NativeArg::Bool(include_header), NativeArg::Int(batch_size), NativeArg::Text(date_format), NativeArg::Text(time_format), NativeArg::Text(datetime_format), NativeArg::Int(float_scientific), NativeArg::Int(float_precision), NativeArg::Bool(decimal_comma), NativeArg::Text(separator), NativeArg::Text(quote_char), NativeArg::Text(null_value), NativeArg::Text(line_terminator), NativeArg::Text(quote_style), NativeArg::Int(n_threads)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "csv_write_options expects its complete flattened option record",
            );
        };
        expression_result_reply(csv_write_options(
            *include_bom,
            *include_header,
            *batch_size,
            date_format,
            time_format,
            datetime_format,
            *float_scientific,
            *float_precision,
            *decimal_comma,
            separator,
            quote_char,
            null_value,
            line_terminator,
            quote_style,
            *n_threads,
        ))
    }

    /// Creates one validated JSON reader option descriptor.
    pub(super) fn json_read_options(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Bool(lines), NativeArg::Int(infer_schema_length), NativeArg::Int(batch_size), NativeArg::Bool(ignore_errors), NativeArg::Strings(projection), NativeArg::Strings(schema_names), NativeArg::Strings(schema_types), NativeArg::Strings(schema_overwrite_names), NativeArg::Strings(schema_overwrite_types), NativeArg::Bool(rechunk)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "json_read_options expects its complete flattened option record",
            );
        };
        expression_result_reply(json_read_options(
            *lines,
            *infer_schema_length,
            *batch_size,
            *ignore_errors,
            projection,
            schema_names,
            schema_types,
            schema_overwrite_names,
            schema_overwrite_types,
            *rechunk,
        ))
    }

    /// Creates one validated Parquet reader option descriptor.
    pub(super) fn parquet_read_options(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Int(slice_offset), NativeArg::Int(slice_length), NativeArg::Strings(columns), NativeArg::Ints(projection), NativeArg::Text(row_index_name), NativeArg::Int(row_index_offset), NativeArg::Bool(low_memory), NativeArg::Text(parallel), NativeArg::Bool(rechunk), NativeArg::Text(include_file_path_name)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "parquet_read_options expects its complete flattened option record",
            );
        };
        expression_result_reply(parquet_read_options(
            *slice_offset,
            *slice_length,
            columns,
            projection,
            row_index_name,
            *row_index_offset,
            *low_memory,
            parallel,
            *rechunk,
            include_file_path_name,
        ))
    }

    /// Creates one validated Parquet writer option descriptor.
    pub(super) fn parquet_write_options(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(compression), NativeArg::Int(compression_level), NativeArg::Text(statistics), NativeArg::Int(row_group_size), NativeArg::Int(data_page_size), NativeArg::Bool(parallel), NativeArg::Strings(metadata_keys), NativeArg::Strings(metadata_values)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "parquet_write_options expects its complete flattened option record",
            );
        };
        expression_result_reply(parquet_write_options(
            compression,
            *compression_level,
            statistics,
            *row_group_size,
            *data_page_size,
            *parallel,
            metadata_keys,
            metadata_values,
        ))
    }

    /// Creates one validated Arrow IPC reader option descriptor.
    pub(super) fn ipc_read_options(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Int(n_rows), NativeArg::Strings(columns), NativeArg::Ints(projection), NativeArg::Text(row_index_name), NativeArg::Int(row_index_offset), NativeArg::Bool(memory_mapped), NativeArg::Bool(rechunk), NativeArg::Text(include_file_path_name)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "ipc_read_options expects its complete flattened option record",
            );
        };
        expression_result_reply(ipc_read_options(
            *n_rows,
            columns,
            projection,
            row_index_name,
            *row_index_offset,
            *memory_mapped,
            *rechunk,
            include_file_path_name,
        ))
    }

    /// Creates one validated Arrow IPC writer option descriptor.
    pub(super) fn ipc_write_options(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Text(compression), NativeArg::Text(compatibility), NativeArg::Int(record_batch_size), NativeArg::Bool(record_batch_statistics), NativeArg::Bool(parallel), NativeArg::Strings(metadata_keys), NativeArg::Strings(metadata_values)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "ipc_write_options expects its complete flattened option record",
            );
        };
        expression_result_reply(ipc_write_options(
            compression,
            compatibility,
            *record_batch_size,
            *record_batch_statistics,
            *parallel,
            metadata_keys,
            metadata_values,
        ))
    }

    /// Reads one file using a serialized option descriptor.
    pub(super) fn read_with_options(
        &mut self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &str,
            &str,
        )
            -> Result<TerlanPolarsDataFrame, terlan_polars_native::TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Text(path), NativeArg::Text(options)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects a path and option descriptor"),
            );
        };
        match function(path, options) {
            Ok(frame) => self.store_result_dataframe(frame),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    /// Writes one DataFrame using a serialized option descriptor.
    pub(super) fn write_with_options(
        &self,
        args: Vec<NativeArg>,
        name: &str,
        function: fn(
            &TerlanPolarsDataFrame,
            &str,
            &str,
        ) -> Result<(), terlan_polars_native::TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(path), NativeArg::Text(options)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects a DataFrame handle, path, and option descriptor"),
            );
        };
        let frame = match self.frame(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match function(frame, path, options) {
            Ok(()) => "result_ok_unit".to_string(),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }
}
