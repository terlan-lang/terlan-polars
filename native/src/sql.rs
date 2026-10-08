//! SQL query contexts backed by Polars logical plans.

use crate::{TerlanPolarsDataFrame, TerlanPolarsError, TerlanPolarsLazyFrame};
#[cfg(feature = "real-polars")]
use polars::prelude::IntoLazy;

const MAX_SQL_QUERY_BYTES: usize = 1_048_576;
const MAX_SQL_TABLE_NAME_BYTES: usize = 1_024;

/// Package-owned Polars SQL context placeholder.
#[cfg(not(feature = "real-polars"))]
pub struct TerlanPolarsSqlContext;

/// Package-owned Polars SQL context.
#[cfg(feature = "real-polars")]
#[derive(Clone)]
pub struct TerlanPolarsSqlContext {
    pub(crate) inner: polars::sql::SQLContext,
}

/// Creates an empty SQL context.
pub fn sql_context_new() -> TerlanPolarsSqlContext {
    #[cfg(feature = "real-polars")]
    {
        TerlanPolarsSqlContext {
            inner: polars::sql::SQLContext::new(),
        }
    }
    #[cfg(not(feature = "real-polars"))]
    {
        TerlanPolarsSqlContext
    }
}

/// Registers a DataFrame as a named SQL table.
#[cfg(feature = "real-polars")]
pub fn sql_context_register(
    context: &TerlanPolarsSqlContext,
    name: &str,
    dataframe: &TerlanPolarsDataFrame,
) -> Result<(), TerlanPolarsError> {
    validate_table_name(name)?;
    context.inner.register(name, dataframe.inner.clone().lazy());
    Ok(())
}

/// Registers a DataFrame as a named SQL table.
#[cfg(not(feature = "real-polars"))]
pub fn sql_context_register(
    _context: &TerlanPolarsSqlContext,
    name: &str,
    _dataframe: &TerlanPolarsDataFrame,
) -> Result<(), TerlanPolarsError> {
    validate_table_name(name)?;
    Err(sql_error(
        "native_unavailable",
        "SQL execution requires the `real-polars` feature",
    ))
}

/// Removes a named SQL table.
#[cfg(feature = "real-polars")]
pub fn sql_context_unregister(
    context: &TerlanPolarsSqlContext,
    name: &str,
) -> Result<(), TerlanPolarsError> {
    validate_table_name(name)?;
    if !context.inner.get_tables().iter().any(|table| table == name) {
        return Err(sql_error(
            "sql_table_missing",
            format!("SQL table `{name}` is not registered"),
        ));
    }
    context.inner.unregister(name);
    Ok(())
}

/// Removes a named SQL table.
#[cfg(not(feature = "real-polars"))]
pub fn sql_context_unregister(
    _context: &TerlanPolarsSqlContext,
    name: &str,
) -> Result<(), TerlanPolarsError> {
    validate_table_name(name)?;
    Err(sql_error(
        "native_unavailable",
        "SQL execution requires the `real-polars` feature",
    ))
}

/// Returns registered SQL table names in deterministic order.
#[cfg(feature = "real-polars")]
pub fn sql_context_tables(context: &TerlanPolarsSqlContext) -> Vec<String> {
    context.inner.get_tables()
}

/// Returns registered SQL table names in deterministic order.
#[cfg(not(feature = "real-polars"))]
pub fn sql_context_tables(_context: &TerlanPolarsSqlContext) -> Vec<String> {
    Vec::new()
}

/// Returns every keyword recognized by the pinned Polars SQL parser.
pub fn sql_keywords() -> Result<Vec<String>, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        Ok(polars::sql::keywords::all_keywords()
            .into_iter()
            .map(str::to_string)
            .collect())
    }
    #[cfg(not(feature = "real-polars"))]
    {
        Err(sql_error(
            "native_unavailable",
            "SQL keyword discovery requires the `real-polars` feature",
        ))
    }
}

/// Returns every function name recognized by the pinned Polars SQL parser.
pub fn sql_functions() -> Result<Vec<String>, TerlanPolarsError> {
    #[cfg(feature = "real-polars")]
    {
        Ok(polars::sql::keywords::all_functions()
            .into_iter()
            .map(str::to_string)
            .collect())
    }
    #[cfg(not(feature = "real-polars"))]
    {
        Err(sql_error(
            "native_unavailable",
            "SQL function discovery requires the `real-polars` feature",
        ))
    }
}

/// Extracts table identifiers from one SQL statement.
pub fn sql_table_identifiers(
    query: &str,
    include_schema: bool,
    unique: bool,
) -> Result<Vec<String>, TerlanPolarsError> {
    validate_query(query)?;
    #[cfg(feature = "real-polars")]
    {
        polars::sql::extract_table_identifiers(query, include_schema, unique)
            .map_err(|error| sql_error("sql_error", error.to_string()))
    }
    #[cfg(not(feature = "real-polars"))]
    {
        let _ = (include_schema, unique);
        Err(sql_error(
            "native_unavailable",
            "SQL identifier extraction requires the `real-polars` feature",
        ))
    }
}

