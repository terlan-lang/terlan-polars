//! Pointer-free tensor exchange owned by the Polars package.
//!
//! Polars chooses and validates tabular columns here. The VM later replaces
//! the returned TNXP packet with a one-shot, authenticated exchange token.

use crate::{TerlanPolarsDataFrame, TerlanPolarsError, TerlanPolarsSeries};
use std::collections::BTreeSet;

#[cfg(feature = "real-polars")]
const TENSOR_PACKET_HEADER_BYTES: usize = 16;
#[cfg(feature = "real-polars")]
const TENSOR_PACKET_RANK: usize = 2;
#[cfg(feature = "real-polars")]
const DLPACK_FLOAT: u8 = 2;
#[cfg(feature = "real-polars")]
const DLPACK_CPU: u8 = 1;

/// Materializes selected DataFrame columns as a contiguous Float64 TNXP packet.
///
/// The column order is preserved and becomes the second tensor dimension.
/// Nulls and non-numeric input are rejected rather than silently imputed or
/// coerced. The consumer is validated here and authenticated by the VM broker.
#[cfg(feature = "real-polars")]
pub fn dataframe_tensor_packet(
    dataframe: &TerlanPolarsDataFrame,
    columns: &[String],
    dtype: &str,
    null_policy: &str,
    consumer: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use polars::prelude::DataType;

    validate_request(
        dataframe.inner.height(),
        columns,
        dtype,
        null_policy,
        consumer,
    )?;
    let row_count = dataframe.inner.height();
    let column_count = columns.len();
    let element_count = row_count
        .checked_mul(column_count)
        .ok_or_else(|| tensor_error("tensor_size_overflow", "tensor element count overflowed"))?;
    let data_bytes = element_count
        .checked_mul(size_of::<f64>())
        .ok_or_else(|| tensor_error("tensor_size_overflow", "tensor byte size overflowed"))?;
    let metadata_bytes = TENSOR_PACKET_HEADER_BYTES + TENSOR_PACKET_RANK * 16;
    let mut packet = Vec::with_capacity(metadata_bytes + data_bytes);

    packet.extend_from_slice(b"TNXP");
    packet.push(1);
    packet.push(native_endian_code());
    packet.push(DLPACK_FLOAT);
    packet.push(64);
    packet.extend_from_slice(&1u16.to_le_bytes());
    packet.push(DLPACK_CPU);
    packet.push(0);
    packet.extend_from_slice(&(TENSOR_PACKET_RANK as u32).to_le_bytes());
    packet.extend_from_slice(&(row_count as i64).to_le_bytes());
    packet.extend_from_slice(&(column_count as i64).to_le_bytes());
    packet.extend_from_slice(&(column_count as i64).to_le_bytes());
    packet.extend_from_slice(&1i64.to_le_bytes());

    let mut converted = Vec::with_capacity(column_count);
    for name in columns {
        let series = dataframe
            .inner
            .column(name)
            .map_err(|_| tensor_error("missing_column", format!("missing column `{name}`")))?
            .as_materialized_series();
        if !is_supported_numeric_type(series.dtype()) {
            return Err(tensor_error(
                "dtype_mismatch",
                format!(
                    "column `{name}` has unsupported dtype `{:?}`; expected a numeric dtype",
                    series.dtype()
                ),
            ));
        }
        if series.null_count() != 0 {
            return Err(tensor_error(
                "null_value",
                format!(
                    "column `{name}` contains {} null value(s); null policy is `error`",
                    series.null_count()
                ),
            ));
        }
        let values = series.cast(&DataType::Float64).map_err(|error| {
            tensor_error(
                "dtype_mismatch",
                format!("column `{name}` cannot be converted to Float64: {error}"),
            )
        })?;
        converted.push(values.rechunk());
    }

    for row in 0..row_count {
        for (column_index, series) in converted.iter().enumerate() {
            let value = series.f64().map_err(|error| {
                tensor_error(
                    "dtype_mismatch",
                    format!("column `{}` is not Float64: {error}", columns[column_index]),
                )
            })?;
            let value = value.get(row).ok_or_else(|| {
                tensor_error(
                    "null_value",
                    format!("column `{}` contains a null value", columns[column_index]),
                )
            })?;
            packet.extend_from_slice(&value.to_ne_bytes());
        }
    }
    debug_assert_eq!(packet.len(), metadata_bytes + data_bytes);
    Ok(packet)
}

