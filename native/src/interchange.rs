//! Apache Arrow IPC interchange for DataFrames.

use crate::{TerlanPolarsDataFrame, TerlanPolarsError, TerlanPolarsSeries};

/// Serializes a DataFrame into an Apache Arrow IPC file payload.
#[cfg(feature = "real-polars")]
pub fn dataframe_to_arrow_ipc(
    dataframe: &TerlanPolarsDataFrame,
) -> Result<Vec<u8>, TerlanPolarsError> {
    use polars::prelude::{IpcWriter, SerWriter};

    let mut bytes = Vec::new();
    let mut inner = dataframe.inner.clone();
    IpcWriter::new(&mut bytes)
        .finish(&mut inner)
        .map_err(arrow_ipc_error)?;
    Ok(bytes)
}

/// Reconstructs a DataFrame from an Apache Arrow IPC file payload.
#[cfg(feature = "real-polars")]
pub fn dataframe_from_arrow_ipc(bytes: &[u8]) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use polars::prelude::{IpcReader, SerReader};

    if bytes.is_empty() {
        return Err(TerlanPolarsError::new(
            "arrow_ipc_error",
            "Arrow IPC payload cannot be empty",
        ));
    }
    let inner = IpcReader::new(std::io::Cursor::new(bytes))
        .finish()
        .map_err(arrow_ipc_error)?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Reports the unavailable real-Polars adapter in the skeleton build.
#[cfg(not(feature = "real-polars"))]
pub fn dataframe_to_arrow_ipc(
    _dataframe: &TerlanPolarsDataFrame,
) -> Result<Vec<u8>, TerlanPolarsError> {
    Err(TerlanPolarsError::new(
        "native_unavailable",
        "Arrow IPC interchange requires the Rust native adapter target capability",
    ))
}

/// Reports the unavailable real-Polars adapter in the skeleton build.
#[cfg(not(feature = "real-polars"))]
pub fn dataframe_from_arrow_ipc(_bytes: &[u8]) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(TerlanPolarsError::new(
        "native_unavailable",
        "Arrow IPC interchange requires the Rust native adapter target capability",
    ))
}

/// Converts a Polars IPC failure into the package's stable error vocabulary.
#[cfg(feature = "real-polars")]
fn arrow_ipc_error(error: polars::error::PolarsError) -> TerlanPolarsError {
    TerlanPolarsError::new(
        "arrow_ipc_error",
        format!("Arrow IPC operation failed: {error}"),
    )
}

/// Serializes one Series through a single-column Arrow IPC file.
pub fn series_to_arrow_ipc(series: &TerlanPolarsSeries) -> Result<Vec<u8>, TerlanPolarsError> {
    let dataframe = crate::series_to_frame(series)?;
    dataframe_to_arrow_ipc(&dataframe)
}

/// Reconstructs one Series from a single-column Arrow IPC file.
pub fn series_from_arrow_ipc(bytes: &[u8]) -> Result<TerlanPolarsSeries, TerlanPolarsError> {
    let dataframe = dataframe_from_arrow_ipc(bytes)?;
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = dataframe;
        Err(TerlanPolarsError::new(
            "native_unavailable",
            "Arrow IPC interchange requires the Rust native adapter target capability",
        ))
    }
    #[cfg(feature = "real-polars")]
    {
        if dataframe.inner.width() != 1 {
            return Err(TerlanPolarsError::new(
                "arrow_ipc_error",
                format!(
                    "Series Arrow IPC requires exactly one column, found {}",
                    dataframe.inner.width()
                ),
            ));
        }
        let series = dataframe
            .inner
            .select_at_idx(0)
            .expect("width was checked before selecting the only column")
            .as_materialized_series()
            .clone();
        Ok(TerlanPolarsSeries { inner: series })
    }
}
