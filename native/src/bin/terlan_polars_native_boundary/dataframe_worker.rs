//! Native worker dispatch for extended immutable DataFrame operations.

use super::*;

impl Worker {
    pub(super) fn dataframe_partition_by(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Strings(columns), NativeArg::Bool(stable), NativeArg::Bool(include_key)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "dataframe_partition_by expects a frame, key columns, stability, and key inclusion",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match terlan_polars_native::dataframe_partition_by(
            dataframe,
            columns,
            *stable,
            *include_key,
        ) {
            Ok(set) => self.store_result_dataframe_set(set),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_set_len(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_set_len expects one DataFrameSet handle",
            );
        };
        match self.dataframe_set(handle) {
            Ok(set) => format!("ok_int {}", terlan_polars_native::dataframe_set_len(set)),
            Err(error) => error,
        }
    }

    pub(super) fn dataframe_set_get(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(index)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_set_get expects a DataFrameSet handle and non-negative index",
            );
        };
        let Ok(index) = usize::try_from(*index) else {
            return protocol_error(
                "row_index_out_of_bounds",
                "DataFrame set index cannot be negative",
            );
        };
        let set = match self.dataframe_set(handle) {
            Ok(set) => set,
            Err(error) => return error,
        };
        match terlan_polars_native::dataframe_set_get(set, index) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_column_at_index(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(index)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_column_at_index expects a DataFrame handle and non-negative index",
            );
        };
        let Ok(index) = usize::try_from(*index) else {
            return protocol_error(
                "row_index_out_of_bounds",
                "DataFrame column index cannot be negative",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match terlan_polars_native::dataframe_column_at_index(dataframe, index) {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_split_chunks(
        &mut self,
        args: Vec<NativeArg>,
        physical: bool,
    ) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_split_chunks expects one DataFrame handle",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match terlan_polars_native::dataframe_split_chunks(dataframe, physical) {
            Ok(set) => self.store_result_dataframe_set(set),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_split_chunks_by_n(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(partitions), NativeArg::Bool(parallel)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "dataframe_split_chunks_by_n expects a frame, positive partition count, and parallel flag",
            );
        };
        let Ok(partitions) = usize::try_from(*partitions) else {
            return protocol_error(
                "invalid_row_limit",
                "DataFrame chunk partition count cannot be negative",
            );
        };
        let dataframe = match self.frame(handle) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        match terlan_polars_native::dataframe_split_chunks_by_n(dataframe, partitions, *parallel) {
            Ok(set) => self.store_result_dataframe_set(set),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_unary(
        &mut self,
        args: Vec<NativeArg>,
        name: &str,
        operation: fn(&TerlanPolarsDataFrame) -> Result<TerlanPolarsDataFrame, TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects one DataFrame handle"),
            );
        };
        let result = match self.frame(handle) {
            Ok(dataframe) => operation(dataframe),
            Err(error) => return error,
        };
        match result {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_empty_with_height(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Int(height)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_empty_with_height expects a non-negative height",
            );
        };
        let Ok(height) = usize::try_from(*height) else {
            return protocol_error("invalid_row_limit", "DataFrame height cannot be negative");
        };
        match terlan_polars_native::dataframe_empty_with_height(height) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_from_schema(
        &mut self,
        args: Vec<NativeArg>,
        full_null: bool,
    ) -> String {
        let (names, data_types, height) = match args.as_slice() {
            [NativeArg::Strings(names), NativeArg::Strings(data_types)] if !full_null => {
                (names, data_types, None)
            }
            [
                NativeArg::Strings(names),
                NativeArg::Strings(data_types),
                NativeArg::Int(height),
            ] if full_null => (names, data_types, Some(height)),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "DataFrame schema construction expects name and data-type lists plus optional height",
                )
            }
        };
        let result = match height {
            Some(height) => {
                let Ok(height) = usize::try_from(*height) else {
                    return protocol_error(
                        "invalid_row_limit",
                        "DataFrame height cannot be negative",
                    );
                };
                dataframe_full_null(names, data_types, height)
            }
            None => dataframe_empty_with_schema(names, data_types),
        };
        match result {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_chunk_lengths(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_chunk_lengths expects one DataFrame handle",
            );
        };
        match self.frame(handle) {
            Ok(dataframe) => {
                let values = terlan_polars_native::dataframe_chunk_lengths(dataframe);
                if values.is_empty() {
                    "ok_ints".to_string()
                } else {
                    format!(
                        "ok_ints {}",
                        values
                            .iter()
                            .map(usize::to_string)
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                }
            }
            Err(error) => error,
        }
    }

    pub(super) fn dataframe_shape(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_shape expects one DataFrame handle",
            );
        };
        match self.frame(handle) {
            Ok(dataframe) => {
                let values = terlan_polars_native::dataframe_shape(dataframe);
                format!(
                    "ok_ints {}",
                    values
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
            Err(error) => error,
        }
    }

    pub(super) fn dataframe_metadata_int(
        &self,
        args: Vec<NativeArg>,
        name: &str,
        operation: fn(&TerlanPolarsDataFrame) -> usize,
    ) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects one DataFrame handle"),
            );
        };
        match self.frame(handle) {
            Ok(dataframe) => format!("ok_int {}", operation(dataframe)),
            Err(error) => error,
        }
    }

    pub(super) fn dataframe_metadata_bool(
        &self,
        args: Vec<NativeArg>,
        name: &str,
        operation: fn(&TerlanPolarsDataFrame) -> bool,
    ) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects one DataFrame handle"),
            );
        };
        match self.frame(handle) {
            Ok(dataframe) => format!("ok_bool {}", operation(dataframe)),
            Err(error) => error,
        }
    }

    pub(super) fn dataframe_schema_equal(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_schema_equal expects two DataFrame handles",
            );
        };
        let left = match self.frame(left) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        let right = match self.frame(right) {
            Ok(dataframe) => dataframe,
            Err(error) => return error,
        };
        format!(
            "ok_bool {}",
            terlan_polars_native::dataframe_schema_equal(left, right)
        )
    }

    pub(super) fn dataframe_column_index(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(name)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_column_index expects a DataFrame handle and column name",
            );
        };
        match self.frame(handle).and_then(|dataframe| {
            match dataframe_column_index(dataframe, name) {
                Ok(index) => Ok(index),
                Err(error) => Err(adapter_result_error(error.code(), error.message())),
            }
        }) {
            Ok(index) => format!("result_ok_int {index}"),
            Err(error) => error,
        }
    }

    pub(super) fn dataframe_row(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(index)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_row expects a DataFrame handle and non-negative row index",
            );
        };
        let Ok(index) = usize::try_from(*index) else {
            return protocol_error(
                "row_index_out_of_bounds",
                "DataFrame row index cannot be negative",
            );
        };
        match self.frame(handle) {
            Ok(dataframe) => match terlan_polars_native::dataframe_row(dataframe, index) {
                Ok(values) if values.is_empty() => "ok_strings".to_string(),
                Ok(values) => format!("ok_strings {}", encode_string_list(&values)),
                Err(error) => adapter_result_error(error.code(), error.message()),
            },
            Err(error) => error,
        }
    }

    pub(super) fn dataframe_split(&mut self, args: Vec<NativeArg>, before: bool) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(offset)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "DataFrame split expects a handle and signed offset",
            );
        };
        let result = match self.frame(handle) {
            Ok(dataframe) if before => dataframe_split_before(dataframe, *offset),
            Ok(dataframe) => dataframe_split_after(dataframe, *offset),
            Err(error) => return error,
        };
        match result {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_column_mutation(&mut self, args: Vec<NativeArg>, kind: &str) -> String {
        let result = match args.as_slice() {
            [NativeArg::Handle(dataframe), NativeArg::Int(index), NativeArg::Handle(series)]
                if kind != "with" =>
            {
                let Ok(index) = usize::try_from(*index) else {
                    return protocol_error(
                        "row_index_out_of_bounds",
                        "DataFrame column index cannot be negative",
                    );
                };
                let dataframe = match self.frame(dataframe) {
                    Ok(dataframe) => dataframe,
                    Err(error) => return error,
                };
                let series = match self.series(series) {
                    Ok(series) => series,
                    Err(error) => return error,
                };
                if kind == "insert" {
                    dataframe_insert_column(dataframe, index, series)
                } else {
                    dataframe_replace_column(dataframe, index, series)
                }
            }
            [NativeArg::Handle(dataframe), NativeArg::Handle(series)] if kind == "with" => {
                let dataframe = match self.frame(dataframe) {
                    Ok(dataframe) => dataframe,
                    Err(error) => return error,
                };
                let series = match self.series(series) {
                    Ok(series) => series,
                    Err(error) => return error,
                };
                dataframe_with_column(dataframe, series)
            }
            _ => return protocol_error(
                "native_bad_args",
                "DataFrame column mutation expects DataFrame, optional index, and Series handles",
            ),
        };
        match result {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_to_struct(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(name)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_to_struct expects a DataFrame handle and Series name",
            );
        };
        let result = match self.frame(handle) {
            Ok(dataframe) => terlan_polars_native::dataframe_to_struct(dataframe, name),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn dataframe_serialize(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_serialize expects one DataFrame handle",
            );
        };
        match self.frame(handle).and_then(|dataframe| {
            match terlan_polars_native::dataframe_serialize(dataframe) {
                Ok(bytes) => Ok(bytes),
                Err(error) => Err(adapter_result_error(error.code(), error.message())),
            }
        }) {
            Ok(bytes) if bytes.is_empty() => "result_ok_bytes".to_string(),
            Ok(bytes) => format!("result_ok_bytes {}", STANDARD.encode(bytes)),
            Err(error) => error,
        }
    }

    pub(super) fn dataframe_deserialize(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Bytes(bytes)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "dataframe_deserialize expects one Bytes payload",
            );
        };
        match terlan_polars_native::dataframe_deserialize(bytes) {
            Ok(dataframe) => self.store_result_dataframe(dataframe),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }
}
