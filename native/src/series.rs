//! Owned Series operations exposed through the Terlan package boundary.
//!
//! Mutating Polars methods are reshaped into immutable operations that return
//! a new owned Series. This keeps aliases deterministic while preserving the
//! observable values, names, data types, and chunk behavior.

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

/// Returns a copy of a Series with a new name.
pub fn series_rename(
    series: &TerlanPolarsSeries,
    name: &str,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, name);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.clone().with_name(name.into())))
    }
}

/// Returns the estimated heap size of a Series in bytes.
pub fn series_estimated_size(series: &TerlanPolarsSeries) -> usize {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        0
    }
    #[cfg(feature = "real-polars")]
    {
        series.inner.estimated_size()
    }
}

/// Returns Polars' estimated Series heap size scaled to a compatible size unit.
pub fn series_estimated_size_with_unit(
    series: &TerlanPolarsSeries,
    unit: &str,
) -> Result<f64, TerlanPolarsError> {
    crate::scale_estimated_size(series_estimated_size(series), unit)
}

/// Returns the number of Arrow chunks backing a Series.
pub fn series_chunk_count(series: &TerlanPolarsSeries) -> usize {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        0
    }
    #[cfg(feature = "real-polars")]
    {
        series.inner.n_chunks()
    }
}

/// Returns the length of every Arrow chunk backing a Series.
pub fn series_chunk_lengths(series: &TerlanPolarsSeries) -> Vec<usize> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Vec::new()
    }
    #[cfg(feature = "real-polars")]
    {
        series.inner.chunk_lengths().collect()
    }
}

/// Returns the ordering metadata recorded on a Series.
pub fn series_sorted_flag(series: &TerlanPolarsSeries) -> String {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        "not_sorted".to_string()
    }
    #[cfg(feature = "real-polars")]
    {
        match series.inner.is_sorted_flag() {
            polars::series::IsSorted::Ascending => "ascending",
            polars::series::IsSorted::Descending => "descending",
            polars::series::IsSorted::Not => "not_sorted",
        }
        .to_string()
    }
}

/// Returns an empty Series with the input name and data type.
pub fn series_clear(series: &TerlanPolarsSeries) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.clear()))
    }
}

/// Returns the first `length` values of a Series.
pub fn series_head(
    series: &TerlanPolarsSeries,
    length: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, length);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.head(Some(length))))
    }
}

/// Returns the last `length` values of a Series.
pub fn series_tail(
    series: &TerlanPolarsSeries,
    length: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, length);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.tail(Some(length))))
    }
}

/// Returns a Series slice using Polars negative-offset semantics.
pub fn series_slice(
    series: &TerlanPolarsSeries,
    offset: i64,
    length: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, offset, length);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.slice(offset, length)))
    }
}

/// Returns the values before a Polars split offset.
pub fn series_split_before(
    series: &TerlanPolarsSeries,
    offset: i64,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, offset);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.split_at(offset).0))
    }
}

/// Returns the values at and after a Polars split offset.
pub fn series_split_after(
    series: &TerlanPolarsSeries,
    offset: i64,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, offset);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.split_at(offset).1))
    }
}

/// Consolidates a Series into one Arrow chunk.
pub fn series_rechunk(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.rechunk()))
    }
}

/// Shrinks cloned Series buffers to fit their current lengths.
pub fn series_shrink_to_fit(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        let mut output = series.inner.clone();
        output.shrink_to_fit();
        Ok(owned(output))
    }
}

/// Reverses the values in a Series.
pub fn series_reverse(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.reverse()))
    }
}

/// Sorts a Series with explicit direction, null placement, and stability.
pub fn series_sort(
    series: &TerlanPolarsSeries,
    descending: bool,
    nulls_last: bool,
    maintain_order: bool,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, descending, nulls_last, maintain_order);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.sort(polars::prelude::SortOptions {
            descending,
            nulls_last,
            maintain_order,
            ..Default::default()
        })?))
    }
}

/// Returns distinct Series values while preserving first-occurrence order.
pub fn series_unique_stable(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.unique_stable()?))
    }
}

