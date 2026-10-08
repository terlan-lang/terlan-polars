//! Extended immutable DataFrame construction, metadata, and interchange.

#[cfg(feature = "real-polars")]
use crate::expressions;
use crate::{TerlanPolarsDataFrame, TerlanPolarsError, TerlanPolarsFrameSet, TerlanPolarsSeries};

#[cfg(feature = "real-polars")]
fn owned(dataframe: polars::prelude::DataFrame) -> TerlanPolarsDataFrame {
    TerlanPolarsDataFrame { inner: dataframe }
}

#[cfg(feature = "real-polars")]
fn owned_series(series: polars::prelude::Series) -> TerlanPolarsSeries {
    TerlanPolarsSeries { inner: series }
}

#[cfg(not(feature = "real-polars"))]
fn unavailable() -> TerlanPolarsError {
    TerlanPolarsError::new(
        "polars_unavailable",
        "real Polars support is not enabled for this adapter build",
    )
}

#[cfg(feature = "real-polars")]
fn schema(
    names: &[String],
    data_types: &[String],
) -> Result<polars::prelude::Schema, TerlanPolarsError> {
    if names.len() != data_types.len() {
        return Err(TerlanPolarsError::new(
            "invalid_columns",
            "DataFrame schema names and data types must have equal lengths",
        ));
    }
    Ok(polars::prelude::Schema::from_iter_check_duplicates(
        names
            .iter()
            .zip(data_types)
            .map(|(name, data_type)| {
                Ok((
                    name.as_str().into(),
                    expressions::parse_data_type(data_type)?,
                ))
            })
            .collect::<Result<Vec<_>, TerlanPolarsError>>()?,
    )?)
}

/// Constructs a column-free DataFrame with an explicit height.
pub fn dataframe_empty_with_height(
    height: usize,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = height;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(polars::prelude::DataFrame::empty_with_height(height)))
    }
}

/// Constructs an empty DataFrame from parallel column names and data types.
pub fn dataframe_empty_with_schema(
    names: &[String],
    data_types: &[String],
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (names, data_types);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(polars::prelude::DataFrame::empty_with_schema(
            &schema(names, data_types)?,
        )))
    }
}

/// Constructs a typed DataFrame whose values are all null.
pub fn dataframe_full_null(
    names: &[String],
    data_types: &[String],
    height: usize,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (names, data_types, height);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(polars::prelude::DataFrame::full_null(
            &schema(names, data_types)?,
            height,
        )))
    }
}

/// Returns chunk lengths from the first materialized column.
pub fn dataframe_chunk_lengths(dataframe: &TerlanPolarsDataFrame) -> Vec<usize> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = dataframe;
        Vec::new()
    }
    #[cfg(feature = "real-polars")]
    {
        dataframe
            .inner
            .columns()
            .first()
            .map(|column| column.as_materialized_series().chunk_lengths().collect())
            .unwrap_or_default()
    }
}

/// Returns DataFrame height and width in that order.
pub fn dataframe_shape(dataframe: &TerlanPolarsDataFrame) -> Vec<usize> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = dataframe;
        Vec::new()
    }
    #[cfg(feature = "real-polars")]
    {
        let (height, width) = dataframe.inner.shape();
        vec![height, width]
    }
}

/// Returns the number of chunks in the first materialized column.
pub fn dataframe_first_col_n_chunks(dataframe: &TerlanPolarsDataFrame) -> usize {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = dataframe;
        0
    }
    #[cfg(feature = "real-polars")]
    {
        dataframe.inner.first_col_n_chunks()
    }
}

/// Returns the greatest chunk count among DataFrame columns.
pub fn dataframe_max_chunk_count(dataframe: &TerlanPolarsDataFrame) -> usize {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = dataframe;
        0
    }
    #[cfg(feature = "real-polars")]
    {
        dataframe.inner.max_n_chunks()
    }
}

/// Reports whether column chunk layouts require consolidation.
pub fn dataframe_should_rechunk(dataframe: &TerlanPolarsDataFrame) -> bool {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = dataframe;
        false
    }
    #[cfg(feature = "real-polars")]
    {
        dataframe.inner.should_rechunk()
    }
}

