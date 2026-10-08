//! Database query ingestion through ConnectorX and Arrow IPC.

use crate::{TerlanPolarsDataFrame, TerlanPolarsError};

#[cfg(any(feature = "real-polars", test))]
const MAX_CONNECTION_URI_BYTES: usize = 64 * 1024;
#[cfg(any(feature = "real-polars", test))]
const MAX_ENVIRONMENT_NAME_BYTES: usize = 256;
#[cfg(any(feature = "real-polars", test))]
const MAX_DATABASE_QUERY_BYTES: usize = 1024 * 1024;

/// Reads one SQL query from a supported database URI.
///
/// The current package build supports PostgreSQL and SQLite through
/// ConnectorX. ConnectorX materializes Arrow record batches, which are passed
/// to Polars through Arrow IPC so no database-specific types enter the Terlan
/// ABI.
#[cfg(feature = "real-polars")]
pub fn read_database_uri(
    connection_uri: &str,
    query: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    use arrow::ipc::writer::FileWriter;
    use connectorx::prelude::{get_arrow, CXQuery, SourceConn};
    use polars::prelude::{IpcReader, SerReader};
    use std::convert::TryFrom;
    use std::io::Cursor;

    validate_connection_uri(connection_uri)?;
    validate_query(query)?;
    let source = SourceConn::try_from(connection_uri).map_err(|_| {
        TerlanPolarsError::new(
            "database_connection_invalid",
            "database connection URI is invalid or unsupported",
        )
    })?;
    let destination = get_arrow(&source, None, &[CXQuery::from(query)], None).map_err(|_| {
        TerlanPolarsError::new(
            "database_query_error",
            "database query failed; connection details were redacted",
        )
    })?;
    let batches = destination.arrow().map_err(|_| {
        TerlanPolarsError::new(
            "database_arrow_error",
            "database result could not be materialized as Arrow",
        )
    })?;
    let first = batches.first().ok_or_else(|| {
        TerlanPolarsError::new(
            "database_empty_result",
            "database connector returned no Arrow result batch",
        )
    })?;
    let mut ipc = Vec::new();
    {
        let mut writer = FileWriter::try_new(&mut ipc, first.schema().as_ref()).map_err(|_| {
            TerlanPolarsError::new(
                "database_arrow_error",
                "database Arrow schema could not be encoded",
            )
        })?;
        for batch in &batches {
            writer.write(batch).map_err(|_| {
                TerlanPolarsError::new(
                    "database_arrow_error",
                    "database Arrow batch could not be encoded",
                )
            })?;
        }
        writer.finish().map_err(|_| {
            TerlanPolarsError::new(
                "database_arrow_error",
                "database Arrow stream could not be finalized",
            )
        })?;
    }
    let inner = IpcReader::new(Cursor::new(ipc)).finish()?;
    Ok(TerlanPolarsDataFrame { inner })
}

/// Reads one SQL query using a connection URI stored in an environment
/// variable.
#[cfg(feature = "real-polars")]
pub fn read_database_env(
    environment_name: &str,
    query: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    validate_environment_name(environment_name)?;
    let connection_uri = std::env::var(environment_name).map_err(|_| {
        TerlanPolarsError::new(
            "database_connection_missing",
            format!("database connection environment variable `{environment_name}` is not set"),
        )
    })?;
    read_database_uri(&connection_uri, query)
}

/// Reports that database ingestion requires the real native adapter.
#[cfg(not(feature = "real-polars"))]
pub fn read_database_uri(
    _connection_uri: &str,
    _query: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(TerlanPolarsError::new(
        "native_unavailable",
        "database ingestion requires the Rust native adapter target capability",
    ))
}

/// Reports that database ingestion requires the real native adapter.
#[cfg(not(feature = "real-polars"))]
pub fn read_database_env(
    _environment_name: &str,
    _query: &str,
) -> Result<TerlanPolarsDataFrame, TerlanPolarsError> {
    Err(TerlanPolarsError::new(
        "native_unavailable",
        "database ingestion requires the Rust native adapter target capability",
    ))
}

#[cfg(any(feature = "real-polars", test))]
fn validate_connection_uri(connection_uri: &str) -> Result<(), TerlanPolarsError> {
    if connection_uri.is_empty()
        || connection_uri.len() > MAX_CONNECTION_URI_BYTES
        || connection_uri.contains('\0')
    {
        return Err(TerlanPolarsError::new(
            "database_connection_invalid",
            "database connection URI must be nonempty, NUL-free, and at most 64 KiB",
        ));
    }
    let scheme = connection_uri.split(':').next().unwrap_or_default();
    if !matches!(scheme, "postgres" | "postgresql" | "sqlite") {
        return Err(TerlanPolarsError::new(
            "database_connection_unsupported",
            "database connection URI must use PostgreSQL or SQLite",
        ));
    }
    Ok(())
}

#[cfg(any(feature = "real-polars", test))]
fn validate_environment_name(environment_name: &str) -> Result<(), TerlanPolarsError> {
    if environment_name.is_empty()
        || environment_name.len() > MAX_ENVIRONMENT_NAME_BYTES
        || !environment_name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(TerlanPolarsError::new(
            "database_environment_invalid",
            "database environment name must contain only uppercase ASCII letters, digits, or underscores",
        ));
    }
    Ok(())
}

#[cfg(any(feature = "real-polars", test))]
fn validate_query(query: &str) -> Result<(), TerlanPolarsError> {
    if query.trim().is_empty() || query.len() > MAX_DATABASE_QUERY_BYTES || query.contains('\0') {
        return Err(TerlanPolarsError::new(
            "database_query_invalid",
            "database query must be nonempty, NUL-free, and at most 1 MiB",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_inputs_are_bounded_and_schemes_are_explicit() {
        assert!(validate_connection_uri("sqlite:///tmp/data.db").is_ok());
        assert!(validate_connection_uri("postgresql://localhost/data").is_ok());
        assert_eq!(
            validate_connection_uri("mysql://localhost/data")
                .expect_err("unsupported scheme")
                .code(),
            "database_connection_unsupported"
        );
        assert_eq!(
            validate_query(" \n ").expect_err("empty query").code(),
            "database_query_invalid"
        );
        assert!(validate_environment_name("DATABASE_URL").is_ok());
        assert_eq!(
            validate_environment_name("DatabaseUrl")
                .expect_err("mixed-case environment name")
                .code(),
            "database_environment_invalid"
        );
    }

    #[cfg(feature = "real-polars")]
    #[test]
    fn sqlite_query_round_trips_through_arrow_ipc() {
        use polars::prelude::DataType;
        use rusqlite::Connection;

        let path = std::env::temp_dir().join(format!(
            "terlan-polars-database-{}-{}.sqlite",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        let connection = Connection::open(&path).expect("open SQLite fixture");
        connection
            .execute_batch(
                "CREATE TABLE people(name TEXT NOT NULL, age INTEGER NOT NULL);
                 INSERT INTO people VALUES ('Ada', 36), ('Grace', 85);",
            )
            .expect("seed SQLite fixture");
        drop(connection);

        let uri = format!("sqlite://{}", path.display());
        let dataframe =
            read_database_uri(&uri, "SELECT name, age FROM people ORDER BY age").expect("query");
        assert_eq!(dataframe.inner.height(), 2);
        assert_eq!(dataframe.inner.width(), 2);
        assert_eq!(dataframe.inner["name"].dtype(), &DataType::String);
        assert!(dataframe.inner["age"].dtype().is_integer());

        std::fs::remove_file(path).expect("remove SQLite fixture");
    }
}