/// Materializes a numeric Series as a contiguous one-column Float64 packet.
///
/// The Series name is used as the packet's only column name. Conversion is
/// delegated to the DataFrame path so row-major layout, casting, null policy,
/// and request validation remain identical for both APIs.
#[cfg(feature = "real-polars")]
pub fn series_tensor_packet(
    series: &TerlanPolarsSeries,
    dtype: &str,
    null_policy: &str,
    consumer: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    let dataframe = crate::series_to_frame(series)?;
    let columns = vec![series.inner.name().to_string()];
    dataframe_tensor_packet(&dataframe, &columns, dtype, null_policy, consumer)
}

#[cfg(feature = "real-polars")]
fn is_supported_numeric_type(dtype: &polars::prelude::DataType) -> bool {
    use polars::prelude::DataType;

    matches!(
        dtype,
        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::Int128
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Boolean
            | DataType::Float32
            | DataType::Float64
    )
}

#[cfg(not(feature = "real-polars"))]
pub fn dataframe_tensor_packet(
    _dataframe: &TerlanPolarsDataFrame,
    columns: &[String],
    dtype: &str,
    null_policy: &str,
    consumer: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    validate_request(1, columns, dtype, null_policy, consumer)?;
    Err(tensor_error(
        "native_unavailable",
        "DataFrame tensor exchange requires the `real-polars` feature",
    ))
}

#[cfg(not(feature = "real-polars"))]
pub fn series_tensor_packet(
    _series: &TerlanPolarsSeries,
    dtype: &str,
    null_policy: &str,
    consumer: &str,
) -> Result<Vec<u8>, TerlanPolarsError> {
    validate_request(1, &["series".to_string()], dtype, null_policy, consumer)?;
    Err(tensor_error(
        "native_unavailable",
        "Series tensor exchange requires the `real-polars` feature",
    ))
}

fn validate_request(
    row_count: usize,
    columns: &[String],
    dtype: &str,
    null_policy: &str,
    consumer: &str,
) -> Result<(), TerlanPolarsError> {
    if columns.is_empty() {
        return Err(tensor_error(
            "empty_columns",
            "DataFrame tensor conversion requires at least one column",
        ));
    }
    let mut unique = BTreeSet::new();
    if let Some(duplicate) = columns
        .iter()
        .find(|column| column.is_empty() || !unique.insert(column.as_str()))
    {
        let message = if duplicate.is_empty() {
            "DataFrame tensor columns must not be empty".to_string()
        } else {
            format!("duplicate tensor column `{duplicate}`")
        };
        return Err(tensor_error("invalid_columns", message));
    }
    if row_count == 0 {
        return Err(tensor_error(
            "empty_dataframe",
            "DataFrame tensor conversion requires at least one row",
        ));
    }
    if dtype != "Float64" {
        return Err(tensor_error(
            "unsupported_dtype",
            format!("unsupported tensor dtype `{dtype}`; expected `Float64`"),
        ));
    }
    if null_policy != "error" {
        return Err(tensor_error(
            "unsupported_null_policy",
            format!("unsupported null policy `{null_policy}`; expected `error`"),
        ));
    }
    if consumer.is_empty()
        || !consumer
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '_')
    {
        return Err(tensor_error(
            "invalid_consumer",
            format!("invalid tensor consumer namespace `{consumer}`"),
        ));
    }
    Ok(())
}

fn tensor_error(code: &'static str, message: impl Into<String>) -> TerlanPolarsError {
    TerlanPolarsError::new(code, message)
}

#[cfg(feature = "real-polars")]
fn native_endian_code() -> u8 {
    if cfg!(target_endian = "little") {
        1
    } else {
        2
    }
}

#[cfg(all(test, feature = "real-polars"))]
mod tests {
    use super::*;
    use polars::prelude::{DataFrame, NamedFrom, Series};

    fn frame(columns: Vec<Series>) -> TerlanPolarsDataFrame {
        let height = columns.first().map_or(0, |series| series.len());
        TerlanPolarsDataFrame {
            inner: DataFrame::new(height, columns.into_iter().map(Into::into).collect())
                .expect("test DataFrame"),
        }
    }