/// Returns whether either DataFrame dimension is zero.
pub fn dataframe_shape_has_zero(dataframe: &TerlanPolarsDataFrame) -> bool {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = dataframe;
        true
    }
    #[cfg(feature = "real-polars")]
    {
        dataframe.inner.shape().0 == 0 || dataframe.inner.shape().1 == 0
    }
}

/// Returns a copy whose column buffers are shrunk to fit.
pub fn dataframe_shrink_to_fit(
    dataframe: &TerlanPolarsDataFrame,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = dataframe;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let mut output = dataframe.inner.clone();
        output.shrink_to_fit();
        Ok(owned(output))
    }
}

/// Returns whether two DataFrames have identical ordered schemas.
pub fn dataframe_schema_equal(
    dataframe: &TerlanPolarsDataFrame,
    other: &TerlanPolarsDataFrame,
) -> bool {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, other);
        false
    }
    #[cfg(feature = "real-polars")]
    {
        dataframe.inner.schema() == other.inner.schema()
    }
}

/// Returns the zero-based index of a named column.
pub fn dataframe_column_index(
    dataframe: &TerlanPolarsDataFrame,
    name: &str,
) -> Result<usize, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, name);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        dataframe.inner.get_column_index(name).ok_or_else(|| {
            TerlanPolarsError::new(
                "missing_column",
                format!("missing DataFrame column `{name}`"),
            )
        })
    }
}

/// Returns one independently owned Series by zero-based column index.
pub fn dataframe_column_at_index(
    dataframe: &TerlanPolarsDataFrame,
    index: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, index);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        dataframe
            .inner
            .select_at_idx(index)
            .map(|column| owned_series(column.as_materialized_series().clone()))
            .ok_or_else(|| {
                TerlanPolarsError::new(
                    "row_index_out_of_bounds",
                    format!(
                        "DataFrame column index {index} is outside width {}",
                        dataframe.inner.width()
                    ),
                )
            })
    }
}

/// Materializes one row as stable display values.
pub fn dataframe_row(
    dataframe: &TerlanPolarsDataFrame,
    index: usize,
) -> Result<Vec<String>, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, index);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        if index >= dataframe.inner.height() {
            return Err(TerlanPolarsError::new(
                "row_index_out_of_bounds",
                format!(
                    "DataFrame row index {index} is outside height {}",
                    dataframe.inner.height()
                ),
            ));
        }
        Ok(dataframe
            .inner
            .get_row(index)?
            .0
            .into_iter()
            .map(crate::display_value)
            .collect())
    }
}

/// Returns rows before a Polars split offset.
pub fn dataframe_split_before(
    dataframe: &TerlanPolarsDataFrame,
    offset: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, offset);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(dataframe.inner.split_at(offset).0))
    }
}

/// Returns rows at and after a Polars split offset.
pub fn dataframe_split_after(
    dataframe: &TerlanPolarsDataFrame,
    offset: i64,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, offset);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(dataframe.inner.split_at(offset).1))
    }
}

/// Returns a copy with a Series inserted at a zero-based column index.
pub fn dataframe_insert_column(
    dataframe: &TerlanPolarsDataFrame,
    index: usize,
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, index, series);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoColumn;
        let mut output = dataframe.inner.clone();
        output.insert_column(index, series.inner.clone().into_column())?;
        Ok(owned(output))
    }
}

/// Returns a copy with a Series replacing a zero-based column index.
pub fn dataframe_replace_column(
    dataframe: &TerlanPolarsDataFrame,
    index: usize,
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, index, series);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoColumn;
        let mut output = dataframe.inner.clone();
        output.replace_column(index, series.inner.clone().into_column())?;
        Ok(owned(output))
    }
}

/// Returns a copy with a Series added or replacing the same named column.
pub fn dataframe_with_column(
    dataframe: &TerlanPolarsDataFrame,
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, series);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoColumn;
        let mut output = dataframe.inner.clone();
        output.with_column(series.inner.clone().into_column())?;
        Ok(owned(output))
    }
}