/// Compiles one SQL statement into a lazy Polars plan.
#[cfg(feature = "real-polars")]
pub fn sql_context_execute(
    context: &mut TerlanPolarsSqlContext,
    query: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_query(query)?;
    context
        .inner
        .execute(query)
        .map(|inner| TerlanPolarsLazyFrame { inner })
        .map_err(|error| sql_error("sql_error", error.to_string()))
}

/// Compiles one SQL statement into a lazy Polars plan.
#[cfg(not(feature = "real-polars"))]
pub fn sql_context_execute(
    _context: &mut TerlanPolarsSqlContext,
    query: &str,
) -> Result<TerlanPolarsLazyFrame, TerlanPolarsError> {
    validate_query(query)?;
    Err(sql_error(
        "native_unavailable",
        "SQL execution requires the `real-polars` feature",
    ))
}

fn validate_table_name(name: &str) -> Result<(), TerlanPolarsError> {
    if name.is_empty() || name.len() > MAX_SQL_TABLE_NAME_BYTES || name.contains('\0') {
        return Err(sql_error(
            "invalid_sql_table_name",
            "SQL table names must be non-empty, NUL-free, and at most 1024 bytes",
        ));
    }
    Ok(())
}

fn validate_query(query: &str) -> Result<(), TerlanPolarsError> {
    if query.trim().is_empty() {
        return Err(sql_error(
            "invalid_sql_query",
            "SQL query must not be empty",
        ));
    }
    if query.len() > MAX_SQL_QUERY_BYTES || query.contains('\0') {
        return Err(sql_error(
            "invalid_sql_query",
            "SQL query must be NUL-free and at most 1048576 bytes",
        ));
    }
    Ok(())
}

fn sql_error(code: &'static str, message: impl Into<String>) -> TerlanPolarsError {
    TerlanPolarsError::new(code, message)
}

#[cfg(all(test, feature = "real-polars"))]
mod tests {
    use super::*;
    use crate::expressions::{compile_many, expr_alias, expr_sql};
    use crate::{collect_lazy, from_rows, height, rows};
    use polars::df;
    use polars::prelude::IntoLazy;

    #[test]
    fn sql_context_registers_queries_and_unregisters_tables() {
        let dataframe = from_rows(
            &["name".to_string(), "score".to_string()],
            &[
                vec!["Ada".to_string(), "2".to_string()],
                vec!["Grace".to_string(), "3".to_string()],
            ],
        )
        .expect("dataframe");
        let mut context = sql_context_new();

        sql_context_register(&context, "people", &dataframe).expect("register");
        assert_eq!(sql_context_tables(&context), vec!["people"]);
        let plan = sql_context_execute(&mut context, "SELECT name FROM people WHERE score = '3'")
            .expect("compile SQL");
        let result = collect_lazy(&plan).expect("execute SQL");
        assert_eq!(height(&result), 1);
        assert_eq!(rows(&result, 1).expect("rows"), vec![vec!["Grace"]]);

        sql_context_unregister(&context, "people").expect("unregister");
        assert!(sql_context_tables(&context).is_empty());
    }

    #[test]
    fn sql_context_rejects_invalid_queries_and_missing_tables() {
        let mut context = sql_context_new();

        let error = match sql_context_execute(&mut context, " ") {
            Ok(_) => panic!("empty SQL query was accepted"),
            Err(error) => error,
        };
        assert_eq!(error.code(), "invalid_sql_query");
        assert_eq!(
            sql_context_unregister(&context, "missing")
                .unwrap_err()
                .code(),
            "sql_table_missing"
        );
        assert_eq!(
            sql_context_register(
                &context,
                "",
                &from_rows(&["value".to_string()], &[vec!["1".to_string()]]).expect("dataframe"),
            )
            .unwrap_err()
            .code(),
            "invalid_sql_table_name"
        );
    }

    #[test]
    fn sql_discovery_identifiers_and_expressions_execute() {
        let keywords = sql_keywords().expect("SQL keywords");
        let functions = sql_functions().expect("SQL functions");
        assert!(keywords.iter().any(|keyword| keyword == "SELECT"));
        assert!(functions
            .iter()
            .any(|function| function.eq_ignore_ascii_case("SUM")));

        assert_eq!(
            sql_table_identifiers(
                "SELECT * FROM analytics.people JOIN events ON true",
                true,
                true,
            )
            .expect("table identifiers"),
            ["analytics.people", "events"]
        );

        let expression = expr_alias(
            &expr_sql("score + 1").expect("SQL expression"),
            "incremented",
        )
        .expect("SQL expression alias");
        let output = df!("score" => [1_i64, 2])
            .expect("SQL expression fixture")
            .lazy()
            .select(compile_many(&[expression]).expect("compile SQL expression"))
            .collect()
            .expect("execute SQL expression");
        assert_eq!(
            output
                .column("incremented")
                .expect("incremented column")
                .i64()
                .expect("integer output")
                .into_no_null_iter()
                .collect::<Vec<_>>(),
            [2, 3]
        );

        assert_eq!(
            sql_table_identifiers("", false, false)
                .expect_err("empty SQL")
                .code(),
            "invalid_sql_query"
        );
        assert_eq!(
            expr_sql("\0").expect_err("NUL SQL expression").code(),
            "invalid_sql_query"
        );
    }
}
