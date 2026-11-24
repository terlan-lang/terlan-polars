//! CSV identity is computed from the same private snapshot consumed by Polars.

use crate::{TerlanPolarsDataFrame, TerlanPolarsError};

/// One caller-owned frame and the lowercase SHA-256 of its original CSV bytes.
pub struct TerlanPolarsIdentifiedDataFrame {
    /// Independently owned DataFrame; normal DataFrame disposal rules apply.
    pub dataframe: TerlanPolarsDataFrame,
    /// Exactly 64 lowercase hexadecimal characters, without a prefix.
    pub sha256: String,
}

/// Reads and identifies a CSV with date inference from one native byte snapshot.
///
/// Capture uses a fixed 64 KiB buffer and a private, automatically removed
/// temporary file. Use a disk-backed temporary directory for large histories.
/// The source is opened once and never reopened for parsing. Concurrent source
/// writes may affect the captured stream, but cannot make its hash disagree
/// with the bytes parsed. Polars' eager result still requires memory for rows.
#[cfg(feature = "real-polars")]
pub fn read_csv_dates_identified(
    path: &str,
) -> Result<TerlanPolarsIdentifiedDataFrame, TerlanPolarsError> {
    let source = std::fs::File::open(path).map_err(|error| {
        TerlanPolarsError::new(
            "csv_read_error",
            format!("failed to open CSV file `{path}`: {error}"),
        )
    })?;
    let snapshot = capture(source)?;
    parse_snapshot(snapshot)
}

/// Reports the same missing-native capability as other CSV readers.
#[cfg(not(feature = "real-polars"))]
pub fn read_csv_dates_identified(
    _path: &str,
) -> Result<TerlanPolarsIdentifiedDataFrame, TerlanPolarsError> {
    Err(crate::unavailable_error())
}

#[cfg(feature = "real-polars")]
const CAPTURE_BUFFER_BYTES: usize = 64 * 1024;

#[cfg(feature = "real-polars")]
pub(crate) struct Snapshot {
    pub(crate) file: std::fs::File,
    pub(crate) sha256: String,
}

#[cfg(feature = "real-polars")]
pub(crate) fn capture(mut source: impl std::io::Read) -> std::io::Result<Snapshot> {
    use sha2::{Digest, Sha256};
    use std::io::{Seek, Write};

    // An anonymous/private file prevents a path replacement between hashing
    // and parsing. RAII also removes it on read, write, seek, or parse errors.
    let mut file = tempfile::tempfile()?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; CAPTURE_BUFFER_BYTES];
    loop {
        let count = match source.read(&mut buffer) {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if count == 0 {
            break;
        }
        file.write_all(&buffer[..count])?;
        hash.update(&buffer[..count]);
    }
    file.rewind()?;
    Ok(Snapshot {
        file,
        sha256: format!("{:x}", hash.finalize()),
    })
}

#[cfg(feature = "real-polars")]
fn parse_snapshot(
    snapshot: Snapshot,
) -> Result<TerlanPolarsIdentifiedDataFrame, TerlanPolarsError> {
    use polars::prelude::{CsvParseOptions, CsvReadOptions, SerReader};

    let inner = CsvReadOptions::default()
        .with_low_memory(true)
        .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
        .into_reader_with_file_handle(snapshot.file)
        .finish()?;
    Ok(TerlanPolarsIdentifiedDataFrame {
        dataframe: TerlanPolarsDataFrame { inner },
        sha256: snapshot.sha256,
    })
}

#[cfg(all(test, feature = "real-polars"))]
#[path = "identified_csv_test.rs"]
mod tests;