/// Packs DataFrame rows into a named Struct Series.
pub fn dataframe_to_struct(
    dataframe: &TerlanPolarsDataFrame,
    name: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, name);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::IntoSeries;
        Ok(owned_series(
            dataframe
                .inner
                .clone()
                .into_struct(name.into())
                .into_series(),
        ))
    }
}

/// Serializes a DataFrame with the pinned Polars binary format.
pub fn dataframe_serialize(
    dataframe: &TerlanPolarsDataFrame,
) -> Result<Vec<u8>, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = dataframe;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let mut output = dataframe.inner.clone();
        let mut bytes = Vec::new();
        output.serialize_into_writer(&mut bytes)?;
        Ok(bytes)
    }
}

/// Deserializes a DataFrame from the pinned Polars binary format.
pub fn dataframe_deserialize(bytes: &[u8]) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = bytes;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let mut reader = std::io::Cursor::new(bytes);
        Ok(owned(polars::prelude::DataFrame::deserialize_from_reader(
            &mut reader,
        )?))
    }
}

/// Partitions rows by one or more key columns.
pub fn dataframe_partition_by(
    dataframe: &TerlanPolarsDataFrame,
    columns: &[String],
    stable: bool,
    include_key: bool,
) -> Result<TerlanPolarsFrameSet, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, columns, stable, include_key);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        if columns.is_empty() {
            return Err(TerlanPolarsError::new(
                "invalid_columns",
                "DataFrame partitioning requires at least one key column",
            ));
        }
        let frames = if stable {
            dataframe.inner.partition_by_stable(columns, include_key)?
        } else {
            dataframe.inner.partition_by(columns, include_key)?
        };
        Ok(TerlanPolarsFrameSet {
            frames: frames.into_iter().map(owned).collect(),
        })
    }
}

/// Returns the number of DataFrames in an owned frame set.
pub fn dataframe_set_len(set: &TerlanPolarsFrameSet) -> usize {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = set;
        0
    }
    #[cfg(feature = "real-polars")]
    {
        set.frames.len()
    }
}

/// Returns an independently owned DataFrame from a frame set.
pub fn dataframe_set_get(
    set: &TerlanPolarsFrameSet,
    index: usize,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (set, index);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        set.frames.get(index).cloned().ok_or_else(|| {
            TerlanPolarsError::new(
                "row_index_out_of_bounds",
                format!(
                    "DataFrame set index {index} is outside length {}",
                    set.frames.len()
                ),
            )
        })
    }
}

/// Splits a DataFrame into independently owned aligned chunks.
pub fn dataframe_split_chunks(
    dataframe: &TerlanPolarsDataFrame,
    physical: bool,
) -> Result<TerlanPolarsFrameSet, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, physical);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let mut output = dataframe.inner.clone();
        if physical {
            output = output
                .columns()
                .iter()
                .map(|column| {
                    Ok(owned_series(
                        column
                            .as_materialized_series()
                            .to_physical_repr()
                            .into_owned(),
                    ))
                })
                .collect::<Result<Vec<_>, TerlanPolarsError>>()?
                .into_iter()
                .try_fold(
                    polars::prelude::DataFrame::empty_with_height(output.height()),
                    |mut frame, series| {
                        frame.with_column(series.inner.into())?;
                        Ok::<_, polars::prelude::PolarsError>(frame)
                    },
                )?;
        }
        let frames = output.split_chunks().map(owned).collect();
        Ok(TerlanPolarsFrameSet { frames })
    }
}

/// Splits a DataFrame into at most the requested number of row partitions.
pub fn dataframe_split_chunks_by_n(
    dataframe: &TerlanPolarsDataFrame,
    partitions: usize,
    parallel: bool,
) -> Result<TerlanPolarsFrameSet, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (dataframe, partitions, parallel);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        if partitions == 0 {
            return Err(TerlanPolarsError::new(
                "invalid_row_limit",
                "DataFrame chunk partition count must be positive",
            ));
        }
        Ok(TerlanPolarsFrameSet {
            frames: dataframe
                .inner
                .clone()
                .split_chunks_by_n(partitions, parallel)
                .into_iter()
                .map(owned)
                .collect(),
        })
    }
}