/// Explodes a List or String Series with explicit empty/null behavior.
pub fn series_explode(
    series: &TerlanPolarsSeries,
    empty_as_null: bool,
    keep_nulls: bool,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, empty_as_null, keep_nulls);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.explode(
            polars::prelude::ExplodeOptions {
                empty_as_null,
                keep_nulls,
            },
        )?))
    }
}

#[cfg(feature = "real-polars")]
pub(crate) fn fill_strategy(
    strategy: &str,
    limit: Option<usize>,
) -> Result<polars::prelude::FillNullStrategy, TerlanPolarsError> {
    if limit.is_some() && !matches!(strategy, "forward" | "backward") {
        return Err(TerlanPolarsError::new(
            "invalid_limit",
            "fill limits are only valid for forward or backward strategies",
        ));
    }
    let limit = limit
        .map(u32::try_from)
        .transpose()
        .map_err(|_| TerlanPolarsError::new("invalid_limit", "fill limit exceeds UInt32"))?;
    match strategy {
        "backward" => Ok(polars::prelude::FillNullStrategy::Backward(limit)),
        "forward" => Ok(polars::prelude::FillNullStrategy::Forward(limit)),
        "mean" => Ok(polars::prelude::FillNullStrategy::Mean),
        "min" => Ok(polars::prelude::FillNullStrategy::Min),
        "max" => Ok(polars::prelude::FillNullStrategy::Max),
        "zero" => Ok(polars::prelude::FillNullStrategy::Zero),
        "one" => Ok(polars::prelude::FillNullStrategy::One),
        other => Err(TerlanPolarsError::new(
            "invalid_fill_strategy",
            format!("unsupported Series fill strategy `{other}`"),
        )),
    }
}

/// Fills null values using a named Polars strategy and optional directional limit.
pub fn series_fill_null(
    series: &TerlanPolarsSeries,
    strategy: &str,
    limit: Option<usize>,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, strategy, limit);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(
            series.inner.fill_null(fill_strategy(strategy, limit)?)?,
        ))
    }
}

/// Selects every `step`th value starting at `offset`.
pub fn series_gather_every(
    series: &TerlanPolarsSeries,
    step: usize,
    offset: usize,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    if step == 0 {
        return Err(TerlanPolarsError::new(
            "invalid_series_step",
            "Series gather step must be positive",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, offset);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.gather_every(step, offset)?))
    }
}

/// Randomly shuffles a Series.
pub fn series_shuffle(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    series_shuffle_with_seed(series, None)
}

/// Randomly shuffles a Series using a deterministic seed.
pub fn series_shuffle_seeded(
    series: &TerlanPolarsSeries,
    seed: u64,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    series_shuffle_with_seed(series, Some(seed))
}

fn series_shuffle_with_seed(
    series: &TerlanPolarsSeries,
    seed: Option<u64>,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, seed);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.shuffle(seed)))
    }
}

/// Samples a fixed number of Series values.
pub fn series_sample_n(
    series: &TerlanPolarsSeries,
    count: usize,
    with_replacement: bool,
    shuffle: bool,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    sample_n_with_seed(series, count, with_replacement, shuffle, None)
}

/// Samples a fixed number of Series values using a deterministic seed.
pub fn series_sample_n_seeded(
    series: &TerlanPolarsSeries,
    count: usize,
    with_replacement: bool,
    shuffle: bool,
    seed: u64,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    sample_n_with_seed(series, count, with_replacement, shuffle, Some(seed))
}

fn sample_n_with_seed(
    series: &TerlanPolarsSeries,
    count: usize,
    with_replacement: bool,
    shuffle: bool,
    seed: Option<u64>,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, count, with_replacement, shuffle, seed);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.sample_n(
            count,
            with_replacement,
            Some(shuffle),
            seed,
        )?))
    }
}

/// Samples a fraction of Series values.
pub fn series_sample_fraction(
    series: &TerlanPolarsSeries,
    fraction: f64,
    with_replacement: bool,
    shuffle: bool,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    sample_fraction_with_seed(series, fraction, with_replacement, shuffle, None)
}