    #[test]
    fn packet_preserves_explicit_column_order_and_row_major_shape() {
        let dataframe = frame(vec![
            Series::new("feature".into(), &[1.0f64, 2.0]),
            Series::new("label".into(), &[10i64, 20]),
        ]);
        let packet = dataframe_tensor_packet(
            &dataframe,
            &["label".to_string(), "feature".to_string()],
            "Float64",
            "error",
            "ndarray",
        )
        .expect("tensor packet");

        assert_eq!(&packet[..4], b"TNXP");
        assert_eq!(u32::from_le_bytes(packet[12..16].try_into().unwrap()), 2);
        assert_eq!(i64::from_le_bytes(packet[16..24].try_into().unwrap()), 2);
        assert_eq!(i64::from_le_bytes(packet[24..32].try_into().unwrap()), 2);
        let values = packet[48..]
            .chunks_exact(8)
            .map(|bytes| f64::from_ne_bytes(bytes.try_into().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(values, vec![10.0, 1.0, 20.0, 2.0]);
    }

    #[test]
    fn packet_rejects_nulls_and_non_numeric_columns() {
        let nullable = frame(vec![Series::new("feature".into(), &[Some(1.0f64), None])]);
        let error = dataframe_tensor_packet(
            &nullable,
            &["feature".to_string()],
            "Float64",
            "error",
            "ndarray",
        )
        .expect_err("null must fail");
        assert_eq!(error.code(), "null_value");

        let strings = frame(vec![Series::new("feature".into(), &["1", "2"])]);
        let error = dataframe_tensor_packet(
            &strings,
            &["feature".to_string()],
            "Float64",
            "error",
            "ndarray",
        )
        .expect_err("string dtype must fail");
        assert_eq!(error.code(), "dtype_mismatch");
    }

    #[test]
    fn packet_consolidates_chunks_before_row_major_encoding() {
        let mut chunked = Series::new("feature".into(), &[1i64, 2]);
        chunked
            .append(&Series::new("feature".into(), &[3i64, 4]))
            .expect("append second chunk");
        assert_eq!(chunked.n_chunks(), 2);
        let dataframe = frame(vec![
            chunked.clone(),
            Series::new("weight".into(), &[0.5f32, 1.5, 2.5, 3.5]),
        ]);

        let packet = dataframe_tensor_packet(
            &dataframe,
            &["feature".to_string(), "weight".to_string()],
            "Float64",
            "error",
            "ndarray",
        )
        .expect("chunked tensor packet");
        let values = packet[48..]
            .chunks_exact(8)
            .map(|bytes| f64::from_ne_bytes(bytes.try_into().unwrap()))
            .collect::<Vec<_>>();

        assert_eq!(values, vec![1.0, 0.5, 2.0, 1.5, 3.0, 2.5, 4.0, 3.5]);
        assert_eq!(
            chunked.n_chunks(),
            2,
            "conversion must not mutate its source"
        );
    }

    #[test]
    fn packet_casts_mixed_supported_primitive_columns_to_float64() {
        let dataframe = frame(vec![
            Series::new("signed".into(), &[-2i8, 3]),
            Series::new("unsigned".into(), &[4u64, 5]),
            Series::new("enabled".into(), &[true, false]),
            Series::new("fraction".into(), &[1.25f32, 2.5]),
        ]);
        let columns = ["signed", "unsigned", "enabled", "fraction"].map(str::to_string);

        let packet = dataframe_tensor_packet(&dataframe, &columns, "Float64", "error", "ndarray")
            .expect("mixed primitive tensor packet");
        let values = packet[48..]
            .chunks_exact(8)
            .map(|bytes| f64::from_ne_bytes(bytes.try_into().unwrap()))
            .collect::<Vec<_>>();

        assert_eq!(values, vec![-2.0, 4.0, 1.0, 1.25, 3.0, 5.0, 0.0, 2.5]);
    }

    #[test]
    fn packet_rejects_temporal_and_nested_physical_layouts() {
        use polars::prelude::{DataType, IntoSeries, ListChunked};

        let dates = Series::new("observed_on".into(), &[1i32, 2])
            .cast(&DataType::Date)
            .expect("Date fixture");
        let date_frame = frame(vec![dates]);
        let error = dataframe_tensor_packet(
            &date_frame,
            &["observed_on".to_string()],
            "Float64",
            "error",
            "ndarray",
        )
        .expect_err("temporal physical storage must not be interpreted as numeric");
        assert_eq!(error.code(), "dtype_mismatch");

        let nested = ListChunked::from_iter(
            [
                Some(Series::new("".into(), &[1i64, 2])),
                Some(Series::new("".into(), &[3i64])),
            ]
            .into_iter(),
        )
        .with_name("items".into())
        .into_series();
        let nested_frame = frame(vec![nested]);
        let error = dataframe_tensor_packet(
            &nested_frame,
            &["items".to_string()],
            "Float64",
            "error",
            "ndarray",
        )
        .expect_err("nested physical storage must not be interpreted as numeric");
        assert_eq!(error.code(), "dtype_mismatch");
    }

    #[test]
    fn series_packet_uses_one_column_row_major_layout() {
        let series = TerlanPolarsSeries {
            inner: Series::new("score".into(), &[1i32, 2, 3]),
        };
        let packet = series_tensor_packet(&series, "Float64", "error", "ndarray")
            .expect("series tensor packet");

        assert_eq!(i64::from_le_bytes(packet[16..24].try_into().unwrap()), 3);
        assert_eq!(i64::from_le_bytes(packet[24..32].try_into().unwrap()), 1);
        let values = packet[48..]
            .chunks_exact(8)
            .map(|bytes| f64::from_ne_bytes(bytes.try_into().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn series_packet_casts_boolean_values_to_float64() {
        let series = TerlanPolarsSeries {
            inner: Series::new("enabled".into(), &[true, false, true]),
        };
        let packet = series_tensor_packet(&series, "Float64", "error", "ndarray")
            .expect("boolean series tensor packet");
        let values = packet[48..]
            .chunks_exact(8)
            .map(|bytes| f64::from_ne_bytes(bytes.try_into().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(values, vec![1.0, 0.0, 1.0]);
    }

    #[test]
    fn series_packet_rejects_nulls_and_unsupported_types() {
        let nullable = TerlanPolarsSeries {
            inner: Series::new("score".into(), &[Some(1i64), None]),
        };
        let error = series_tensor_packet(&nullable, "Float64", "error", "ndarray")
            .expect_err("null must fail");
        assert_eq!(error.code(), "null_value");

        let strings = TerlanPolarsSeries {
            inner: Series::new("score".into(), &["1", "2"]),
        };
        let error = series_tensor_packet(&strings, "Float64", "error", "ndarray")
            .expect_err("string dtype must fail");
        assert_eq!(error.code(), "dtype_mismatch");
    }

    #[test]
    fn tensor_requests_reject_invalid_shape_policy_and_columns() {
        let dataframe = frame(vec![Series::new("value".into(), &[1.0f64, 2.0])]);
        let cases = [
            (&[][..], "Float64", "error", "ndarray", "empty_columns"),
            (
                &["value", "value"][..],
                "Float64",
                "error",
                "ndarray",
                "invalid_columns",
            ),
            (&[""][..], "Float64", "error", "ndarray", "invalid_columns"),
            (
                &["missing"][..],
                "Float64",
                "error",
                "ndarray",
                "missing_column",
            ),
            (
                &["value"][..],
                "Int64",
                "error",
                "ndarray",
                "unsupported_dtype",
            ),
            (
                &["value"][..],
                "Float64",
                "impute_zero",
                "ndarray",
                "unsupported_null_policy",
            ),
            (
                &["value"][..],
                "Float64",
                "error",
                "NdArray",
                "invalid_consumer",
            ),
        ];
        for (columns, dtype, null_policy, consumer, code) in cases {
            let error = dataframe_tensor_packet(
                &dataframe,
                &columns
                    .iter()
                    .map(|column| (*column).to_string())
                    .collect::<Vec<_>>(),
                dtype,
                null_policy,
                consumer,
            )
            .expect_err("invalid tensor request must fail");
            assert_eq!(error.code(), code);
        }

        let empty = frame(Vec::new());
        let error = dataframe_tensor_packet(
            &empty,
            &["value".to_string()],
            "Float64",
            "error",
            "ndarray",
        )
        .expect_err("empty DataFrame must fail");
        assert_eq!(error.code(), "empty_dataframe");
    }

    #[test]
    fn iris_features_form_an_exact_dense_array_packet() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../examples/iris_dataset_audit/data/iris.csv");
        let dataframe =
            crate::read_csv(fixture.to_str().expect("UTF-8 Iris path")).expect("read Iris fixture");
        let columns = [
            "sepal_length_cm",
            "sepal_width_cm",
            "petal_length_cm",
            "petal_width_cm",
        ]
        .map(str::to_string);
        let packet = dataframe_tensor_packet(&dataframe, &columns, "Float64", "error", "ndarray")
            .expect("Iris ndarray packet");

        assert_eq!(i64::from_le_bytes(packet[16..24].try_into().unwrap()), 150);
        assert_eq!(i64::from_le_bytes(packet[24..32].try_into().unwrap()), 4);
        let values = packet[48..]
            .chunks_exact(8)
            .map(|bytes| f64::from_ne_bytes(bytes.try_into().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(values.len(), 600);
        assert_eq!(&values[..4], &[5.1, 3.5, 1.4, 0.2]);
        assert_eq!(&values[596..], &[5.9, 3.0, 5.1, 1.8]);
        assert!(values.iter().all(|value| (0.1..=7.9).contains(value)));
    }
}
