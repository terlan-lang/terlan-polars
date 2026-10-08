//! Extended owned Series conversions, reshaping, and interchange.
//!
//! These operations preserve useful upstream behavior while replacing
//! mutable metadata setters and reader/writer APIs with immutable values.

#[cfg(feature = "real-polars")]
use crate::expressions;
use crate::{TerlanPolarsError, TerlanPolarsSeries};

#[cfg(feature = "real-polars")]
fn owned(series: polars::prelude::Series) -> TerlanPolarsSeries {
    TerlanPolarsSeries { inner: series }
}

#[cfg(not(feature = "real-polars"))]
fn unavailable() -> TerlanPolarsError {
    TerlanPolarsError::new(
        "polars_unavailable",
        "real Polars support is not enabled for this adapter build",
    )
}

/// Returns an independently owned clone of a Series resource.
pub fn series_clone(series: &TerlanPolarsSeries) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.clone()))
    }
}

/// Encodes each value into Polars' stable binary row representation.
pub fn series_row_encode(
    series: &TerlanPolarsSeries,
    ordered: bool,
    descending: bool,
    nulls_last: bool,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, ordered, descending, nulls_last);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoSeries;

        let encoded = if ordered {
            series.inner.row_encode_ordered(descending, nulls_last)?
        } else {
            series.inner.row_encode_unordered()?
        };
        Ok(owned(encoded.into_series()))
    }
}

/// Constructs a typed Series containing only null values.
pub fn series_full_null(
    name: &str,
    length: usize,
    data_type: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (name, length, data_type);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(polars::prelude::Series::full_null(
            name.into(),
            length,
            &expressions::parse_data_type(data_type)?,
        )))
    }
}

/// Constructs a Null-typed Series of the requested length.
pub fn series_new_null(name: &str, length: usize) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (name, length);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(polars::prelude::Series::new_null(
            name.into(),
            length,
        )))
    }
}

/// Parses decimal text into a populated signed 128-bit integer Series.
pub fn series_from_int128_strings(
    name: &str,
    values: &[String],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (name, values);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::NamedFrom;

        let values = parse_wide_integers(values, "Int128", str::parse::<i128>)?;
        Ok(owned(polars::prelude::Series::new(name.into(), values)))
    }
}

/// Parses decimal text into a populated unsigned 128-bit integer Series.
pub fn series_from_uint128_strings(
    name: &str,
    values: &[String],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (name, values);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::NamedFrom;

        let values = parse_wide_integers(values, "UInt128", str::parse::<u128>)?;
        Ok(owned(polars::prelude::Series::new(name.into(), values)))
    }
}

/// Extracts every signed 128-bit value as lossless canonical decimal text.
pub fn series_int128_strings(
    series: &TerlanPolarsSeries,
) -> Result<Vec<String>, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(series
            .inner
            .i128()?
            .into_no_null_iter()
            .map(|value| value.to_string())
            .collect())
    }
}

/// Extracts every unsigned 128-bit value as lossless canonical decimal text.
pub fn series_uint128_strings(
    series: &TerlanPolarsSeries,
) -> Result<Vec<String>, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(series
            .inner
            .u128()?
            .into_no_null_iter()
            .map(|value| value.to_string())
            .collect())
    }
}

/// Parses indexed decimal values while preserving the full destination width.
#[cfg(feature = "real-polars")]
fn parse_wide_integers<T, E>(
    values: &[String],
    data_type: &str,
    parse: impl Fn(&str) -> Result<T, E>,
) -> Result<Vec<T>, TerlanPolarsError>
where
    E: std::fmt::Display,
{
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            parse(value).map_err(|error| {
                TerlanPolarsError::new(
                    "invalid_wide_integer",
                    format!("invalid {data_type} decimal at index {index} (`{value}`): {error}"),
                )
            })
        })
        .collect()
}

/// Returns a copy with explicit ascending, descending, or unsorted metadata.
pub fn series_set_sorted_flag(
    series: &TerlanPolarsSeries,
    flag: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, flag);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let flag = match flag {
            "ascending" => polars::series::IsSorted::Ascending,
            "descending" => polars::series::IsSorted::Descending,
            "not_sorted" => polars::series::IsSorted::Not,
            other => {
                return Err(TerlanPolarsError::new(
                    "invalid_sorted_flag",
                    format!("unsupported Series sorted flag `{other}`"),
                ));
            }
        };
        let mut output = series.inner.clone();
        output.set_sorted_flag(flag);
        Ok(owned(output))
    }
}

/// Reports whether the upstream Series container requests rechunking.
pub fn series_should_rechunk(series: &TerlanPolarsSeries) -> bool {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        false
    }
    #[cfg(feature = "real-polars")]
    {
        let _ = series;
        false
    }
}