/// Samples a fraction of Series values using a deterministic seed.
pub fn series_sample_fraction_seeded(
    series: &TerlanPolarsSeries,
    fraction: f64,
    with_replacement: bool,
    shuffle: bool,
    seed: u64,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    sample_fraction_with_seed(series, fraction, with_replacement, shuffle, Some(seed))
}

fn sample_fraction_with_seed(
    series: &TerlanPolarsSeries,
    fraction: f64,
    with_replacement: bool,
    shuffle: bool,
    seed: Option<u64>,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    if !fraction.is_finite() || fraction < 0.0 {
        return Err(TerlanPolarsError::new(
            "invalid_sample_fraction",
            "Series sample fraction must be finite and non-negative",
        ));
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (series, with_replacement, shuffle, seed);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        Ok(owned(series.inner.sample_frac(
            fraction,
            with_replacement,
            Some(shuffle),
            seed,
        )?))
    }
}

/// Appends another Series while retaining its chunk boundaries.
pub fn series_append(
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
        let mut output = series.inner.clone();
        output.append(&other.inner)?;
        Ok(owned(output))
    }
}

/// Extends another Series into the current terminal chunk.
pub fn series_extend(
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
        let mut output = series.inner.clone();
        output.extend(&other.inner)?;
        Ok(owned(output))
    }
}

#[cfg(feature = "real-polars")]
fn reduction(
    series: &TerlanPolarsSeries,
    operation: impl FnOnce(
        &polars::prelude::Series,
    ) -> polars::prelude::PolarsResult<polars::prelude::Scalar>,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    Ok(owned(
        operation(&series.inner)?.into_series(series.inner.name().clone()),
    ))
}

/// Reduces a Series to its sum as a one-element Series preserving data type.
pub fn series_sum(series: &TerlanPolarsSeries) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        reduction(series, |input| input.sum_reduce())
    }
}

/// Reduces a Series to its minimum as a one-element Series preserving data type.
pub fn series_min(series: &TerlanPolarsSeries) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        reduction(series, |input| input.min_reduce())
    }
}

/// Reduces a Series to its maximum as a one-element Series preserving data type.
pub fn series_max(series: &TerlanPolarsSeries) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        reduction(series, |input| input.max_reduce())
    }
}

/// Reduces a Series to its product as a one-element Series preserving data type.
pub fn series_product(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        reduction(series, |input| input.product())
    }
}

/// Reduces a Series to its mean as a one-element Series preserving data type.
pub fn series_mean(series: &TerlanPolarsSeries) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        reduction(series, |input| input.mean_reduce())
    }
}

#[cfg(feature = "real-polars")]
fn binary_values(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
    operation: impl FnOnce(
        &polars::prelude::Series,
        &polars::prelude::Series,
    ) -> polars::prelude::PolarsResult<polars::prelude::Series>,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::DataType;

    let signed_uint64 = (left.inner.dtype().is_signed_integer()
        && right.inner.dtype() == &DataType::UInt64)
        || (right.inner.dtype().is_signed_integer() && left.inner.dtype() == &DataType::UInt64);
    if signed_uint64 {
        let left = left.inner.cast(&DataType::Int128)?;
        let right = right.inner.cast(&DataType::Int128)?;
        Ok(owned(operation(&left, &right)?))
    } else {
        Ok(owned(operation(&left.inner, &right.inner)?))
    }
}

/// Adds two Series with Polars broadcasting and coercion semantics.
pub fn series_add(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        binary_values(left, right, |left, right| left + right)
    }
}

/// Subtracts two Series with Polars broadcasting and coercion semantics.
pub fn series_subtract(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        binary_values(left, right, |left, right| left - right)
    }
}

/// Multiplies two Series with Polars broadcasting and coercion semantics.
pub fn series_multiply(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        binary_values(left, right, |left, right| left * right)
    }
}

/// Divides two Series with Polars broadcasting and coercion semantics.
pub fn series_divide(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        binary_values(left, right, |left, right| left / right)
    }
}

/// Computes the remainder of two Series with Polars coercion semantics.
pub fn series_remainder(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        binary_values(left, right, |left, right| left % right)
    }
}

