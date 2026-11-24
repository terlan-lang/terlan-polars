//! Identified CSV ingestion with bounded record and batch buffers.
use crate::{TerlanPolarsDataFrame, TerlanPolarsError};

/// Maximum raw CSV bytes retained for a batch, excluding its bounded header.
pub const MAX_BATCH_BYTES: usize = 1024 * 1024;
/// Maximum requested records in one batch.
pub const MAX_BATCH_ROWS: usize = 10_000;

/// Owns a private disk snapshot and bounded CSV buffers, never an eager full frame.
pub struct IdentifiedCsvCursor {
    /// Digest of precisely the captured bytes from which batches are parsed.
    pub sha256: String,
    #[cfg(feature = "real-polars")]
    reader: Option<std::io::BufReader<std::fs::File>>,
    #[cfg(feature = "real-polars")]
    header: Vec<u8>,
    #[cfg(feature = "real-polars")]
    pending: Option<Vec<u8>>,
    #[cfg(feature = "real-polars")]
    schema: Option<polars::prelude::SchemaRef>,
    #[cfg(feature = "real-polars")]
    rows: usize,
    #[cfg(feature = "real-polars")]
    failed: bool,
}

impl IdentifiedCsvCursor {
    /// Captures once, hashes the snapshot, and opens a bounded CSV cursor.
    /// Schema is inferred from the first batch and enforced on later batches.
    pub fn open(path: &str, rows: usize) -> Result<Self, TerlanPolarsError> {
        if rows == 0 || rows > MAX_BATCH_ROWS {
            return Err(TerlanPolarsError::new(
                "csv_batch_size",
                "batch rows must be between 1 and 10000",
            ));
        }
        #[cfg(feature = "real-polars")]
        {
            let source = std::fs::File::open(path)
                .map_err(|error| TerlanPolarsError::new("csv_read_error", error.to_string()))?;
            let snapshot = crate::identified_csv::capture(source)?;
            let mut reader = std::io::BufReader::new(snapshot.file);
            let header = read_record(&mut reader)?
                .ok_or_else(|| TerlanPolarsError::new("csv_read_error", "CSV header is missing"))?;
            Ok(Self {
                sha256: snapshot.sha256,
                reader: Some(reader),
                header,
                pending: None,
                schema: None,
                rows,
                failed: false,
            })
        }
        #[cfg(not(feature = "real-polars"))]
        {
            let _ = path;
            Err(crate::unavailable_error())
        }
    }

    /// Returns an independently owned frame, or EOF. Errors poison the cursor.
    /// Dropping the cursor closes its private snapshot independently of frames.
    pub fn next_batch(&mut self) -> Result<Option<TerlanPolarsDataFrame>, TerlanPolarsError> {
        #[cfg(feature = "real-polars")]
        {
            if self.failed {
                return Err(TerlanPolarsError::new(
                    "csv_cursor_failed",
                    "cursor failed; dispose it",
                ));
            }
            let result = self.read_batch();
            if result.is_err() {
                self.failed = true;
                self.reader = None;
                self.pending = None;
            }
            result
        }
        #[cfg(not(feature = "real-polars"))]
        {
            Err(crate::unavailable_error())
        }
    }

    #[cfg(feature = "real-polars")]
    fn read_batch(&mut self) -> Result<Option<TerlanPolarsDataFrame>, TerlanPolarsError> {
        use polars::prelude::{CsvParseOptions, CsvReadOptions, SerReader};
        let Some(reader) = self.reader.as_mut() else {
            return Ok(None);
        };
        let mut bytes = Vec::new();
        let mut count = 0;
        let mut eof = false;
        while count < self.rows {
            let record = match self.pending.take() {
                Some(record) => Some(record),
                None => read_record(reader)?,
            };
            let Some(record) = record else {
                eof = true;
                break;
            };
            if record.iter().all(|b| matches!(b, b'\r' | b'\n')) {
                continue;
            }
            if bytes.len() + record.len() > MAX_BATCH_BYTES {
                self.pending = Some(record);
                break;
            }
            bytes.extend(record);
            count += 1;
        }
        if eof {
            self.reader = None;
        }
        if count == 0 {
            return Ok(None);
        }
        let mut csv = self.header.clone();
        if !csv.ends_with(b"\n") {
            csv.push(b'\n');
        }
        csv.extend(bytes);
        let inner = CsvReadOptions::default()
            .with_low_memory(true)
            .with_n_threads(Some(1))
            .with_infer_schema_length(None)
            .with_schema(self.schema.clone())
            .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
            .into_reader_with_file_handle(std::io::Cursor::new(csv))
            .finish()?;
        self.schema = Some(inner.schema().clone());
        Ok(Some(TerlanPolarsDataFrame { inner }))
    }
}

#[cfg(feature = "real-polars")]
fn read_record(reader: &mut impl std::io::Read) -> Result<Option<Vec<u8>>, TerlanPolarsError> {
    let mut record = Vec::new();
    let mut quoted = false;
    let mut byte = [0];
    loop {
        let count = match reader.read(&mut byte) {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if count == 0 {
            if quoted {
                return Err(TerlanPolarsError::new(
                    "csv_read_error",
                    "unterminated quoted CSV record",
                ));
            }
            return Ok((!record.is_empty()).then_some(record));
        }
        if record.len() == MAX_BATCH_BYTES {
            return Err(TerlanPolarsError::new(
                "csv_record_limit",
                "CSV record exceeds 1 MiB",
            ));
        }
        record.push(byte[0]);
        if byte[0] == b'"' {
            quoted = !quoted;
        }
        if byte[0] == b'\n' && !quoted {
            return Ok(Some(record));
        }
    }
}

#[cfg(all(test, feature = "real-polars"))]
#[path = "csv_cursor_test.rs"]
mod tests;