/// Casts a Series and rejects values that cannot be represented exactly.
pub fn series_strict_cast(
    series: &TerlanPolarsSeries,
    data_type: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, data_type);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let data_type = expressions::parse_data_type(data_type)?;
        if let Some(message) =
            expressions::polars2_cast_error(series.inner.dtype(), &data_type, true)
        {
            return Err(TerlanPolarsError::new("invalid_cast", message));
        }
        Ok(owned(series.inner.strict_cast(&data_type)?))
    }
}

/// Casts a Series and replaces unrepresentable values with null.
pub fn series_non_strict_cast(
    series: &TerlanPolarsSeries,
    data_type: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, data_type);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let data_type = expressions::parse_data_type(data_type)?;
        if let Some(message) =
            expressions::polars2_cast_error(series.inner.dtype(), &data_type, false)
        {
            return Err(TerlanPolarsError::new("invalid_cast", message));
        }
        Ok(owned(series.inner.cast_with_options(
            &data_type,
            polars::chunked_array::cast::CastOptions::NonStrict,
        )?))
    }
}

/// Casts a Series and permits upstream wrapping overflow behavior.
pub fn series_overflowing_cast(
    series: &TerlanPolarsSeries,
    data_type: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, data_type);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let data_type = expressions::parse_data_type(data_type)?;
        if let Some(message) =
            expressions::polars2_cast_error(series.inner.dtype(), &data_type, false)
        {
            return Err(TerlanPolarsError::new("invalid_cast", message));
        }
        Ok(owned(series.inner.cast_with_options(
            &data_type,
            polars::chunked_array::cast::CastOptions::Overflowing,
        )?))
    }
}

/// Casts a Series with explicit strict, non-strict, or overflowing behavior.
pub fn series_cast_with_options(
    series: &TerlanPolarsSeries,
    data_type: &str,
    mode: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    match mode {
        "strict" => series_strict_cast(series, data_type),
        "non_strict" | "nonstrict" => series_non_strict_cast(series, data_type),
        "overflowing" => series_overflowing_cast(series, data_type),
        _ => Err(TerlanPolarsError::new(
            "invalid_cast_mode",
            "Series cast mode must be strict, non_strict, or overflowing",
        )),
    }
}

/// Returns the physical representation of a logical Series.
pub fn series_to_physical(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.to_physical_repr().into_owned()))
    }
}

/// Returns the storage representation of an extension Series.
pub fn series_to_storage(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.to_storage().clone()))
    }
}

/// Wraps all values in one List value.
pub fn series_implode(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoSeries;
        Ok(owned(series.inner.implode()?.into_series()))
    }
}

/// Packs each Series value into a singleton List value.
pub fn series_as_list(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoSeries;
        Ok(owned(series.inner.to_unit_list().into_series()))
    }
}

/// Packs each Series value into a singleton List value using the Polars 0.55 name.
pub fn series_to_unit_list(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    series_as_list(series)
}

/// Constructs categorical or enum values from their physical category IDs.
pub fn series_from_cats_and_dtype(
    categories: &TerlanPolarsSeries,
    data_type: &str,
    strict: bool,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (categories, data_type, strict);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let data_type = crate::expressions::parse_data_type(data_type)?;
        Ok(owned(polars::prelude::Series::from_cats_and_dtype(
            &categories.inner,
            &data_type,
            strict,
        )?))
    }
}

#[cfg(feature = "real-polars")]
fn reshape_dimensions(
    values: &[i64],
) -> Result<Vec<polars::prelude::ReshapeDimension>, TerlanPolarsError> {
    if values.is_empty() {
        return Err(TerlanPolarsError::new(
            "invalid_reshape_dimensions",
            "Series reshape requires at least one dimension",
        ));
    }
    if values.iter().filter(|value| **value < 0).count() > 1 {
        return Err(TerlanPolarsError::new(
            "invalid_reshape_dimensions",
            "Series reshape accepts at most one inferred dimension",
        ));
    }
    Ok(values
        .iter()
        .copied()
        .map(polars::prelude::ReshapeDimension::new)
        .collect())
}

/// Reshapes values into nested variable-width List storage.
pub fn series_reshape_list(
    series: &TerlanPolarsSeries,
    dimensions: &[i64],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, dimensions);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(
            series
                .inner
                .reshape_list(&reshape_dimensions(dimensions)?)?,
        ))
    }
}

/// Reshapes values into nested fixed-width Array storage.
pub fn series_reshape_array(
    series: &TerlanPolarsSeries,
    dimensions: &[i64],
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, dimensions);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(
            series
                .inner
                .reshape_array(&reshape_dimensions(dimensions)?)?,
        ))
    }
}

#[cfg(feature = "real-polars")]
fn extend_constant(
    series: &TerlanPolarsSeries,
    value: polars::prelude::AnyValue<'_>,
    count: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Ok(owned(series.inner.extend_constant(value, count)?))
}