/// Applies value-wise bitwise AND to two integer or Boolean Series.
pub fn series_bit_and(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        if (left.inner.dtype().is_bool() && right.inner.dtype().is_integer())
            || (right.inner.dtype().is_bool() && left.inner.dtype().is_integer())
        {
            return Err(TerlanPolarsError::new(
                "invalid_operation",
                "bitwise operations between Boolean and integer Series are not supported in Polars 2.0",
            ));
        }
        binary_values(left, right, |left, right| left & right)
    }
}

/// Applies value-wise bitwise OR to two integer or Boolean Series.
pub fn series_bit_or(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        if (left.inner.dtype().is_bool() && right.inner.dtype().is_integer())
            || (right.inner.dtype().is_bool() && left.inner.dtype().is_integer())
        {
            return Err(TerlanPolarsError::new(
                "invalid_operation",
                "bitwise operations between Boolean and integer Series are not supported in Polars 2.0",
            ));
        }
        binary_values(left, right, |left, right| left | right)
    }
}

/// Applies value-wise bitwise XOR to two integer or Boolean Series.
pub fn series_bit_xor(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        if (left.inner.dtype().is_bool() && right.inner.dtype().is_integer())
            || (right.inner.dtype().is_bool() && left.inner.dtype().is_integer())
        {
            return Err(TerlanPolarsError::new(
                "invalid_operation",
                "bitwise operations between Boolean and integer Series are not supported in Polars 2.0",
            ));
        }
        binary_values(left, right, |left, right| left ^ right)
    }
}

#[cfg(feature = "real-polars")]
fn comparison(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
    operation: impl FnOnce(
        &polars::prelude::Series,
        &polars::prelude::Series,
    ) -> polars::prelude::PolarsResult<polars::prelude::BooleanChunked>,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::IntoSeries;
    Ok(owned(operation(&left.inner, &right.inner)?.into_series()))
}

/// Compares two Series value-wise for equality.
pub fn series_equal_values(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::ChunkCompareEq;
        comparison(left, right, |left, right| left.equal(right))
    }
}

/// Compares two Series value-wise for inequality.
pub fn series_not_equal_values(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::ChunkCompareEq;
        comparison(left, right, |left, right| left.not_equal(right))
    }
}

/// Compares two Series value-wise using less-than ordering.
pub fn series_less_than(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::ChunkCompareIneq;
        comparison(left, right, |left, right| left.lt(right))
    }
}

/// Compares two Series value-wise using less-than-or-equal ordering.
pub fn series_less_than_or_equal(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::ChunkCompareIneq;
        comparison(left, right, |left, right| left.lt_eq(right))
    }
}

/// Compares two Series value-wise using greater-than ordering.
pub fn series_greater_than(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::ChunkCompareIneq;
        comparison(left, right, |left, right| left.gt(right))
    }
}

/// Compares two Series value-wise using greater-than-or-equal ordering.
pub fn series_greater_than_or_equal(
    left: &TerlanPolarsSeries,
    right: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (left, right);
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        use polars::prelude::ChunkCompareIneq;
        comparison(left, right, |left, right| left.gt_eq(right))
    }
}

#[cfg(feature = "real-polars")]
fn floating_mask(
    series: &TerlanPolarsSeries,
    operation: impl FnOnce(
        &polars::prelude::Series,
    ) -> polars::prelude::PolarsResult<polars::prelude::BooleanChunked>,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    use polars::prelude::IntoSeries;
    Ok(owned(operation(&series.inner)?.into_series()))
}

/// Returns a Boolean mask identifying NaN values.
pub fn series_is_nan(series: &TerlanPolarsSeries) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        floating_mask(series, |input| input.is_nan())
    }
}

/// Returns a Boolean mask identifying values that are not NaN.
pub fn series_is_not_nan(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        floating_mask(series, |input| input.is_not_nan())
    }
}

/// Returns a Boolean mask identifying finite floating-point values.
pub fn series_is_finite(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        floating_mask(series, |input| input.is_finite())
    }
}

/// Returns a Boolean mask identifying infinite floating-point values.
pub fn series_is_infinite(
    series: &TerlanPolarsSeries,
) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = series;
        Err(unavailable())
    }
    #[cfg(feature = "real-polars")]
    {
        floating_mask(series, |input| input.is_infinite())
    }
}
