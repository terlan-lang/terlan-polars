//! Native worker dispatch for owned Series operations.

use super::*;

type UnarySeriesOperation =
    fn(&TerlanPolarsSeries) -> Result<TerlanPolarsSeries, TerlanPolarsError>;

impl Worker {
    /// Constructs a populated signed or unsigned 128-bit Series from text.
    pub(super) fn series_from_wide_integer_strings(
        &mut self,
        args: Vec<NativeArg>,
        signed: bool,
    ) -> String {
        let [NativeArg::Text(name), NativeArg::Strings(values)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "wide integer construction expects a name and decimal string list",
            );
        };
        let result = if signed {
            terlan_polars_native::series_from_int128_strings(name, values)
        } else {
            terlan_polars_native::series_from_uint128_strings(name, values)
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    /// Extracts signed or unsigned 128-bit values as canonical decimal text.
    pub(super) fn series_wide_integer_strings(&self, args: Vec<NativeArg>, signed: bool) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "wide integer extraction expects one Series handle",
            );
        };
        let series = match self.series(handle) {
            Ok(series) => series,
            Err(error) => return error,
        };
        let result = if signed {
            terlan_polars_native::series_int128_strings(series)
        } else {
            terlan_polars_native::series_uint128_strings(series)
        };
        match result {
            Ok(values) if values.is_empty() => "result_ok_strings".to_string(),
            Ok(values) => format!("result_ok_strings {}", encode_string_list(&values)),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    /// Clones a Series into an independently owned worker resource.
    pub(super) fn series_clone(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "series_clone expects one Series handle");
        };
        let series = match self.series(handle) {
            Ok(series) => series,
            Err(error) => return error,
        };
        match terlan_polars_native::series_clone(series) {
            Ok(output) => self.store_result_series(output),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    /// Encodes Series values into an owned binary Series.
    pub(super) fn series_row_encode(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Bool(ordered), NativeArg::Bool(descending), NativeArg::Bool(nulls_last)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "series_row_encode expects a Series and three ordering flags",
            );
        };
        let series = match self.series(handle) {
            Ok(series) => series,
            Err(error) => return error,
        };
        match terlan_polars_native::series_row_encode(series, *ordered, *descending, *nulls_last) {
            Ok(output) => self.store_result_series(output),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_to_arrow_ipc(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_to_arrow_ipc expects one Series handle",
            );
        };
        match self.series(handle) {
            Ok(series) => match terlan_polars_native::series_to_arrow_ipc(series) {
                Ok(bytes) if bytes.is_empty() => "result_ok_bytes".to_string(),
                Ok(bytes) => format!("result_ok_bytes {}", STANDARD.encode(bytes)),
                Err(error) => adapter_result_error(error.code(), error.message()),
            },
            Err(error) => error,
        }
    }

    pub(super) fn series_from_arrow_ipc(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Bytes(bytes)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_from_arrow_ipc expects one Bytes payload",
            );
        };
        match terlan_polars_native::series_from_arrow_ipc(bytes) {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_typed_values(&self, args: Vec<NativeArg>, kind: &str) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("series_{kind}_values expects one Series handle"),
            );
        };
        let series = match self.series(handle) {
            Ok(series) => series,
            Err(error) => return error,
        };
        match kind {
            "integer" => match terlan_polars_native::series_integer_values(series) {
                Ok(values) if values.is_empty() => "result_ok_ints".to_string(),
                Ok(values) => format!(
                    "result_ok_ints {}",
                    values
                        .iter()
                        .map(i64::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                Err(error) => adapter_result_error(error.code(), error.message()),
            },
            "float" => match terlan_polars_native::series_float_values(series) {
                Ok(values) if values.is_empty() => "result_ok_floats".to_string(),
                Ok(values) => format!(
                    "result_ok_floats {}",
                    values
                        .iter()
                        .map(f64::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                Err(error) => adapter_result_error(error.code(), error.message()),
            },
            "boolean" => match terlan_polars_native::series_boolean_values(series) {
                Ok(values) if values.is_empty() => "result_ok_bools".to_string(),
                Ok(values) => format!(
                    "result_ok_bools {}",
                    values
                        .iter()
                        .map(bool::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                Err(error) => adapter_result_error(error.code(), error.message()),
            },
            "string" => match terlan_polars_native::series_string_values(series) {
                Ok(values) if values.is_empty() => "result_ok_strings".to_string(),
                Ok(values) => format!("result_ok_strings {}", encode_string_list(&values)),
                Err(error) => adapter_result_error(error.code(), error.message()),
            },
            _ => protocol_error(
                "native_bad_args",
                &format!("unsupported typed Series extraction kind `{kind}`"),
            ),
        }
    }

    pub(super) fn series_unary(
        &mut self,
        args: Vec<NativeArg>,
        name: &str,
        operation: UnarySeriesOperation,
    ) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects one Series handle"),
            );
        };
        let result = match self.series(handle) {
            Ok(series) => operation(series),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_rename(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(name)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_rename expects a Series handle and name",
            );
        };
        let result = match self.series(handle) {
            Ok(series) => terlan_polars_native::series_rename(series, name),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_size(&self, args: Vec<NativeArg>, chunks: bool) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error("native_bad_args", "Series size expects one Series handle");
        };
        match self.series(handle) {
            Ok(series) => {
                let value = if chunks {
                    series_chunk_count(series)
                } else {
                    series_estimated_size(series)
                };
                format!("ok_int {value}")
            }
            Err(error) => error,
        }
    }

    pub(super) fn series_estimated_size_with_unit(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(unit)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "Series estimated size expects a Series handle and size unit",
            );
        };
        let series = match self.series(handle) {
            Ok(value) => value,
            Err(error) => return error,
        };
        match series_estimated_size_with_unit(series, unit) {
            Ok(value) => format!("result_ok_float {value}"),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_chunk_lengths(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_chunk_lengths expects one Series handle",
            );
        };
        match self.series(handle) {
            Ok(series) => {
                let values = terlan_polars_native::series_chunk_lengths(series);
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

    pub(super) fn series_sorted_flag(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_sorted_flag expects one Series handle",
            );
        };
        match self.series(handle) {
            Ok(series) => format!(
                "ok_string {}",
                STANDARD.encode(terlan_polars_native::series_sorted_flag(series))
            ),
            Err(error) => error,
        }
    }

    pub(super) fn series_length_transform(
        &mut self,
        args: Vec<NativeArg>,
        name: &str,
        operation: fn(&TerlanPolarsSeries, usize) -> Result<TerlanPolarsSeries, TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(length)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects a Series handle and non-negative length"),
            );
        };
        let Ok(length) = usize::try_from(*length) else {
            return protocol_error("invalid_slice_length", "Series length cannot be negative");
        };
        let result = match self.series(handle) {
            Ok(series) => operation(series, length),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_slice(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(offset), NativeArg::Int(length)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "series_slice expects a Series handle, offset, and non-negative length",
            );
        };
        let Ok(length) = usize::try_from(*length) else {
            return protocol_error(
                "invalid_slice_length",
                "Series slice length cannot be negative",
            );
        };
        let result = match self.series(handle) {
            Ok(series) => terlan_polars_native::series_slice(series, *offset, length),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_split(&mut self, args: Vec<NativeArg>, before: bool) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(offset)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_split expects a Series handle and offset",
            );
        };
        let result = match self.series(handle) {
            Ok(series) if before => series_split_before(series, *offset),
            Ok(series) => series_split_after(series, *offset),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_sort(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Bool(descending), NativeArg::Bool(nulls_last), NativeArg::Bool(maintain_order)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "series_sort expects a Series handle and three Boolean options",
            );
        };
        let result = match self.series(handle) {
            Ok(series) => series_sort(series, *descending, *nulls_last, *maintain_order),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_explode(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Bool(empty_as_null), NativeArg::Bool(keep_nulls)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "series_explode expects a Series handle and two Boolean options",
            );
        };
        let result = match self.series(handle) {
            Ok(series) => series_explode(series, *empty_as_null, *keep_nulls),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_fill_null(&mut self, args: Vec<NativeArg>, limited: bool) -> String {
        let (handle, strategy, limit) = match args.as_slice() {
            [NativeArg::Handle(handle), NativeArg::Text(strategy)] if !limited => {
                (handle, strategy.as_str(), None)
            }
            [NativeArg::Handle(handle), NativeArg::Text(strategy), NativeArg::Int(limit)]
                if limited =>
            {
                let Ok(limit) = usize::try_from(*limit) else {
                    return protocol_error("invalid_limit", "fill limit cannot be negative");
                };
                (handle, strategy.as_str(), Some(limit))
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_fill_null expects a Series handle, strategy, and optional limit",
                )
            }
        };
        let result = match self.series(handle) {
            Ok(series) => terlan_polars_native::series_fill_null(series, strategy, limit),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_gather_every(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(step), NativeArg::Int(offset)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "series_gather_every expects a Series handle, step, and offset",
            );
        };
        let (Ok(step), Ok(offset)) = (usize::try_from(*step), usize::try_from(*offset)) else {
            return protocol_error(
                "invalid_row_index",
                "Series gather step and offset must be non-negative",
            );
        };
        let result = match self.series(handle) {
            Ok(series) => terlan_polars_native::series_gather_every(series, step, offset),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_shuffle(&mut self, args: Vec<NativeArg>, seeded: bool) -> String {
        let (handle, seed) = match args.as_slice() {
            [NativeArg::Handle(handle)] if !seeded => (handle, None),
            [NativeArg::Handle(handle), NativeArg::Int(seed)] if seeded => {
                let Ok(seed) = u64::try_from(*seed) else {
                    return protocol_error("invalid_sample_seed", "Series seed cannot be negative");
                };
                (handle, Some(seed))
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_shuffle expects a Series handle and optional seed",
                )
            }
        };
        let result = match (self.series(handle), seed) {
            (Ok(series), Some(seed)) => series_shuffle_seeded(series, seed),
            (Ok(series), None) => series_shuffle(series),
            (Err(error), _) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_sample_n(&mut self, args: Vec<NativeArg>, seeded: bool) -> String {
        let (handle, count, with_replacement, shuffle, seed) = match args.as_slice() {
            [
                NativeArg::Handle(handle),
                NativeArg::Int(count),
                NativeArg::Bool(with_replacement),
                NativeArg::Bool(shuffle),
            ] if !seeded => (handle, count, with_replacement, shuffle, None),
            [
                NativeArg::Handle(handle),
                NativeArg::Int(count),
                NativeArg::Bool(with_replacement),
                NativeArg::Bool(shuffle),
                NativeArg::Int(seed),
            ] if seeded => (handle, count, with_replacement, shuffle, Some(seed)),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_sample_n expects a Series handle, count, replacement, shuffle, and optional seed",
                )
            }
        };
        let (Ok(count), Ok(seed)) = (
            usize::try_from(*count),
            seed.map(|value| u64::try_from(*value)).transpose(),
        ) else {
            return protocol_error(
                "invalid_sample_size",
                "Series sample count and seed must be non-negative",
            );
        };
        let result = match (self.series(handle), seed) {
            (Ok(series), Some(seed)) => {
                series_sample_n_seeded(series, count, *with_replacement, *shuffle, seed)
            }
            (Ok(series), None) => series_sample_n(series, count, *with_replacement, *shuffle),
            (Err(error), _) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_sample_fraction(&mut self, args: Vec<NativeArg>, seeded: bool) -> String {
        let (handle, fraction, with_replacement, shuffle, seed) = match args.as_slice() {
            [
                NativeArg::Handle(handle),
                NativeArg::Float(fraction),
                NativeArg::Bool(with_replacement),
                NativeArg::Bool(shuffle),
            ] if !seeded => (handle, fraction, with_replacement, shuffle, None),
            [
                NativeArg::Handle(handle),
                NativeArg::Float(fraction),
                NativeArg::Bool(with_replacement),
                NativeArg::Bool(shuffle),
                NativeArg::Int(seed),
            ] if seeded => (handle, fraction, with_replacement, shuffle, Some(seed)),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "series_sample_fraction expects a Series handle, fraction, replacement, shuffle, and optional seed",
                )
            }
        };
        let Ok(seed) = seed.map(|value| u64::try_from(*value)).transpose() else {
            return protocol_error(
                "invalid_sample_seed",
                "Series sample seed cannot be negative",
            );
        };
        let result = match (self.series(handle), seed) {
            (Ok(series), Some(seed)) => {
                series_sample_fraction_seeded(series, *fraction, *with_replacement, *shuffle, seed)
            }
            (Ok(series), None) => {
                series_sample_fraction(series, *fraction, *with_replacement, *shuffle)
            }
            (Err(error), _) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_binary(
        &mut self,
        args: Vec<NativeArg>,
        name: &str,
        operation: fn(
            &TerlanPolarsSeries,
            &TerlanPolarsSeries,
        ) -> Result<TerlanPolarsSeries, TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Handle(left), NativeArg::Handle(right)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects two Series handles"),
            );
        };
        let left = match self.series(left) {
            Ok(series) => series,
            Err(error) => return error,
        };
        let right = match self.series(right) {
            Ok(series) => series,
            Err(error) => return error,
        };
        let result = operation(left, right);
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_full_null(&mut self, args: Vec<NativeArg>, null_dtype: bool) -> String {
        let (name, length, data_type) = match args.as_slice() {
            [NativeArg::Text(name), NativeArg::Int(length)] if null_dtype => {
                (name, length, None)
            }
            [
                NativeArg::Text(name),
                NativeArg::Int(length),
                NativeArg::Text(data_type),
            ] if !null_dtype => (name, length, Some(data_type)),
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "Series null construction expects a name, non-negative length, and optional data type",
                )
            }
        };
        let Ok(length) = usize::try_from(*length) else {
            return protocol_error(
                "invalid_slice_length",
                "Series null length cannot be negative",
            );
        };
        let result = match data_type {
            Some(data_type) => terlan_polars_native::series_full_null(name, length, data_type),
            None => terlan_polars_native::series_new_null(name, length),
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_set_sorted_flag(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(flag)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_set_sorted_flag expects a Series handle and sorted flag",
            );
        };
        let result = match self.series(handle) {
            Ok(series) => terlan_polars_native::series_set_sorted_flag(series, flag),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_should_rechunk(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_should_rechunk expects one Series handle",
            );
        };
        match self.series(handle) {
            Ok(series) => format!(
                "ok_bool {}",
                terlan_polars_native::series_should_rechunk(series)
            ),
            Err(error) => error,
        }
    }

    pub(super) fn series_cast_mode(
        &mut self,
        args: Vec<NativeArg>,
        name: &str,
        operation: fn(&TerlanPolarsSeries, &str) -> Result<TerlanPolarsSeries, TerlanPolarsError>,
    ) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(data_type)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                &format!("{name} expects a Series handle and data type"),
            );
        };
        let result = match self.series(handle) {
            Ok(series) => operation(series, data_type),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_cast_with_options(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(data_type), NativeArg::Text(mode)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "series_cast_with_options expects a Series handle, data type, and cast mode",
            );
        };
        let result = match self.series(handle) {
            Ok(series) => terlan_polars_native::series_cast_with_options(series, data_type, mode),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_from_cats_and_dtype(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Text(data_type), NativeArg::Bool(strict)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "series_from_cats_and_dtype expects a Series handle, data type, and strict flag",
            );
        };
        let series = match self.series(handle) {
            Ok(series) => series,
            Err(error) => return error,
        };
        match terlan_polars_native::series_from_cats_and_dtype(series, data_type, *strict) {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_reshape(&mut self, args: Vec<NativeArg>, array: bool) -> String {
        let [NativeArg::Handle(handle), NativeArg::Ints(dimensions)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "Series reshape expects a Series handle and integer dimensions",
            );
        };
        let result = match self.series(handle) {
            Ok(series) if array => series_reshape_array(series, dimensions),
            Ok(series) => series_reshape_list(series, dimensions),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_extend_constant(&mut self, args: Vec<NativeArg>, kind: &str) -> String {
        let result = match args.as_slice() {
            [NativeArg::Handle(handle), NativeArg::Text(value), NativeArg::Int(count)]
                if kind == "string" =>
            {
                let Ok(count) = usize::try_from(*count) else {
                    return protocol_error(
                        "invalid_slice_length",
                        "Series extension count cannot be negative",
                    );
                };
                match self.series(handle) {
                    Ok(series) => series_extend_string(series, value, count),
                    Err(error) => return error,
                }
            }
            [NativeArg::Handle(handle), NativeArg::Int(value), NativeArg::Int(count)]
                if kind == "int" =>
            {
                let Ok(count) = usize::try_from(*count) else {
                    return protocol_error(
                        "invalid_slice_length",
                        "Series extension count cannot be negative",
                    );
                };
                match self.series(handle) {
                    Ok(series) => series_extend_int(series, *value, count),
                    Err(error) => return error,
                }
            }
            [NativeArg::Handle(handle), NativeArg::Float(value), NativeArg::Int(count)]
                if kind == "float" =>
            {
                let Ok(count) = usize::try_from(*count) else {
                    return protocol_error(
                        "invalid_slice_length",
                        "Series extension count cannot be negative",
                    );
                };
                match self.series(handle) {
                    Ok(series) => series_extend_float(series, *value, count),
                    Err(error) => return error,
                }
            }
            [NativeArg::Handle(handle), NativeArg::Bool(value), NativeArg::Int(count)]
                if kind == "bool" =>
            {
                let Ok(count) = usize::try_from(*count) else {
                    return protocol_error(
                        "invalid_slice_length",
                        "Series extension count cannot be negative",
                    );
                };
                match self.series(handle) {
                    Ok(series) => series_extend_bool(series, *value, count),
                    Err(error) => return error,
                }
            }
            [NativeArg::Handle(handle), NativeArg::Int(count)] if kind == "null" => {
                let Ok(count) = usize::try_from(*count) else {
                    return protocol_error(
                        "invalid_slice_length",
                        "Series extension count cannot be negative",
                    );
                };
                match self.series(handle) {
                    Ok(series) => series_extend_null(series, count),
                    Err(error) => return error,
                }
            }
            _ => {
                return protocol_error(
                    "native_bad_args",
                    "Series constant extension expects a handle, compatible value, and non-negative count",
                )
            }
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_zip_with(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Handle(mask), NativeArg::Handle(other)] =
            args.as_slice()
        else {
            return protocol_error(
                "native_bad_args",
                "series_zip_with expects value, Boolean mask, and alternate Series handles",
            );
        };
        let series = match self.series(handle) {
            Ok(series) => series,
            Err(error) => return error,
        };
        let mask = match self.series(mask) {
            Ok(series) => series,
            Err(error) => return error,
        };
        let other = match self.series(other) {
            Ok(series) => series,
            Err(error) => return error,
        };
        match terlan_polars_native::series_zip_with(series, mask, other) {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_serialize(&self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_serialize expects one Series handle",
            );
        };
        match self.series(handle).and_then(|series| {
            match terlan_polars_native::series_serialize(series) {
                Ok(bytes) => Ok(bytes),
                Err(error) => Err(adapter_result_error(error.code(), error.message())),
            }
        }) {
            Ok(bytes) if bytes.is_empty() => "result_ok_bytes".to_string(),
            Ok(bytes) => format!("result_ok_bytes {}", STANDARD.encode(bytes)),
            Err(error) => error,
        }
    }

    pub(super) fn series_deserialize(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Bytes(bytes)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_deserialize expects one Bytes payload",
            );
        };
        match terlan_polars_native::series_deserialize(bytes) {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }

    pub(super) fn series_select_chunk(&mut self, args: Vec<NativeArg>) -> String {
        let [NativeArg::Handle(handle), NativeArg::Int(index)] = args.as_slice() else {
            return protocol_error(
                "native_bad_args",
                "series_select_chunk expects a Series handle and non-negative chunk index",
            );
        };
        let Ok(index) = usize::try_from(*index) else {
            return protocol_error(
                "row_index_out_of_bounds",
                "Series chunk index cannot be negative",
            );
        };
        let result = match self.series(handle) {
            Ok(series) => terlan_polars_native::series_select_chunk(series, index),
            Err(error) => return error,
        };
        match result {
            Ok(series) => self.store_result_series(series),
            Err(error) => adapter_result_error(error.code(), error.message()),
        }
    }
}