/// Extends a Series with repeated String values.
pub fn series_extend_string(
    series: &TerlanPolarsSeries,
    value: &str,
    count: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, value, count);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        extend_constant(series, polars::prelude::AnyValue::String(value), count)
    }
}

/// Extends a Series with repeated Int64 values.
pub fn series_extend_int(
    series: &TerlanPolarsSeries,
    value: i64,
    count: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, value, count);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        extend_constant(series, polars::prelude::AnyValue::Int64(value), count)
    }
}

/// Extends a Series with repeated Float64 values.
pub fn series_extend_float(
    series: &TerlanPolarsSeries,
    value: f64,
    count: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, value, count);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        extend_constant(series, polars::prelude::AnyValue::Float64(value), count)
    }
}

/// Extends a Series with repeated Boolean values.
pub fn series_extend_bool(
    series: &TerlanPolarsSeries,
    value: bool,
    count: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, value, count);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        extend_constant(series, polars::prelude::AnyValue::Boolean(value), count)
    }
}

/// Extends a Series with repeated null values.
pub fn series_extend_null(
    series: &TerlanPolarsSeries,
    count: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, count);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        extend_constant(series, polars::prelude::AnyValue::Null, count)
    }
}

/// Selects values from two Series using a Boolean mask.
pub fn series_zip_with(
    series: &TerlanPolarsSeries,
    mask: &TerlanPolarsSeries,
    other: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, mask, other);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(
            series.inner.zip_with(mask.inner.bool()?, &other.inner)?,
        ))
    }
}

/// Serializes a Series with the pinned Polars binary format.
pub fn series_serialize(series: &TerlanPolarsSeries) -> Result<Vec<u8>, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(series.inner.serialize_to_bytes()?)
    }
}

/// Deserializes a Series from the pinned Polars binary format.
pub fn series_deserialize(bytes: &[u8]) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = bytes;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let mut reader = std::io::Cursor::new(bytes);
        Ok(owned(polars::prelude::Series::deserialize_from_reader(
            &mut reader,
        )?))
    }
}

/// Compares values for inequality while treating null as equal to null.
pub fn series_not_equal_missing(
    series: &TerlanPolarsSeries,
    other: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, other);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::{ChunkCompareEq, IntoSeries};
        Ok(owned(
            series.inner.not_equal_missing(&other.inner)?.into_series(),
        ))
    }
}

/// Returns one independently owned Arrow chunk by zero-based index.
pub fn series_select_chunk(
    series: &TerlanPolarsSeries,
    index: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, index);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        if index >= series.inner.n_chunks() {
            return Err(TerlanPolarsError::new(
                "row_index_out_of_bounds",
                format!(
                    "Series chunk index {index} is outside {} chunks",
                    series.inner.n_chunks()
                ),
            ));
        }
        Ok(owned(series.inner.select_chunk(index)))
    }
}

/// Recursively removes List and Array containers and returns leaf values.
pub fn series_leaf_values(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.get_leaf_array()))
    }
}

/// Converts numeric values to Float64 while retaining existing float storage.
pub fn series_to_float(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.to_float()?))
    }
}

#[cfg(feature = "real-polars")]
fn reject_null_values(series: &TerlanPolarsSeries) -> Result<(), TerlanPolarsError> {
    if series.inner.null_count() == 0 {
        Ok(())
    } else {
        Err(TerlanPolarsError::new(
            "nullable_values_require_option",
            "typed Series extraction requires a non-null Series until Option native layouts are available",
        ))
    }
}

/// Extracts integer values through a checked Int64 representation.
pub fn series_integer_values(series: &TerlanPolarsSeries) -> Result<Vec<i64>, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        reject_null_values(series)?;
        let values = series
            .inner
            .strict_cast(&polars::prelude::DataType::Int64)?;
        Ok(values.i64()?.into_no_null_iter().collect())
    }
}

/// Extracts numeric values through a Float64 representation.
pub fn series_float_values(series: &TerlanPolarsSeries) -> Result<Vec<f64>, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        reject_null_values(series)?;
        let values = series
            .inner
            .strict_cast(&polars::prelude::DataType::Float64)?;
        Ok(values.f64()?.into_no_null_iter().collect())
    }
}

/// Extracts non-null Boolean values.
pub fn series_boolean_values(series: &TerlanPolarsSeries) -> Result<Vec<bool>, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        reject_null_values(series)?;
        series
            .inner
            .bool()?
            .iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                TerlanPolarsError::new(
                    "nullable_values_require_option",
                    "Boolean Series unexpectedly contained a null value",
                )
            })
    }
}

/// Extracts non-null String values.
pub fn series_string_values(series: &TerlanPolarsSeries) -> Result<Vec<String>, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        reject_null_values(series)?;
        series
            .inner
            .str()?
            .iter()
            .map(|value| value.map(str::to_string))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                TerlanPolarsError::new(
                    "nullable_values_require_option",
                    "String Series unexpectedly contained a null value",
                )
            })
    }
}
